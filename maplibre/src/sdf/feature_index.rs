#[derive(Clone)]
pub struct IndexedSubfeature {
    pub ref_: RefIndexedSubfeature,
    pub sourceLayerNameCopy: String,
    pub bucketLeaderIDCopy: String,
}

impl IndexedSubfeature {
    pub fn new(indexedFeature: IndexedSubfeature, bucketInstanceId: u32, collisionGroupId: u16) -> IndexedSubfeature {
        IndexedSubfeature {
            ref_: RefIndexedSubfeature {
                index: indexedFeature.ref_.index,
                sortIndex: indexedFeature.ref_.sortIndex,
                sourceLayerName: indexedFeature.ref_.sourceLayerName.to_string(),
                bucketLeaderID: indexedFeature.ref_.bucketLeaderID.to_string(),
                bucketInstanceId,
                collisionGroupId,
            },
            sourceLayerNameCopy: indexedFeature.ref_.sourceLayerName.to_string(),
            bucketLeaderIDCopy: indexedFeature.ref_.bucketLeaderID.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct RefIndexedSubfeature {
    pub index: usize,
    pub sortIndex: usize,

    pub sourceLayerName: String,
    pub bucketLeaderID: String,

    // Only used for symbol features
    pub bucketInstanceId: u32,
    pub collisionGroupId: u16,
}
