use std::collections::HashMap;
use std::{any::Any, cell::RefCell, marker::PhantomData};

use wgpu::TextureView;

use super::{DeclaredDependency, GraphResource, RenderTargetHandle, RpassTemporaryPool};

/// Handle to arbitrary graph-stored data.
pub struct DataHandle<T> {
    pub(super) idx: usize,
    pub(super) _phantom: PhantomData<T>,
}

impl<T> std::fmt::Debug for DataHandle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataHandle")
            .field("idx", &self.idx)
            .finish()
    }
}

impl<T> Copy for DataHandle<T> {}

impl<T> Clone for DataHandle<T> {
    fn clone(&self) -> Self {
        Self {
            idx: self.idx,
            _phantom: self._phantom,
        }
    }
}

impl<T> PartialEq for DataHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && self._phantom == other._phantom
    }
}

/// Provides read-only access to the renderer and access to graph resources.
///
/// This is how you turn [DeclaredDependency] into actual wgpu resources.
pub struct RenderGraphDataStore<'a> {
    pub(super) texture_mapping: &'a HashMap<usize, TextureView>,
    pub(super) data: &'a [Box<dyn Any>], // Any is RefCell<Option<T>> where T is the stored data
    pub(super) output: Option<&'a TextureView>,
}

impl<'a> RenderGraphDataStore<'a> {
    /// Get a rendertarget from the handle to one.
    pub fn get_render_target(
        &self,
        dep: DeclaredDependency<RenderTargetHandle>,
    ) -> &'a TextureView {
        match dep.handle.resource {
            GraphResource::Texture(name) => self
                .texture_mapping
                .get(&name)
                .expect("internal rendergraph error: failed to get named texture"),
            GraphResource::OutputTexture => self
                .output
                .expect("internal rendergraph error: tried to get unacquired surface image"),
            r => {
                panic!(
                    "internal rendergraph error: tried to get a {:?} as a render target",
                    r
                )
            }
        }
    }

    /// Set the custom data behind a data handle.
    ///
    /// # Panics
    ///
    /// If get_data was called in the same renderpass, calling this will panic.
    pub fn set_data<T: 'static>(&self, dep: DeclaredDependency<DataHandle<T>>, data: Option<T>) {
        *self
            .data
            .get(dep.handle.idx)
            .expect("internal rendergraph error: failed to get buffer")
            .downcast_ref::<RefCell<Option<T>>>()
            .expect("internal rendergraph error: downcasting failed")
            .try_borrow_mut()
            .expect("tried to call set_data on a handle that has an outstanding borrow through get_data") = data
    }

    /// Gets the custom data behind a data handle. If it has not been set, or
    /// set to None, this will return None.
    pub fn get_data<T: 'static>(
        &self,
        temps: &'a RpassTemporaryPool<'a>,
        dep: DeclaredDependency<DataHandle<T>>,
    ) -> Option<&'a T> {
        let borrow = self
            .data
            .get(dep.handle.idx)
            .expect("internal rendergraph error: failed to get buffer")
            .downcast_ref::<RefCell<Option<T>>>()
            .expect("internal rendergraph error: downcasting failed")
            .try_borrow()
            .expect("internal rendergraph error: read-only borrow failed");
        match *borrow {
            Some(_) => {
                let r = temps.add(borrow);
                Some(r.as_ref().unwrap())
            }
            None => None,
        }
    }
}
