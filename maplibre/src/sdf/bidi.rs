
// TODO
pub struct u16string;


// TODO
pub fn applyArabicShaping(str: &u16string) -> u16string {
    todo!();
}

// StyledText pairs each code point in a string with an integer indicating
// the styling options to use for rendering that code point
// The data structure is intended to accomodate the reordering/interleaving
// of formatting that can happen when BiDi rearranges inputs
pub type StyledText = (u16string, Vec<u8>);

// TODO
pub struct BiDi;