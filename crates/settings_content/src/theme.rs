use collections::{HashMap, IndexMap};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use settings_macros::{MergeFrom, with_fallible_options};
use std::{borrow::Cow, fmt::Display, ops::Deref, sync::Arc};

use crate::serialize_f32_with_two_decimal_places;

/// OpenType 字体特性，以特性标签到值的映射形式表示。
/// 这是一个镜像 `gpui::FontFeatures` 的内容类型，但不含 Arc 包装器。
/// 值可以指定为布尔值（true=1，false=0）或整数
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, MergeFrom)]
#[serde(transparent)]
pub struct FontFeaturesContent(pub IndexMap<String, u32>);

impl FontFeaturesContent {
    pub fn new() -> Self {
        Self(IndexMap::default())
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum FeatureValue {
    Bool(bool),
    Number(serde_json::Number),
}

fn is_valid_feature_tag(tag: &str) -> bool {
    tag.len() == 4 && tag.chars().all(|c| c.is_ascii_alphanumeric())
}

impl<'de> Deserialize<'de> for FontFeaturesContent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{MapAccess, Visitor};
        use std::fmt;

        struct FontFeaturesVisitor;

        impl<'de> Visitor<'de> for FontFeaturesVisitor {
            type Value = FontFeaturesContent;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map of font features")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut feature_map = IndexMap::default();

                while let Some((key, value)) =
                    access.next_entry::<String, Option<FeatureValue>>()?
                {
                    if !is_valid_feature_tag(&key) {
                        log::error!("Incorrect font feature tag: {}", key);
                        continue;
                    }
                    if let Some(value) = value {
                        match value {
                            FeatureValue::Bool(enable) => {
                                feature_map.insert(key, if enable { 1 } else { 0 });
                            }
                            FeatureValue::Number(value) => {
                                if value.is_u64() {
                                    feature_map.insert(key, value.as_u64().unwrap() as u32);
                                } else {
                                    log::error!(
                                        "Incorrect font feature value {} for feature tag {}",
                                        value,
                                        key
                                    );
                                    continue;
                                }
                            }
                        }
                    }
                }

                Ok(FontFeaturesContent(feature_map))
            }
        }

        deserializer.deserialize_map(FontFeaturesVisitor)
    }
}

impl JsonSchema for FontFeaturesContent {
    fn schema_name() -> Cow<'static, str> {
        "FontFeaturesContent".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        use schemars::json_schema;
        json_schema!({
            "type": "object",
            "patternProperties": {
                "[0-9a-zA-Z]{4}$": {
                    "type": ["boolean", "integer"],
                    "minimum": 0,
                    "multipleOf": 1
                }
            },
            "additionalProperties": false
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, MergeFrom)]
#[serde(transparent)]
pub struct ThemeColor(String);

impl JsonSchema for ThemeColor {
    fn schema_name() -> Cow<'static, str> {
        "Color".into()
    }

    fn json_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        use schemars::json_schema;
        json_schema!({
            "type": "string",
            "pattern": "^#([0-9a-fA-F]{3}|[0-9a-fA-F]{4}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$",
            "format": "color"
        })
    }
}

impl From<String> for ThemeColor {
    fn from(value: String) -> Self {
        ThemeColor(value)
    }
}

impl From<&str> for ThemeColor {
    fn from(value: &str) -> Self {
        ThemeColor(value.to_string())
    }
}

impl Deref for ThemeColor {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<ThemeColor> for String {
    fn from(value: ThemeColor) -> Self {
        value.0
    }
}

impl Display for ThemeColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<&ThemeColor> for gpui::Rgba {
    type Error = anyhow::Error;

