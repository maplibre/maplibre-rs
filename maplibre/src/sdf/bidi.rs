use std::collections::BTreeSet;
use widestring::U16String;

pub type Char16 = u16; // was char16_t

pub fn applyArabicShaping(str: &U16String) -> U16String {
    // todo!();
    str.clone()
}

// StyledText pairs each code point in a string with an integer indicating
// the styling options to use for rendering that code point
// The data structure is intended to accomodate the reordering/interleaving
// of formatting that can happen when BiDi rearranges inputs
pub type StyledText = (U16String, Vec<u8>);

pub struct BiDi;

impl BiDi {
    // TODO: This implementation is from the QT backend and lacks ICU support
    /// Given text in logical ordering and a set of line break points,
    /// return a set of lines in visual order with bidi and line breaking applied
    pub fn processText(
        &self,
        input: &U16String,
        mut lineBreakPoints: BTreeSet<usize>, // TODO: Make sure this is no output
    ) -> Vec<U16String> {
        lineBreakPoints.insert(input.len());

        let mut transformedLines = Vec::new();
        let mut start = 0;
        for lineBreakPoint in lineBreakPoints {
            transformedLines.push(U16String::from(&input[start..lineBreakPoint])); // TODO verify if this is correct
            start = lineBreakPoint;
        }

        return transformedLines;
    }

    /// Same as processText but preserves per-code-point formatting information
    pub fn processStyledText(
        &self,
        input: &StyledText,
        mut lineBreakPoints: BTreeSet<usize>, // TODO: Make sure this is no output
    ) -> Vec<StyledText> {
        lineBreakPoints.insert(input.0.len());

        let mut transformedLines = Vec::new();
        let mut start = 0;
        for lineBreakPoint in lineBreakPoints {
            if (lineBreakPoint <= input.1.len()) {
                transformedLines.push((
                    U16String::from(&input.0[start..lineBreakPoint]),
                    Vec::from(&input.1[start..lineBreakPoint]),
                )); // TODO verify if this is correct
                start = lineBreakPoint;
            }
        }

        return transformedLines;
    }
}
