use crate::sdf::layout::symbol_feature::SymbolFeature;
use std::collections::BTreeSet;
use std::marker::PhantomData;

/// Types belonging to style

#[derive(Clone, Copy, PartialEq)]
pub enum SymbolPlacementType {
    Point,
    Line,
    LineCenter,
}
#[derive(Clone, Copy, PartialEq)]
pub enum SymbolAnchorType {
    Center,
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TextJustifyType {
    Auto,
    Center,
    Left,
    Right,
}
#[derive(Clone, Copy, PartialEq)]
pub enum IconTextFitType {
    None,
    Both,
    Width,
    Height,
}

#[derive(Clone, Copy, PartialEq)]
pub enum TextWritingModeType {
    Horizontal,
    Vertical,
}
pub type TextVariableAnchorType = SymbolAnchorType;

#[derive(Clone, Copy, PartialEq)]
pub enum AlignmentType {
    Map,
    Viewport,
    Auto,
}
#[derive(Clone, Copy, PartialEq)]
pub enum TextTransformType {
    None,
    Uppercase,
    Lowercase,
}
#[derive(Clone, Copy, PartialEq)]
pub enum SymbolZOrderType {
    Auto,
    ViewportY,
    Source,
}
#[derive(Clone, PartialEq)]
pub struct PropertyValue<T> {
    value: expression::Value,
    _phandom: PhantomData<T>,
}
#[derive(Clone, PartialEq)]
pub struct PossiblyEvaluatedPropertyValue<T> {
    value: expression::Value,
    _phandom: PhantomData<T>,
}

impl<T> PossiblyEvaluatedPropertyValue<T> {
    pub fn constantOr(&self, constant: T) -> T {
        todo!()
    }
}

pub trait LayoutProperty {
    // type TransitionableType = std::nullptr_t;
    type UnevaluatedType;
    // type EvaluatorType = PropertyEvaluator<T>;
    type PossiblyEvaluatedType;
    type Type;
    const IsDataDriven: bool = false;
    const IsOverridable: bool = false;
}

pub trait DataDrivenLayoutProperty {
    // type TransitionableType = std::nullptr_t;
    type UnevaluatedType;
    //type EvaluatorType = DataDrivenPropertyEvaluator<T>;
    type PossiblyEvaluatedTyp;
    type Type;
    const IsDataDriven: bool = true;
    const IsOverridable: bool = false;
}

// text
pub struct IconAllowOverlap {}

impl IconAllowOverlap {
    fn name() -> &'static str {
        return "icon-allow-overlap";
    }

    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

impl LayoutProperty for IconAllowOverlap {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct IconAnchor {}

impl IconAnchor {
    fn name() -> &'static str {
        return "icon-anchor";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return SymbolAnchorType::Center;
    }
}
impl DataDrivenLayoutProperty for IconAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = SymbolAnchorType;
}

pub struct IconIgnorePlacement {}

impl IconIgnorePlacement {
    fn name() -> &'static str {
        return "icon-ignore-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for IconIgnorePlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct IconImage {}

impl IconImage {
    fn name() -> &'static str {
        return "icon-image";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return expression::Image::default();
    }
}
impl DataDrivenLayoutProperty for IconImage {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = expression::Image;
}

pub struct IconKeepUpright {}

impl IconKeepUpright {
    fn name() -> &'static str {
        return "icon-keep-upright";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for IconKeepUpright {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct IconOffset {}

impl IconOffset {
    fn name() -> &'static str {
        return "icon-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return [0.0, 0.0];
    }
}
impl DataDrivenLayoutProperty for IconOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = [f64; 2];
}

pub struct IconOptional {}

impl IconOptional {
    fn name() -> &'static str {
        return "icon-optional";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for IconOptional {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct IconPadding {}

impl IconPadding {
    fn name() -> &'static str {
        return "icon-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 2.0;
    }
}
impl LayoutProperty for IconPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;
}

pub struct IconPitchAlignment {}

impl IconPitchAlignment {
    fn name() -> &'static str {
        return "icon-pitch-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}
impl LayoutProperty for IconPitchAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;
}

pub struct IconRotate {}

impl IconRotate {
    fn name() -> &'static str {
        return "icon-rotate";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}
impl DataDrivenLayoutProperty for IconRotate {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct IconRotationAlignment {}

impl IconRotationAlignment {
    fn name() -> &'static str {
        return "icon-rotation-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}
impl LayoutProperty for IconRotationAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;
}
pub struct IconSize {}

impl IconSize {
    fn name() -> &'static str {
        return "icon-size";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 1.0;
    }
}
impl DataDrivenLayoutProperty for IconSize {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct IconTextFit {}

impl IconTextFit {
    fn name() -> &'static str {
        return "icon-text-fit";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return IconTextFitType::None;
    }
}
impl LayoutProperty for IconTextFit {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = IconTextFitType;
}
pub struct IconTextFitPadding {}
impl IconTextFitPadding {
    fn name() -> &'static str {
        return "icon-text-fit-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return [0.0, 0.0, 0.0, 0.0];
    }
}
impl LayoutProperty for IconTextFitPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = [f64; 4];
}

