use std::collections::BTreeSet;

use maplibre::coords::WorldTileCoords;

pub(crate) fn source_tile_coords(
    required: &[WorldTileCoords],
    min_zoom: Option<u8>,
    max_zoom: Option<u8>,
) -> BTreeSet<WorldTileCoords> {
    let mut pending = required.to_vec();
    let mut selected = BTreeSet::new();

    while let Some(mut coords) = pending.pop() {
        while max_zoom.is_some_and(|zoom| u8::from(coords.z) > zoom) {
            let Some(parent) = coords.get_parent() else {
                break;
            };
            coords = parent;
        }

        if min_zoom.is_some_and(|zoom| u8::from(coords.z) < zoom) {
            pending.extend(coords.get_children());
        } else {
            selected.insert(coords);
        }
    }

    selected
}

#[cfg(test)]
mod tests {
    use maplibre::coords::{WorldTileCoords, ZoomLevel};

    use super::source_tile_coords;

    #[test]
    fn expands_visible_parent_to_source_minimum_zoom() {
        let root = WorldTileCoords::default();
        let selected = source_tile_coords(&[root], Some(1), Some(1));

        assert_eq!(selected, root.get_children().into_iter().collect());
    }

    #[test]
    fn clamps_visible_children_to_source_maximum_zoom() {
        let child = WorldTileCoords {
            x: 3,
            y: 2,
            z: ZoomLevel::new(2),
        };
        let selected = source_tile_coords(&[child], None, Some(1));

        assert_eq!(
            selected,
            [child.get_parent().expect("z2 has a parent")].into()
        );
    }
}
