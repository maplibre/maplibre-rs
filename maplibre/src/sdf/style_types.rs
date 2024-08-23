use crate::sdf::layout::symbol_feature::SymbolGeometryTileFeature;
use crate::sdf::CanonicalTileID;
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

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
pub enum TextWritingModeType {
    Horizontal = 0,
    Vertical = 1,
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

impl<T> PropertyValue<T> {
    pub fn isUndefined(&self) -> bool {
        todo!()
    }
    pub fn isDataDriven(&self) -> bool {
        todo!()
    }

    pub fn isZoomant(&self) -> bool {
        todo!()
    }
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
    pub fn name() -> &'static str {
        return "icon-allow-overlap";
    }

    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-anchor";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-ignore-placement";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-image";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-keep-upright";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-offset";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-optional";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-padding";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-pitch-alignment";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-rotate";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-rotation-alignment";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-size";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-text-fit";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "icon-text-fit-padding";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "symbol-avoid-edges";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "symbol-placement";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "symbol-sort-key";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "symbol-spacing";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "symbol-z-order";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-allow-overlap";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-anchor";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-field";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-font";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-ignore-placement";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-justify";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-keep-upright";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-letter-spacing";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-line-height";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-max-angle";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-max-width";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-offset";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-optional";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-padding";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-pitch-alignment";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-radial-offset";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-rotate";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-rotation-alignment";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-size";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-transform";
    }
    pub fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-variable-anchor";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
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
    pub fn name() -> &'static str {
        return "text-writing-mode";
    }
    pub fn defaultValue() -> <Self as LayoutProperty>::Type {
        return Vec::new();
    }
}

impl LayoutProperty for TextWritingMode {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = Vec<TextWritingModeType>;
}

#[derive(Clone)]
pub struct SymbolLayoutProperties_Unevaluated;
#[derive(Clone)]
pub struct SymbolLayoutProperties_PossiblyEvaluated;

impl SymbolLayoutProperties_PossiblyEvaluated {
    pub fn has<T>(&self) -> bool {
        todo!()
        //     return layout.get<Property>().match([](const typename Property::Type& t) { return !t.is_empty(); },
        //                                         [](let) { return true; });
    }
}

#[derive(Clone)]
pub struct SymbolLayoutProperties_Evaluated;

pub mod expression {
    use crate::sdf::font_stack::FontStack;
    use crate::sdf::layout::symbol_feature::SymbolGeometryTileFeature;
    use crate::sdf::CanonicalTileID;
    use csscolorparser::Color;
    use std::collections::{BTreeSet, HashMap};
    use std::rc::Rc;

    #[derive(Clone, PartialEq)]
    pub enum Value {
        Color(Color),
        f64(f64),
        Object(HashMap<String, Value>),
    }

    // TODO
    #[derive(Default, Clone)]
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

    // TODO
    pub type FeatureState = Value;

    pub struct EvaluationContext {
        zoom: Option<f64>,
        accumulated: Option<Value>,
        feature: Rc<SymbolGeometryTileFeature>,
        colorRampParameter: Option<f64>,
        // Contains formatted section object, std::unordered_map<std::string, Value>.
        formattedSection: Rc<Value>,
        featureState: Rc<FeatureState>,
        availableImages: Rc<BTreeSet<String>>,
        canonical: Rc<CanonicalTileID>,
    }
}

// TODO
pub struct PropertyEvaluationParameters(pub f64);

impl SymbolLayoutProperties_Unevaluated {
    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::UnevaluatedType {
        todo!()
    }

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
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate2<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate4<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        availableImages: &BTreeSet<String>,
        p2: CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
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
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        todo!()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        todo!()
    }
}
