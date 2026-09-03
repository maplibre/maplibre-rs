use geozero::GeomProcessor;

use super::{IndexDataType, ZeroTessellator};

#[test]
fn globe_line_tessellation_inserts_grid_crossings() {
    let mut mercator = ZeroTessellator::<IndexDataType>::default();
    tessellate_reference_line(&mut mercator);
    let mut globe =
        ZeroTessellator::<IndexDataType>::default().with_globe_subdivision(4, false, false, false);
    tessellate_reference_line(&mut globe);

    assert!(globe.buffer.vertices.len() > mercator.buffer.vertices.len());
    assert!(globe.buffer.indices.len() > mercator.buffer.indices.len());
}

#[test]
fn globe_fill_tessellation_cuts_triangle_interior() {
    let mut mercator = ZeroTessellator::<IndexDataType>::default();
    tessellate_reference_triangle(&mut mercator);
    let mut globe =
        ZeroTessellator::<IndexDataType>::default().with_globe_subdivision(2, false, false, false);
    tessellate_reference_triangle(&mut globe);

    assert!(globe.buffer.indices.len() > mercator.buffer.indices.len());
    assert!(globe
        .buffer
        .vertices
        .iter()
        .any(|vertex| vertex.position == [2048.0, 0.0]));
}

fn tessellate_reference_line(tessellator: &mut ZeroTessellator<IndexDataType>) {
    tessellator
        .linestring_begin(true, 2, 0)
        .expect("line should begin");
    tessellator
        .xy(0.0, 0.0, 0)
        .expect("line start should be valid");
    tessellator
        .xy(4096.0, 0.0, 1)
        .expect("line end should be valid");
    tessellator
        .linestring_end(true, 0)
        .expect("line should tessellate");
}

fn tessellate_reference_triangle(tessellator: &mut ZeroTessellator<IndexDataType>) {
    tessellator
        .polygon_begin(true, 1, 0)
        .expect("polygon should begin");
    tessellator
        .linestring_begin(false, 4, 0)
        .expect("ring should begin");
    for (index, point) in [[0.0, 0.0], [4096.0, 0.0], [0.0, 4096.0], [0.0, 0.0]]
        .into_iter()
        .enumerate()
    {
        tessellator
            .xy(point[0], point[1], index)
            .expect("ring coordinate should be valid");
    }
    tessellator
        .linestring_end(false, 0)
        .expect("ring should end");
    tessellator
        .polygon_end(true, 0)
        .expect("polygon should tessellate");
}
