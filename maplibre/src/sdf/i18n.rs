use crate::sdf::bidi::u16string;

pub fn allowsWordBreaking(chr: char) -> bool {
    todo!()
}

pub fn charAllowsLetterSpacing(chr: char) -> bool {
    todo!()
}

pub fn allowsLetterSpacing(string: &u16string) -> bool {
    todo!()
}

pub fn allowsIdeographicBreaking_str(string: &u16string) -> bool {
    todo!()
}

pub fn allowsIdeographicBreaking(chr: char) -> bool {
    todo!()
}

pub fn allowsFixedWidthGlyphGeneration(chr: char) -> bool {
    todo!()
}

pub fn allowsVerticalWritingMode(string: &u16string) -> bool {
    todo!()
}

// The following logic comes from
// <http://www.unicode.org/Public/12.0.0/ucd/VerticalOrientation.txt>.
// Keep it synchronized with
// <http://www.unicode.org/Public/UCD/latest/ucd/VerticalOrientation.txt>.
// The data file denotes with “U” or “Tu” any codepoint that may be drawn
// upright in vertical text but does not distinguish between upright and
// “neutral” characters.

pub fn hasUprightVerticalOrientation(chr: char) -> bool {
    todo!()
}

pub fn hasNeutralVerticalOrientation(chr: char) -> bool {
    todo!()
}

pub fn hasRotatedVerticalOrientation(chr: char) -> bool {
    todo!()
}


// Replaces "horizontal" with "vertical" punctuation in place
// Does not re-order or change length of string
// (TaggedString::verticalizePunctuation depends on this behavior)
pub fn verticalizePunctuation_str(input: &u16string) -> u16string {
    todo!()
}

pub fn verticalizePunctuation(chr: char) -> char {
    todo!()
}

pub fn charInSupportedScript(chr: char) -> bool {
    todo!()
}

pub fn isStringInSupportedScript(input: &str) -> bool {
    todo!()
}

pub fn isCharInComplexShapingScript(chr: char) -> bool {
    todo!()
}

pub fn isWhitespace(chr: char) -> bool {
    // TODO verify that his is correct \v and \f where not available
    return chr == ' '
        || chr == '\t'
        || chr == '\n'
        || chr == '\u{000B}'
        || chr == '\u{000C}'
        || chr == '\r';
}
