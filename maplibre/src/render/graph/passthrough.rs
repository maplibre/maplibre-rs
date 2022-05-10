use std::{
    marker::PhantomData,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Reference to data that is passed through.
pub struct PassthroughDataRef<T> {
    node_id: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

/// Mutable reference to data that is passed through.
pub struct PassthroughDataRefMut<T> {
    node_id: usize,
    index: usize,
    _phantom: PhantomData<T>,
}

/// Container which promotes data from &'node outside a node to &'rpass inside a
/// node.
pub struct PassthroughDataContainer<'node> {
    node_id: usize,
    data: Vec<Option<*const ()>>,
    _phantom: PhantomData<&'node ()>,
}

impl<'node> PassthroughDataContainer<'node> {
    pub(super) fn new() -> Self {
        static NODE_ID: AtomicUsize = AtomicUsize::new(0);
        Self {
            node_id: NODE_ID.fetch_add(1, Ordering::Relaxed),
            data: Vec::new(),
            _phantom: PhantomData,
        }
    }

    pub(super) fn add_ref<T: 'node>(&mut self, data: &'node T) -> PassthroughDataRef<T> {
        let index = self.data.len();
        self.data.push(Some(<*const _>::cast(data)));
        PassthroughDataRef {
            node_id: self.node_id,
            index,
            _phantom: PhantomData,
        }
    }

    pub(super) fn add_ref_mut<T: 'node>(&mut self, data: &'node mut T) -> PassthroughDataRefMut<T> {
        let index = self.data.len();
        self.data.push(Some(<*const _>::cast(data)));
        PassthroughDataRefMut {
            node_id: self.node_id,
            index,
            _phantom: PhantomData,
        }
    }
}

impl<'rpass> PassthroughDataContainer<'rpass> {
    /// Gets a piece of immutable data passed through from outside the node.
    ///
    /// Use [RenderGraphNodeBuilder::passthrough_ref][pt] to add data on the
    /// outside.
    ///
    /// [pt]: super::RenderGraphNodeBuilder::passthrough_ref
    pub fn get<T>(&mut self, handle: PassthroughDataRef<T>) -> &'rpass T {
        assert_eq!(
            handle.node_id, self.node_id,
            "Trying to use a passthrough data reference from another node"
        );
        unsafe {
            &*(self
                .data
                .get_mut(handle.index)
                .expect("internal rendergraph error: passthrough data handle corresponds to no passthrough data")
                .take()
                .expect("tried to retreve passthrough data more than once") as *const T)
        }
    }

    /// Gets a piece of mutable data passed through from outside the node.
    ///
    /// Use [RenderGraphNodeBuilder::passthrough_ref_mut][pt] to add data on the
    /// outside.
    ///
    /// [pt]: super::RenderGraphNodeBuilder::passthrough_ref_mut
    pub fn get_mut<T>(&mut self, handle: PassthroughDataRefMut<T>) -> &'rpass mut T {
        assert_eq!(
            handle.node_id, self.node_id,
            "Trying to use a passthrough data reference from another node"
        );
        unsafe {
            &mut *(self
                .data
                .get_mut(handle.index)
                .expect("internal rendergraph error: passthrough data handle corresponds to no passthrough data")
                .take()
                .expect("tried to retreve passthrough data more than once") as *const T as *mut T)
        }
    }
}
