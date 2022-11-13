#![deny(unused_imports)]

mod traits;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Result};

use crate::traits::{Derivable, MemoryTransferable};

/// Derive the `MemoryTransferable` trait for a struct
///
/// The macro ensures that the struct follows all the the safety requirements
/// for the `MemoryTransferable` trait.
///
/// The following constraints need to be satisfied for the macro to succeed
///
/// - All fields in the struct must implement `MemoryTransferable`
/// - The struct must NOT be `#[repr(C)]` or `#[repr(transparent)]`
/// - The struct must not contain any padding bytes
/// - The struct contains no generic parameters
///
/// ## Example
///
/// ```rust
/// # use transferable_memory::{MemoryTransferable, Zeroable};
///
/// #[derive(Copy, Clone, MemoryTransferable)]
/// struct Test {
///   a: u16,
///   b: u16,
/// }
/// ```
#[proc_macro_derive(MemoryTransferable)]
pub fn derive_memory_transferable(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let expanded =
        derive_marker_trait::<MemoryTransferable>(parse_macro_input!(input as DeriveInput));

    proc_macro::TokenStream::from(expanded)
}

/// Basic wrapper for error handling
fn derive_marker_trait<Trait: Derivable>(input: DeriveInput) -> TokenStream {
    derive_marker_trait_inner::<Trait>(input).unwrap_or_else(|err| err.into_compile_error())
}

fn derive_marker_trait_inner<Trait: Derivable>(mut input: DeriveInput) -> Result<TokenStream> {
    // Enforce MemoryTransferable on all generic fields.
    let trait_ = Trait::ident(&input)?;
    add_trait_marker(&mut input.generics, &trait_);

    let name = &input.ident;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Trait::check_attributes(&input.data, &input.attrs)?;
    let asserts = Trait::asserts(&input)?;
    let (trait_impl_extras, trait_impl) = Trait::trait_impl(&input)?;

    let implies_trait = if let Some(implies_trait) = Trait::implies_trait() {
        quote!(unsafe impl #implies_trait for #name {})
    } else {
        quote!()
    };

    Ok(quote! {
      #asserts

      #trait_impl_extras

      unsafe impl #impl_generics #trait_ for #name #ty_generics #where_clause {
        #trait_impl
      }

      #implies_trait
    })
}

/// Add a trait marker to the generics if it is not already present
fn add_trait_marker(generics: &mut syn::Generics, trait_name: &syn::Path) {
    // Get each generic type parameter.
    let type_params = generics
        .type_params()
        .map(|param| &param.ident)
        .map(|param| {
            syn::parse_quote!(
              #param: #trait_name
            )
        })
        .collect::<Vec<syn::WherePredicate>>();

    generics.make_where_clause().predicates.extend(type_params);
}
