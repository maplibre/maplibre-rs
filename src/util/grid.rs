use tile_grid::{extent_wgs84_to_merc, Extent, Grid, GridIterator, Origin, Unit};

pub fn google_mercator() -> Grid {
    Grid::new(
        256,
        256,
        Extent {
            minx: -20037508.342789248,
            miny: -20037508.342789248,
            maxx: 20037508.342789248,
            maxy: 20037508.342789248,
        },
        3857,
        Unit::Meters,
        vec![
            156543.033928041,
            78271.5169640205,
            39135.75848201025,
            19567.879241005125,
            9783.939620502562,
            4891.969810251281,
            2445.9849051256406,
            1222.9924525628203,
            611.4962262814101,
            305.7481131407051,
            152.87405657035254,
            76.43702828517627,
            38.218514142588134,
            19.109257071294067,
            9.554628535647034,
            4.777314267823517,
            2.3886571339117584,
            1.1943285669558792,
            0.5971642834779396,
            0.2985821417389698,
            0.1492910708694849,
            0.07464553543474245,
            0.037322767717371225,
        ],
        Origin::TopLeft,
    )
}

///
/// Returns coordinates for tiles within bavaria according to the specified grid.
/// The grid is responsible for defining the coordinate system. For example whether
/// [Slippy map tilenames](https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames) (also known as
/// XYZ) or [TMS](https://wiki.osgeo.org/wiki/Tile_Map_Service_Specification#TileMap_Diagram) is
/// used.
///
/// ## Additional Resources:
///
/// * https://www.maptiler.com/google-maps-coordinates-tile-bounds-projection
/// * https://gist.github.com/maptiler/fddb5ce33ba995d5523de9afdf8ef118
pub fn tile_coordinates_bavaria(grid: &Grid, zoom: u8) -> Vec<(u8, u32, u32)> {
    let tile_limits = grid.tile_limits(
        extent_wgs84_to_merc(&Extent {
            minx: 8.9771580802,
            miny: 47.2703623267,
            maxx: 13.8350427083,
            maxy: 50.5644529365,
        }),
        0,
    );

    GridIterator::new(zoom, zoom, tile_limits).collect()
}
