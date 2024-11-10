//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/quads.cpp

use std::f64::consts::PI;

use crate::{
    euclid::{Point2D, Rect, Size2D, Vector2D},
    legacy::{
        glyph::{Shaping, WritingModeType},
        image::{ImageMap, ImageStretches},
        image_atlas::ImagePosition,
        layout::symbol_instance::SymbolContent,
        shaping::PositionedIcon,
        style_types::{
            AlignmentType, SymbolLayoutProperties_Evaluated, SymbolPlacementType, TextRotate,
            TextRotationAlignment,
        },
        util::{
            constants::ONE_EM,
            math::{deg2radf, rotate},
        },
        TileSpace,
    },
};

/// maplibre/maplibre-native#4add9ea original name: SymbolQuad
pub struct SymbolQuad {
    pub tl: Point2D<f64, TileSpace>,
    pub tr: Point2D<f64, TileSpace>,
    pub bl: Point2D<f64, TileSpace>,
    pub br: Point2D<f64, TileSpace>,
    pub tex: Rect<u16, TileSpace>,
    pub pixel_offset_tl: Point2D<f64, TileSpace>,
    pub pixel_offset_br: Point2D<f64, TileSpace>,
    pub glyph_offset: Point2D<f64, TileSpace>,
    pub writing_mode: WritingModeType,
    pub is_sdf: bool,
    pub section_index: usize,
    pub min_font_scale: Point2D<f64, TileSpace>,
}

/// maplibre/maplibre-native#4add9ea original name: SymbolQuads
pub type SymbolQuads = Vec<SymbolQuad>;

const BORDER: u16 = ImagePosition::PADDING;

/// maplibre/maplibre-native#4add9ea original name: computeStretchSum
fn compute_stretch_sum(stretches: &ImageStretches) -> f64 {
    let mut sum = 0.;
    for stretch in stretches {
        sum += stretch.1 - stretch.0;
    }
    sum
}

/// maplibre/maplibre-native#4add9ea original name: sumWithinRange
fn sum_within_range(stretches: &ImageStretches, min: f64, max: f64) -> f64 {
    let mut sum = 0.;
    for stretch in stretches {
        sum += min.max(max.min(stretch.1)) - min.max(max.min(stretch.0));
    }
    sum
}

/// maplibre/maplibre-native#4add9ea original name: getEmOffset
fn get_em_offset(stretch_offset: f64, stretch_size: f64, icon_size: f64, icon_offset: f64) -> f64 {
    icon_offset + icon_size * stretch_offset / stretch_size
}

/// maplibre/maplibre-native#4add9ea original name: getPxOffset
fn get_px_offset(
    fixed_offset: f64,
    fixed_size: f64,
    stretch_offset: f64,
    stretch_size: f64,
) -> f64 {
    fixed_offset - fixed_size * stretch_offset / stretch_size
}

/// maplibre/maplibre-native#4add9ea original name: Cut
struct Cut {
    fixed: f64,
    stretch: f64,
}

/// maplibre/maplibre-native#4add9ea original name: Cuts
type Cuts = Vec<Cut>;

/// maplibre/maplibre-native#4add9ea original name: stretchZonesToCuts
fn stretch_zones_to_cuts(
    stretch_zones: &ImageStretches,
    fixed_size: f64,
    stretch_size: f64,
) -> Cuts {
    let mut cuts = vec![Cut {
        fixed: -(BORDER as f64),
        stretch: 0.0,
    }];

    for zone in stretch_zones {
        let c1 = zone.0;
        let c2 = zone.1;
        let last_stretch = cuts.last().unwrap().stretch;
        cuts.push(Cut {
            fixed: c1 - last_stretch,
            stretch: last_stretch,
        });
        cuts.push(Cut {
            fixed: c1 - last_stretch,
            stretch: last_stretch + (c2 - c1),
        });
    }
    cuts.push(Cut {
        fixed: fixed_size + BORDER as f64,
        stretch: stretch_size,
    });
    cuts
}

