use downcast_rs::{impl_downcast, Downcast};

/// A component is data associated with an [`Entity`](crate::ecs::entity::Entity). Each entity can have
/// multiple different types of components, but only one of them per type.
pub trait TileComponent: Downcast + 'static {}
impl_downcast!(TileComponent);
