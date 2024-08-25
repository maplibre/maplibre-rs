use std::any::{TypeId};
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

#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
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

impl <T> Default for PropertyValue<T> {
    fn default() -> Self {
        // TODO
        PropertyValue {
            value: expression::Value::f64(0.0),
            _phandom: Default::default(),
        }
    }
}

impl<T> PropertyValue<T> {
    pub fn isUndefined(&self) -> bool {
       // todo!()
        false
    }
    pub fn isDataDriven(&self) -> bool {
       // todo!()
        false
    }

    pub fn isZoomant(&self) -> bool {
      //  todo!()
        false
    }
}

#[derive(Clone, PartialEq)]
pub struct PossiblyEvaluatedPropertyValue<T> {
    value: expression::Value,
    _phandom: PhantomData<T>,
}

impl <T> Default for PossiblyEvaluatedPropertyValue<T> {
    fn default() -> Self {
        // TODO
        PossiblyEvaluatedPropertyValue {
            value: expression::Value::f64(0.0),
            _phandom: Default::default(),
        }
    }
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

    fn name() -> &'static str;
    fn defaultValue() -> Self::Type;
}

pub trait DataDrivenLayoutProperty {
    // type TransitionableType = std::nullptr_t;
    type UnevaluatedType: Default;
    //type EvaluatorType = DataDrivenPropertyEvaluator<T>;
    type PossiblyEvaluatedTyp: Default;
    type Type;
    const IsDataDriven: bool = true;
    const IsOverridable: bool = false;

    fn name() -> &'static str;
    fn defaultValue() -> Self::Type;
}

// text
pub struct IconAllowOverlap {}

impl LayoutProperty for IconAllowOverlap {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "icon-allow-overlap";
    }

    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct IconAnchor {}

impl DataDrivenLayoutProperty for IconAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = SymbolAnchorType;

    fn name() -> &'static str {
        return "icon-anchor";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return SymbolAnchorType::Center;
    }
}

pub struct IconIgnorePlacement {}

impl LayoutProperty for IconIgnorePlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "icon-ignore-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct IconImage {}

impl DataDrivenLayoutProperty for IconImage {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = expression::Image;

    fn name() -> &'static str {
        return "icon-image";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return expression::Image::default();
    }
}

pub struct IconKeepUpright {}

impl LayoutProperty for IconKeepUpright {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "icon-keep-upright";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct IconOffset {}

impl DataDrivenLayoutProperty for IconOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = [f64; 2];

    fn name() -> &'static str {
        return "icon-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return [0.0, 0.0];
    }
}

pub struct IconOptional {}

impl LayoutProperty for IconOptional {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "icon-optional";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct IconPadding {}

impl LayoutProperty for IconPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;

    fn name() -> &'static str {
        return "icon-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 2.0;
    }
}

pub struct IconPitchAlignment {}

impl LayoutProperty for IconPitchAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;

    fn name() -> &'static str {
        return "icon-pitch-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}

pub struct IconRotate {}

impl DataDrivenLayoutProperty for IconRotate {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "icon-rotate";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}

pub struct IconRotationAlignment {}

impl LayoutProperty for IconRotationAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;

    fn name() -> &'static str {
        return "icon-rotation-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}
pub struct IconSize {}

impl DataDrivenLayoutProperty for IconSize {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "icon-size";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 1.0;
    }
}

pub struct IconTextFit {}

impl LayoutProperty for IconTextFit {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = IconTextFitType;

    fn name() -> &'static str {
        return "icon-text-fit";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return IconTextFitType::None;
    }
}
pub struct IconTextFitPadding {}
impl LayoutProperty for IconTextFitPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = [f64; 4];

    fn name() -> &'static str {
        return "icon-text-fit-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return [0.0, 0.0, 0.0, 0.0];
    }
}

pub struct SymbolAvoidEdges {}

impl LayoutProperty for SymbolAvoidEdges {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "symbol-avoid-edges";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct SymbolPlacement {}

impl LayoutProperty for SymbolPlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = SymbolPlacementType;

    fn name() -> &'static str {
        return "symbol-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return SymbolPlacementType::Point;
    }
}

pub struct SymbolSortKey {}

impl DataDrivenLayoutProperty for SymbolSortKey {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "symbol-sort-key";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}

pub struct SymbolSpacing {}


impl LayoutProperty for SymbolSpacing {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;

    fn name() -> &'static str {
        return "symbol-spacing";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 250.0;
    }
}

pub struct SymbolZOrder {}


impl LayoutProperty for SymbolZOrder {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = SymbolZOrderType;

    fn name() -> &'static str {
        return "symbol-z-order";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return SymbolZOrderType::Auto;
    }
}
pub struct TextAllowOverlap {}

impl LayoutProperty for TextAllowOverlap {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "text-allow-overlap";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct TextAnchor {}

impl DataDrivenLayoutProperty for TextAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = SymbolAnchorType;

    fn name() -> &'static str {
        return "text-anchor";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return SymbolAnchorType::Center;
    }
}

pub struct TextField {}
impl DataDrivenLayoutProperty for TextField {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = expression::Formatted;

    fn name() -> &'static str {
        return "text-field";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return expression::Formatted::default();
    }
}

pub struct TextFont {}


impl DataDrivenLayoutProperty for TextFont {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = Vec<String>;

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

pub struct TextIgnorePlacement {}

impl LayoutProperty for TextIgnorePlacement {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "text-ignore-placement";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct TextJustify {}

impl DataDrivenLayoutProperty for TextJustify {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = TextJustifyType;

    fn name() -> &'static str {
        return "text-justify";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return TextJustifyType::Center;
    }
}

pub struct TextKeepUpright {}

impl TextKeepUpright {

}
impl LayoutProperty for TextKeepUpright {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "text-keep-upright";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return true;
    }
}

