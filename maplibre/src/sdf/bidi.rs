use std::collections::HashSet;
use widestring::U16String;

pub type Char16 = u16; // was char16_t

// TODO
pub fn applyArabicShaping(str: &U16String) -> U16String {
    todo!();
}

// StyledText pairs each code point in a string with an integer indicating
// the styling options to use for rendering that code point
// The data structure is intended to accomodate the reordering/interleaving
// of formatting that can happen when BiDi rearranges inputs
pub type StyledText = (U16String, Vec<u8>);

// TODO
pub struct BiDi;


impl BiDi {
pub fn processText(&self, input:  & U16String ,lineBreakPoints: HashSet<usize> ) -> Vec<U16String> {
    todo!()
}

    pub fn processStyledText(&self, input:  &StyledText,lineBreakPoints: HashSet<usize> ) -> Vec<StyledText> {
   todo!()
}
}