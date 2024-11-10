//! Translated from https://github.com/maplibre/maplibre-native/blob/4add9ea/include/mbgl/style/types.hpp
//! and https://github.com/maplibre/maplibre-native/blob/4add9ea/src/mbgl/style/layers/symbol_layer_properties.hpp

use std::{any::TypeId, collections::BTreeSet, marker::PhantomData};

use crate::legacy::{layout::symbol_feature::SymbolGeometryTileFeature, CanonicalTileID};

/// maplibre/maplibre-native#4add9ea original name: SymbolPlacementType
#[derive(Clone, Copy, PartialEq)]
pub enum SymbolPlacementType {
    Point,
    Line,
    LineCenter,
}
/// maplibre/maplibre-native#4add9ea original name: SymbolAnchorType
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

/// maplibre/maplibre-native#4add9ea original name: TextJustifyType
#[derive(Clone, Copy, PartialEq)]
pub enum TextJustifyType {
    Auto,
    Center,
    Left,
    Right,
}
/// maplibre/maplibre-native#4add9ea original name: IconTextFitType
#[derive(Clone, Copy, PartialEq)]
pub enum IconTextFitType {
    None,
    Both,
    Width,
    Height,
}

/// maplibre/maplibre-native#4add9ea original name: TextWritingModeType
#[derive(PartialOrd, Ord, PartialEq, Eq, Copy, Clone, Debug)]
pub enum TextWritingModeType {
    Horizontal = 0,
    Vertical = 1,
}

/// maplibre/maplibre-native#4add9ea original name: TextVariableAnchorType
pub type TextVariableAnchorType = SymbolAnchorType;

/// maplibre/maplibre-native#4add9ea original name: AlignmentType
#[derive(Clone, Copy, PartialEq)]
pub enum AlignmentType {
    Map,
    Viewport,
    Auto,
}
/// maplibre/maplibre-native#4add9ea original name: TextTransformType
#[derive(Clone, Copy, PartialEq)]
pub enum TextTransformType {
    None,
    Uppercase,
    Lowercase,
}
/// maplibre/maplibre-native#4add9ea original name: SymbolZOrderType
#[derive(Clone, Copy, PartialEq)]
pub enum SymbolZOrderType {
    Auto,
    ViewportY,
    Source,
}
/// maplibre/maplibre-native#4add9ea original name: PropertyValue
#[derive(Clone, PartialEq)]
pub struct PropertyValue<T> {
    value: expression::Value,
    _phandom: PhantomData<T>,
}

impl<T> Default for PropertyValue<T> {
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        // TODO
        PropertyValue {
            value: expression::Value::f64(0.0),
            _phandom: Default::default(),
        }
    }
}

impl<T> PropertyValue<T> {
    /// maplibre/maplibre-native#4add9ea original name: isUndefined
    pub fn isUndefined(&self) -> bool {
        // todo!()
        false
    }
    /// maplibre/maplibre-native#4add9ea original name: isDataDriven
    pub fn isDataDriven(&self) -> bool {
        // todo!()
        false
    }

    /// maplibre/maplibre-native#4add9ea original name: isZoomant
    pub fn isZoomant(&self) -> bool {
        //  todo!()
        false
    }
}

/// maplibre/maplibre-native#4add9ea original name: PossiblyEvaluatedPropertyValue
#[derive(Clone, PartialEq)]
pub struct PossiblyEvaluatedPropertyValue<T> {
    value: expression::Value,
    _phandom: PhantomData<T>,
}

impl<T> Default for PossiblyEvaluatedPropertyValue<T> {
    /// maplibre/maplibre-native#4add9ea original name: default
    fn default() -> Self {
        // TODO
        PossiblyEvaluatedPropertyValue {
            value: expression::Value::f64(0.0),
            _phandom: Default::default(),
        }
    }
}

impl<T> PossiblyEvaluatedPropertyValue<T> {
    /// maplibre/maplibre-native#4add9ea original name: constantOr
    pub fn constantOr(&self, constant: T) -> T {
        todo!()
    }
}

pub trait LayoutProperty {
    /// maplibre/maplibre-native#4add9ea original name: TransitionableType
    // type TransitionableType = std::nullptr_t;
    
    type UnevaluatedType;
    /// maplibre/maplibre-native#4add9ea original name: EvaluatorType
    // type EvaluatorType = PropertyEvaluator<T>;

    type PossiblyEvaluatedType;
    
    type Type;
    const IsDataDriven: bool = false;
    const IsOverridable: bool = false;

    
    fn name() -> &'static str;
    
    fn defaultValue() -> Self::Type;
}

pub trait DataDrivenLayoutProperty {
    /// maplibre/maplibre-native#4add9ea original name: TransitionableType
    // type TransitionableType = std::nullptr_t;