    fn try_from(value: &ThemeColor) -> anyhow::Result<Self> {
        let s: &str = &value.0;
        gpui::Rgba::try_from(s)
    }
}

/// 用于在 UI 和文本缓冲区中渲染文本的设置

#[with_fallible_options]
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ThemeSettingsContent {
    /// UI 文字的默认字体大小
    pub ui_font_size: Option<FontSize>,
    /// 用于在 UI 中渲染的字体名称
    pub ui_font_family: Option<FontFamilyName>,
    /// UI 渲染时使用的字体回退列表
    #[schemars(default = "default_font_fallbacks")]
    #[schemars(extend("uniqueItems" = true))]
    pub ui_font_fallbacks: Option<Vec<FontFamilyName>>,
    /// 为 UI 文本启用的 OpenType 特性
    #[schemars(default = "default_font_features")]
    pub ui_font_features: Option<FontFeaturesContent>,
    /// UI 字体的粗细，以 CSS 单位表示，范围为 100 到 900
    #[schemars(default = "default_buffer_font_weight")]
    pub ui_font_weight: Option<FontWeightContent>,
    /// 用于在文本缓冲区中渲染的字体名称
    pub buffer_font_family: Option<FontFamilyName>,
    /// 文本缓冲区渲染时使用的字体回退列表
    #[schemars(extend("uniqueItems" = true))]
    pub buffer_font_fallbacks: Option<Vec<FontFamilyName>>,
    /// 文本缓冲区中渲染文本的默认字体大小
    pub buffer_font_size: Option<FontSize>,
    /// 编辑器字体的粗细，以 CSS 单位表示，范围为 100 到 900
    #[schemars(default = "default_buffer_font_weight")]
    pub buffer_font_weight: Option<FontWeightContent>,
    /// 缓冲区的行高
    pub buffer_line_height: Option<BufferLineHeight>,
    /// 在文本缓冲区渲染时启用的 OpenType 特性
    #[schemars(default = "default_font_features")]
    pub buffer_font_features: Option<FontFeaturesContent>,
    /// 智能体面板中智能体回复所用的字体名称。未设置时回退到 UI 字体。
    pub agent_ui_font_family: Option<FontFamilyName>,
    /// 智能体面板中智能体回复的字体大小，未设置时回退到 UI 字体大小
    pub agent_ui_font_size: Option<FontSize>,
    /// 智能体面板中用户消息所用的字体名称。未设置时回退到缓冲区字体。
    pub agent_buffer_font_family: Option<FontFamilyName>,
    /// 智能体面板中用户消息的字体大小
    pub agent_buffer_font_size: Option<FontSize>,
    pub git_commit_buffer_font_size: Option<FontSize>,
    /// 用于渲染智能体面板和
    /// Markdown 预览中 Mermaid 图表的字体名称。未设置时回退到 UI 字体。
    pub mermaid_font_family: Option<FontFamilyName>,
    /// 要使用的 Zed 主题名称
    pub theme: Option<ThemeSelection>,
    /// 要使用的图标主题名称
    pub icon_theme: Option<IconThemeSelection>,

    /// 不稳定：预计许多元素可能损坏
    ///
    // Controls the density of the UI.
    #[serde(rename = "unstable.ui_density")]
    pub ui_density: Option<UiDensity>,

    /// 未使用代码的淡出程度
    #[schemars(range(min = 0.0, max = 0.9))]
    pub unnecessary_code_fade: Option<CodeFade>,

    /// 实验性功能：当前主题的覆盖项
    ///
    /// 这些值将覆盖 `theme` 中指定的当前主题的对应值
    #[serde(rename = "experimental.theme_overrides")]
    pub experimental_theme_overrides: Option<ThemeStyleContent>,

    /// 按主题覆盖
    ///
    /// 这些值将覆盖指定主题的对应值
    #[serde(default)]
    pub theme_overrides: HashMap<String, ThemeStyleContent>,
}

/// 以像素为单位的字体大小值，封装 `f32` 用于自定义设置 UI 渲染
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    PartialOrd,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct FontSize(#[serde(serialize_with = "serialize_f32_with_two_decimal_places")] pub f32);

impl Display for FontSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl From<f32> for FontSize {
    fn from(value: f32) -> Self {
        Self(value)
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    PartialOrd,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct CodeFade(#[serde(serialize_with = "serialize_f32_with_two_decimal_places")] pub f32);

impl Display for CodeFade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl From<f32> for CodeFade {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

fn default_font_features() -> Option<FontFeaturesContent> {
    Some(FontFeaturesContent::default())
}

fn default_font_fallbacks() -> Option<Vec<FontFamilyName>> {
    Some(Vec::new())
}

fn default_buffer_font_weight() -> Option<FontWeightContent> {
    Some(FontWeightContent::NORMAL)
}

/// 表示主题选择，可以是静态或动态类型
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::VariantArray, strum::VariantNames, strum::FromRepr))]
#[serde(untagged)]
pub enum ThemeSelection {
    /// 静态主题选择，由单个主题名称表示
    #[strum_discriminants(strum(serialize = "静态"))]
    Static(ThemeName),
    /// 动态主题选择，可根据 [ThemeMode] 变化
    #[strum_discriminants(strum(serialize = "动态"))]
    Dynamic {
        /// 用于确定所用主题的模式
        #[serde(default)]
        mode: ThemeAppearanceMode,
        /// 浅色模式使用的主题
        light: ThemeName,
        /// 深色模式使用的主题
        dark: ThemeName,
    },
}

pub const DEFAULT_LIGHT_THEME: &'static str = "One Light";
pub const DEFAULT_DARK_THEME: &'static str = "One Dark";

impl Default for ThemeSelection {
    fn default() -> Self {
        Self::Dynamic {
            mode: ThemeAppearanceMode::default(),
            light: ThemeName(DEFAULT_LIGHT_THEME.into()),
            dark: ThemeName(DEFAULT_DARK_THEME.into()),
        }
    }
}

/// 表示图标主题选择，可以是静态或动态类型
#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::VariantArray, strum::VariantNames, strum::FromRepr))]
#[serde(untagged)]
pub enum IconThemeSelection {
    /// 静态图标主题选择，由单个图标主题名称表示
    #[strum_discriminants(strum(serialize = "静态"))]
    Static(IconThemeName),
    /// 动态图标主题选择，可根据 [`ThemeMode`] 变化
    #[strum_discriminants(strum(serialize = "动态"))]
    Dynamic {
        /// 用于确定所用主题的模式
        #[serde(default)]
        mode: ThemeAppearanceMode,
        /// 浅色模式下使用的图标主题
        light: IconThemeName,
        /// 深色模式下使用的图标主题
        dark: IconThemeName,
    },
}

/// 用于选择主题的模式
///
/// `Light` 和 `Dark` 将分别选择对应的主题
///
/// `System` 将根据系统外观选择主题。
#[derive(
    Debug,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ThemeAppearanceMode {
    /// 使用指定的 `light` 主题。
    #[strum(serialize = "浅色")]
    Light,

    /// 使用指定的 `dark` 主题。
    #[strum(serialize = "深色")]
    Dark,

    /// 根据系统外观使用对应主题
    #[default]
    #[strum(serialize = "系统")]
    System,
}

/// 指定 UI 的密度。
/// 注意：此设置仍处于实验阶段。参阅[此跟踪议题](https://github.com/zed-industries/zed/issues/18078)
#[derive(
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum UiDensity {
    /// 更紧凑的 UI，间距更小，元素更小
    #[serde(alias = "compact")]
    Compact,
    #[default]
    #[serde(alias = "default")]
    /// 默认 UI 密度
    Default,
    #[serde(alias = "comfortable")]
    /// 更宽松的 UI，间距更大，元素更大
    Comfortable,
}

impl UiDensity {
    /// 给定密度的间距比例
    /// TODO: Standardize usage throughout the app or remove
    pub fn spacing_ratio(self) -> f32 {
        match self {
            UiDensity::Compact => 0.75,
            UiDensity::Default => 1.0,
            UiDensity::Comfortable => 1.25,
        }
    }
}

/// 字体族名称
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct FontFamilyName(pub Arc<str>);

impl AsRef<str> for FontFamilyName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for FontFamilyName {
    fn from(value: String) -> Self {
        Self(Arc::from(value))
    }
}

impl From<FontFamilyName> for String {
    fn from(value: FontFamilyName) -> Self {
        value.0.to_string()
    }
}

/// 缓冲区的行高
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    JsonSchema,
    MergeFrom,
    Default,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::VariantArray, strum::VariantNames, strum::FromRepr))]
