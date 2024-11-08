use std::f64::consts::PI;

use crate::{
    euclid::{Point2D, Rect, Size2D, Vector2D},
    sdf::{
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

pub struct SymbolQuad {
    pub tl: Point2D<f64, TileSpace>,
    pub tr: Point2D<f64, TileSpace>,
    pub bl: Point2D<f64, TileSpace>,
    pub br: Point2D<f64, TileSpace>,
    pub tex: Rect<u16, TileSpace>,
    pub pixelOffsetTL: Point2D<f64, TileSpace>,
    pub pixelOffsetBR: Point2D<f64, TileSpace>,
    pub glyphOffset: Point2D<f64, TileSpace>,
    pub writingMode: WritingModeType,
    pub isSDF: bool,
    pub sectionIndex: usize,
    pub minFontScale: Point2D<f64, TileSpace>,
}

pub type SymbolQuads = Vec<SymbolQuad>;

const border: u16 = ImagePosition::padding;

fn computeStretchSum(stretches: &ImageStretches) -> f64 {
    let mut sum = 0.;
    for stretch in stretches {
        sum += stretch.1 - stretch.0;
    }
    return sum;
}

fn sumWithinRange(stretches: &ImageStretches, min: f64, max: f64) -> f64 {
    let mut sum = 0.;
    for stretch in stretches {
        sum += min.max(max.min(stretch.1)) - min.max(max.min(stretch.0));
    }
    return sum;
}

fn getEmOffset(stretchOffset: f64, stretchSize: f64, iconSize: f64, iconOffset: f64) -> f64 {
    return iconOffset + iconSize * stretchOffset / stretchSize;
}

fn getPxOffset(fixedOffset: f64, fixedSize: f64, stretchOffset: f64, stretchSize: f64) -> f64 {
    return fixedOffset - fixedSize * stretchOffset / stretchSize;
}

struct Cut {
    fixed: f64,
    stretch: f64,
}

type Cuts = Vec<Cut>;

fn stretchZonesToCuts(stretchZones: &ImageStretches, fixedSize: f64, stretchSize: f64) -> Cuts {
    let mut cuts = vec![Cut {
        fixed: -(border as f64),
        stretch: 0.0,
    }];

    for zone in stretchZones {
        let c1 = zone.0;
        let c2 = zone.1;
        let lastStretch = cuts.last().unwrap().stretch;
        cuts.push(Cut {
            fixed: c1 - lastStretch,
            stretch: lastStretch,
        });
        cuts.push(Cut {
            fixed: c1 - lastStretch,
            stretch: lastStretch + (c2 - c1),
        });
    }
    cuts.push(Cut {
        fixed: fixedSize + border as f64,
        stretch: stretchSize,
    });
    return cuts;
}

fn matrixMultiply<U>(m: &[f64; 4], p: Point2D<f64, U>) -> Point2D<f64, U> {
    return Point2D::<f64, U>::new(m[0] * p.x + m[1] * p.y, m[2] * p.x + m[3] * p.y);
}

pub fn getIconQuads(
    shapedIcon: &PositionedIcon,
    iconRotate: f64,
    iconType: SymbolContent,
    hasIconTextFit: bool,
) -> SymbolQuads {
    let mut quads = Vec::new();

    let image = &shapedIcon.image;
    let pixelRatio = image.pixelRatio;
    let imageWidth: u16 = image.paddedRect.width() - 2 * border;
    let imageHeight: u16 = image.paddedRect.height() - 2 * border;

    let iconWidth = shapedIcon.right - shapedIcon.left;
    let iconHeight = shapedIcon.bottom - shapedIcon.top;

    let stretchXFull: ImageStretches = vec![(0.0, imageWidth as f64)];
    let stretchYFull: ImageStretches = vec![(0.0, imageHeight as f64)];
    let stretchX: &ImageStretches = if !image.stretchX.is_empty() {
        &image.stretchX
    } else {
        &stretchXFull
    };
    let stretchY: &ImageStretches = if !image.stretchY.is_empty() {
        &image.stretchY
    } else {
        &stretchYFull
    };

    let stretchWidth = computeStretchSum(stretchX);
    let stretchHeight = computeStretchSum(stretchY);
    let fixedWidth = imageWidth as f64 - stretchWidth;
    let fixedHeight = imageHeight as f64 - stretchHeight;

    let mut stretchOffsetX = 0.;
    let mut stretchContentWidth = stretchWidth;
    let mut stretchOffsetY = 0.;
    let mut stretchContentHeight = stretchHeight;
    let mut fixedOffsetX = 0.;
    let mut fixedContentWidth = fixedWidth;
    let mut fixedOffsetY = 0.;
    let mut fixedContentHeight = fixedHeight;

    if (hasIconTextFit) {
        if let Some(content) = &image.content {
            stretchOffsetX = sumWithinRange(stretchX, 0., content.left);
            stretchOffsetY = sumWithinRange(stretchY, 0., content.top);
            stretchContentWidth = sumWithinRange(stretchX, content.left, content.right);
            stretchContentHeight = sumWithinRange(stretchY, content.top, content.bottom);
            fixedOffsetX = content.left - stretchOffsetX;
            fixedOffsetY = content.top - stretchOffsetY;
            fixedContentWidth = content.right - content.left - stretchContentWidth;
            fixedContentHeight = content.bottom - content.top - stretchContentHeight;
        }
    }

    let mut matrix: Option<[f64; 4]> = None;
    if (iconRotate != 0.0) {
        // TODO is this correct?
        let angle = deg2radf(iconRotate);
        let angle_sin = (angle.sin());
        let angle_cos = (angle.cos());
        matrix = Some([angle_cos, -angle_sin, angle_sin, angle_cos]);
    }

    let mut makeBox = |left: &Cut, top: &Cut, right: &Cut, bottom: &Cut| {
        let leftEm = getEmOffset(
            left.stretch - stretchOffsetX,
            stretchContentWidth,
            iconWidth,
            shapedIcon.left,
        );
        let leftPx = getPxOffset(
            left.fixed - fixedOffsetX,
            fixedContentWidth,
            left.stretch,
            stretchWidth,
        );

        let topEm = getEmOffset(
            top.stretch - stretchOffsetY,
            stretchContentHeight,
            iconHeight,
            shapedIcon.top,
        );
        let topPx = getPxOffset(
            top.fixed - fixedOffsetY,
            fixedContentHeight,
            top.stretch,
            stretchHeight,
        );

        let rightEm = getEmOffset(
            right.stretch - stretchOffsetX,
            stretchContentWidth,
            iconWidth,
            shapedIcon.left,
        );
        let rightPx = getPxOffset(
            right.fixed - fixedOffsetX,
            fixedContentWidth,
            right.stretch,
            stretchWidth,
        );

        let bottomEm = getEmOffset(
            bottom.stretch - stretchOffsetY,
            stretchContentHeight,
            iconHeight,
            shapedIcon.top,
        );
        let bottomPx = getPxOffset(
            bottom.fixed - fixedOffsetY,
            fixedContentHeight,
            bottom.stretch,
            stretchHeight,
        );

        let mut tl = Point2D::<f64, TileSpace>::new(leftEm, topEm);
        let mut tr = Point2D::<f64, TileSpace>::new(rightEm, topEm);
        let mut br = Point2D::<f64, TileSpace>::new(rightEm, bottomEm);
        let mut bl = Point2D::<f64, TileSpace>::new(leftEm, bottomEm);
        let pixelOffsetTL = Point2D::<f64, TileSpace>::new(leftPx / pixelRatio, topPx / pixelRatio);
        let pixelOffsetBR =
            Point2D::<f64, TileSpace>::new(rightPx / pixelRatio, bottomPx / pixelRatio);

        if let Some(matrix) = matrix {
            tl = matrixMultiply(&matrix, tl);
            tr = matrixMultiply(&matrix, tr);
            bl = matrixMultiply(&matrix, bl);
            br = matrixMultiply(&matrix, br);
        }

        let x1 = left.stretch + left.fixed;
        let x2 = right.stretch + right.fixed;
        let y1 = top.stretch + top.fixed;
        let y2 = bottom.stretch + bottom.fixed;

        // TODO: consider making texture coordinates f64 instead of uint16_t
        let subRect: Rect<u16, TileSpace> = Rect::new(
            Point2D::new(
                (image.paddedRect.origin.x as f64 + border as f64 + x1) as u16,
                (image.paddedRect.origin.y as f64 + border as f64 + y1) as u16,
            ),
            Size2D::new((x2 - x1) as u16, (y2 - y1) as u16),
        );

        let minFontScaleX = fixedContentWidth / pixelRatio / iconWidth;
        let minFontScaleY = fixedContentHeight / pixelRatio / iconHeight;

        // Icon quad is padded, so texture coordinates also need to be padded.
        quads.push(SymbolQuad {
            tl,
            tr,
            bl,
            br,
            tex: subRect,
            pixelOffsetTL,
            pixelOffsetBR,
            glyphOffset: Point2D::new(0.0, 0.0),
            writingMode: WritingModeType::None,
            isSDF: iconType == SymbolContent::IconSDF,
            sectionIndex: 0,
            minFontScale: Point2D::new(minFontScaleX, minFontScaleY),
        });
    };

    if (!hasIconTextFit || (image.stretchX.is_empty() && image.stretchY.is_empty())) {
        makeBox(
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
                stretch: (imageWidth + 1) as f64,
            },
            &Cut {
                fixed: 0.,
                stretch: (imageHeight + 1) as f64,
            },
        );
    } else {
        let xCuts = stretchZonesToCuts(stretchX, fixedWidth, stretchWidth);
        let yCuts = stretchZonesToCuts(stretchY, fixedHeight, stretchHeight);

        for xi in 0..xCuts.len() - 1 {
            let x1 = &xCuts[xi];
            let x2 = &xCuts[xi + 1];
            for yi in 0..yCuts.len() - 1 {
                let y1 = &yCuts[yi];
                let y2 = &yCuts[yi + 1];
                makeBox(x1, y1, x2, y2);
            }
        }
    }

    return quads;
}

pub fn getGlyphQuads(
    shapedText: &Shaping,
    textOffset: [f64; 2],
    layout: &SymbolLayoutProperties_Evaluated,
    placement: SymbolPlacementType,
    imageMap: &ImageMap,
    allowVerticalPlacement: bool,
) -> SymbolQuads {
    let textRotate: f64 = deg2radf(layout.get_eval::<TextRotate>());
    let alongLine: bool = layout.get::<TextRotationAlignment>() == AlignmentType::Map
        && placement != SymbolPlacementType::Point;

    let mut quads = Vec::new();

    for line in &shapedText.positionedLines {
        for positionedGlyph in &line.positionedGlyphs {
            if (positionedGlyph.rect.is_empty()) {
                continue;
            }

            // The rects have an addditional buffer that is not included in their size;
            let glyphPadding = 1.0;
            let mut rectBuffer = 3.0 + glyphPadding;
            let mut pixelRatio = 1.0;
            let mut lineOffset = 0.0;
            let rotateVerticalGlyph =
                (alongLine || allowVerticalPlacement) && positionedGlyph.vertical;
            let halfAdvance = positionedGlyph.metrics.advance as f64 * positionedGlyph.scale / 2.0;
            let rect = positionedGlyph.rect;
            let mut isSDF = true;

            // Align images and scaled glyphs in the middle of a vertical line.
            if (allowVerticalPlacement && shapedText.verticalizable) {
                let scaledGlyphOffset = (positionedGlyph.scale - 1.) * ONE_EM;
                let imageOffset =
                    (ONE_EM - positionedGlyph.metrics.width as f64 * positionedGlyph.scale) / 2.0;
                lineOffset = line.lineOffset / 2.0
                    - (if positionedGlyph.imageID.is_some() {
                        -imageOffset
                    } else {
                        scaledGlyphOffset
                    });
            }

            if let Some(imageID) = (&positionedGlyph.imageID) {
                let image = imageMap.get(imageID);
                if let Some(image) = image {
                    pixelRatio = image.pixelRatio;
                    rectBuffer = ImagePosition::padding as f64 / pixelRatio;
                    isSDF = image.sdf;
                }
            }

            let glyphOffset = if alongLine {
                Point2D::new(positionedGlyph.x + halfAdvance, positionedGlyph.y)
            } else {
                Point2D::new(0.0, 0.0)
            };

            let mut builtInOffset = if alongLine {
                Vector2D::new(0.0, 0.0)
            } else {
                Vector2D::new(
                    positionedGlyph.x + halfAdvance + textOffset[0],
                    positionedGlyph.y + textOffset[1] - lineOffset,
                )
            };

            let mut verticalizedLabelOffset = Vector2D::<f64, TileSpace>::new(0.0, 0.0);
            if (rotateVerticalGlyph) {
                // Vertical POI labels, that are rotated 90deg CW and whose
                // glyphs must preserve upright orientation need to be rotated
                // 90deg CCW. After quad is rotated, it is translated to the
                // original built-in offset.
                verticalizedLabelOffset = builtInOffset;
                builtInOffset = Vector2D::new(0.0, 0.0);
            }

            let x1 = (positionedGlyph.metrics.left as f64 - rectBuffer) * positionedGlyph.scale
                - halfAdvance
                + builtInOffset.x;
            let y1 = (-positionedGlyph.metrics.top as f64 - rectBuffer) * positionedGlyph.scale
                + builtInOffset.y;
            let x2 = x1 + rect.width() as f64 * positionedGlyph.scale / pixelRatio;
            let y2 = y1 + rect.height() as f64 * positionedGlyph.scale / pixelRatio;

            let mut tl: Point2D<f64, TileSpace> = Point2D::new(x1, y1);
            let mut tr: Point2D<f64, TileSpace> = Point2D::new(x2, y1);
            let mut bl: Point2D<f64, TileSpace> = Point2D::new(x1, y2);
            let mut br: Point2D<f64, TileSpace> = Point2D::new(x2, y2);

            if (rotateVerticalGlyph) {
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

                let center = Point2D::new(-halfAdvance, halfAdvance - Shaping::yOffset as f64);
                let verticalRotation = -PI / 2.;

                // xHalfWidhtOffsetcorrection is a difference between full-width
                // and half-width advance, should be 0 for full-width glyphs and
                // will pull up half-width glyphs.
                let xHalfWidhtOffsetcorrection = ONE_EM / 2. - halfAdvance;
                let yImageOffsetCorrection = if positionedGlyph.imageID.is_some() {
                    xHalfWidhtOffsetcorrection
                } else {
                    0.0
                };

                let xOffsetCorrection = Vector2D::<f64, TileSpace>::new(
                    5.0 - Shaping::yOffset as f64 - xHalfWidhtOffsetcorrection,
                    -yImageOffsetCorrection,
                );

                tl = center
                    + rotate(&(tl - center), verticalRotation)
                    + xOffsetCorrection
                    + verticalizedLabelOffset;
                tr = center
                    + rotate(&(tr - center), verticalRotation)
                    + xOffsetCorrection
                    + verticalizedLabelOffset;
                bl = center
                    + rotate(&(bl - center), verticalRotation)
                    + xOffsetCorrection
                    + verticalizedLabelOffset;
                br = center
                    + rotate(&(br - center), verticalRotation)
                    + xOffsetCorrection
                    + verticalizedLabelOffset;
            }

            if (textRotate != 0.0) {
                // TODO is this correct?
                // Compute the transformation matrix.
                let angle_sin = textRotate.sin();
                let angle_cos = textRotate.cos();
                let matrix = [angle_cos, -angle_sin, angle_sin, angle_cos];

                tl = matrixMultiply(&matrix, tl);
                tr = matrixMultiply(&matrix, tr);
                bl = matrixMultiply(&matrix, bl);
                br = matrixMultiply(&matrix, br);
            }

            let pixelOffsetTL = Point2D::default();
            let pixelOffsetBR = Point2D::default();
            let minFontScale = Point2D::default();

            quads.push(SymbolQuad {
                tl,
                tr,
                bl,
                br,
                tex: rect,
                pixelOffsetTL,
                pixelOffsetBR,
                glyphOffset: glyphOffset,
                writingMode: shapedText.writingMode,
                isSDF: isSDF,
                sectionIndex: positionedGlyph.sectionIndex,
                minFontScale: minFontScale,
            });
        }
    }

    return quads;
}
#[cfg(test)]
mod tests {
    use cgmath::ulps_eq;

    use crate::{
        euclid::{Point2D, Rect, Size2D},
        sdf::{
            geometry::Anchor,
            geometry_tile_data::GeometryCoordinates,
            glyph::{PositionedGlyph, PositionedLine, Shaping, WritingModeType},
            image_atlas::ImagePosition,
            layout::symbol_instance::SymbolContent,
            quads::getIconQuads,
            shaping::PositionedIcon,
            style_types::{IconTextFitType, SymbolAnchorType, SymbolLayoutProperties_Evaluated},
        },
    };

    #[test]
    pub fn getIconQuads_normal() {
        let layout = SymbolLayoutProperties_Evaluated;
        let anchor = Anchor {
            point: Point2D::new(2.0, 3.0),
            angle: 0.0,
            segment: Some(0),
        };
        let image: ImagePosition = ImagePosition {
            pixelRatio: 1.0,
            paddedRect: Rect::new(Point2D::origin(), Size2D::new(15, 11)),
            version: 0,
            stretchX: vec![],
            stretchY: vec![],
            content: None,
        };

        let shapedIcon =
            PositionedIcon::shapeIcon(image.clone(), &[-6.5, -4.5], SymbolAnchorType::Center);

        let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
    pub fn getIconQuads_style() {
        let anchor = Anchor {
            point: Point2D::new(0.0, 0.0),
            angle: 0.0,
            segment: Some(0),
        };

        let image: ImagePosition = ImagePosition {
            pixelRatio: 1.0,
            paddedRect: Rect::new(Point2D::origin(), Size2D::new(20, 20)),
            version: 0,
            stretchX: vec![],
            stretchY: vec![],
            content: None,
        };

        let line = GeometryCoordinates::default();
        let mut shapedText: Shaping = Shaping {
            top: -10.,
            bottom: 30.0,
            left: -60.,
            right: 20.0,

            positionedLines: vec![],
            writingMode: WritingModeType::None,
            verticalizable: false,
            iconsInText: false,
        };

        // shapedText.positionedGlyphs.emplace_back(PositionedGlyph(32, 0.0, 0.0, false, 0, 1.0));

        shapedText.positionedLines.push(PositionedLine::default());
        shapedText
            .positionedLines
            .last_mut()
            .unwrap()
            .positionedGlyphs
            .push(PositionedGlyph {
                glyph: 32,
                x: 0.0,
                y: 0.0,
                vertical: false,
                font: 0,
                scale: 0.0,
                rect: Default::default(),
                metrics: Default::default(),
                imageID: None,
                sectionIndex: 0,
            });

        // none
        {
            let shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);

            ulps_eq!(-18.5, shapedIcon.top);
            ulps_eq!(-0.5, shapedIcon.right);
            ulps_eq!(-0.5, shapedIcon.bottom);
            ulps_eq!(-18.5, shapedIcon.left);

            let layout = SymbolLayoutProperties_Evaluated;
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Width,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Width,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Width,
                &[5., 10., 5., 10.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Height,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Height,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Height,
                &[5., 10., 5., 20.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Both,
                &[0., 0., 0., 0.],
                &[0., 0.],
                24.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Both,
                &[0., 0., 0., 0.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Both,
                &[5., 10., 5., 10.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
            let mut shapedIcon =
                PositionedIcon::shapeIcon(image.clone(), &[-9.5, -9.5], SymbolAnchorType::Center);
            shapedIcon.fitIconToText(
                &shapedText,
                IconTextFitType::Both,
                &[0., 5., 10., 15.],
                &[0., 0.],
                12.0 / 24.0,
            );
            let quads = getIconQuads(&shapedIcon, 0., SymbolContent::IconRGBA, false);

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