    type UnevaluatedType: Default;
    /// maplibre/maplibre-native#4add9ea original name: EvaluatorType
    //type EvaluatorType = DataDrivenPropertyEvaluator<T>;

    type PossiblyEvaluatedTyp: Default;
    
    type Type;
    const IsDataDriven: bool = true;
    const IsOverridable: bool = false;

    
    fn name() -> &'static str;
    
    fn defaultValue() -> Self::Type;
}

// text
/// maplibre/maplibre-native#4add9ea original name: IconAllowOverlap
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

/// maplibre/maplibre-native#4add9ea original name: IconAnchor
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

/// maplibre/maplibre-native#4add9ea original name: IconIgnorePlacement
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

/// maplibre/maplibre-native#4add9ea original name: IconImage
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

/// maplibre/maplibre-native#4add9ea original name: IconKeepUpright
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

/// maplibre/maplibre-native#4add9ea original name: IconOffset
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

/// maplibre/maplibre-native#4add9ea original name: IconOptional
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

/// maplibre/maplibre-native#4add9ea original name: IconPadding
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

/// maplibre/maplibre-native#4add9ea original name: IconPitchAlignment
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

/// maplibre/maplibre-native#4add9ea original name: IconRotate
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

/// maplibre/maplibre-native#4add9ea original name: IconRotationAlignment
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
/// maplibre/maplibre-native#4add9ea original name: IconSize
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

/// maplibre/maplibre-native#4add9ea original name: IconTextFit
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
/// maplibre/maplibre-native#4add9ea original name: IconTextFitPadding
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

/// maplibre/maplibre-native#4add9ea original name: SymbolAvoidEdges
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

/// maplibre/maplibre-native#4add9ea original name: SymbolPlacement
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

/// maplibre/maplibre-native#4add9ea original name: SymbolSortKey
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

/// maplibre/maplibre-native#4add9ea original name: SymbolSpacing
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

/// maplibre/maplibre-native#4add9ea original name: SymbolZOrder
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
/// maplibre/maplibre-native#4add9ea original name: TextAllowOverlap
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

/// maplibre/maplibre-native#4add9ea original name: TextAnchor
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

/// maplibre/maplibre-native#4add9ea original name: TextField
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

/// maplibre/maplibre-native#4add9ea original name: TextFont
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

/// maplibre/maplibre-native#4add9ea original name: TextIgnorePlacement
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

/// maplibre/maplibre-native#4add9ea original name: TextJustify
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

/// maplibre/maplibre-native#4add9ea original name: TextKeepUpright
pub struct TextKeepUpright {}

impl TextKeepUpright {}
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

/// maplibre/maplibre-native#4add9ea original name: TextLetterSpacing
pub struct TextLetterSpacing {}

impl TextLetterSpacing {}
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

/// maplibre/maplibre-native#4add9ea original name: TextLineHeight
pub struct TextLineHeight {}

impl TextLineHeight {}
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

/// maplibre/maplibre-native#4add9ea original name: TextMaxAngle
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

/// maplibre/maplibre-native#4add9ea original name: TextMaxWidth
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

/// maplibre/maplibre-native#4add9ea original name: TextOffset
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

/// maplibre/maplibre-native#4add9ea original name: TextOptional
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

/// maplibre/maplibre-native#4add9ea original name: TextPadding
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

/// maplibre/maplibre-native#4add9ea original name: TextPitchAlignment
pub struct TextPitchAlignment {}

impl TextPitchAlignment {}
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

/// maplibre/maplibre-native#4add9ea original name: TextRadialOffset
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

/// maplibre/maplibre-native#4add9ea original name: TextRotate
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

/// maplibre/maplibre-native#4add9ea original name: TextRotationAlignment
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

/// maplibre/maplibre-native#4add9ea original name: TextSize
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

/// maplibre/maplibre-native#4add9ea original name: TextTransform
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

/// maplibre/maplibre-native#4add9ea original name: TextVariableAnchor
pub struct TextVariableAnchor {}

impl TextVariableAnchor {}
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

/// maplibre/maplibre-native#4add9ea original name: TextWritingMode
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

/// maplibre/maplibre-native#4add9ea original name: SymbolLayoutProperties_Unevaluated
#[derive(Clone, Debug)]
pub struct SymbolLayoutProperties_Unevaluated;
/// maplibre/maplibre-native#4add9ea original name: SymbolLayoutProperties_PossiblyEvaluated
#[derive(Clone, Debug)]
pub struct SymbolLayoutProperties_PossiblyEvaluated;