pub struct SymbolAvoidEdges {}

impl SymbolAvoidEdges {
    fn name() -> &'static str {
        return "symbol-avoid-edges";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for SymbolAvoidEdges {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct SymbolPlacement {}

impl SymbolPlacement {
    fn name() -> &'static str {
        return "symbol-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return SymbolPlacementType::Point;
    }
}
impl LayoutProperty for SymbolPlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = SymbolPlacementType;
}

pub struct SymbolSortKey {}

impl SymbolSortKey {
    fn name() -> &'static str {
        return "symbol-sort-key";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}
impl DataDrivenLayoutProperty for SymbolSortKey {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct SymbolSpacing {}

impl SymbolSpacing {
    fn name() -> &'static str {
        return "symbol-spacing";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 250.0;
    }
}
impl LayoutProperty for SymbolSpacing {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;
}

pub struct SymbolZOrder {}

impl SymbolZOrder {
    fn name() -> &'static str {
        return "symbol-z-order";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return SymbolZOrderType::Auto;
    }
}
impl LayoutProperty for SymbolZOrder {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = SymbolZOrderType;
}
pub struct TextAllowOverlap {}

impl TextAllowOverlap {
    fn name() -> &'static str {
        return "text-allow-overlap";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for TextAllowOverlap {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct TextAnchor {}

impl TextAnchor {
    fn name() -> &'static str {
        return "text-anchor";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return SymbolAnchorType::Center;
    }
}
impl DataDrivenLayoutProperty for TextAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = SymbolAnchorType;
}

pub struct TextField {}
impl TextField {
    fn name() -> &'static str {
        return "text-field";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return expression::Formatted::default();
    }
}
impl DataDrivenLayoutProperty for TextField {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = expression::Formatted;
}

pub struct TextFont {}

impl TextFont {
    fn name() -> &'static str {
        return "text-font";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return vec![
            "Open Sans Regular".to_string(),
            "Arial Unicode MS Regular".to_string(),
        ];
    }
}
impl DataDrivenLayoutProperty for TextFont {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = Vec<String>;
}

pub struct TextIgnorePlacement {}

impl TextIgnorePlacement {
    fn name() -> &'static str {
        return "text-ignore-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for TextIgnorePlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct TextJustify {}

impl TextJustify {
    fn name() -> &'static str {
        return "text-justify";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return TextJustifyType::Center;
    }
}
impl DataDrivenLayoutProperty for TextJustify {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = TextJustifyType;
}

pub struct TextKeepUpright {}

impl TextKeepUpright {
    fn name() -> &'static str {
        return "text-keep-upright";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return true;
    }
}
impl LayoutProperty for TextKeepUpright {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct TextLetterSpacing {}

impl TextLetterSpacing {
    fn name() -> &'static str {
        return "text-letter-spacing";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}
impl DataDrivenLayoutProperty for TextLetterSpacing {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct TextLineHeight {}

impl TextLineHeight {
    fn name() -> &'static str {
        return "text-line-height";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 1.2;
    }
}
impl LayoutProperty for TextLineHeight {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;
}

pub struct TextMaxAngle {}

impl TextMaxAngle {
    fn name() -> &'static str {
        return "text-max-angle";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 45.0;
    }
}
impl LayoutProperty for TextMaxAngle {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;
}

pub struct TextMaxWidth {}

impl TextMaxWidth {
    fn name() -> &'static str {
        return "text-max-width";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 10.0;
    }
}
impl DataDrivenLayoutProperty for TextMaxWidth {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct TextOffset {}

impl TextOffset {
    fn name() -> &'static str {
        return "text-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return [0.0, 0.0];
    }
}
impl DataDrivenLayoutProperty for TextOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = [f64; 2];
}

pub struct TextOptional {}

impl TextOptional {
    fn name() -> &'static str {
        return "text-optional";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}
impl LayoutProperty for TextOptional {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;
}

pub struct TextPadding {}

impl TextPadding {
    fn name() -> &'static str {
        return "text-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 2.0;
    }
}
impl LayoutProperty for TextPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;
}

pub struct TextPitchAlignment {}

impl TextPitchAlignment {
    fn name() -> &'static str {
        return "text-pitch-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}
impl LayoutProperty for TextPitchAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;
}

