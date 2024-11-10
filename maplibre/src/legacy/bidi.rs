//! Translated from the QT BIDI implementation https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/bidi.cpp

use std::collections::BTreeSet;

use widestring::U16String;

/// maplibre/maplibre-native#4add9ea original name: Char16
pub type Char16 = u16; // was char16_t

/// maplibre/maplibre-native#4add9ea original name: applyArabicShaping
pub fn apply_arabic_shaping(str: &U16String) -> U16String {
    // TODO: Add real implementation
    str.clone()
}

// StyledText pairs each code point in a string with an integer indicating
// the styling options to use for rendering that code point
// The data structure is intended to accomodate the reordering/interleaving
// of formatting that can happen when BiDi rearranges inputs
/// maplibre/maplibre-native#4add9ea original name: StyledText
pub type StyledText = (U16String, Vec<u8>);

/// maplibre/maplibre-native#4add9ea original name: BiDi
pub struct BiDi;

impl BiDi {
    // TODO: This implementation is from the QT backend and lacks ICU support
    /// Given text in logical ordering and a set of line break points,
    /// return a set of lines in visual order with bidi and line breaking applied
    /// maplibre/maplibre-native#4add9ea original name: processText
    pub fn process_text(
        &self,
        input: &U16String,
        mut line_break_points: BTreeSet<usize>, // TODO: Make sure this is no output
    ) -> Vec<U16String> {
        line_break_points.insert(input.len());

        let mut transformed_lines = Vec::new();
        let mut start = 0;
        for lineBreakPoint in line_break_points {
            transformed_lines.push(U16String::from(&input[start..lineBreakPoint])); // TODO verify if this is correct
            start = lineBreakPoint;
        }

        transformed_lines
    }

    /// Same as processText but preserves per-code-point formatting information
    /// maplibre/maplibre-native#4add9ea original name: processStyledText
    pub fn process_styled_text(
        &self,
        input: &StyledText,
        mut line_break_points: BTreeSet<usize>, // TODO: Make sure this is no output
    ) -> Vec<StyledText> {
        line_break_points.insert(input.0.len());

        let mut transformed_lines = Vec::new();
        let mut start = 0;
        for lineBreakPoint in line_break_points {
            if lineBreakPoint <= input.1.len() {
                transformed_lines.push((
                    U16String::from(&input.0[start..lineBreakPoint]),
                    Vec::from(&input.1[start..lineBreakPoint]),
                )); // TODO verify if this is correct
                start = lineBreakPoint;
            }
        }

        transformed_lines
    }
}
