- Start Date: 2022-12-11
- RFC PR: [maplibre/maplibre-rs#223](https://github.com/maplibre/maplibre-rs/pull/223)
- maplibre-rs Issue: 
[maplibre/maplibre-rs#190](https://github.com/maplibre/maplibre-rs/pull/190) 
[maplibre/maplibre-rs#174](https://github.com/maplibre/maplibre-rs/pull/174)

# Summary

Rendering data in real-time requires developers to carefully decide which work to 
perform on the main rendering thread and which work can be done asynchronously.

This RFC focuses on describing how asynchronous work can be done on the Web platform, while still allowing
other platform to use other paradigms.

# Motivation

On the Web platform we do not have threads or processes available.
Instead, we have WebWorkers. WebWorkers are very similar to processes in the Unix-world.
With the recent "atomics" proposals in WebAssembly and its shared-memory support it is actually possible
to lift WebWorkers from be being processes to fully fledged threads (i.e. synchronizing on shared-memory using mutexes).

Though, using shared-memory requires
[settings special HTTP-headers](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer#security_requirements)
, which limit a websites cross-site capabilities.
For this reason maplibre-rs needs a way to do work asynchronously without relying on shared-memory.
Other platforms (Linux, Android, iOS) should still be able to leverage shared-memory though.

# Detailed design

## Asynchronous Procedure Calls (APCs)

Analogous to remote procedure calls, we are using asynchronous procedure calls which call a procedure in a foreign 
process/thread.
Depending on the platform a different implementation for APCs are used. On Linux we can use threads to do work 
asynchronously and use a multiple-consumer single-consumer channel to send data back and forth.
On the Web platform we use `WebWorkers` to do work asynchronously and its`postMessage` API to send data back and forth.

## Exchanging Data

We are using the browser API `postMessage(message, targetOrigin, transfer)` to send data between WebWorkers and
the main event loop. We are using the `postMessage` call which includes the `transfer` parameter. By using this parameter
we can transfer the ownership of the `message`. After the message has been sent it can no longer be used from the origin
WebWorker/main event loop.

On Linux or all other platforms, we have shared-memory so the data does not really need to be exchanged or transferred.
We only need to transfer pointers to the data and transfer ownership of the data. Rust allows us to safely do this.

## Serialization

Because we are using WebAssembly we can use `ArrayBuffers` (which are transferable to/from WebWorkers) and
avoid expensive serialization and deserialization using for example JSON.

We can more or less skip serialization because we can use zero-copy data formats when exchanging data. 
There are several ways to do this:

1. Reinterpreting between Rust structs (WebAssembly memory) and bytes
2. **Using a well-defined format like Cap'n Proto or Flatbuffers** (Accepted solution)

The second option has the benefit of being safer. Therefore, the current implementation for APCs is using it.
By using established libraries for doing  the reinterpretation
we do not have to deal with alignment, padding bytes, and avoiding undefined behavior.

One might ask why we can not use the well-known library [bytemuck](https://docs.rs/bytemuck/latest/bytemuck/) to do 
the reinterpretation. We go into detail about this in the [Alternatives for Serialization](#for-serialization) section.

A good library for doing zero-copy serialization is [Flatbuffers](https://google.github.io/flatbuffers/).
By defining a schema and generating code from it, we can serialize between a byte array and structures with pointers
into that byte array.

An ideal implementation would only allocate once in the worker thread, send that over to the main thread
and then read from it. Only two copies are required here, because we need to copy the buffer from JS-world 
to the linear WebAssembly memory.

For example when tessellating from geo data to vertices:

1. Allocate once some buffer of sufficient length (if it becomes larger, then reallocate).
2. Start the serialization by using a Flatbuffer builder.
3. Generate the vertices and directly pass them to the builder which stores it in the buffer.
4. Copy the buffer to a new ArrayBuffer.
5. Transfer the ArrayBuffer using `postMessage`. (No copy happens)
6. Copy the ArrayBuffer directly to the WebAssembly linear memory.
7. Without any additional copies, directly pass the buffer to WebGPU.

# Alternatives

## For APCs

No alternative designs were considered.

## For Exchanging Data

There is no other way except for shared-memory and `postMessage` to exchange data between a WebWorker and other threads.

## For Serialization

1. Reinterpreting using `bytemuck`

   The main reason why this is not feasible, is that it does not allow for arbitrary length data.
   A first implementation for APCs using WebWorkers, actually used `bytemuck`. This was great for a proof-of-concept, but
   supporting arbitrary length data (e.g. variable sized vertex buffers) is important. Also `bytemuck` has several [safety](https://docs.rs/bytemuck/1.12.3/bytemuck/trait.Pod.html#safety)
   constraints which can be hard to follow sometimes like: 1) `#[repr(C)]` 2) no padding bytes 3) no enums.
2. Reinterpreting a `bytemuck`-like library
   
   As an experiment I implemented a similar library like `bytemuck` with less safety constrains. For example without
   requirement of `#[repr(C)]`. When passing data between a WebWorker and the main thread all WebAssembly instances
   are using the same memory layout for structs. This means in order to be interchangeable, the `#[repr(C)]` is not
   required, as long as the data being passed is processed by the same binaries on the same platform. Through
   requirements like padding bytes stayed. Also, the solution did still not support variable sized data. See [^1] and [^2] for an
   example implementation called `transferable-memory`.

3. Cap'n Proto
   
   The Cap'n Proto library is a very similar library compared to Flatbuffers. I selected Flatbuffers over Cap'n Proto
   after noticing that the API is not easy to use and not a lot of documentation is available.
   


# Unresolved questions

* How performant is the APC design and its implementations?
* Is shared-memory faster or `postMessage` in different browsers?


[^1]: Implementation for `transferable-memory` (for reference, not used in maplibre-rs)
```rust
pub mod intransfer {
    use js_sys::Uint8Array;

    use crate::{bytes_of, from_bytes, MemoryTransferable};

    pub struct InTransferMemory {
        pub type_id: u32,
        pub buffer: js_sys::ArrayBuffer,
    }

    pub trait InTransfer
        where
            Self: Copy,
    {
        fn to_in_transfer(&self, type_id: u32) -> InTransferMemory {
            let data = unsafe { bytes_of(self) };
            let serialized_array_buffer = js_sys::ArrayBuffer::new(data.len() as u32);
            let serialized_array = js_sys::Uint8Array::new(&serialized_array_buffer);
            unsafe {
                serialized_array.set(&js_sys::Uint8Array::view(data), 0);
            }

            InTransferMemory {
                type_id,
                buffer: serialized_array_buffer,
            }
        }

        fn from_in_transfer(in_transfer: InTransferMemory) -> Self
            where
                Self: Sized,
        {
            unsafe { *from_bytes(&Uint8Array::new(&in_transfer.buffer).to_vec()) }
        }

        fn from_in_transfer_boxed(in_transfer: InTransferMemory) -> Box<Self>
            where
                Self: Sized,
        {
            unsafe {
                let data = Uint8Array::new(&in_transfer.buffer);
                let mut uninit = Box::<Self>::new_zeroed();
                data.raw_copy_to_ptr(uninit.as_mut_ptr() as *mut u8);
                uninit.assume_init()
            }
        }
    }

    impl<T> InTransfer for T where T: MemoryTransferable + Copy {}
}

use std::mem::{align_of, size_of};

pub unsafe trait MemoryTransferable {}
unsafe impl<T, const N: usize> MemoryTransferable for [T; N] where T: MemoryTransferable {}

unsafe impl MemoryTransferable for () {}
unsafe impl MemoryTransferable for u8 {}
unsafe impl MemoryTransferable for i8 {}
unsafe impl MemoryTransferable for u16 {}
unsafe impl MemoryTransferable for i16 {}
unsafe impl MemoryTransferable for u32 {}
unsafe impl MemoryTransferable for i32 {}
unsafe impl MemoryTransferable for u64 {}
unsafe impl MemoryTransferable for i64 {}
unsafe impl MemoryTransferable for usize {}
unsafe impl MemoryTransferable for isize {}
unsafe impl MemoryTransferable for u128 {}
unsafe impl MemoryTransferable for i128 {}
unsafe impl MemoryTransferable for f32 {}
unsafe impl MemoryTransferable for f64 {}

/// Immediately panics.
#[cold]
#[inline(never)]
pub(crate) fn something_went_wrong<D: core::fmt::Display>(_src: &str, _err: D) -> ! {
    // Note(Lokathor): Keeping the panic here makes the panic _formatting_ go
    // here too, which helps assembly readability and also helps keep down
    // the inline pressure.
    panic!("{src}>{err}", src = _src, err = _err);
}

/// The things that can go wrong when casting between [`Pod`] data forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTransferableError {
    /// You tried to cast a slice to an element type with a higher alignment
    /// requirement but the slice wasn't aligned.
    TargetAlignmentGreaterAndInputNotAligned,
    /// If the element size changes then the output slice changes length
    /// accordingly. If the output slice wouldn't be a whole number of elements
    /// then the conversion fails.
    OutputSliceWouldHaveSlop,
    /// When casting a slice you can't convert between ZST elements and non-ZST
    /// elements. When casting an individual `T`, `&T`, or `&mut T` value the
    /// source size and destination size must be an exact match.
    SizeMismatch,
}

impl core::fmt::Display for MemoryTransferableError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Re-interprets `&[u8]` as `&T`.
///
/// ## Panics
///
/// This is [`try_from_bytes`] but will panic on error.
#[inline]
unsafe fn from_bytes<T: Copy>(s: &[u8]) -> &T {
    match try_from_bytes(s) {
        Ok(t) => t,
        Err(e) => something_went_wrong("from_bytes", e),
    }
}

/// Re-interprets `&[u8]` as `&T`.
///
/// ## Failure
///
/// * If the slice isn't aligned for the new type
/// * If the slice's length isn’t exactly the size of the new type
#[inline]
unsafe fn try_from_bytes<T: Copy>(s: &[u8]) -> Result<&T, MemoryTransferableError> {
    if s.len() != size_of::<T>() {
        Err(MemoryTransferableError::SizeMismatch)
    } else if (s.as_ptr() as usize) % align_of::<T>() != 0 {
        Err(MemoryTransferableError::TargetAlignmentGreaterAndInputNotAligned)
    } else {
        Ok(unsafe { &*(s.as_ptr() as *const T) })
    }
}

/// Re-interprets `&T` as `&[u8]`.
///
/// Any ZST becomes an empty slice, and in that case the pointer value of that
/// empty slice might not match the pointer value of the input reference.
#[inline(always)]
unsafe fn bytes_of<T: Copy>(t: &T) -> &[u8] {
    if size_of::<T>() == 0 {
        &[]
    } else {
        match try_cast_slice::<T, u8>(core::slice::from_ref(t)) {
            Ok(s) => s,
            Err(_) => unreachable!(),
        }
    }
}

/// Try to convert `&[A]` into `&[B]` (possibly with a change in length).
///
/// * `input.as_ptr() as usize == output.as_ptr() as usize`
/// * `input.len() * size_of::<A>() == output.len() * size_of::<B>()`
///
/// ## Failure
///
/// * If the target type has a greater alignment requirement and the input slice
///   isn't aligned.
/// * If the target element type is a different size from the current element
///   type, and the output slice wouldn't be a whole number of elements when
///   accounting for the size change (eg: 3 `u16` values is 1.5 `u32` values, so
///   that's a failure).
/// * Similarly, you can't convert between a [ZST](https://doc.rust-lang.org/nomicon/exotic-sizes.html#zero-sized-types-zsts)
///   and a non-ZST.
#[inline]
unsafe fn try_cast_slice<A: Copy, B: Copy>(a: &[A]) -> Result<&[B], MemoryTransferableError> {
    // Note(Lokathor): everything with `align_of` and `size_of` will optimize away
    // after monomorphization.
    if align_of::<B>() > align_of::<A>() && (a.as_ptr() as usize) % align_of::<B>() != 0 {
        Err(MemoryTransferableError::TargetAlignmentGreaterAndInputNotAligned)
    } else if size_of::<B>() == size_of::<A>() {
        Ok(unsafe { core::slice::from_raw_parts(a.as_ptr() as *const B, a.len()) })
    } else if size_of::<A>() == 0 || size_of::<B>() == 0 {
        Err(MemoryTransferableError::SizeMismatch)
    } else if core::mem::size_of_val(a) % size_of::<B>() == 0 {
        let new_len = core::mem::size_of_val(a) / size_of::<B>();
        Ok(unsafe { core::slice::from_raw_parts(a.as_ptr() as *const B, new_len) })
    } else {
        Err(MemoryTransferableError::OutputSliceWouldHaveSlop)
    }
}
```

[^2] Implementation for `transferable-memory-derive` (for reference, not used in maplibre-rs)
```rust
mod traits {
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
}

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
```
