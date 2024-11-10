//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/text/tagged_string.cpp

use csscolorparser::Color;
use widestring::{U16Str, U16String};

use crate::legacy::{
    bidi::{Char16, StyledText},
    font_stack::{FontStack, FontStackHash, FontStackHasher},
    util::{
        i18n,
        i18n::{BACKSLACK_F, BACKSLACK_V},
    },
};

/// maplibre/maplibre-native#4add9ea original name: SectionOptions
#[derive(Clone, Default)]
pub struct SectionOptions {
    pub scale: f64,
    pub font_stack_hash: FontStackHash,
    pub font_stack: FontStack,
    pub text_color: Option<Color>,
    pub image_id: Option<String>,
}
impl SectionOptions {
    /// maplibre/maplibre-native#4add9ea original name: from_image_id
    pub fn from_image_id(image_id: String) -> Self {
        Self {
            scale: 1.0,
            image_id: Some(image_id),
            ..SectionOptions::default()
        }
    }
    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(scale: f64, font_stack: FontStack, text_color: Option<Color>) -> Self {
        Self {
            scale,
            font_stack_hash: FontStackHasher::new(&font_stack),
            font_stack,
            text_color,
            image_id: None,
        }
    }
}

const PUABEGIN: Char16 = '\u{E000}' as Char16;
const PUAEND: Char16 = '\u{F8FF}' as Char16;

/**
 * A TaggedString is the shaping-code counterpart of the Formatted type
 * Whereas Formatted matches the logical structure of a 'format' expression,
 * a TaggedString represents the same data at a per-character level so that
 * character-rearranging operations (e.g. BiDi) preserve formatting.
 * Text is represented as:
 * - A string of characters
 * - A matching array of indices, pointing to:
 * - An array of SectionsOptions, representing the evaluated formatting
 *    options of the original sections.
 *
 * Once the guts of a TaggedString have been re-arranged by BiDi, you can
 * iterate over the contents in order, using getCharCodeAt and getSection
 * to get the formatting options for each character in turn.
 */
/// maplibre/maplibre-native#4add9ea original name: TaggedString
#[derive(Clone)]
pub struct TaggedString {
    pub styled_text: StyledText,
    pub sections: Vec<SectionOptions>,
    pub supports_vertical_writing_mode: Option<bool>,
    // Max number of images within a text is 6400 U+E000–U+F8FF
    // that covers Basic Multilingual Plane Unicode Private Use Area (PUA).
    pub image_section_id: Char16,
}

impl Default for TaggedString {
    /// Returns an empty string
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        Self {
            styled_text: (U16String::new(), vec![]), // TODO is this correct?
            sections: vec![],
            supports_vertical_writing_mode: None,
            image_section_id: 0 as Char16, // TODO is this correct?
        }
    }
}