pub struct TextLetterSpacing {}

impl TextLetterSpacing {

}
impl DataDrivenLayoutProperty for TextLetterSpacing {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;
    fn name() -> &'static str {
        return "text-letter-spacing";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}

pub struct TextLineHeight {}

impl TextLineHeight {

}
impl LayoutProperty for TextLineHeight {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;

    fn name() -> &'static str {
        return "text-line-height";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 1.2;
    }
}

pub struct TextMaxAngle {}


impl LayoutProperty for TextMaxAngle {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;

    fn name() -> &'static str {
        return "text-max-angle";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 45.0;
    }
}

pub struct TextMaxWidth {}


impl DataDrivenLayoutProperty for TextMaxWidth {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "text-max-width";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 10.0;
    }
}

pub struct TextOffset {}

impl DataDrivenLayoutProperty for TextOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = [f64; 2];

    fn name() -> &'static str {
        return "text-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return [0.0, 0.0];
    }
}

pub struct TextOptional {}

impl LayoutProperty for TextOptional {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = bool;

    fn name() -> &'static str {
        return "text-optional";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return false;
    }
}

pub struct TextPadding {}


impl LayoutProperty for TextPadding {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = f64;

    fn name() -> &'static str {
        return "text-padding";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return 2.0;
    }
}

pub struct TextPitchAlignment {}

impl TextPitchAlignment {

}
impl LayoutProperty for TextPitchAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;

    fn name() -> &'static str {
        return "text-pitch-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}

pub struct TextRadialOffset {}


impl DataDrivenLayoutProperty for TextRadialOffset {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "text-radial-offset";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}

pub struct TextRotate {}


impl DataDrivenLayoutProperty for TextRotate {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "text-rotate";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 0.0;
    }
}

pub struct TextRotationAlignment {}


impl LayoutProperty for TextRotationAlignment {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = AlignmentType;

    fn name() -> &'static str {
        return "text-rotation-alignment";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return AlignmentType::Auto;
    }
}

pub struct TextSize {}


impl DataDrivenLayoutProperty for TextSize {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = f64;

    fn name() -> &'static str {
        return "text-size";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return 16.0;
    }
}

pub struct TextTransform {}

impl DataDrivenLayoutProperty for TextTransform {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedTyp = PossiblyEvaluatedPropertyValue<Self::Type>;
    type Type = TextTransformType;

    fn name() -> &'static str {
        return "text-transform";
    }
    fn defaultValue() -> <Self as DataDrivenLayoutProperty>::Type {
        return TextTransformType::None;
    }
}

pub struct TextVariableAnchor {}

impl TextVariableAnchor {

}
impl LayoutProperty for TextVariableAnchor {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = Vec<TextVariableAnchorType>;

    fn name() -> &'static str {
        return "text-variable-anchor";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return Vec::new();
    }
}

pub struct TextWritingMode {}



impl LayoutProperty for TextWritingMode {
    type UnevaluatedType = PropertyValue<Self::Type>;
    type PossiblyEvaluatedType = Self::Type;
    type Type = Vec<TextWritingModeType>;

    fn name() -> &'static str {
        return "text-writing-mode";
    }
    fn defaultValue() -> <Self as LayoutProperty>::Type {
        return Vec::new();
    }
}

#[derive(Clone, Debug)]
pub struct SymbolLayoutProperties_Unevaluated;
#[derive(Clone,Debug)]
pub struct SymbolLayoutProperties_PossiblyEvaluated;

impl SymbolLayoutProperties_PossiblyEvaluated {
    pub fn has<T:'static>(&self) -> bool {
        // todo!()
        //     return layout.get<Property>().match([](const typename Property::Type& t) { return !t.is_empty(); },
        //                                         [](let) { return true; });
        TypeId::of::<T>() ==  TypeId::of::<TextField>() ||
        TypeId::of::<T>() ==  TypeId::of::<TextFont>()
    }
}

#[derive(Clone)]
pub struct SymbolLayoutProperties_Evaluated;

pub mod expression {
    use crate::sdf::font_stack::{FontStack};
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
    pub struct Formatted {
        pub sections: Vec<FormattedSection>,
    }

    impl Default for Formatted {
        fn default() -> Self {
            // TODO remove
            Formatted {
                sections: vec![FormattedSection {
                    text: "中中中中".to_string(),
                    image: None,
                    fontScale: None,
                    fontStack: None,
                    textColor: None,
                }],
            }
        }
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
        T::UnevaluatedType::default()
    }

    pub fn evaluate(
        &self,
        p0: PropertyEvaluationParameters,
    ) -> SymbolLayoutProperties_PossiblyEvaluated {
        // TODO
        SymbolLayoutProperties_PossiblyEvaluated
    }
}

// TODO generated
impl SymbolLayoutProperties_PossiblyEvaluated {
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        // todo!()
        T::defaultValue()
    }
    pub fn set<T: LayoutProperty>(&mut self, value: T::Type) {
        // todo!()
    }

    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        T::PossiblyEvaluatedTyp::default()
    }

    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    pub fn evaluate_feature(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
    ) -> SymbolLayoutProperties_Evaluated {
        //
        SymbolLayoutProperties_Evaluated
    }

    pub fn evaluate4<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        availableImages: &BTreeSet<String>,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }
}

impl SymbolLayoutProperties_Evaluated {
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        //todo!()
        T::defaultValue()
    }
    pub fn set<T: LayoutProperty>(&mut self, value: T::Type) {
        // todo!()
    }

    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        // todo!()
        T::PossiblyEvaluatedTyp::default()
    }

    pub fn get_eval<T: DataDrivenLayoutProperty>(&self) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    pub fn evaluate_static<T: LayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }
}