#[serde(rename_all = "snake_case")]
pub enum BufferLineHeight {
    /// 较宽松的行高
    #[default]
    #[strum_discriminants(strum(serialize = "舒适"))]
    Comfortable,
    /// 默认行高
    #[strum_discriminants(strum(serialize = "标准"))]
    Standard,
    /// 自定义行高，1.0 等于字体高度，最小值为 1.0
    #[strum_discriminants(strum(serialize = "自定义"))]
    Custom(#[serde(deserialize_with = "deserialize_line_height")] f32),
}

fn deserialize_line_height<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = f32::deserialize(deserializer)?;
    if value < 1.0 {
        return Err(serde::de::Error::custom(
            "buffer_line_height.custom must be at least 1.0",
        ));
    }

    Ok(value)
}

/// 已序列化主题的内容
#[with_fallible_options]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(default)]
pub struct ThemeStyleContent {
    #[serde(rename = "background.appearance")]
    pub window_background_appearance: Option<WindowBackgroundContent>,

    #[serde(default)]
    pub accents: Vec<AccentContent>,

    #[serde(flatten, default)]
    pub colors: ThemeColorsContent,

    #[serde(flatten, default)]
    pub status: StatusColorsContent,

    #[serde(default)]
    pub players: Vec<PlayerColorContent>,

    /// 语法节点的样式
    #[serde(default)]
    pub syntax: IndexMap<String, HighlightStyleContent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct AccentContent(pub Option<ThemeColor>);

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct PlayerColorContent {
    pub cursor: Option<ThemeColor>,
    pub background: Option<ThemeColor>,
    pub selection: Option<ThemeColor>,
}

/// 主题名称
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct ThemeName(pub Arc<str>);

/// 图标主题名称
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct IconThemeName(pub Arc<str>);

#[with_fallible_options]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(default)]
pub struct ThemeColorsContent {
    /// 边框颜色，用于大多数边框，通常为高对比度颜色
    #[serde(rename = "border")]
    pub border: Option<ThemeColor>,

    /// 边框颜色，用于弱化边框，如两个区域之间的视觉分隔线
    #[serde(rename = "border.variant")]
    pub border_variant: Option<ThemeColor>,

    /// 边框颜色，用于已聚焦元素，如键盘聚焦的列表项
    #[serde(rename = "border.focused")]
    pub border_focused: Option<ThemeColor>,

    /// 边框颜色，用于已选中元素，如激活的搜索筛选器或已选中的复选框
    #[serde(rename = "border.selected")]
    pub border_selected: Option<ThemeColor>,

    /// 边框颜色，用于透明边框。在元素因状态变化而获得边框时用作占位边框。
    #[serde(rename = "border.transparent")]
    pub border_transparent: Option<ThemeColor>,

    /// 边框颜色，用于已禁用元素，如禁用的输入框或按钮
    #[serde(rename = "border.disabled")]
    pub border_disabled: Option<ThemeColor>,

    /// 背景色。用于上下文菜单、弹出框或对话框等浮层表面。
    #[serde(rename = "elevated_surface.background")]
    pub elevated_surface_background: Option<ThemeColor>,

    /// 背景色。用于面板或标签页等基础表面。
    #[serde(rename = "surface.background")]
    pub surface_background: Option<ThemeColor>,

    /// 背景色。用于应用程序背景及空白面板或窗口。
    #[serde(rename = "background")]
    pub background: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景不同的元素的背景。
    ///
    /// 元素可能包括：按钮、输入框、复选框、单选按钮等
    ///
    /// 对于背景应与所在表面相同的元素，使用 `ghost_element_background`
    #[serde(rename = "element.background")]
    pub element_background: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景不同的元素的悬停状态。
    ///
    /// 悬停状态由鼠标进入元素或手指触摸触屏上的元素触发
    #[serde(rename = "element.hover")]
    pub element_hover: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景不同的元素的活动状态。
    ///
    /// 活动状态由鼠标按钮按下元素或按下 Return 键或其他激活键触发
    #[serde(rename = "element.active")]
    pub element_active: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景不同的元素的选中状态。
    ///
    /// 当用户选中（或“激活”）元素时，会触发选中状态
    ///
    /// 这可能包括已选中的复选框、处于开启状态的可切换按钮等
    #[serde(rename = "element.selected")]
    pub element_selected: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景不同的元素的禁用状态。
    ///
    /// 当用户无法与元素（例如禁用的按钮或输入框）交互时，显示禁用状态
    #[serde(rename = "element.disabled")]
    pub element_disabled: Option<ThemeColor>,

    /// 背景色。用于 UI 元素中选中区域的背景。
    #[serde(rename = "element.selection_background")]
    pub element_selection_background: Option<ThemeColor>,

    /// 背景色。用于显示拖拽元素放置目标区域。
    #[serde(rename = "drop_target.background")]
    pub drop_target_background: Option<ThemeColor>,

    /// 边框颜色，用于显示拖动元素放置位置的边框
    #[serde(rename = "drop_target.border")]
    pub drop_target_border: Option<ThemeColor>,

    /// 用于与所在表面背景相同的幽灵元素的背景色
    ///
    /// 元素可能包括：按钮、输入框、复选框、单选按钮等
    ///
    /// 对于背景应与所在表面不同的元素，使用 `element_background`
    #[serde(rename = "ghost_element.background")]
    pub ghost_element_background: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景相同的幽灵元素的悬停状态。
    ///
    /// 悬停状态由鼠标进入元素或手指触摸触屏上的元素触发
    #[serde(rename = "ghost_element.hover")]
    pub ghost_element_hover: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景相同的幽灵元素的活动状态。
    ///
    /// 活动状态由鼠标按钮按下元素或按下 Return 键或其他激活键触发
    #[serde(rename = "ghost_element.active")]
    pub ghost_element_active: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景相同的幽灵元素的选中状态。
    ///
    /// 当用户选中（或“激活”）元素时，会触发选中状态
    ///
    /// 这可能包括已选中的复选框、处于开启状态的可切换按钮等
    #[serde(rename = "ghost_element.selected")]
    pub ghost_element_selected: Option<ThemeColor>,

