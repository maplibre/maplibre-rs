use crate::sdf::bidi::Char16;
use widestring::U16String;

pub fn allowsWordBreaking(chr: Char16) -> bool {
    todo!()
}

pub fn charAllowsLetterSpacing(chr: Char16) -> bool {
    todo!()
}

pub fn allowsLetterSpacing(string: &U16String) -> bool {
    todo!()
}

pub fn allowsIdeographicBreaking_str(string: &U16String) -> bool {
    todo!()
}

pub fn allowsIdeographicBreaking(chr: Char16) -> bool {
    todo!()
}

pub fn allowsFixedWidthGlyphGeneration(chr: Char16) -> bool {
    todo!()
}

pub fn allowsVerticalWritingMode(string: &U16String) -> bool {
    todo!()
}

// The following logic comes from
// <http://www.unicode.org/Public/12.0.0/ucd/VerticalOrientation.txt>.
// Keep it synchronized with
// <http://www.unicode.org/Public/UCD/latest/ucd/VerticalOrientation.txt>.
// The data file denotes with “U” or “Tu” any codepoint that may be drawn
// upright in vertical text but does not distinguish between upright and
// “neutral” characters.

pub fn hasUprightVerticalOrientation(chr: Char16) -> bool {
    todo!()
}

pub fn hasNeutralVerticalOrientation(chr: Char16) -> bool {
    todo!()
}

pub fn hasRotatedVerticalOrientation(chr: Char16) -> bool {
    todo!()
}

// Replaces "horizontal" with "vertical" punctuation in place
// Does not re-order or change length of string
// (TaggedString::verticalizePunctuation depends on this behavior)
pub fn verticalizePunctuation_str(input: &U16String) -> U16String {
    todo!()
}

pub fn verticalizePunctuation(chr: Char16) -> char {
    todo!()
}

pub fn charInSupportedScript(chr: Char16) -> bool {
    todo!()
}

pub fn isStringInSupportedScript(input: &str) -> bool {
    todo!()
}

pub fn isCharInComplexShapingScript(chr: Char16) -> bool {
    todo!()
}

pub const BACKSLACK_V: Char16 = '\u{000B}' as Char16;
pub const BACKSLACK_F: Char16 = '\u{000C}' as Char16;

pub fn isWhitespace(chr: Char16) -> bool {
    // TODO verify that his is correct \v and \f where not available
    return chr == ' ' as Char16
        || chr == '\t' as Char16
        || chr == '\n' as Char16
        || chr == BACKSLACK_V
        || chr == BACKSLACK_F
        || chr == '\r' as Char16;
}
