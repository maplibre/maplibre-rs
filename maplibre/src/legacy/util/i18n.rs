//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/util/i18n.cpp


use widestring::U16String;

use crate::legacy::bidi::Char16;

pub fn allowsWordBreaking(chr: Char16) -> bool {
    return (chr == 0x0a      /* newline */
        || chr == 0x20   /* space */
        || chr == 0x26   /* ampersand */
        || chr == 0x28   /* open parenthesis */
        || chr == 0x29   /* close parenthesis */
        || chr == 0x2b   /* plus sign */
        || chr == 0x2d   /* hyphen-minus */
        || chr == 0x2f   /* solidus */
        || chr == 0xad   /* soft hyphen */
        || chr == 0xb7   /* middle dot */
        || chr == 0x200b /* zero-width space */
        || chr == 0x2010 /* hyphen */
        || chr == 0x2013/* en dash */);
}

pub fn charAllowsLetterSpacing(chr: Char16) -> bool {
    return false;
    todo!()
}

pub fn allowsLetterSpacing(string: &U16String) -> bool {
    return false;
    todo!()
}

pub fn allowsIdeographicBreaking_str(string: &U16String) -> bool {
    return false;
    todo!()
}

pub fn allowsIdeographicBreaking(chr: Char16) -> bool {
    return false;
    todo!()
}

pub fn allowsFixedWidthGlyphGeneration(chr: Char16) -> bool {
    return false;
    todo!()
}

pub fn allowsVerticalWritingMode(string: &U16String) -> bool {
    return false;
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
    return false;
    todo!()
}

pub fn hasNeutralVerticalOrientation(chr: Char16) -> bool {
    return false;
    todo!()
}

pub fn hasRotatedVerticalOrientation(chr: Char16) -> bool {
    return !(hasUprightVerticalOrientation(chr) || hasNeutralVerticalOrientation(chr));
}

// Replaces "horizontal" with "vertical" punctuation in place
// Does not re-order or change length of string
// (TaggedString::verticalizePunctuation depends on this behavior)
pub fn verticalizePunctuation_str(input: &U16String) -> U16String {
    return input.clone();
    todo!()
}

pub fn verticalizePunctuation(chr: Char16) -> Char16 {
    return 0;
    todo!()
}

pub fn charInSupportedScript(chr: Char16) -> bool {
    return true;
    todo!()
}

pub fn isStringInSupportedScript(input: &str) -> bool {
    let u16string = U16String::from(input); // TODO: verify if this is correct
    for chr in u16string.as_slice() {
        if (!charInSupportedScript(*chr)) {
            return false;
        }
    }
    return true;
}

pub fn isCharInComplexShapingScript(chr: Char16) -> bool {
    return false;
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