    /// 背景色。用于与所在表面背景相同的幽灵元素的禁用状态。
    ///
    /// 当用户无法与元素（例如禁用的按钮或输入框）交互时，显示禁用状态
    #[serde(rename = "ghost_element.disabled")]
    pub ghost_element_disabled: Option<ThemeColor>,

    /// 文本颜色。大多数文本使用的默认文本颜色。
    #[serde(rename = "text")]
    pub text: Option<ThemeColor>,

    /// 文本颜色。弱化或淡化文本的颜色，是标准文本颜色的柔和版本
    #[serde(rename = "text.muted")]
    pub text_muted: Option<ThemeColor>,

    /// 文本颜色。通常显示在输入框中的占位文本颜色，用于引导用户输入有效数据
    #[serde(rename = "text.placeholder")]
    pub text_placeholder: Option<ThemeColor>,

    /// 文本颜色。用于表示禁用元素的文本颜色，通常为褪色或灰色以强调禁用状态
    #[serde(rename = "text.disabled")]
    pub text_disabled: Option<ThemeColor>,

    /// 文本颜色。用于强调或高亮特定文本的颜色，如活动筛选器或搜索中匹配的字符
    #[serde(rename = "text.accent")]
    pub text_accent: Option<ThemeColor>,

    /// 填充颜色，用于图标的默认填充色
    #[serde(rename = "icon")]
    pub icon: Option<ThemeColor>,

    /// 填充颜色，用于图标的弱化或淡化填充色
    ///
    /// 可用于在非活动窗格中显示图标，或弱化一组图标以降低其视觉权重。
    #[serde(rename = "icon.muted")]
    pub icon_muted: Option<ThemeColor>,

    /// 填充颜色，用于图标的禁用填充色
    ///
    /// 当用户无法与元素（例如图标按钮）交互时，显示禁用状态
    #[serde(rename = "icon.disabled")]
    pub icon_disabled: Option<ThemeColor>,

    /// 填充颜色，用于图标的占位符填充色
    ///
    /// 可用于在输入框中显示一个图标，该图标在用户输入文本时消失。
    #[serde(rename = "icon.placeholder")]
    pub icon_placeholder: Option<ThemeColor>,

    /// 填充颜色，用于图标的强调填充色
    ///
    /// 可用于显示可切换图标按钮被选中的状态。
    #[serde(rename = "icon.accent")]
    pub icon_accent: Option<ThemeColor>,

    /// 用于突出显示调试器部分元素的颜色
    /// 目前仅对断点及断点相关符号应用强调色
    #[serde(rename = "debugger.accent")]
    pub debugger_accent: Option<ThemeColor>,

    #[serde(rename = "status_bar.background")]
    pub status_bar_background: Option<ThemeColor>,

    #[serde(rename = "title_bar.background")]
    pub title_bar_background: Option<ThemeColor>,

    #[serde(rename = "title_bar.inactive_background")]
    pub title_bar_inactive_background: Option<ThemeColor>,

    #[serde(rename = "toolbar.background")]
    pub toolbar_background: Option<ThemeColor>,

    #[serde(rename = "tab_bar.background")]
    pub tab_bar_background: Option<ThemeColor>,

    #[serde(rename = "tab.inactive_background")]
    pub tab_inactive_background: Option<ThemeColor>,

    #[serde(rename = "tab.active_background")]
    pub tab_active_background: Option<ThemeColor>,

    #[serde(rename = "search.match_background")]
    pub search_match_background: Option<ThemeColor>,

    #[serde(rename = "search.active_match_background")]
    pub search_active_match_background: Option<ThemeColor>,

    #[serde(rename = "panel.background")]
    pub panel_background: Option<ThemeColor>,

    #[serde(rename = "panel.focused_border")]
    pub panel_focused_border: Option<ThemeColor>,

    #[serde(rename = "panel.indent_guide")]
    pub panel_indent_guide: Option<ThemeColor>,

    #[serde(rename = "panel.indent_guide_hover")]
    pub panel_indent_guide_hover: Option<ThemeColor>,

    #[serde(rename = "panel.indent_guide_active")]
    pub panel_indent_guide_active: Option<ThemeColor>,

    #[serde(rename = "panel.overlay_background")]
    pub panel_overlay_background: Option<ThemeColor>,

    #[serde(rename = "panel.overlay_hover")]
    pub panel_overlay_hover: Option<ThemeColor>,

    #[serde(rename = "pane.focused_border")]
    pub pane_focused_border: Option<ThemeColor>,

    #[serde(rename = "pane_group.border")]
    pub pane_group_border: Option<ThemeColor>,

    /// `scrollbar.thumb.background` 的已弃用版本
    ///
    /// 不要使用此字段
    #[serde(rename = "scrollbar_thumb.background", skip_serializing)]
    #[schemars(skip)]
    pub deprecated_scrollbar_thumb_background: Option<ThemeColor>,

    /// 滚动条滑块的颜色
    #[serde(rename = "scrollbar.thumb.background")]
    pub scrollbar_thumb_background: Option<ThemeColor>,

    /// 悬停时滚动条滑块的颜色
    #[serde(rename = "scrollbar.thumb.hover_background")]
    pub scrollbar_thumb_hover_background: Option<ThemeColor>,

    /// 拖动时滚动条滑块的颜色
    #[serde(rename = "scrollbar.thumb.active_background")]
    pub scrollbar_thumb_active_background: Option<ThemeColor>,

    /// 滚动条滑块的边框颜色
    #[serde(rename = "scrollbar.thumb.border")]
    pub scrollbar_thumb_border: Option<ThemeColor>,

