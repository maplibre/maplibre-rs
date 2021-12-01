use tile_grid::{extent_wgs84_to_merc, Extent, Grid, GridIterator, Origin, Unit};

fn web_mercator() -> Grid {
    Grid::new(
        256,
        256,
        Extent {
            minx: -20037508.3427892480,
            miny: -20037508.3427892480,
            maxx: 20037508.3427892480,
            maxy: 20037508.3427892480,
        },
        3857,
        Unit::Meters,
        // for calculation see fn test_resolutions
        vec![
            156543.0339280410,
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

/// z, x, z
pub fn get_tile_coordinates_bavaria() -> Vec<(u8, u32, u32)> {
    let grid = web_mercator();
    let tile_limits = grid.tile_limits(
        extent_wgs84_to_merc(&Extent {
            minx: 10.0,
            miny: 48.0,
            maxx: 12.0,
            maxy: 50.0,
        }),
        0,
    );

    println!("{:?}", grid.tile_extent(0, 0, 0));
    println!("{:?}", grid.tile_extent(33, 21, 6));
    println!("{:?}", grid.tile_extent_xyz(0, 0, 0));

    let z = 6;
    let griditer = GridIterator::new(z, z, tile_limits);
    griditer.collect()
}

pub fn get_tile_coordinates_tutzing() -> Vec<(u8, u32, u32)> {
    let grid = web_mercator();
    let tile_limits = grid.tile_limits(
        extent_wgs84_to_merc(&Extent {
            minx: 11.2772666,
            miny: 47.9125117,
            maxx: 11.2772666,
            maxy: 47.9125117,
        }),
        1,
    );

    println!("{:?}", grid.tile_extent(0, 0, 0));
    println!("{:?}", grid.tile_extent(33, 21, 6));
    println!("{:?}", grid.tile_extent_xyz(0, 0, 0));

    let z = 12;
    let griditer = GridIterator::new(z, z, tile_limits);
    griditer.collect()
}
