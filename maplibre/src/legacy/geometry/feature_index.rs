//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/geometry/feature_index.cpp

/// maplibre/maplibre-native#4add9ea original name: IndexedSubfeature
#[derive(Clone)]
pub struct IndexedSubfeature {
    pub ref_: RefIndexedSubfeature,
    pub source_layer_name_copy: String,
    pub bucket_leader_idcopy: String,
}

impl IndexedSubfeature {
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(
        indexed_feature: IndexedSubfeature,
        bucket_instance_id: u32,
        collision_group_id: u16,
    ) -> IndexedSubfeature {
        IndexedSubfeature {
            ref_: RefIndexedSubfeature {
                index: indexed_feature.ref_.index,
                sort_index: indexed_feature.ref_.sort_index,
                source_layer_name: indexed_feature.ref_.source_layer_name.to_string(),
                bucket_leader_id: indexed_feature.ref_.bucket_leader_id.to_string(),
                bucket_instance_id,
                collision_group_id,
            },
            source_layer_name_copy: indexed_feature.ref_.source_layer_name.to_string(),
            bucket_leader_idcopy: indexed_feature.ref_.bucket_leader_id.to_string(),
        }
    }
}

/// maplibre/maplibre-native#4add9ea original name: RefIndexedSubfeature
#[derive(Clone)]
pub struct RefIndexedSubfeature {
    pub index: usize,
    pub sort_index: usize,

    pub source_layer_name: String,
    pub bucket_leader_id: String,

    // Only used for symbol features
    pub bucket_instance_id: u32,
    pub collision_group_id: u16,
}