    /// 滚动条轨道的背景颜色
    #[serde(rename = "scrollbar.track.background")]
    pub scrollbar_track_background: Option<ThemeColor>,

    /// 滚动条轨道的边框颜色
    #[serde(rename = "scrollbar.track.border")]
    pub scrollbar_track_border: Option<ThemeColor>,

    /// 小地图滑块的颜色
    #[serde(rename = "minimap.thumb.background")]
    pub minimap_thumb_background: Option<ThemeColor>,

    /// 悬停时小地图滑块的颜色
    #[serde(rename = "minimap.thumb.hover_background")]
    pub minimap_thumb_hover_background: Option<ThemeColor>,

    /// 拖动时小地图滑块的颜色
    #[serde(rename = "minimap.thumb.active_background")]
    pub minimap_thumb_active_background: Option<ThemeColor>,

    /// 小地图滑块的边框颜色
    #[serde(rename = "minimap.thumb.border")]
    pub minimap_thumb_border: Option<ThemeColor>,

    #[serde(rename = "editor.foreground")]
    pub editor_foreground: Option<ThemeColor>,

    /// 编辑器中 Code Lens 项使用的文本颜色
    #[serde(rename = "editor.code_lens.foreground")]
    pub editor_code_lens_foreground: Option<ThemeColor>,

    #[serde(rename = "editor.background")]
    pub editor_background: Option<ThemeColor>,

    #[serde(rename = "editor.gutter.background")]
    pub editor_gutter_background: Option<ThemeColor>,

    #[serde(rename = "editor.subheader.background")]
    pub editor_subheader_background: Option<ThemeColor>,

    #[serde(rename = "editor.active_line.background")]
    pub editor_active_line_background: Option<ThemeColor>,

    #[serde(rename = "editor.highlighted_line.background")]
    pub editor_highlighted_line_background: Option<ThemeColor>,

    /// 调试器活动行的背景
    #[serde(rename = "editor.debugger_active_line.background")]
    pub editor_debugger_active_line_background: Option<ThemeColor>,

    /// 文本颜色。编辑器装订线中行号的文本颜色
    #[serde(rename = "editor.line_number")]
    pub editor_line_number: Option<ThemeColor>,

    /// 文本颜色。编辑器装订线中高亮行的行号文本颜色
    #[serde(rename = "editor.active_line_number")]
    pub editor_active_line_number: Option<ThemeColor>,

    /// 文本颜色。编辑器装订线中鼠标悬停行的行号文本颜色
    #[serde(rename = "editor.hover_line_number")]
    pub editor_hover_line_number: Option<ThemeColor>,

    /// 文本颜色。用于标记编辑器中不可见字符的颜色
    ///
    /// 示例：空格、制表符、回车符等
    #[serde(rename = "editor.invisible")]
    pub editor_invisible: Option<ThemeColor>,

    #[serde(rename = "editor.wrap_guide")]
    pub editor_wrap_guide: Option<ThemeColor>,

    #[serde(rename = "editor.active_wrap_guide")]
    pub editor_active_wrap_guide: Option<ThemeColor>,

    #[serde(rename = "editor.indent_guide")]
    pub editor_indent_guide: Option<ThemeColor>,

    #[serde(rename = "editor.indent_guide_active")]
    pub editor_indent_guide_active: Option<ThemeColor>,

    /// 符号的读取访问，例如读取变量
    ///
    /// 文档高亮是文本文档中值得
    /// 特别关注的范围。通常，文档高亮的呈现方式是更改
    /// 其范围的背景色。
    #[serde(rename = "editor.document_highlight.read_background")]
    pub editor_document_highlight_read_background: Option<ThemeColor>,

    /// 符号的读取访问，例如读取变量
    ///
    /// 文档高亮是文本文档中值得
    /// 特别关注的范围。通常，文档高亮的呈现方式是更改
    /// 其范围的背景色。
    #[serde(rename = "editor.document_highlight.write_background")]
    pub editor_document_highlight_write_background: Option<ThemeColor>,

    /// 高亮括号的背景颜色
    ///
    /// 光标范围内匹配的括号将以此背景颜色高亮显示
    #[serde(rename = "editor.document_highlight.bracket_background")]
    pub editor_document_highlight_bracket_background: Option<ThemeColor>,

    /// 编辑器中已添加差异区块行高亮的填充背景色
    #[serde(rename = "editor.diff_hunk.added.background")]
    pub editor_diff_hunk_added_background: Option<ThemeColor>,

    /// 编辑器中已添加差异区块行高亮的空心背景色
    #[serde(rename = "editor.diff_hunk.added.hollow_background")]
    pub editor_diff_hunk_added_hollow_background: Option<ThemeColor>,

    /// 编辑器中已添加差异区块行高亮的空心边框色
    #[serde(rename = "editor.diff_hunk.added.hollow_border")]
    pub editor_diff_hunk_added_hollow_border: Option<ThemeColor>,

    /// 编辑器中已删除差异区块行高亮的填充背景色
    #[serde(rename = "editor.diff_hunk.deleted.background")]
    pub editor_diff_hunk_deleted_background: Option<ThemeColor>,

    /// 编辑器中已删除差异区块行高亮的空心背景色
    #[serde(rename = "editor.diff_hunk.deleted.hollow_background")]
    pub editor_diff_hunk_deleted_hollow_background: Option<ThemeColor>,

    /// 编辑器中已删除差异区块行高亮的空心边框色
    #[serde(rename = "editor.diff_hunk.deleted.hollow_border")]
    pub editor_diff_hunk_deleted_hollow_border: Option<ThemeColor>,

    /// 终端背景颜色
    #[serde(rename = "terminal.background")]
    pub terminal_background: Option<ThemeColor>,

    /// 终端前景颜色
    #[serde(rename = "terminal.foreground")]
    pub terminal_foreground: Option<ThemeColor>,

    /// 终端 ANSI 背景颜色
    #[serde(rename = "terminal.ansi.background")]
    pub terminal_ansi_background: Option<ThemeColor>,