impl SymbolLayoutProperties_PossiblyEvaluated {
    /// maplibre/maplibre-native#4add9ea original name: has
    pub fn has<T: 'static>(&self) -> bool {
        // todo!() check actual style if property is not empty
        //     return layout.get<Property>().match([](const typename Property::Type& t) { return !t.is_empty(); },
        //                                         [](let) { return true; });
        TypeId::of::<T>() == TypeId::of::<TextField>()
            || TypeId::of::<T>() == TypeId::of::<TextFont>()
    }
}

/// maplibre/maplibre-native#4add9ea original name: SymbolLayoutProperties_Evaluated
#[derive(Clone)]
pub struct SymbolLayoutProperties_Evaluated;

pub mod expression {
    use std::{
        collections::{BTreeSet, HashMap},
        rc::Rc,
    };

    use csscolorparser::Color;

    use crate::legacy::{
        font_stack::FontStack, layout::symbol_feature::SymbolGeometryTileFeature, CanonicalTileID,
    };

    /// maplibre/maplibre-native#4add9ea original name: Value
#[derive(Clone, PartialEq)]
    pub enum Value {
        Color(Color),
        f64(f64),
        Object(HashMap<String, Value>),
    }

    // TODO
    /// maplibre/maplibre-native#4add9ea original name: Image
    #[derive(Default, Clone)]
    pub struct Image {
        pub imageID: String,
        pub available: bool,
    }
    /// maplibre/maplibre-native#4add9ea original name: Formatted
    pub struct Formatted {
        pub sections: Vec<FormattedSection>,
    }

    impl Default for Formatted {
        /// maplibre/maplibre-native#4add9ea original name: default
        fn default() -> Self {
            // TODO remove
            Formatted {
                sections: vec![FormattedSection {
                    text: "AllerAnfangistschwer".to_string(),
                    image: None,
                    fontScale: None,
                    fontStack: None,
                    textColor: None,
                }],
            }
        }
    }

    impl Formatted {
        /// maplibre/maplibre-native#4add9ea original name: toString
        fn toString() -> String {
            todo!()
        }
        /// maplibre/maplibre-native#4add9ea original name: toObject
        fn toObject() -> Value {
            todo!()
        }

        /// maplibre/maplibre-native#4add9ea original name: empty
        fn empty() -> bool {
            todo!()
        }
    }

    impl PartialEq for Formatted {
        /// maplibre/maplibre-native#4add9ea original name: eq
        fn eq(&self, other: &Self) -> bool {
            todo!()
        }
    }

    /// maplibre/maplibre-native#4add9ea original name: FormattedSection
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
    /// maplibre/maplibre-native#4add9ea original name: FeatureState
    pub type FeatureState = Value;

    /// maplibre/maplibre-native#4add9ea original name: EvaluationContext
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
/// maplibre/maplibre-native#4add9ea original name: PropertyEvaluationParameters(pub
pub struct PropertyEvaluationParameters(pub f64);

impl SymbolLayoutProperties_Unevaluated {
    /// maplibre/maplibre-native#4add9ea original name: get_dynamic
    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::UnevaluatedType {
        T::UnevaluatedType::default()
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate
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
    /// maplibre/maplibre-native#4add9ea original name: get
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        // todo!()
        T::defaultValue()
    }
    /// maplibre/maplibre-native#4add9ea original name: set
    pub fn set<T: LayoutProperty>(&mut self, value: T::Type) {
        // todo!()
    }

    /// maplibre/maplibre-native#4add9ea original name: get_dynamic
    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        T::PossiblyEvaluatedTyp::default()
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate
    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate_feature
    pub fn evaluate_feature(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
    ) -> SymbolLayoutProperties_Evaluated {
        //
        SymbolLayoutProperties_Evaluated
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate4
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

    /// maplibre/maplibre-native#4add9ea original name: evaluate_static
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
    /// maplibre/maplibre-native#4add9ea original name: get
    pub fn get<T: LayoutProperty>(&self) -> T::Type {
        //todo!()
        T::defaultValue()
    }
    /// maplibre/maplibre-native#4add9ea original name: set
    pub fn set<T: LayoutProperty>(&mut self, value: T::Type) {
        // todo!()
    }

    /// maplibre/maplibre-native#4add9ea original name: get_dynamic
    pub fn get_dynamic<T: DataDrivenLayoutProperty>(&self) -> T::PossiblyEvaluatedTyp {
        // todo!()
        T::PossiblyEvaluatedTyp::default()
    }

    /// maplibre/maplibre-native#4add9ea original name: get_eval
    pub fn get_eval<T: DataDrivenLayoutProperty>(&self) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate
    pub fn evaluate<T: DataDrivenLayoutProperty>(
        &self,
        p0: f64,
        p1: &SymbolGeometryTileFeature,
        p2: CanonicalTileID,
    ) -> T::Type {
        //todo!()
        T::defaultValue()
    }

    /// maplibre/maplibre-native#4add9ea original name: evaluate_static
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
