use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Result, *};

macro_rules! bail {
    ($msg:expr $(,)?) => {
        return Err(Error::new(Span::call_site(), &$msg[..]))
    };

    ( $msg:expr => $span_to_blame:expr $(,)? ) => {
        return Err(Error::new_spanned(&$span_to_blame, $msg))
    };
}

pub trait Derivable {
    fn ident(input: &DeriveInput) -> Result<syn::Path>;
    fn implies_trait() -> Option<TokenStream> {
        None
    }
    fn asserts(_input: &DeriveInput) -> Result<TokenStream> {
        Ok(quote!())
    }
    fn check_attributes(_ty: &Data, _attributes: &[Attribute]) -> Result<()> {
        Ok(())
    }
    fn trait_impl(_input: &DeriveInput) -> Result<(TokenStream, TokenStream)> {
        Ok((quote!(), quote!()))
    }
}

pub struct MemoryTransferable;

impl Derivable for MemoryTransferable {
    fn ident(_: &DeriveInput) -> Result<syn::Path> {
        Ok(syn::parse_quote!(::transferable_memory::MemoryTransferable))
    }

    fn asserts(input: &DeriveInput) -> Result<TokenStream> {
        /* FIXME       if !input.generics.params.is_empty() {
              bail!("\
          MemoryTransferable requires cannot be derived for types containing \
          generic parameters because the padding requirements can't be verified \
          for generic structs\
        " => input.generics.params.first().unwrap());
          }*/

        match &input.data {
            Data::Struct(_) => {
                //FIXME: padding calc odes not work with generics: let assert_no_padding = Some(generate_assert_no_padding(input)?);
                let assert_fields_are_memory_transferable =
                    generate_fields_are_trait(input, Self::ident(input)?)?;

                Ok(quote!(
                  //#assert_no_padding
                  #assert_fields_are_memory_transferable
                ))
            }
            Data::Enum(_) => bail!("Deriving MemoryTransferable is not supported for enums"),
            Data::Union(_) => bail!("Deriving MemoryTransferable is not supported for unions"),
        }
    }

    fn check_attributes(_ty: &Data, _attributes: &[Attribute]) -> Result<()> {
        Ok(())
    }
}

/// Check that a struct has no padding by asserting that the size of the struct
/// is equal to the sum of the size of it's fields
fn generate_assert_no_padding(input: &DeriveInput) -> Result<TokenStream> {
    let struct_type = &input.ident;
    let span = input.ident.span();
    let fields = get_fields(input)?;

    let mut field_types = get_field_types(&fields);
    let size_sum = if let Some(first) = field_types.next() {
        let size_first = quote_spanned!(span => ::core::mem::size_of::<#first>());
        let size_rest = quote_spanned!(span => #( + ::core::mem::size_of::<#field_types>() )*);

        quote_spanned!(span => #size_first #size_rest)
    } else {
        quote_spanned!(span => 0)
    };

    Ok(quote_spanned! {span => const _: fn() = || {
      struct TypeWithoutPadding([u8; #size_sum]);
      let _ = ::core::mem::transmute::<#struct_type, TypeWithoutPadding>;
    };})
}

/// Check that all fields implement a given trait
fn generate_fields_are_trait(input: &DeriveInput, trait_: syn::Path) -> Result<TokenStream> {
    let (impl_generics, _ty_generics, where_clause) = input.generics.split_for_impl();
    let fields = get_fields(input)?;
    let span = input.span();
    let field_types = get_field_types(&fields);
    Ok(quote_spanned! {span => #(const _: fn() = || {
        #[allow(clippy::missing_const_for_fn)]
        fn check #impl_generics () #where_clause {
          fn assert_impl<T: #trait_>() {}
          assert_impl::<#field_types>();
        }
      };)*
    })
}

fn get_struct_fields(input: &DeriveInput) -> Result<&Fields> {
    if let Data::Struct(DataStruct { fields, .. }) = &input.data {
        Ok(fields)
    } else {
        bail!("deriving this trait is only supported for structs")
    }
}

fn get_field_types<'a>(fields: &'a Fields) -> impl Iterator<Item = &'a Type> + 'a {
    fields.iter().map(|field| &field.ty)
}

fn get_fields(input: &DeriveInput) -> Result<Fields> {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => Ok(fields.clone()),
        Data::Union(DataUnion { fields, .. }) => Ok(Fields::Named(fields.clone())),
        Data::Enum(_) => bail!("deriving this trait is not supported for enums"),
    }
}
