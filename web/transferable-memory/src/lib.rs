#![feature(allocator_api, new_uninit)]
#![deny(unused_imports)]

use std::mem::{align_of, size_of};

use js_sys::{ArrayBuffer, Uint8Array};

pub struct InTransferMemory {
    pub type_id: u32,
    pub buffer: ArrayBuffer,
}

pub unsafe trait MemoryTransferable
where
    Self: Copy,
{
    fn to_in_transfer(&self, type_id: u32) -> InTransferMemory {
        let data = unsafe { bytes_of(self) };
        let serialized_array_buffer = ArrayBuffer::new(data.len() as u32);
        let serialized_array = Uint8Array::new(&serialized_array_buffer);
        unsafe {
            serialized_array.set(&Uint8Array::view(data), 0);
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