    /// 亮色终端前景颜色
    #[serde(rename = "terminal.bright_foreground")]
    pub terminal_bright_foreground: Option<ThemeColor>,

    /// 暗淡终端前景色
    #[serde(rename = "terminal.dim_foreground")]
    pub terminal_dim_foreground: Option<ThemeColor>,

    /// 黑色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.black")]
    pub terminal_ansi_black: Option<ThemeColor>,

    /// 亮黑色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_black")]
    pub terminal_ansi_bright_black: Option<ThemeColor>,

    /// 暗黑色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_black")]
    pub terminal_ansi_dim_black: Option<ThemeColor>,

    /// 红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.red")]
    pub terminal_ansi_red: Option<ThemeColor>,

    /// 亮红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_red")]
    pub terminal_ansi_bright_red: Option<ThemeColor>,

    /// 暗红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_red")]
    pub terminal_ansi_dim_red: Option<ThemeColor>,

    /// 绿色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.green")]
    pub terminal_ansi_green: Option<ThemeColor>,

    /// 亮绿色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_green")]
    pub terminal_ansi_bright_green: Option<ThemeColor>,

    /// 暗绿色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_green")]
    pub terminal_ansi_dim_green: Option<ThemeColor>,

    /// 黄色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.yellow")]
    pub terminal_ansi_yellow: Option<ThemeColor>,

    /// 亮黄色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_yellow")]
    pub terminal_ansi_bright_yellow: Option<ThemeColor>,

    /// 暗黄色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_yellow")]
    pub terminal_ansi_dim_yellow: Option<ThemeColor>,

    /// 蓝色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.blue")]
    pub terminal_ansi_blue: Option<ThemeColor>,

    /// 亮蓝色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_blue")]
    pub terminal_ansi_bright_blue: Option<ThemeColor>,

    /// 暗蓝色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_blue")]
    pub terminal_ansi_dim_blue: Option<ThemeColor>,

    /// 品红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.magenta")]
    pub terminal_ansi_magenta: Option<ThemeColor>,

    /// 亮品红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_magenta")]
    pub terminal_ansi_bright_magenta: Option<ThemeColor>,

    /// 暗品红色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_magenta")]
    pub terminal_ansi_dim_magenta: Option<ThemeColor>,

    /// 青色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.cyan")]
    pub terminal_ansi_cyan: Option<ThemeColor>,

    /// 亮青色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_cyan")]
    pub terminal_ansi_bright_cyan: Option<ThemeColor>,

    /// 暗青色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_cyan")]
    pub terminal_ansi_dim_cyan: Option<ThemeColor>,

    /// 白色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.white")]
    pub terminal_ansi_white: Option<ThemeColor>,

    /// 亮白色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.bright_white")]
    pub terminal_ansi_bright_white: Option<ThemeColor>,

    /// 暗淡白色 ANSI 终端颜色
    #[serde(rename = "terminal.ansi.dim_white")]
    pub terminal_ansi_dim_white: Option<ThemeColor>,

    #[serde(rename = "link_text.hover")]
    pub link_text_hover: Option<ThemeColor>,

    /// 版本控制中新增状态的颜色
    #[serde(rename = "version_control.added")]
    pub version_control_added: Option<ThemeColor>,

    /// 版本控制中删除状态的颜色
    #[serde(rename = "version_control.deleted")]
    pub version_control_deleted: Option<ThemeColor>,

    /// 修改状态的版本控制颜色
    #[serde(rename = "version_control.modified")]
    pub version_control_modified: Option<ThemeColor>,

    /// 版本控制中重命名状态的颜色
    #[serde(rename = "version_control.renamed")]
    pub version_control_renamed: Option<ThemeColor>,

    /// 版本控制冲突颜色
    #[serde(rename = "version_control.conflict")]
    pub version_control_conflict: Option<ThemeColor>,

    /// 已忽略状态的版本控制颜色。
    #[serde(rename = "version_control.ignored")]
    pub version_control_ignored: Option<ThemeColor>,

    /// 单词差异中新增单词的颜色
    #[serde(rename = "version_control.word_added")]
    pub version_control_word_added: Option<ThemeColor>,

    /// 单词差异中删除单词的颜色
    #[serde(rename = "version_control.word_deleted")]
    pub version_control_word_deleted: Option<ThemeColor>,

    /// 合并冲突中“我方”区域行高亮的背景色。
    #[serde(rename = "version_control.conflict_marker.ours")]
    pub version_control_conflict_marker_ours: Option<ThemeColor>,

    /// 合并冲突中“对方”区域行高亮的背景色。
    #[serde(rename = "version_control.conflict_marker.theirs")]
    pub version_control_conflict_marker_theirs: Option<ThemeColor>,

    /// 已弃用，改用 `version_control_conflict_marker_ours`
    #[deprecated]
    pub version_control_conflict_ours_background: Option<ThemeColor>,

    /// 已弃用，改用 `version_control_conflict_marker_theirs`
    #[deprecated]
    pub version_control_conflict_theirs_background: Option<ThemeColor>,