/// maplibre/maplibre-native#4add9ea original name: matrixMultiply
fn matrix_multiply<U>(m: &[f64; 4], p: Point2D<f64, U>) -> Point2D<f64, U> {
    Point2D::<f64, U>::new(m[0] * p.x + m[1] * p.y, m[2] * p.x + m[3] * p.y)
}

/// maplibre/maplibre-native#4add9ea original name: getIconQuads
pub fn get_icon_quads(
    shaped_icon: &PositionedIcon,
    icon_rotate: f64,
    icon_type: SymbolContent,
    has_icon_text_fit: bool,
) -> SymbolQuads {
    let mut quads = Vec::new();

    let image = &shaped_icon.image;
    let pixel_ratio = image.pixel_ratio;
    let image_width: u16 = image.padded_rect.width() - 2 * BORDER;
    let image_height: u16 = image.padded_rect.height() - 2 * BORDER;

    let icon_width = shaped_icon.right - shaped_icon.left;
    let icon_height = shaped_icon.bottom - shaped_icon.top;

    let stretch_xfull: ImageStretches = vec![(0.0, image_width as f64)];
    let stretch_yfull: ImageStretches = vec![(0.0, image_height as f64)];
    let stretch_x: &ImageStretches = if !image.stretch_x.is_empty() {
        &image.stretch_x
    } else {
        &stretch_xfull
    };
    let stretch_y: &ImageStretches = if !image.stretch_y.is_empty() {
        &image.stretch_y
    } else {
        &stretch_yfull
    };

    let stretch_width = compute_stretch_sum(stretch_x);
    let stretch_height = compute_stretch_sum(stretch_y);
    let fixed_width = image_width as f64 - stretch_width;
    let fixed_height = image_height as f64 - stretch_height;

    let mut stretch_offset_x = 0.;
    let mut stretch_content_width = stretch_width;
    let mut stretch_offset_y = 0.;
    let mut stretch_content_height = stretch_height;
    let mut fixed_offset_x = 0.;
    let mut fixed_content_width = fixed_width;
    let mut fixed_offset_y = 0.;
    let mut fixed_content_height = fixed_height;

    if has_icon_text_fit {
        if let Some(content) = &image.content {
            stretch_offset_x = sum_within_range(stretch_x, 0., content.left);
            stretch_offset_y = sum_within_range(stretch_y, 0., content.top);
            stretch_content_width = sum_within_range(stretch_x, content.left, content.right);
            stretch_content_height = sum_within_range(stretch_y, content.top, content.bottom);
            fixed_offset_x = content.left - stretch_offset_x;
            fixed_offset_y = content.top - stretch_offset_y;
            fixed_content_width = content.right - content.left - stretch_content_width;
            fixed_content_height = content.bottom - content.top - stretch_content_height;
        }
    }

    let mut matrix: Option<[f64; 4]> = None;
    if icon_rotate != 0.0 {
        // TODO is this correct?
        let angle = deg2radf(icon_rotate);
        let angle_sin = angle.sin();
        let angle_cos = angle.cos();
        matrix = Some([angle_cos, -angle_sin, angle_sin, angle_cos]);
    }

    let mut make_box = |left: &Cut, top: &Cut, right: &Cut, bottom: &Cut| {
        let left_em = get_em_offset(
            left.stretch - stretch_offset_x,
            stretch_content_width,
            icon_width,
            shaped_icon.left,
        );
        let left_px = get_px_offset(
            left.fixed - fixed_offset_x,
            fixed_content_width,
            left.stretch,
            stretch_width,
        );

        let top_em = get_em_offset(
            top.stretch - stretch_offset_y,
            stretch_content_height,
            icon_height,
            shaped_icon.top,
        );
        let top_px = get_px_offset(
            top.fixed - fixed_offset_y,
            fixed_content_height,
            top.stretch,
            stretch_height,
        );

        let right_em = get_em_offset(
            right.stretch - stretch_offset_x,
            stretch_content_width,
            icon_width,
            shaped_icon.left,
        );
        let right_px = get_px_offset(
            right.fixed - fixed_offset_x,
            fixed_content_width,
            right.stretch,
            stretch_width,
        );

        let bottom_em = get_em_offset(
            bottom.stretch - stretch_offset_y,
            stretch_content_height,
            icon_height,
            shaped_icon.top,
        );
        let bottom_px = get_px_offset(
            bottom.fixed - fixed_offset_y,
            fixed_content_height,
            bottom.stretch,
            stretch_height,
        );

        let mut tl = Point2D::<f64, TileSpace>::new(left_em, top_em);
        let mut tr = Point2D::<f64, TileSpace>::new(right_em, top_em);
        let mut br = Point2D::<f64, TileSpace>::new(right_em, bottom_em);
        let mut bl = Point2D::<f64, TileSpace>::new(left_em, bottom_em);
        let pixel_offset_tl =
            Point2D::<f64, TileSpace>::new(left_px / pixel_ratio, top_px / pixel_ratio);
        let pixel_offset_br =
            Point2D::<f64, TileSpace>::new(right_px / pixel_ratio, bottom_px / pixel_ratio);

        if let Some(matrix) = matrix {
            tl = matrix_multiply(&matrix, tl);
            tr = matrix_multiply(&matrix, tr);
            bl = matrix_multiply(&matrix, bl);
            br = matrix_multiply(&matrix, br);
        }

        let x1 = left.stretch + left.fixed;
        let x2 = right.stretch + right.fixed;
        let y1 = top.stretch + top.fixed;
        let y2 = bottom.stretch + bottom.fixed;

        // TODO: consider making texture coordinates f64 instead of uint16_t
        let sub_rect: Rect<u16, TileSpace> = Rect::new(
            Point2D::new(
                (image.padded_rect.origin.x as f64 + BORDER as f64 + x1) as u16,
                (image.padded_rect.origin.y as f64 + BORDER as f64 + y1) as u16,
            ),
            Size2D::new((x2 - x1) as u16, (y2 - y1) as u16),
        );

        let min_font_scale_x = fixed_content_width / pixel_ratio / icon_width;
        let min_font_scale_y = fixed_content_height / pixel_ratio / icon_height;

        // Icon quad is padded, so texture coordinates also need to be padded.
        quads.push(SymbolQuad {
            tl,
            tr,
            bl,
            br,
            tex: sub_rect,
            pixel_offset_tl,
            pixel_offset_br,
            glyph_offset: Point2D::new(0.0, 0.0),
            writing_mode: WritingModeType::None,
            is_sdf: icon_type == SymbolContent::IconSDF,
            section_index: 0,
            min_font_scale: Point2D::new(min_font_scale_x, min_font_scale_y),
        });
    };

    if !has_icon_text_fit || (image.stretch_x.is_empty() && image.stretch_y.is_empty()) {
        make_box(
            &Cut {
                fixed: 0.,
                stretch: -1.,
            },
            &Cut {
                fixed: 0.,
                stretch: -1.,
            },
            &Cut {
                fixed: 0.,
                stretch: (image_width + 1) as f64,
            },
            &Cut {
                fixed: 0.,
                stretch: (image_height + 1) as f64,
            },
        );
    } else {
        let x_cuts = stretch_zones_to_cuts(stretch_x, fixed_width, stretch_width);
        let y_cuts = stretch_zones_to_cuts(stretch_y, fixed_height, stretch_height);

        for xi in 0..x_cuts.len() - 1 {
            let x1 = &x_cuts[xi];
            let x2 = &x_cuts[xi + 1];
            for yi in 0..y_cuts.len() - 1 {
                let y1 = &y_cuts[yi];
                let y2 = &y_cuts[yi + 1];
                make_box(x1, y1, x2, y2);
            }
        }
    }

    quads
}