pub struct TextRadialOffset {}

impl TextRadialOffset {
    fn name() -> &'static str {
        return "text-radial-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}
impl DataDrivenLayoutProperty for TextRadialOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct TextRotate {}

impl TextRotate {
    fn name() -> &'static str {
        return "text-rotate";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}
impl DataDrivenLayoutProperty for TextRotate {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct TextRotationAlignment {}

impl TextRotationAlignment {
    fn name() -> &'static str {
        return "text-rotation-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}
impl LayoutProperty for TextRotationAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;
}

pub struct TextSize {}

impl TextSize {
    fn name() -> &'static str {
        return "text-size";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 16.0;
    }
}
impl DataDrivenLayoutProperty for TextSize {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
}

pub struct TextTransform {}

impl TextTransform {
    fn name() -> &'static str {
        return "text-transform";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return TextTransformType::None;
    }
}
impl DataDrivenLayoutProperty for TextTransform {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = TextTransformType;
}

pub struct TextVariableAnchor {}

impl TextVariableAnchor {
    fn name() -> &'static str {
        return "text-variable-anchor";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return Vec::new();
    }
}
impl LayoutProperty for TextVariableAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = Vec<TextVariableAnchorType>;
}

pub struct TextWritingMode {}

impl TextWritingMode {
    fn name() -> &'static str {
        return "text-writing-mode";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return Vec::new();
    }
}

impl LayoutProperty for TextWritingMode {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = Vec<TextWritingModeType>;
}

pub struct SymbolLayerProperties;

pub struct LayerProperties;

pub struct PropertyEvaluationParameters(pub f64);
pub struct SymbolLayoutProperties_Unevaluated;
pub struct SymbolLayoutProperties_PossiblyEvaluated;
pub struct SymbolLayoutProperties_Evaluated;

pub mod expression {
    use crate::sdf::font_stack::FontStack;
    use csscolorparser::Color;
    use std::collections::HashMap;

    #[derive(Clone, PartialEq)]
    pub enum Value {
        Color(Color),
        f64(f64),
        Object(HashMap<String, Value>),
    }

    // TODO
    #[derive(Default)]
    pub struct Image {
        pub imageID: String,
        pub available: bool,
    }
    #[derive(Default)]
    pub struct Formatted {
        pub sections: Vec<FormattedSection>,
    }

    impl Formatted {
        fn toString() -> String {
            todo!()
        }
        fn toObject() -> Value {
            todo!()
        }

        fn empty() -> bool {
            todo!()
        }
    }

    impl PartialEq for Formatted {
        fn eq(&self, other: &Self) -> bool {
            todo!()
        }
    }

    #[derive(Default)]
    pub struct FormattedSection {
        pub text: String,
        pub image: Option<Image>,
        pub fontScale: Option<f64>,
        pub fontStack: Option<FontStack>,
        pub textColor: Option<Color>,
    }

    pub const kFormattedSectionFontScale: &'static str = "font-scale";
    pub const kFormattedSectionTextFont: &'static str = "text-font";
    pub const kFormattedSectionTextColor: &'static str = "text-color";
}

impl SymbolLayoutProperties_Unevaluated {
    pub fn evaluate(
        &self,
        p0: PropertyEvaluationParameters,
    ) -> SymbolLayoutProperties_PossiblyEvaluated {
        todo!()
    }
}

// TODO generated
impl SymbolLayoutProperties_PossiblyEvaluated {
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        todo!()
    }
    pub fn get_mut<T: LayoutProperty>(&mut self) -> &mut T::Type {
        todo!()
    }

    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        todo!()
    }

    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolFeature,
        p2: crate::sdf::layout::symbol_layout::CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate2<T: DataDrivenLayoutProperty>(&self, p0: f64, p1: &SymbolFeature) -> T::Type {
        todo!()
    }

    pub fn evaluate4<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolFeature,
        availableImages: &BTreeSet<String>,
        p2: crate::sdf::layout::symbol_layout::CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolFeature,
        p2: crate::sdf::layout::symbol_layout::CanonicalTileID,
    ) -> T::Type {
        todo!()
    }
}

impl SymbolLayoutProperties_Evaluated {
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        todo!()
    }
    pub fn get_mut<T: LayoutProperty>(&mut self) -> &mut T::Type {
        todo!()
    }

    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        todo!()
    }

    pub fn get_eval<T: DataDrivenLayoutProperty>(&self) -> T::Type {
        todo!()
    }

    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolFeature,
        p2: crate::sdf::layout::symbol_layout::CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolFeature,
        p2: crate::sdf::layout::symbol_layout::CanonicalTileID,
    ) -> T::Type {
        todo!()
    }
}
