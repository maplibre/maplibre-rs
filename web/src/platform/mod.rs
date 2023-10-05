use crate::WHATWGOffscreenKernelEnvironment;

pub mod http_client;

#[cfg(target_feature = "atomics")]
pub mod multithreaded;

#[cfg(not(target_feature = "atomics"))]
pub mod singlethreaded;

#[cfg(target_feature = "atomics")]
pub type UsedRasterTransferables = maplibre::raster::DefaultRasterTransferables;
#[cfg(not(target_feature = "atomics"))]
pub type UsedRasterTransferables = singlethreaded::transferables::FlatTransferables;

#[cfg(target_feature = "atomics")]
pub type UsedVectorTransferables = maplibre::vector::DefaultVectorTransferables;
#[cfg(not(target_feature = "atomics"))]
pub type UsedVectorTransferables = singlethreaded::transferables::FlatTransferables;

pub type UsedOffscreenKernelEnvironment = WHATWGOffscreenKernelEnvironment;
