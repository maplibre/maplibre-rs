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
