use crate::sdf::bidi::{Char16, StyledText};
use crate::sdf::font_stack::{FontStack, FontStackHash, FontStackHasher};
use crate::sdf::util::i18n;
use crate::sdf::util::i18n::{BACKSLACK_F, BACKSLACK_V};
use csscolorparser::Color;
use widestring::{U16Str, U16String};

#[derive(Clone, Default)]
pub struct SectionOptions {
    pub scale: f64,
    pub fontStackHash: FontStackHash,
    pub fontStack: FontStack,
    pub textColor: Option<Color>,
    pub imageID: Option<String>,
}
impl SectionOptions {
    pub fn from_image_id(imageID_: String) -> Self {
        Self {
            scale: 1.0,
            imageID: Some(imageID_),
            ..SectionOptions::default()
        }
    }
    pub fn new(scale: f64, fontStack: FontStack, textColor: Option<Color>) -> Self {
        Self {
            scale,
            fontStackHash: FontStackHasher::new(&fontStack),
            fontStack,
            textColor,
            imageID: None,
        }
    }
}

const PUAbegin: Char16 = '\u{E000}' as Char16;
const PUAend: Char16 = '\u{F8FF}' as Char16;

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
pub struct TaggedString {
    pub styledText: StyledText,
    pub sections: Vec<SectionOptions>,
    pub supportsVerticalWritingMode: Option<bool>,
    // Max number of images within a text is 6400 U+E000–U+F8FF
    // that covers Basic Multilingual Plane Unicode Private Use Area (PUA).
    pub imageSectionID: Char16,
}

impl Default for TaggedString {
    /// Returns an empty string
    fn default() -> Self {
        Self {
            styledText: (U16String::new(), vec![]), // TODO is this correct?
            sections: vec![],
            supportsVerticalWritingMode: None,
            imageSectionID: 0 as Char16, // TODO is this correct?
        }
    }
}

impl TaggedString {
    pub fn new_from_raw(text_: U16String, options: SectionOptions) -> Self {
        let text_len = text_.len();
        Self {
            styledText: (text_, vec![0; text_len]), // TODO is this correct?
            sections: vec![options],
            supportsVerticalWritingMode: None,
            imageSectionID: 0 as Char16, // TODO is this correct?
        }
    }

    pub fn new(styledText_: StyledText, sections_: Vec<SectionOptions>) -> Self {
        Self {
            styledText: styledText_,
            sections: sections_,
            supportsVerticalWritingMode: None,
            imageSectionID: 0 as Char16, // TODO is this correct?
        }
    }

    pub fn length(&self) -> usize {
        return self.styledText.0.len();
    }

    pub fn sectionCount(&self) -> usize {
        return self.sections.len();
    }

    pub fn empty(&self) -> bool {
        return self.styledText.0.is_empty();
    }

    pub fn getSection(&self, index: usize) -> &SectionOptions {
        return &self.sections[self.styledText.1[index] as usize]; // TODO Index does not honor encoding, fine? previously it was .at()
    }

    pub fn getCharCodeAt(&self, index: usize) -> u16 {
        return self.styledText.0.as_slice()[index];
    }

    pub fn rawText(&self) -> &U16String {
        return &self.styledText.0;
    }

    pub fn getStyledText(&self) -> &StyledText {
        return &self.styledText;
    }

    pub fn addTextSection(
        &mut self,
        sectionText: &U16String,
        scale: f64,
        fontStack: FontStack,
        textColor: Option<Color>,
    ) {
        self.styledText.0.push(sectionText);
        self.sections
            .push(SectionOptions::new(scale, fontStack, textColor));
        self.styledText
            .1
            .resize(self.styledText.0.len(), (self.sections.len() - 1) as u8);
        self.supportsVerticalWritingMode = None;
    }

    pub fn addImageSection(&mut self, imageID: String) {
        let nextImageSectionCharCode = self.getNextImageSectionCharCode();

        if let Some(nextImageSectionCharCode) = nextImageSectionCharCode {
            self.styledText
                .0
                .push(U16Str::from_slice(&[nextImageSectionCharCode])); // TODO is this correct?
            self.sections.push(SectionOptions::from_image_id(imageID));
            self.styledText
                .1
                .resize(self.styledText.0.len(), (self.sections.len() - 1) as u8);
        } else {
            log::warn!("Exceeded maximum number of images in a label.");
        }
    }

    pub fn sectionAt(&self, index: usize) -> &SectionOptions {
        return &self.sections[index];
    }

    pub fn getSections(&self) -> &Vec<SectionOptions> {
        return &self.sections;
    }