/// maplibre/maplibre-native#4add9ea original name: getGlyphQuads
pub fn get_glyph_quads(
    shaped_text: &Shaping,
    text_offset: [f64; 2],
    layout: &SymbolLayoutProperties_Evaluated,
    placement: SymbolPlacementType,
    image_map: &ImageMap,
    allow_vertical_placement: bool,
) -> SymbolQuads {
    let text_rotate: f64 = deg2radf(layout.get_eval::<TextRotate>());
    let along_line: bool = layout.get::<TextRotationAlignment>() == AlignmentType::Map
        && placement != SymbolPlacementType::Point;

    let mut quads = Vec::new();

    for line in &shaped_text.positioned_lines {
        for positionedGlyph in &line.positioned_glyphs {
            if positionedGlyph.rect.is_empty() {
                continue;
            }

            // The rects have an addditional buffer that is not included in their size;
            let glyph_padding = 1.0;
            let mut rect_buffer = 3.0 + glyph_padding;
            let mut pixel_ratio = 1.0;
            let mut line_offset = 0.0;
            let rotate_vertical_glyph =
                (along_line || allow_vertical_placement) && positionedGlyph.vertical;
            let half_advance = positionedGlyph.metrics.advance as f64 * positionedGlyph.scale / 2.0;
            let rect = positionedGlyph.rect;
            let mut is_sdf = true;

            // Align images and scaled glyphs in the middle of a vertical line.
            if allow_vertical_placement && shaped_text.verticalizable {
                let scaled_glyph_offset = (positionedGlyph.scale - 1.) * ONE_EM;
                let image_offset =
                    (ONE_EM - positionedGlyph.metrics.width as f64 * positionedGlyph.scale) / 2.0;
                line_offset = line.line_offset / 2.0
                    - (if positionedGlyph.image_id.is_some() {
                        -image_offset
                    } else {
                        scaled_glyph_offset
                    });
            }

            if let Some(imageID) = &positionedGlyph.image_id {
                let image = image_map.get(imageID);
                if let Some(image) = image {
                    pixel_ratio = image.pixel_ratio;
                    rect_buffer = ImagePosition::PADDING as f64 / pixel_ratio;
                    is_sdf = image.sdf;
                }
            }

            let glyph_offset = if along_line {
                Point2D::new(positionedGlyph.x + half_advance, positionedGlyph.y)
            } else {
                Point2D::new(0.0, 0.0)
            };

            let mut built_in_offset = if along_line {
                Vector2D::new(0.0, 0.0)
            } else {
                Vector2D::new(
                    positionedGlyph.x + half_advance + text_offset[0],
                    positionedGlyph.y + text_offset[1] - line_offset,
                )
            };

            let mut verticalized_label_offset = Vector2D::<f64, TileSpace>::new(0.0, 0.0);
            if rotate_vertical_glyph {
                // Vertical POI labels, that are rotated 90deg CW and whose
                // glyphs must preserve upright orientation need to be rotated
                // 90deg CCW. After quad is rotated, it is translated to the
                // original built-in offset.
                verticalized_label_offset = built_in_offset;
                built_in_offset = Vector2D::new(0.0, 0.0);
            }

            let x1 = (positionedGlyph.metrics.left as f64 - rect_buffer) * positionedGlyph.scale
                - half_advance
                + built_in_offset.x;
            let y1 = (-positionedGlyph.metrics.top as f64 - rect_buffer) * positionedGlyph.scale
                + built_in_offset.y;
            let x2 = x1 + rect.width() as f64 * positionedGlyph.scale / pixel_ratio;
            let y2 = y1 + rect.height() as f64 * positionedGlyph.scale / pixel_ratio;

            let mut tl: Point2D<f64, TileSpace> = Point2D::new(x1, y1);
            let mut tr: Point2D<f64, TileSpace> = Point2D::new(x2, y1);
            let mut bl: Point2D<f64, TileSpace> = Point2D::new(x1, y2);
            let mut br: Point2D<f64, TileSpace> = Point2D::new(x2, y2);

            if rotate_vertical_glyph {
                // Vertical-supporting glyphs are laid out in 24x24 point boxes
                // (1 square em) In horizontal orientation, the y values for
                // glyphs are below the midline and we use a "yOffset" of -17 to
                // pull them up to the middle. By rotating counter-clockwise
                // around the point at the center of the left edge of a 24x24
                // layout box centered below the midline, we align the center of
                // the glyphs with the horizontal midline, so the yOffset is no
                // longer necessary, but we also pull the glyph to the left
                // along the x axis. The y coordinate includes baseline yOffset,
                // therefore, needs to be accounted for when glyph is rotated
                // and translated.

                let center = Point2D::new(-half_advance, half_advance - Shaping::Y_OFFSET as f64);
                let vertical_rotation = -PI / 2.;

                // xHalfWidhtOffsetcorrection is a difference between full-width
                // and half-width advance, should be 0 for full-width glyphs and
                // will pull up half-width glyphs.
                let x_half_widht_offsetcorrection = ONE_EM / 2. - half_advance;
                let y_image_offset_correction = if positionedGlyph.image_id.is_some() {
                    x_half_widht_offsetcorrection
                } else {
                    0.0
                };

                let x_offset_correction = Vector2D::<f64, TileSpace>::new(
                    5.0 - Shaping::Y_OFFSET as f64 - x_half_widht_offsetcorrection,
                    -y_image_offset_correction,
                );

                tl = center
                    + rotate(&(tl - center), vertical_rotation)
                    + x_offset_correction
                    + verticalized_label_offset;
                tr = center
                    + rotate(&(tr - center), vertical_rotation)
                    + x_offset_correction
                    + verticalized_label_offset;
                bl = center
                    + rotate(&(bl - center), vertical_rotation)
                    + x_offset_correction
                    + verticalized_label_offset;
                br = center
                    + rotate(&(br - center), vertical_rotation)
                    + x_offset_correction
                    + verticalized_label_offset;
            }

            if text_rotate != 0.0 {
                // TODO is this correct?
                // Compute the transformation matrix.
                let angle_sin = text_rotate.sin();
                let angle_cos = text_rotate.cos();
                let matrix = [angle_cos, -angle_sin, angle_sin, angle_cos];

                tl = matrix_multiply(&matrix, tl);
                tr = matrix_multiply(&matrix, tr);
                bl = matrix_multiply(&matrix, bl);
                br = matrix_multiply(&matrix, br);
            }

            let pixel_offset_tl = Point2D::default();
            let pixel_offset_br = Point2D::default();
            let min_font_scale = Point2D::default();

            quads.push(SymbolQuad {
                tl,
                tr,
                bl,
                br,
                tex: rect,
                pixel_offset_tl,
                pixel_offset_br,
                glyph_offset,
                writing_mode: shaped_text.writing_mode,
                is_sdf,
                section_index: positionedGlyph.section_index,
                min_font_scale,
            });
        }
    }

    quads
}
#[cfg(test)]
mod tests {
    use cgmath::ulps_eq;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        legacy::{
            geometry::anchor::Anchor,
            geometry_tile_data::GeometryCoordinates,
            glyph::{PositionedGlyph, PositionedLine, Shaping, WritingModeType},
            image_atlas::ImagePosition,
            layout::symbol_instance::SymbolContent,
            quads::get_icon_quads,
            shaping::PositionedIcon,
            style_types::{IconTextFitType, SymbolAnchorType, SymbolLayoutProperties_Evaluated},
        },
    };

    #[test]
    /// maplibre/maplibre-native#4add9ea original name: getIconQuads_normal
    pub fn get_icon_quads_normal() {
        let layout = SymbolLayoutProperties_Evaluated;
        let anchor = Anchor {
            point: Point2D::new(2.0, 3.0),
            angle: 0.0,
            segment: Some(0),
        };
        let image: ImagePosition = ImagePosition {
            pixel_ratio: 1.0,
            padded_rect: Rect::new(Point2D::origin(), Size2D::new(15, 11)),
            version: 0,
            stretch_x: vec![],
            stretch_y: vec![],
            content: None,
        };

        let shaped_icon =
            PositionedIcon::shape_icon(image.clone(), &[-6.5, -4.5], SymbolAnchorType::Center);

        let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

        assert_eq!(quads.len(), 1);
        let quad = &quads[0];
        ulps_eq!(quad.tl.x, -14.);
        ulps_eq!(quad.tl.y, -10.);
        ulps_eq!(quad.tr.x, 1.);
        ulps_eq!(quad.tr.y, -10.);
        ulps_eq!(quad.bl.x, -14.);
        ulps_eq!(quad.bl.y, 1.);
        ulps_eq!(quad.br.x, 1.);
        ulps_eq!(quad.br.y, 1.);
    }

    #[test]
    /// maplibre/maplibre-native#4add9ea original name: getIconQuads_style
    pub fn get_icon_quads_style() {
        let anchor = Anchor {
            point: Point2D::new(0.0, 0.0),
            angle: 0.0,
            segment: Some(0),
        };

        let image: ImagePosition = ImagePosition {
            pixel_ratio: 1.0,
            padded_rect: Rect::new(Point2D::origin(), Size2D::new(20, 20)),
            version: 0,
            stretch_x: vec![],
            stretch_y: vec![],
            content: None,
        };

        let line = GeometryCoordinates::default();
        let mut shaped_text: Shaping = Shaping {
            top: -10.,
            bottom: 30.0,
            left: -60.,
            right: 20.0,

            positioned_lines: vec![],
            writing_mode: WritingModeType::None,
            verticalizable: false,
            icons_in_text: false,
        };

        // shapedText.positionedGlyphs.emplace_back(PositionedGlyph(32, 0.0, 0.0, false, 0, 1.0));

        shaped_text.positioned_lines.push(PositionedLine::default());
        shaped_text
            .positioned_lines
            .last_mut()
            .unwrap()
            .positioned_glyphs
            .push(PositionedGlyph {
                glyph: 32,
                x: 0.0,
                y: 0.0,
                vertical: false,
                font: 0,
                scale: 0.0,
                rect: Default::default(),
                metrics: Default::default(),
                image_id: None,
                section_index: 0,
            });

        // none
        {
            let shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);

            ulps_eq!(-18.5, shaped_icon.top);
            ulps_eq!(-0.5, shaped_icon.right);
            ulps_eq!(-0.5, shaped_icon.bottom);
            ulps_eq!(-18.5, shaped_icon.left);

            let layout = SymbolLayoutProperties_Evaluated;
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -19.5);
            ulps_eq!(quad.tl.y, -19.5);
            ulps_eq!(quad.tr.x, 0.5);
            ulps_eq!(quad.tr.y, -19.5);
            ulps_eq!(quad.bl.x, -19.5);
            ulps_eq!(quad.bl.y, 0.5);
            ulps_eq!(quad.br.x, 0.5);
            ulps_eq!(quad.br.y, 0.5);
        }

        // width
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Width,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -64.4444427);
            ulps_eq!(quad.tl.y, 0.);
            ulps_eq!(quad.tr.x, 24.4444427);
            ulps_eq!(quad.tr.y, 0.);
            ulps_eq!(quad.bl.x, -64.4444427);
            ulps_eq!(quad.bl.y, 20.);
            ulps_eq!(quad.br.x, 24.4444427);
            ulps_eq!(quad.br.y, 20.);
        }

        // width x textSize
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Width,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -32.2222214);
            ulps_eq!(quad.tl.y, -5.);
            ulps_eq!(quad.tr.x, 12.2222214);
            ulps_eq!(quad.tr.y, -5.);
            ulps_eq!(quad.bl.x, -32.2222214);
            ulps_eq!(quad.bl.y, 15.);
            ulps_eq!(quad.br.x, 12.2222214);
            ulps_eq!(quad.br.y, 15.);
        }

        // width x textSize + padding
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Width,
                &[5., 10., 5., 10.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -43.3333321);
            ulps_eq!(quad.tl.y, -5.);
            ulps_eq!(quad.tr.x, 23.3333321);
            ulps_eq!(quad.tr.y, -5.);
            ulps_eq!(quad.bl.x, -43.3333321);
            ulps_eq!(quad.bl.y, 15.);
            ulps_eq!(quad.br.x, 23.3333321);
            ulps_eq!(quad.br.y, 15.);
        }

        // height
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Height,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -30.);
            ulps_eq!(quad.tl.y, -12.2222214);
            ulps_eq!(quad.tr.x, -10.);
            ulps_eq!(quad.tr.y, -12.2222214);
            ulps_eq!(quad.bl.x, -30.);
            ulps_eq!(quad.bl.y, 32.2222214);
            ulps_eq!(quad.br.x, -10.);
            ulps_eq!(quad.br.y, 32.2222214);
        }

        // height x textSize
        {
            let layout = SymbolLayoutProperties_Evaluated;
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Height,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -20.);
            ulps_eq!(quad.tl.y, -6.11111069);
            ulps_eq!(quad.tr.x, 0.);
            ulps_eq!(quad.tr.y, -6.11111069);
            ulps_eq!(quad.bl.x, -20.);
            ulps_eq!(quad.bl.y, 16.1111107);
            ulps_eq!(quad.br.x, 0.);
            ulps_eq!(quad.br.y, 16.1111107);
        }

        // height x textSize + padding
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Height,
                &[5., 10., 5., 20.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -20.);
            ulps_eq!(quad.tl.y, -11.666666);
            ulps_eq!(quad.tr.x, 0.);
            ulps_eq!(quad.tr.y, -11.666666);
            ulps_eq!(quad.bl.x, -20.);
            ulps_eq!(quad.bl.y, 21.666666);
            ulps_eq!(quad.br.x, 0.);
            ulps_eq!(quad.br.y, 21.666666);
        }

        // both
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Both,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -64.4444427);
            ulps_eq!(quad.tl.y, -12.2222214);
            ulps_eq!(quad.tr.x, 24.4444427);
            ulps_eq!(quad.tr.y, -12.2222214);
            ulps_eq!(quad.bl.x, -64.4444427);
            ulps_eq!(quad.bl.y, 32.2222214);
            ulps_eq!(quad.br.x, 24.4444427);
            ulps_eq!(quad.br.y, 32.2222214);
        }

        // both x textSize
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Both,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -32.2222214);
            ulps_eq!(quad.tl.y, -6.11111069);
            ulps_eq!(quad.tr.x, 12.2222214);
            ulps_eq!(quad.tr.y, -6.11111069);
            ulps_eq!(quad.bl.x, -32.2222214);
            ulps_eq!(quad.bl.y, 16.1111107);
            ulps_eq!(quad.br.x, 12.2222214);
            ulps_eq!(quad.br.y, 16.1111107);
        }

        // both x textSize + padding
        {
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Both,
                &[5., 10., 5., 10.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -43.3333321);
            ulps_eq!(quad.tl.y, -11.666666);
            ulps_eq!(quad.tr.x, 23.3333321);
            ulps_eq!(quad.tr.y, -11.666666);
            ulps_eq!(quad.bl.x, -43.3333321);
            ulps_eq!(quad.bl.y, 21.666666);
            ulps_eq!(quad.br.x, 23.3333321);
            ulps_eq!(quad.br.y, 21.666666);
        }

        // both x textSize + padding t/r/b/l
        {
            let layout = SymbolLayoutProperties_Evaluated;
            // FIXME add layout.get::<TextSize>() = 12.0; this test also works without this, which makes sense because text size does not affect glyph quads
            let mut shaped_icon =
                PositionedIcon::shape_icon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shaped_icon.fit_icon_to_text(
                &shaped_text,
                IconTextFitType::Both,
                &[0., 5., 10., 15.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = get_icon_quads(&shaped_icon, 0., SymbolContent::IconRGBA, false);

            assert_eq!(quads.len(), 1);
            let quad = &quads[0];

            ulps_eq!(quad.tl.x, -48.3333321);
            ulps_eq!(quad.tl.y, -6.66666603);
            ulps_eq!(quad.tr.x, 18.3333321);
            ulps_eq!(quad.tr.y, -6.66666603);
            ulps_eq!(quad.bl.x, -48.3333321);
            ulps_eq!(quad.bl.y, 26.666666);
            ulps_eq!(quad.br.x, 18.3333321);
            ulps_eq!(quad.br.y, 26.666666);
        }
    }
}