    /// Vim 普通模式指示器的背景色。
    #[serde(rename = "vim.normal.background")]
    pub vim_normal_background: Option<ThemeColor>,
    /// Vim 插入模式指示器的背景色。
    #[serde(rename = "vim.insert.background")]
    pub vim_insert_background: Option<ThemeColor>,
    /// Vim 替换模式指示器的背景色。
    #[serde(rename = "vim.replace.background")]
    pub vim_replace_background: Option<ThemeColor>,
    /// Vim 可视模式指示器的背景色。
    #[serde(rename = "vim.visual.background")]
    pub vim_visual_background: Option<ThemeColor>,
    /// Vim 可视行模式指示器的背景色。
    #[serde(rename = "vim.visual_line.background")]
    pub vim_visual_line_background: Option<ThemeColor>,
    /// Vim 可视块模式指示器的背景色。
    #[serde(rename = "vim.visual_block.background")]
    pub vim_visual_block_background: Option<ThemeColor>,
    /// Vim 复制高亮的背景色。
    #[serde(rename = "vim.yank.background")]
    pub vim_yank_background: Option<ThemeColor>,
    /// Helix 跳转标签的前景色
    #[serde(rename = "vim.helix_jump_label.foreground")]
    pub vim_helix_jump_label_foreground: Option<ThemeColor>,
    /// Vim Helix 普通模式指示器的背景色。
    #[serde(rename = "vim.helix_normal.background")]
    pub vim_helix_normal_background: Option<ThemeColor>,
    /// Vim Helix 选择模式指示器的背景色。
    #[serde(rename = "vim.helix_select.background")]
    pub vim_helix_select_background: Option<ThemeColor>,
    /// Vim 普通模式指示器的背景色。
    #[serde(rename = "vim.normal.foreground")]
    pub vim_normal_foreground: Option<ThemeColor>,
    /// Vim 插入模式指示器的前景色
    #[serde(rename = "vim.insert.foreground")]
    pub vim_insert_foreground: Option<ThemeColor>,
    /// Vim 替换模式指示器的前景色
    #[serde(rename = "vim.replace.foreground")]
    pub vim_replace_foreground: Option<ThemeColor>,
    /// Vim 可视模式指示器的前景色
    #[serde(rename = "vim.visual.foreground")]
    pub vim_visual_foreground: Option<ThemeColor>,
    /// Vim 可视行模式指示器的前景色
    #[serde(rename = "vim.visual_line.foreground")]
    pub vim_visual_line_foreground: Option<ThemeColor>,
    /// Vim 可视块模式指示器的前景色
    #[serde(rename = "vim.visual_block.foreground")]
    pub vim_visual_block_foreground: Option<ThemeColor>,
    /// Vim Helix 普通模式指示器的前景色
    #[serde(rename = "vim.helix_normal.foreground")]
    pub vim_helix_normal_foreground: Option<ThemeColor>,
    /// Vim Helix 选择模式指示器的前景色
    #[serde(rename = "vim.helix_select.foreground")]
    pub vim_helix_select_foreground: Option<ThemeColor>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(default)]
pub struct HighlightStyleContent {
    pub color: Option<ThemeColor>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub background_color: Option<ThemeColor>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub font_style: Option<FontStyleContent>,

    #[serde(
        skip_serializing_if = "Option::is_none",
        deserialize_with = "treat_error_as_none"
    )]
    pub font_weight: Option<FontWeightContent>,
}

impl HighlightStyleContent {
    pub fn is_empty(&self) -> bool {
        self.color.is_none()
            && self.background_color.is_none()
            && self.font_style.is_none()
            && self.font_weight.is_none()
    }
}

fn treat_error_as_none<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    Ok(T::deserialize(value).ok())
}

#[with_fallible_options]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(default)]
pub struct StatusColorsContent {
    /// 表示某种冲突，如文件在打开时磁盘上已被修改，或
    /// Git 仓库中的合并冲突。
    #[serde(rename = "conflict")]
    pub conflict: Option<ThemeColor>,

    #[serde(rename = "conflict.background")]
    pub conflict_background: Option<ThemeColor>,

    #[serde(rename = "conflict.border")]
    pub conflict_border: Option<ThemeColor>,

    /// 表示新建内容，如添加到 Git 仓库的新文件。
    #[serde(rename = "created")]
    pub created: Option<ThemeColor>,

    #[serde(rename = "created.background")]
    pub created_background: Option<ThemeColor>,

    #[serde(rename = "created.border")]
    pub created_border: Option<ThemeColor>,

    /// 表示某内容已不存在，如已删除的文件。
    #[serde(rename = "deleted")]
    pub deleted: Option<ThemeColor>,

    #[serde(rename = "deleted.background")]
    pub deleted_background: Option<ThemeColor>,

    #[serde(rename = "deleted.border")]
    pub deleted_border: Option<ThemeColor>,

    /// 表示系统错误、操作失败或诊断错误。
    #[serde(rename = "error")]
    pub error: Option<ThemeColor>,

    #[serde(rename = "error.background")]
    pub error_background: Option<ThemeColor>,

    #[serde(rename = "error.border")]
    pub error_border: Option<ThemeColor>,

    /// 表示隐藏状态，例如文件树中被隐藏的文件
    #[serde(rename = "hidden")]
    pub hidden: Option<ThemeColor>,

    #[serde(rename = "hidden.background")]
    pub hidden_background: Option<ThemeColor>,

    #[serde(rename = "hidden.border")]
    pub hidden_border: Option<ThemeColor>,

    /// 表示提示或某种附加信息。
    #[serde(rename = "hint")]
    pub hint: Option<ThemeColor>,

    #[serde(rename = "hint.background")]
    pub hint_background: Option<ThemeColor>,

    #[serde(rename = "hint.border")]
    pub hint_border: Option<ThemeColor>,

    /// 表示被主动忽略的内容，如被 Git 忽略的文件或操作。
    #[serde(rename = "ignored")]
    pub ignored: Option<ThemeColor>,

    #[serde(rename = "ignored.background")]
    pub ignored_background: Option<ThemeColor>,

    #[serde(rename = "ignored.border")]
    pub ignored_border: Option<ThemeColor>,

    /// 表示信息类状态更新或消息
    #[serde(rename = "info")]
    pub info: Option<ThemeColor>,

    #[serde(rename = "info.background")]
    pub info_background: Option<ThemeColor>,

    #[serde(rename = "info.border")]
    pub info_border: Option<ThemeColor>,

    /// 表示已更改或修改的状态，如已编辑的文件。
    #[serde(rename = "modified")]
    pub modified: Option<ThemeColor>,

    #[serde(rename = "modified.background")]
    pub modified_background: Option<ThemeColor>,

    #[serde(rename = "modified.border")]
    pub modified_border: Option<ThemeColor>,

    /// 表示预测性内容，如自动代码补全或生成的代码。
    #[serde(rename = "predictive")]
    pub predictive: Option<ThemeColor>,

    #[serde(rename = "predictive.background")]
    pub predictive_background: Option<ThemeColor>,