impl TaggedString {
    /// maplibre/maplibre-native#4add9ea original name: new_from_raw
    pub fn new_from_raw(text_: U16String, options: SectionOptions) -> Self {
        let text_len = text_.len();
        Self {
            styled_text: (text_, vec![0; text_len]), // TODO is this correct?
            sections: vec![options],
            supports_vertical_writing_mode: None,
            image_section_id: 0 as Char16, // TODO is this correct?
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: new
    pub fn new(styled_text: StyledText, sections_: Vec<SectionOptions>) -> Self {
        Self {
            styled_text,
            sections: sections_,
            supports_vertical_writing_mode: None,
            image_section_id: 0 as Char16, // TODO is this correct?
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: length
    pub fn length(&self) -> usize {
        self.styled_text.0.len()
    }

    /// maplibre/maplibre-native#4add9ea original name: sectionCount
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// maplibre/maplibre-native#4add9ea original name: empty
    pub fn empty(&self) -> bool {
        self.styled_text.0.is_empty()
    }

    /// maplibre/maplibre-native#4add9ea original name: getSection
    pub fn get_section(&self, index: usize) -> &SectionOptions {
        &self.sections[self.styled_text.1[index] as usize] // TODO Index does not honor encoding, fine? previously it was .at()
    }

    /// maplibre/maplibre-native#4add9ea original name: getCharCodeAt
    pub fn get_char_code_at(&self, index: usize) -> u16 {
        return self.styled_text.0.as_slice()[index];
    }

    /// maplibre/maplibre-native#4add9ea original name: rawText
    pub fn raw_text(&self) -> &U16String {
        &self.styled_text.0
    }

    /// maplibre/maplibre-native#4add9ea original name: getStyledText
    pub fn get_styled_text(&self) -> &StyledText {
        &self.styled_text
    }

    /// maplibre/maplibre-native#4add9ea original name: addTextSection
    pub fn add_text_section(
        &mut self,
        section_text: &U16String,
        scale: f64,
        font_stack: FontStack,
        text_color: Option<Color>,
    ) {
        self.styled_text.0.push(section_text);
        self.sections
            .push(SectionOptions::new(scale, font_stack, text_color));
        self.styled_text
            .1
            .resize(self.styled_text.0.len(), (self.sections.len() - 1) as u8);
        self.supports_vertical_writing_mode = None;
    }

    /// maplibre/maplibre-native#4add9ea original name: addImageSection
    pub fn add_image_section(&mut self, image_id: String) {
        let next_image_section_char_code = self.get_next_image_section_char_code();

        if let Some(nextImageSectionCharCode) = next_image_section_char_code {
            self.styled_text
                .0
                .push(U16Str::from_slice(&[nextImageSectionCharCode])); // TODO is this correct?
            self.sections.push(SectionOptions::from_image_id(image_id));
            self.styled_text
                .1
                .resize(self.styled_text.0.len(), (self.sections.len() - 1) as u8);
        } else {
            log::warn!("Exceeded maximum number of images in a label.");
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: sectionAt
    pub fn section_at(&self, index: usize) -> &SectionOptions {
        &self.sections[index]
    }

    /// maplibre/maplibre-native#4add9ea original name: getSections
    pub fn get_sections(&self) -> &Vec<SectionOptions> {
        &self.sections
    }

    /// maplibre/maplibre-native#4add9ea original name: getSectionIndex
    pub fn get_section_index(&self, character_index: usize) -> u8 {
        self.styled_text.1[character_index] // TODO Index does not honor encoding, fine? previously it was .at()
    }

    /// maplibre/maplibre-native#4add9ea original name: getMaxScale
    pub fn get_max_scale(&self) -> f64 {
        let mut max_scale: f64 = 0.0;
        for i in 0..self.styled_text.0.len() {
            max_scale = max_scale.max(self.get_section(i).scale)
        }
        max_scale
    }

    const WHITESPACE_CHARS: &'static [Char16] = &[
        ' ' as Char16,
        '\t' as Char16,
        '\n' as Char16,
        BACKSLACK_V as Char16,
        BACKSLACK_F as Char16,
        '\r' as Char16,
    ];

    /// maplibre/maplibre-native#4add9ea original name: trim
    pub fn trim(&mut self) {
        let beginning_whitespace: Option<usize> = self
            .styled_text
            .0
            .as_slice()
            .iter()
            .position(|c| !Self::WHITESPACE_CHARS.contains(c));

        if let Some(beginningWhitespace) = beginning_whitespace {
            let trailing_whitespace: usize = self
                .styled_text
                .0
                .as_slice()
                .iter()
                .rposition(|c| !Self::WHITESPACE_CHARS.contains(c))
                .expect("there is a whitespace char")
                + 1;

            self.styled_text.0 =
                U16String::from(&self.styled_text.0[beginningWhitespace..trailing_whitespace]); // TODO write test for this
            self.styled_text.1 =
                Vec::from(&self.styled_text.1[beginningWhitespace..trailing_whitespace]);
        } else {
            // Entirely whitespace
            self.styled_text.0.clear();
            self.styled_text.1.clear();
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: verticalizePunctuation
    pub fn verticalize_punctuation(&mut self) {
        // Relies on verticalization changing characters in place so that style indices don't need updating
        self.styled_text.0 = i18n::verticalize_punctuation_str(&self.styled_text.0);
    }
    /// maplibre/maplibre-native#4add9ea original name: allowsVerticalWritingMode
    pub fn allows_vertical_writing_mode(&mut self) -> bool {
        if self.supports_vertical_writing_mode.is_none() {
            let new_value = i18n::allows_vertical_writing_mode(self.raw_text());
            self.supports_vertical_writing_mode = Some(new_value);
            return new_value;
        }
        self.supports_vertical_writing_mode
            .expect("supportsVerticalWritingMode mut be set")
    }
}

impl TaggedString {
    /// maplibre/maplibre-native#4add9ea original name: getNextImageSectionCharCode
    fn get_next_image_section_char_code(&mut self) -> Option<Char16> {
        if self.image_section_id == 0 {
            self.image_section_id = PUABEGIN;
            return Some(self.image_section_id);
        }

        self.image_section_id += 1;
        if self.image_section_id > PUAEND {
            return None;
        }

        Some(self.image_section_id)
    }
}

#[cfg(test)]
mod tests {
    use widestring::U16String;

    use crate::legacy::{
        bidi::Char16,
        tagged_string::{SectionOptions, TaggedString},
        util::i18n::BACKSLACK_V,
    };

    #[test]
    /// maplibre/maplibre-native#4add9ea original name: TaggedString_Trim
    fn tagged_string_trim() {
        let mut basic = TaggedString::new_from_raw(
            " \t\ntrim that and not this  \n\t".into(),
            SectionOptions::new(1.0, vec![], None),
        );
        basic.trim();
        assert_eq!(basic.raw_text(), &U16String::from("trim that and not this"));

        let mut two_sections = TaggedString::default();
        two_sections.add_text_section(&" \t\ntrim that".into(), 1.5, vec![], None);
        two_sections.add_text_section(&" and not this  \n\t".into(), 0.5, vec![], None);

        two_sections.trim();
        assert_eq!(
            two_sections.raw_text(),
            &U16String::from("trim that and not this")
        );

        let mut empty = TaggedString::new_from_raw(
            format!(
                "\n\t{} \r  \t\n",
                char::from_u32(BACKSLACK_V as u32).unwrap()
            )
            .into(),
            SectionOptions::new(1.0, vec![], None),
        );
        empty.trim();
        assert_eq!(empty.raw_text(), &U16String::from(""));

        let mut no_trim =
            TaggedString::new_from_raw("no trim!".into(), SectionOptions::new(1.0, vec![], None));
        no_trim.trim();
        assert_eq!(no_trim.raw_text(), &U16String::from("no trim!"));
    }
    #[test]
    /// maplibre/maplibre-native#4add9ea original name: TaggedString_ImageSections
    fn tagged_string_image_sections() {
        let mut string = TaggedString::new_from_raw(U16String::new(), SectionOptions::default());
        string.add_image_section("image_name".to_string());
        assert_eq!(string.raw_text(), &U16String::from("\u{E000}"));
        assert!(string.get_section(0).image_id.is_some());
        assert_eq!(
            string.get_section(0).image_id.as_ref().unwrap(),
            &"image_name".to_string()
        );

        let mut max_sections = TaggedString::default();
        for i in 0..6401 {
            max_sections.add_image_section(i.to_string());
        }

        assert_eq!(max_sections.get_sections().len(), 6400);
        assert_eq!(max_sections.get_char_code_at(0), '\u{E000}' as Char16);
        assert_eq!(max_sections.get_char_code_at(6399), '\u{F8FF}' as Char16);
    }
}
