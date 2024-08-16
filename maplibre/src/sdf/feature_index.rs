#[derive(Clone)]
pub struct IndexedSubfeature {
    pub ref_: RefIndexedSubfeature,
    pub sourceLayerNameCopy: String,
    pub bucketLeaderIDCopy: String
}

#[derive(Clone)]
pub struct RefIndexedSubfeature {
    pub index: usize,
    pub sortIndex: usize,

    pub sourceLayerName: String,
    pub bucketLeaderID: String,

    // Only used for symbol features
    pub bucketInstanceId: u32,
    pub collisionGroupId: u16
}