    #[serde(rename = "predictive.border")]
    pub predictive_border: Option<ThemeColor>,

    /// 表示重命名状态，例如已被重命名的文件
    #[serde(rename = "renamed")]
    pub renamed: Option<ThemeColor>,

    #[serde(rename = "renamed.background")]
    pub renamed_background: Option<ThemeColor>,

    #[serde(rename = "renamed.border")]
    pub renamed_border: Option<ThemeColor>,

    /// 表示操作成功或任务完成。
    #[serde(rename = "success")]
    pub success: Option<ThemeColor>,

    #[serde(rename = "success.background")]
    pub success_background: Option<ThemeColor>,

    #[serde(rename = "success.border")]
    pub success_border: Option<ThemeColor>,

    /// 表示某种不可达状态，如永远无法执行到的代码块。
    #[serde(rename = "unreachable")]
    pub unreachable: Option<ThemeColor>,

    #[serde(rename = "unreachable.background")]
    pub unreachable_background: Option<ThemeColor>,

    #[serde(rename = "unreachable.border")]
    pub unreachable_border: Option<ThemeColor>,

    /// 表示警告状态，例如即将失败的操作
    #[serde(rename = "warning")]
    pub warning: Option<ThemeColor>,

    #[serde(rename = "warning.background")]
    pub warning_background: Option<ThemeColor>,

    #[serde(rename = "warning.border")]
    pub warning_border: Option<ThemeColor>,
}

/// 窗口的背景外观
#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum WindowBackgroundContent {
    Opaque,
    Transparent,
    Blurred,
    MicaBackdrop,
    MicaAltBackdrop,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FontStyleContent {
    Normal,
    Italic,
    Oblique,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    PartialOrd,
    Serialize,
    Deserialize,
    MergeFrom,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct FontWeightContent(pub f32);

impl Display for FontWeightContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<f32> for FontWeightContent {
    fn from(weight: f32) -> Self {
        FontWeightContent(weight)
    }
}

impl Default for FontWeightContent {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl FontWeightContent {
    pub const THIN: FontWeightContent = FontWeightContent(100.0);
    pub const EXTRA_LIGHT: FontWeightContent = FontWeightContent(200.0);
    pub const LIGHT: FontWeightContent = FontWeightContent(300.0);
    pub const NORMAL: FontWeightContent = FontWeightContent(400.0);
    pub const MEDIUM: FontWeightContent = FontWeightContent(500.0);
    pub const SEMIBOLD: FontWeightContent = FontWeightContent(600.0);
    pub const BOLD: FontWeightContent = FontWeightContent(700.0);
    pub const EXTRA_BOLD: FontWeightContent = FontWeightContent(800.0);
    pub const BLACK: FontWeightContent = FontWeightContent(900.0);
}

impl schemars::JsonSchema for FontWeightContent {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FontWeightContent".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        use schemars::json_schema;
        json_schema!({
            "type": "number",
            "minimum": Self::THIN.0,
            "maximum": Self::BLACK.0,
            "default": Self::NORMAL.0,
            "description": "Font weight value between 100 (thin) and 900 (black)"
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_buffer_line_height_deserialize_valid() {
        assert_eq!(
            serde_json::from_value::<BufferLineHeight>(json!("comfortable")).unwrap(),
            BufferLineHeight::Comfortable
        );
        assert_eq!(
            serde_json::from_value::<BufferLineHeight>(json!("standard")).unwrap(),
            BufferLineHeight::Standard
        );
        assert_eq!(
            serde_json::from_value::<BufferLineHeight>(json!({"custom": 1.0})).unwrap(),
            BufferLineHeight::Custom(1.0)
        );
        assert_eq!(
            serde_json::from_value::<BufferLineHeight>(json!({"custom": 1.5})).unwrap(),
            BufferLineHeight::Custom(1.5)
        );
    }

    #[test]
    fn test_buffer_line_height_deserialize_invalid() {
        assert!(
            serde_json::from_value::<BufferLineHeight>(json!({"custom": 0.99}))
                .err()
                .unwrap()
                .to_string()
                .contains("buffer_line_height.custom must be at least 1.0")
        );
        assert!(
            serde_json::from_value::<BufferLineHeight>(json!({"custom": 0.0}))
                .err()
                .unwrap()
                .to_string()
                .contains("buffer_line_height.custom must be at least 1.0")
        );
        assert!(
            serde_json::from_value::<BufferLineHeight>(json!({"custom": -1.0}))
                .err()
                .unwrap()
                .to_string()
                .contains("buffer_line_height.custom must be at least 1.0")
        );
    }

    #[test]
    fn test_buffer_font_weight_schema_has_default() {
        use schemars::schema_for;

        let schema = schema_for!(ThemeSettingsContent);
        let schema_value = serde_json::to_value(&schema).unwrap();

        let properties = &schema_value["properties"];
        let buffer_font_weight = &properties["buffer_font_weight"];

        assert!(
            buffer_font_weight.get("default").is_some(),
            "buffer_font_weight should have a default value in the schema"
        );

        let default_value = &buffer_font_weight["default"];
        assert_eq!(
            default_value.as_f64(),
            Some(FontWeightContent::NORMAL.0 as f64),
            "buffer_font_weight default should be 400.0 (FontWeightContent::NORMAL)"
        );

        let defs = &schema_value["$defs"];
        let font_weight_def = &defs["FontWeightContent"];

        assert_eq!(
            font_weight_def["minimum"].as_f64(),
            Some(FontWeightContent::THIN.0 as f64),
            "FontWeightContent should have minimum of 100.0"
        );
        assert_eq!(
            font_weight_def["maximum"].as_f64(),
            Some(FontWeightContent::BLACK.0 as f64),
            "FontWeightContent should have maximum of 900.0"
        );
        assert_eq!(
            font_weight_def["default"].as_f64(),
            Some(FontWeightContent::NORMAL.0 as f64),
            "FontWeightContent should have default of 400.0"
        );
    }
}
