use mvt::{Error, GeomEncoder, GeomType, Tile};
use pointy::Transform;

fn main() -> Result<(), Error> {
    let mut tile = Tile::new(4096);
    let layer = tile.create_layer("First Layer");
    // NOTE: normally, the Transform would come from MapGrid::tile_transform
    let b = GeomEncoder::new(GeomType::Linestring, Transform::default())
        .point(0.0, 0.0)?
        .point(1024.0, 0.0)?
        .point(1024.0, 2048.0)?
        .point(2048.0, 2048.0)?
        .point(2048.0, 4096.0)?
        .encode()?;
    let mut feature = layer.into_feature(b);
    feature.set_id(1);
    feature.add_tag_string("key", "value");
    let layer = feature.into_layer();
    tile.add_layer(layer)?;
    let data = tile.to_bytes()?;
    println!("encoded {} bytes: {:?}", data.len(), data);
    Ok(())
}
