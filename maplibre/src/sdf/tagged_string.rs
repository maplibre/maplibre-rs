use crate::sdf::bidi::{StyledText, u16string};
use crate::sdf::font_stack::{FontStack, FontStackHash};
use crate::sdf::i18n;

// TODO
struct Color;

#[derive(Default)]
pub struct SectionOptions {
    pub scale: f64,
    pub fontStackHash: FontStackHash,
    pub fontStack: FontStack,
    pub textColor: Option<Color>,
    pub imageID: Option<String>,
}
impl SectionOptions {
    pub fn new(imageID_: String) -> Self {
        Self {
            scale: 1.0,
            imageID: Some(imageID_),
            ..SectionOptions::default()
        }
    }
}

const PUAbegin: char = '\u{E000}';
const PUAend: char = '\u{F8FF}';

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
    pub imageSectionID: char,
}

impl TaggedString {
    pub fn new_from_raw(text_: u16string, options: SectionOptions) -> Self {
        Self {
            styledText: StyledText::new(text_), // == std::move(text_), std::vector<uint8_t>(text_.size(), 0)
            sections: vec![options],
            supportsVerticalWritingMode: None,
            imageSectionID: 0 as char, // TODO is this correct?
        }
    }

    pub fn new(styledText_: StyledText, sections_: Vec<SectionOptions>) -> Self {
        Self {
            styledText: styledText_,
            sections: sections_,
            supportsVerticalWritingMode: None,
            imageSectionID: 0 as char, // TODO is this correct?
        }
    }

    pub fn length(&self) -> usize {
        return self.styledText.0.length();
    }

    pub fn sectionCount(&self) -> usize {
        return self.sections.size();
    }

    pub fn empty(&self) -> bool {
        return self.styledText.0.empty();
    }

    pub fn getSection(&self, index: usize) -> &SectionOptions {
        return self.sections.at(self.styledText.1.at(index));
    }

    pub fn getCharCodeAt(&self, index: usize) -> char {
        return self.styledText.0[index];
    }

    pub fn rawText(&self) -> &u16string {
        return &self.styledText.0;
    }

    pub fn getStyledText(&self) -> &StyledText {
        return &self.styledText;
    }

    pub fn addTextSection(
        &mut self,
        sectionText: &u16string,
        scale: f64,
        fontStack: &FontStack,
        textColor: Option<Color>,
    ) {
        self.styledText.0 += sectionText;
        self.sections.push(scale, fontStack, textColor);
        self.styledText.1.resize(
            self.styledText.0.size(),
            (self.sections.len() - 1) as u8,
        );
        self.supportsVerticalWritingMode = None;
    }

    pub fn addImageSection(&mut self, imageID: &String) {
        let nextImageSectionCharCode = self.getNextImageSectionCharCode();
        if (!nextImageSectionCharCode) {
            log::warn!("Exceeded maximum number of images in a label.");
            return;
        }

        self.styledText.0 += *nextImageSectionCharCode;
        self.sections.push(imageID);
        self.styledText.1.resize(
            self.styledText.0.size(),
            (self.sections.size() - 1) as u8,
        );
    }

    pub fn sectionAt(&self, index: usize) -> &SectionOptions {
        return &self.sections[index];
    }

    pub fn getSections(&self) -> &Vec<SectionOptions> {
        return &self.sections;
    }

    pub fn getSectionIndex(&self, characterIndex: usize) -> u8 {
        return self.styledText.1.at(characterIndex);
    }

    pub fn getMaxScale(&self) -> f64 {
    let mut maxScale = 0.0;
    for i in 0..self.styledText.0.length() {
        maxScale = maxScale.max(self.getSection(i).scale)
    }
    return maxScale;
    }
    pub fn trim(&mut self) {
        let beginningWhitespace: usize = self.styledText.0.find_first_not_of(" \t\n\v\f\r");
        if (beginningWhitespace == u16string::npos) {
            // Entirely whitespace
            self.styledText.0.clear();
            self.styledText.1.clear();
        } else {
            let trailingWhitespace: usize = self.styledText.0.find_last_not_of(" \t\n\v\f\r") + 1;

            self.styledText.0 = self.styledText.0.substr(beginningWhitespace, trailingWhitespace - beginningWhitespace);
            self.styledText.1 = (self.styledText.1.begin() + beginningWhitespace,
                                      self.styledText.1.begin() + trailingWhitespace) as u8;
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
        return self.supportsVerticalWritingMode.expect("supportsVerticalWritingMode is set");
    }
}

impl TaggedString {
    fn getNextImageSectionCharCode(&mut self) -> Option<char> {
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