    pub fn getSectionIndex(&self, characterIndex: usize) -> u8 {
        return self.styledText.1[characterIndex]; // TODO Index does not honor encoding, fine? previously it was .at()
    }

    pub fn getMaxScale(&self) -> f64 {
        let mut maxScale: f64 = 0.0;
        for i in 0..self.styledText.0.len() {
            maxScale = maxScale.max(self.getSection(i).scale)
        }
        return maxScale;
    }

    const WHITESPACE_CHARS: &'static [Char16] = &[
        ' ' as Char16,
        '\t' as Char16,
        '\n' as Char16,
        BACKSLACK_V as Char16,
        BACKSLACK_F as Char16,
        '\r' as Char16,
    ];

    pub fn trim(&mut self) {
        let beginningWhitespace: Option<usize> = self
            .styledText
            .0
            .as_slice()
            .iter()
            .position(|c| !Self::WHITESPACE_CHARS.contains(c));

        if let Some(beginningWhitespace) = beginningWhitespace {
            let trailingWhitespace: usize = self
                .styledText
                .0
                .as_slice()
                .iter()
                .rposition(|c| !Self::WHITESPACE_CHARS.contains(c))
                .expect("there is a whitespace char")
                + 1;

            self.styledText.0 =
                U16String::from(&self.styledText.0[beginningWhitespace..trailingWhitespace]); // TODO write test for this
            self.styledText.1 =
                Vec::from(&self.styledText.1[beginningWhitespace..trailingWhitespace]);
        } else {
            // Entirely whitespace
            self.styledText.0.clear();
            self.styledText.1.clear();
        }
    }

    pub fn verticalizePunctuation(&mut self) {
        // Relies on verticalization changing characters in place so that style indices don't need updating
        self.styledText.0 = i18n::verticalizePunctuation_str(&self.styledText.0);
    }
    pub fn allowsVerticalWritingMode(&mut self) -> bool {
        if (self.supportsVerticalWritingMode.is_none()) {
            let new_value = i18n::allowsVerticalWritingMode(self.rawText());
            self.supportsVerticalWritingMode = Some(new_value);
            return new_value;
        }
        return self
            .supportsVerticalWritingMode
            .expect("supportsVerticalWritingMode is set");
    }
}

impl TaggedString {
    fn getNextImageSectionCharCode(&mut self) -> Option<Char16> {
        if (self.imageSectionID == 0) {
            self.imageSectionID = PUAbegin;
            return Some(self.imageSectionID);
        }

        self.imageSectionID += 1;
        if (self.imageSectionID > PUAend) {
            return None;
        }

        return Some(self.imageSectionID);
    }
}

#[cfg(test)]
mod tests {
    use crate::sdf::bidi::Char16;
    use crate::sdf::tagged_string::{SectionOptions, TaggedString};
    use crate::sdf::util::i18n::BACKSLACK_V;
    use widestring::U16String;

    #[test]
    fn TaggedString_Trim() {
        let mut basic = TaggedString::new_from_raw(
            " \t\ntrim that and not this  \n\t".into(),
            SectionOptions::new(1.0, vec![], None),
        );
        basic.trim();
        assert_eq!(basic.rawText(), &U16String::from("trim that and not this"));

        let mut twoSections = TaggedString::default();
        twoSections.addTextSection(&" \t\ntrim that".into(), 1.5, vec![], None);
        twoSections.addTextSection(&" and not this  \n\t".into(), 0.5, vec![], None);

        twoSections.trim();
        assert_eq!(
            twoSections.rawText(),
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
        assert_eq!(empty.rawText(), &U16String::from(""));

        let mut noTrim =
            TaggedString::new_from_raw("no trim!".into(), SectionOptions::new(1.0, vec![], None));
        noTrim.trim();
        assert_eq!(noTrim.rawText(), &U16String::from("no trim!"));
    }
    #[test]
    fn TaggedString_ImageSections() {
        let mut string = TaggedString::new_from_raw(U16String::new(), SectionOptions::default());
        string.addImageSection("image_name".to_string());
        assert_eq!(string.rawText(), &U16String::from("\u{E000}"));
        assert!(string.getSection(0).imageID.is_some());
        assert_eq!(
            string.getSection(0).imageID.as_ref().unwrap(),
            &"image_name".to_string()
        );

        let mut maxSections = TaggedString::default();
        for i in 0..6401 {
            maxSections.addImageSection(i.to_string());
        }

        assert_eq!(maxSections.getSections().len(), 6400);
        assert_eq!(maxSections.getCharCodeAt(0), '\u{E000}' as Char16);
        assert_eq!(maxSections.getCharCodeAt(6399), '\u{F8FF}' as Char16);
    }
}
