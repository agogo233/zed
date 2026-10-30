use std::fmt::Display;
use std::num;

use collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

use crate::{
    DelayMs, DiagnosticSeverityContent, ShowScrollbar, serialize_f32_with_two_decimal_places,
};

#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct EditorSettingsContent {
    /// 编辑器中光标是否闪烁
    ///
    /// 默认值：true
    pub cursor_blink: Option<bool>,
    /// 默认编辑器的光标形状
    /// 可以是 "bar"、"block"、"underline" 或 "hollow"。
    ///
    /// 默认值：bar
    pub cursor_shape: Option<CursorShape>,
    /// 光标移动动画设置
    pub cursor_animation: Option<CursorAnimationSettingsContent>,
    /// 确定代码片段相对于其他补全项的排序方式
    ///
    /// 默认值：inline
    pub snippet_sort_order: Option<SnippetSortOrder>,
    /// 当前行在编辑器中的高亮方式
    ///
    /// 默认值：all
    pub current_line_highlight: Option<CurrentLineHighlight>,
    /// 是否在编辑器中高亮显示所选文本的所有出现位置
    ///
    /// 默认值：true
    pub selection_highlight: Option<bool>,
    /// 文本选区是否使用圆角
    ///
    /// 默认值：true
    pub rounded_selection: Option<bool>,
    /// 从语言服务器查询高亮之前的防抖延迟
    /// （基于当前光标位置）。
    ///
    /// 默认值：75
    pub lsp_highlight_debounce: Option<DelayMs>,
    /// 将鼠标移动到符号上时是否显示信息悬停框
    /// （在编辑器中）。
    ///
    /// 默认值：true
    pub hover_popover_enabled: Option<bool>,
    /// 显示信息悬停框前等待的毫秒数。
    /// 当启用 `auto_signature_help` 时，此延迟也适用于自动签名帮助
    ///
    /// 默认值：300
    pub hover_popover_delay: Option<DelayMs>,
    /// 鼠标移向悬停弹出框时是否保持显示，
    /// 允许在悬停框消失前与其内容进行交互
    ///
    /// 默认值：true
    pub hover_popover_sticky: Option<bool>,
    /// 鼠标移离悬停目标后，隐藏悬停弹出框前
    /// 需要等待的毫秒数。
    /// 仅在启用 `hover_popover_sticky` 时生效。
    ///
    /// 默认值：300
    pub hover_popover_hiding_delay: Option<DelayMs>,
    /// 工具栏相关设置
    pub toolbar: Option<ToolbarContent>,
    /// 滚动条相关设置
    pub scrollbar: Option<ScrollbarContent>,
    /// 小地图相关设置
    pub minimap: Option<MinimapContent>,
    /// 装订线相关设置
    pub gutter: Option<GutterContent>,
    /// 编辑器是否可以滚动到最后一行之后
    ///
    /// 默认值：one_page
    pub scroll_beyond_last_line: Option<ScrollBeyondLastLine>,
    /// 自动滚动时光标上方/下方保留的行数
    ///
    /// 默认值：3.
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub vertical_scroll_margin: Option<f32>,
    /// 点击可见文本区域边缘附近时是否滚动
    ///
    /// 默认值：false
    pub autoscroll_on_clicks: Option<bool>,
    /// 使用鼠标滚动时两侧保留的字符数
    ///
    /// 默认值：5.
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub horizontal_scroll_margin: Option<f32>,
    /// 滚动灵敏度倍数。滚动时，此倍数同时应用于
    /// 水平和垂直方向的增量值
    ///
    /// 默认值：1.0
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub scroll_sensitivity: Option<f32>,
    /// 是否在
    /// 按住主修饰键（macOS 上为 Cmd，其他平台上为 Ctrl）时使用鼠标滚轮缩放编辑器字体大小
    ///
    /// 默认值：false
    pub mouse_wheel_zoom: Option<bool>,
    /// 快速滚动的灵敏度倍数。滚动时，此倍数同时应用于
    /// 水平和垂直方向的增量值。
    /// 按住 Alt 或 Option 键滚动时会触发快速滚动。
    ///
    /// 默认值：4.0
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub fast_scroll_sensitivity: Option<f32>,
    /// 将作用域固定到编辑器顶部的设置
    ///
    /// 默认值：粘性滚动已禁用
    pub sticky_scroll: Option<StickyScrollContent>,
    /// 编辑器行号栏中的行号是否为相对行号。
    /// "enabled" 时显示缓冲区行的相对编号，"wrapped" 时显示
    /// 显示行的相对行号。
    ///
    /// 默认值："disabled"
    pub relative_line_numbers: Option<RelativeLineNumbers>,
    /// 根据光标下文本填充新搜索查询的时机
    ///
    /// 默认值：always
    pub seed_search_query_from_cursor: Option<SeedQuerySetting>,
    pub use_smartcase_search: Option<bool>,
    /// 确定用鼠标添加多光标时使用的修饰键。悬停链接鼠标手势将自动调整，以避免与多光标修饰键冲突
    ///
    /// 默认值：alt
    pub multi_cursor_modifier: Option<MultiCursorModifier>,
    /// 隐藏 `private` 文件中变量的值（由
    /// private_files 设置定义）。这仅改变视觉呈现，
    /// 这些值仍存在于文件中，可以被选中/复制/粘贴
    ///
    /// 默认值：false
    pub redact_private_values: Option<bool>,

    /// 多缓冲区摘录默认展开多少行
    ///
    /// 默认值：3
    pub expand_excerpt_lines: Option<u32>,

    /// 多缓冲区摘录中默认提供多少行上下文
    ///
    /// 默认值：2
    pub excerpt_context_lines: Option<u32>,

    /// 是否在 Linux 上启用中键粘贴
    ///
    /// 默认值：true
    pub middle_click_paste: Option<bool>,

    /// 在多缓冲区的某个摘录
    /// （单例缓冲区的片段）中双击时执行的操作
    ///
    /// 默认值：select
    pub double_click_in_multibuffer: Option<DoubleClickInMultibuffer>,
    /// 编辑器搜索结果是否循环
    ///
    /// 默认值：true
    pub search_wrap: Option<bool>,

    /// 打开新的缓冲区搜索项和项目搜索项时使用的默认设置
    ///
    /// 默认值：未启用任何选项
    pub search: Option<SearchSettingsContent>,

    /// 是否自动显示签名帮助弹出框
    ///
    /// 默认值：false
    pub auto_signature_help: Option<bool>,

    /// 是否根据内容自动检测未命名缓冲区的语言。
    /// 通过语言选择器显式选择的语言不会被更改。
    ///
    /// 默认值：true
    pub language_detection: Option<bool>,

    /// 是否在补全或括号对插入后显示签名帮助弹出框
    ///
    /// 默认值：false
    pub show_signature_help_after_edits: Option<bool>,
    /// 在编辑器高亮背景上渲染文本时要保持的
    /// 最小 APCA 感知对比度。
    ///
    /// 值的范围为 0 到 106。设为 0 可禁用调整
    /// 默认值：45
    #[schemars(range(min = 0, max = 106))]
    pub minimum_contrast_for_highlights: Option<MinimumContrast>,

    /// 是否对语言服务器返回的空“转到定义”响应进行后续处理
    /// 使用 `FindAllReferences` 可查找同一符号的引用
    /// `None` 将禁用回退
    ///
    /// 默认值：FindAllReferences
    pub go_to_definition_fallback: Option<GoToDefinitionFallback>,

    /// 可能包含多个位置的 LSP 结果的显示位置
    /// （转到定义、转到实现、查找所有引用）。单个
    /// 结果始终直接打开。各项操作可通过
    /// 各自的 `open_results_in` 参数覆盖此设置。
    ///
    /// 默认值：multi_buffer
    pub lsp_results_location: Option<OpenResultsIn>,

    /// 跳转到定义或引用时将目标滚动到可见区域的方式
    /// （如：转到定义、转到类型定义、查找所有引用）。
    ///
    /// 默认值：center
    pub go_to_definition_scroll_strategy: Option<GoToDefinitionScrollStrategy>,

    /// Jupyter REPL 设置
    pub jupyter: Option<JupyterContent>,

    /// 使用哪个级别过滤编辑器中显示的诊断信息
    ///
    /// 仅影响编辑器渲染，不会中断
    /// 诊断获取或项目诊断编辑器的运行。
    /// 在标签页中标记哪些包含诊断错误/警告的文件
    /// 仅在文件图标也启用时才显示诊断信息。
    ///
    /// 如未指定，则显示所有诊断信息
    ///
    /// 默认值：warning
    pub diagnostics_max_severity: Option<DiagnosticSeverityContent>,

    /// 是否在缓冲区行首显示代码操作按钮
    ///
    /// 默认值：true
    pub inline_code_actions: Option<bool>,

    /// 拖放相关设置
    pub drag_and_drop_selection: Option<DragAndDropSelectionContent>,

    /// 是否以及如何显示语言服务器提供的 Code Lens
    ///
    /// 默认值："off"
    pub code_lens: Option<CodeLens>,

    /// LSP `textDocument/documentColor` 颜色在编辑器中的渲染方式
    ///
    /// 默认值：[`DocumentColorsRenderMode::Inlay`]
    pub lsp_document_colors: Option<DocumentColorsRenderMode>,
    /// 是否在编辑器中查询并显示 LSP `textDocument/documentLink` 链接
    ///
    /// 默认值：true
    pub lsp_document_links: Option<bool>,
    /// 在补全菜单中显示滚动条的时机
    /// 此设置可取以下四个值：
    ///
    /// 1. 若有重要信息则显示滚动条，或
    ///    跟随系统配置的行为
    ///   "auto"
    /// 2. 与系统配置的行为保持一致：
    ///    "system"
    /// 3. 始终显示滚动条：
    ///    "always"
    /// 4. 从不显示滚动条：
    ///    "never"（默认）
    pub completion_menu_scrollbar: Option<ShowScrollbar>,

    /// 代码补全上下文菜单中的详情文本是否左对齐或右对齐
    ///
    /// 默认值：left
    pub completion_detail_alignment: Option<CompletionDetailAlignment>,

    /// 补全菜单中各条目的 LSP 条目类型（函数、方法、变量等）的
    /// 显示方式。
    ///
    /// - "off": 不显示条目类型（默认）。
    /// - "symbol": 显示单字母徽章，并根据
    ///   当前语法主题着色。
    ///
    /// 默认值：off
    pub completion_menu_item_kind: Option<CompletionMenuItemKind>,

    /// 差异在编辑器中的显示方式
    ///
    /// 默认值：split
    pub diff_view_style: Option<DiffViewStyle>,

    /// 使用分割差异视图的最小宽度（以 em 为单位）
    /// 当编辑器宽度小于此值时，差异视图将自动
    /// 切换到统一模式，并在编辑器足够宽时
    /// 切换回来。设置为 0 可禁用自动切换。
    ///
    /// 默认值：100
    pub minimum_split_diff_width: Option<f32>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum RelativeLineNumbers {
    #[strum(serialize = "已禁用")]
    Disabled,
    #[strum(serialize = "已启用")]
    Enabled,
    #[strum(serialize = "折行")]
    Wrapped,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CompletionDetailAlignment {
    #[default]
    #[strum(serialize = "左")]
    Left,
    #[strum(serialize = "右")]
    Right,
}

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CompletionMenuItemKind {
    #[default]
    #[strum(serialize = "关闭")]
    Off,
    #[strum(serialize = "符号")]
    Symbol,
}

impl RelativeLineNumbers {
    pub fn enabled(&self) -> bool {
        match self {
            RelativeLineNumbers::Enabled | RelativeLineNumbers::Wrapped => true,
            RelativeLineNumbers::Disabled => false,
        }
    }
    pub fn wrapped(&self) -> bool {
        match self {
            RelativeLineNumbers::Enabled | RelativeLineNumbers::Disabled => false,
            RelativeLineNumbers::Wrapped => true,
        }
    }
}

#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct CursorAnimationSettingsContent {
    /// 是否启用光标移动动画
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
}

// Toolbar related settings
#[with_fallible_options]
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
pub struct ToolbarContent {
    /// 是否在编辑器工具栏中显示面包屑
    ///
    /// 默认值：true
    pub breadcrumbs: Option<bool>,
    /// 是否在编辑器工具栏中显示快速操作按钮
    ///
    /// 默认值：true
    pub quick_actions: Option<bool>,
    /// 是否在编辑器工具栏中显示选区菜单
    ///
    /// 默认值：true
    pub selections_menu: Option<bool>,
    /// 是否在编辑器工具栏中显示智能体审阅按钮。
    /// 仅在审阅智能体编辑的文件时适用。
    ///
    /// 默认值：true
    pub agent_review: Option<bool>,
    /// 是否在编辑器工具栏中显示代码操作按钮
    ///
    /// 默认值：false
    pub code_actions: Option<bool>,
}

/// 滚动条相关设置
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Default)]
pub struct ScrollbarContent {
    /// 在编辑器中显示滚动条的时机
    ///
    /// 默认值：auto
    pub show: Option<ShowScrollbar>,
    /// 是否在滚动条中显示 Git 差异指示器
    ///
    /// 默认值：true
    pub git_diff: Option<bool>,
    /// 是否在滚动条中显示缓冲区搜索结果指示器
    ///
    /// 默认值：true
    pub search_results: Option<bool>,
    /// 是否在滚动条中显示选中文本的出现位置
    ///
    /// 默认值：true
    pub selected_text: Option<bool>,
    /// 是否在滚动条中显示选中符号的出现位置
    ///
    /// 默认值：true
    pub selected_symbol: Option<bool>,
    /// 在滚动条中显示哪些诊断指示器：
    ///
    /// 默认值：all
    pub diagnostics: Option<ScrollbarDiagnostics>,
    /// 是否在滚动条中显示光标位置
    ///
    /// 默认值：true
    pub cursors: Option<bool>,
    /// 强制启用或禁用各轴的滚动条
    pub axes: Option<ScrollbarAxesContent>,
}

/// 粘性滚动相关设置
#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct StickyScrollContent {
    /// 是否启用粘性滚动
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
}

/// 小地图相关设置
#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct MinimapContent {
    /// 在编辑器中显示小地图的时机
    ///
    /// 默认值：never
    pub show: Option<ShowMinimap>,

    /// 在编辑器中显示小地图的位置
    ///
    /// 默认值：[`DisplayIn::ActiveEditor`]
    pub display_in: Option<DisplayIn>,

    /// 显示小地图滑块的时机
    ///
    /// 默认值：always
    pub thumb: Option<MinimapThumb>,

    /// 定义小地图滚动条滑块的边框样式。
    ///
    /// 默认值：left_open
    pub thumb_border: Option<MinimapThumbBorder>,

    /// 当前行在小地图中的高亮方式
    ///
    /// 默认值：继承编辑器行高亮设置
    pub current_line_highlight: Option<CurrentLineHighlight>,

    /// 小地图中显示的最大列数
    ///
    /// 默认值：80
    pub max_width_columns: Option<num::NonZeroU32>,
}

/// 强制启用或禁用各轴的滚动条
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Default)]
pub struct ScrollbarAxesContent {
    /// 为 false 时，强制禁用水平滚动条；否则遵循其他设置
    ///
    /// 默认值：true
    pub horizontal: Option<bool>,

    /// 为 false 时，强制禁用垂直滚动条；否则遵循其他设置
    ///
    /// 默认值：true
    pub vertical: Option<bool>,
}

/// 控制装订线中 Git 差异区块指示器的宽度
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::VariantArray, strum::VariantNames, strum::FromRepr))]
#[serde(rename_all = "snake_case")]
pub enum GitGutterWidth {
    /// 宽度随缓冲区字体大小自动缩放
    #[default]
    #[strum_discriminants(strum(serialize = "默认"))]
    Default,
    /// Git 差异指示器的固定像素宽度
    #[strum_discriminants(strum(serialize = "自定义"))]
    Custom(crate::PixelSetting),
}

/// 装订线相关设置
#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct GutterContent {
    /// 是否在装订线中显示行号
    ///
    /// 默认值：true
    pub line_numbers: Option<bool>,
    /// 在装订线中预留空间的最少字符数
    ///
    /// 默认值：4
    pub min_line_number_digits: Option<usize>,
    /// 是否在装订线中显示运行按钮
    ///
    /// 默认值：true
    pub runnables: Option<bool>,
    /// 是否在装订线中显示断点
    ///
    /// 默认值：true
    pub breakpoints: Option<bool>,
    /// 是否在装订线中显示书签
    ///
    /// 默认值：true
    pub bookmarks: Option<bool>,
    /// 是否在装订线中显示折叠按钮
    ///
    /// 默认值：true
    pub folds: Option<bool>,
    /// 装订线中 Git 差异区块指示器的宽度。
    /// 使用 "default" 可使宽度随缓冲区字体大小缩放，或使用 {"custom": <pixels>} 设置固定宽度
    ///
    /// 默认值："default"
    pub git_gutter_width: Option<GitGutterWidth>,
}

/// 是否在代码元素上方显示来自语言服务器的 Code Lens
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CodeLens {
    /// 不查询也不显示 Code Lens
    #[default]
    #[strum(serialize = "关闭")]
    Off,
    /// 在代码元素上方显示来自语言服务器的 Code Lens
    #[strum(serialize = "开启")]
    On,
    /// 在代码操作菜单中显示 Code Lens
    #[strum(serialize = "菜单")]
    Menu,
}

impl CodeLens {
    pub fn enabled(&self) -> bool {
        self != &Self::Off
    }

    pub fn inline(&self) -> bool {
        *self == Self::On
    }

    pub fn show_in_menu(&self) -> bool {
        *self == Self::Menu
    }
}

/// LSP `textDocument/documentColor` 颜色在编辑器中的渲染方式
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DocumentColorsRenderMode {
    /// 不查询也不渲染文档颜色
    #[strum(serialize = "无")]
    None,
    /// 将文档颜色渲染为颜色文本附近的内嵌提示
    #[default]
    #[strum(serialize = "内嵌提示")]
    Inlay,
    /// 在颜色文本周围绘制边框
    #[strum(serialize = "边框")]
    Border,
    /// 在颜色文本后面绘制背景
    #[strum(serialize = "背景")]
    Background,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CurrentLineHighlight {
    // Don't highlight the current line.
    #[strum(serialize = "无")]
    None,
    // Highlight the gutter area.
    #[strum(serialize = "装订线")]
    Gutter,
    // Highlight the editor area.
    #[strum(serialize = "行")]
    Line,
    // Highlight the full line.
    #[strum(serialize = "全部")]
    All,
}

/// 根据光标下文本填充新搜索查询的时机
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum SeedQuerySetting {
    /// 始终使用光标下的单词填充搜索查询。
    #[strum(serialize = "始终")]
    Always,
    /// 仅当有文本被选中时填充搜索查询。
    #[strum(serialize = "选择")]
    Selection,
    /// 从不填充搜索查询
    #[strum(serialize = "从不")]
    Never,
}

/// 在多缓冲区的某个摘录（单例缓冲区的片段）中双击时执行的操作
#[derive(
    Default,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DoubleClickInMultibuffer {
    /// 作为普通缓冲区，选中整个单词。
    #[default]
    #[strum(serialize = "选择")]
    Select,
    /// 如果双击时未按下 `alt` 修饰键，则将所点击的摘录作为新缓冲区在新标签页中打开。
    /// 否则，按普通缓冲区方式运行并选中整个单词。
    #[strum(serialize = "打开")]
    Open,
}

/// 显示小地图滑块的时机
///
/// 默认值：always
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum MinimapThumb {
    /// 仅在鼠标悬停于小地图时显示小地图滑块
    #[strum(serialize = "悬停")]
    Hover,
    /// 始终显示小地图滑块。
    #[default]
    #[strum(serialize = "始终")]
    Always,
}

/// 定义小地图滚动条滑块的边框样式。
///
/// 默认值：left_open
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum MinimapThumbBorder {
    /// 在滑块所有边显示边框
    #[strum(serialize = "完整")]
    Full,
    /// 在滑块除左侧以外的所有边显示边框
    #[default]
    #[strum(serialize = "左侧不封闭")]
    LeftOpen,
    /// 在滑块除右侧以外的所有边显示边框
    #[strum(serialize = "右侧不封闭")]
    RightOpen,
    /// 仅在滑块左侧显示边框
    #[strum(serialize = "仅左侧")]
    LeftOnly,
    /// 显示无边框的滑块
    #[strum(serialize = "无")]
    None,
}

/// 在滚动条中显示哪些诊断指示器
///
/// 默认值：all
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
pub enum ScrollbarDiagnostics {
    /// 显示所有诊断级别：提示、信息、警告、错误
    #[strum(serialize = "全部")]
    All,
    /// 仅显示以下诊断级别：信息、警告、错误
    #[strum(serialize = "信息")]
    Information,
    /// 仅显示以下诊断级别：警告、错误
    #[strum(serialize = "警告")]
    Warning,
    /// 仅显示以下诊断级别：错误
    #[strum(serialize = "错误")]
    Error,
    /// 不显示诊断信息
    #[strum(serialize = "无")]
    None,
}

/// 用于添加多光标的按键
///
/// 默认值：alt
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum MultiCursorModifier {
    #[strum(serialize = "Alt")]
    Alt,
    #[serde(alias = "cmd", alias = "ctrl")]
    #[strum(serialize = "Cmd 或 Ctrl")]
    CmdOrCtrl,
}

/// 编辑器是否可以滚动到最后一行之后
///
/// 默认值：one_page
#[derive(
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ScrollBeyondLastLine {
    /// 编辑器不会滚动超出最后一行
    #[strum(serialize = "关闭")]
    Off,

    /// 编辑器可在最后一行之后再滚动一页
    #[strum(serialize = "一页")]
    OnePage,

    /// 编辑器可在最后一行之后滚动与 vertical_scroll_margin 相同的行数
    #[strum(serialize = "垂直滚动边距")]
    VerticalScrollMargin,
}

/// 选区光标的形状
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CursorShape {
    /// 竖线
    #[default]
    #[strum(serialize = "竖线")]
    Bar,
    /// 围绕下一个字符的方块
    #[strum(serialize = "块状")]
    Block,
    /// 沿后续字符下方显示的下划线
    #[strum(serialize = "下划线")]
    Underline,
    /// 绘制在下一个字符周围的空心框
    #[strum(serialize = "空心框")]
    Hollow,
}

/// 转到定义无结果时执行的操作
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum GoToDefinitionFallback {
    /// 禁用回退
    #[strum(serialize = "无")]
    None,
    /// 改为查找相同符号的引用
    #[default]
    #[strum(serialize = "查找所有引用")]
    FindAllReferences,
}

/// 可能包含多个位置的 LSP 结果的显示位置
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum OpenResultsIn {
    /// 在多缓冲区中打开结果
    #[default]
    #[strum(serialize = "多缓冲区")]
    MultiBuffer,
    /// 在可筛选的选择器中打开结果
    #[strum(serialize = "选择器")]
    Picker,
}

/// 跳转到定义或引用时将目标滚动至视图的方式
///
/// 默认值：center
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum GoToDefinitionScrollStrategy {
    /// 将目标垂直居中显示在视口中
    #[default]
    #[strum(serialize = "居中")]
    Center,
    /// 滚动最小距离以使目标可见
    #[strum(serialize = "最小")]
    Minimum,
    /// 滚动使目标出现在视口顶部附近
    #[strum(serialize = "顶部")]
    Top,
    /// 保留光标在视口中的垂直位置，
    /// 当光标位于屏幕外时回退为居中显示。
    #[strum(serialize = "保持位置")]
    Preserve,
}

/// 确定代码片段相对于其他补全项的排序方式
///
/// 默认值：inline
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum SnippetSortOrder {
    /// 将代码片段放置在补全列表顶部
    #[strum(serialize = "顶部")]
    Top,
    /// 使用默认比较逻辑对代码片段进行常规排序
    #[default]
    #[strum(serialize = "内联")]
    Inline,
    /// 将代码片段放置在补全列表底部
    #[strum(serialize = "底部")]
    Bottom,
    /// 不在补全列表中显示代码片段
    #[strum(serialize = "无")]
    None,
}

/// 差异在编辑器中的显示方式
///
/// 默认值：unified
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::Display,
    strum::EnumIter,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DiffViewStyle {
    /// 以统一视图显示差异
    #[strum(serialize = "统一视图")]
    Unified,
    /// 以分割视图显示差异
    #[default]
    #[strum(serialize = "拆分视图")]
    Split,
}

/// 缓冲区和项目搜索项的默认选项
#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
pub struct SearchSettingsContent {
    /// 是否在状态栏中显示项目搜索按钮
    pub button: Option<bool>,
    /// 是否仅匹配完整单词
    pub whole_word: Option<bool>,
    /// 是否区分大小写匹配
    pub case_sensitive: Option<bool>,
    /// 是否在搜索结果中包含被 Git 忽略的文件
    pub include_ignored: Option<bool>,
    /// 是否将搜索查询解释为正则表达式
    pub regex: Option<bool>,
    /// 是否在导航时将光标居中显示在每个搜索匹配项上
    pub center_on_match: Option<bool>,
    /// 在项目搜索中，输入时即开始搜索，无需按 Enter
    pub search_on_type: Option<bool>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct JupyterContent {
    /// 是否启用 Jupyter 功能
    ///
    /// 默认值：true
    pub enabled: Option<bool>,

    /// 为每种语言选择的默认内核
    ///
    /// 默认值：`{}`
    pub kernel_selections: Option<HashMap<String, String>>,
}

/// 是否允许在缓冲区中拖放文本选区
#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
pub struct DragAndDropSelectionContent {
    /// 为 true 时，启用缓冲区中的拖放文本选区
    ///
    /// 默认值：true
    pub enabled: Option<bool>,

    /// 允许拖放操作前必须经过的延迟（毫秒）。否则将创建新的文本选区
    ///
    /// 默认值：300
    pub delay: Option<DelayMs>,
}

/// 在编辑器中显示小地图的时机
///
/// 默认值：never
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ShowMinimap {
    /// 跟随滚动条的可见性
    #[strum(serialize = "自动")]
    Auto,
    /// 始终显示小地图
    #[strum(serialize = "始终")]
    Always,
    /// 从不显示小地图
    #[default]
    #[strum(serialize = "从不")]
    Never,
}

/// 在编辑器中显示小地图的位置
///
/// 默认值：all_editors
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DisplayIn {
    /// 在所有已打开的编辑器中显示
    #[strum(serialize = "所有编辑器")]
    AllEditors,
    /// 仅在当前编辑器中显示小地图
    #[default]
    #[strum(serialize = "当前编辑器")]
    ActiveEditor,
}

/// 高亮背景上文本所需的最低 APCA 感知对比度
///
/// 有效范围：0.0 到 106.0
/// 默认值：45.0
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
pub struct MinimumContrast(
    #[serde(serialize_with = "crate::serialize_f32_with_two_decimal_places")] pub f32,
);

impl Display for MinimumContrast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

impl From<f32> for MinimumContrast {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

/// 非活动窗格的不透明度。0 表示透明，1 表示不透明。
///
/// 有效范围：0.0 到 1.0
/// 默认值：1.0
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
pub struct InactiveOpacity(
    #[serde(serialize_with = "serialize_f32_with_two_decimal_places")] pub f32,
);

impl Display for InactiveOpacity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}", self.0)
    }
}

impl From<f32> for InactiveOpacity {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

/// 居中布局相关设置（左/右）
///
/// 有效范围：0.0 到 0.4
/// 默认值：2.0
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    MergeFrom,
    PartialEq,
    PartialOrd,
    derive_more::FromStr,
)]
#[serde(transparent)]
pub struct CenteredPaddingSettings(
    #[serde(serialize_with = "serialize_f32_with_two_decimal_places")] pub f32,
);

impl CenteredPaddingSettings {
    pub const MIN_PADDING: f32 = 0.0;
    // This is an f64 so serde_json can give a type hint without random numbers in the back
    pub const DEFAULT_PADDING: f64 = 0.2;
    pub const MAX_PADDING: f32 = 0.4;
}

impl Display for CenteredPaddingSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

impl From<f32> for CenteredPaddingSettings {
    fn from(x: f32) -> Self {
        Self(x)
    }
}

impl Default for CenteredPaddingSettings {
    fn default() -> Self {
        Self(Self::DEFAULT_PADDING as f32)
    }
}

impl schemars::JsonSchema for CenteredPaddingSettings {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "CenteredPaddingSettings".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        use schemars::json_schema;
        json_schema!({
            "type": "number",
            "minimum": Self::MIN_PADDING,
            "maximum": Self::MAX_PADDING,
            "default": Self::DEFAULT_PADDING,
            "description": "Centered layout related setting (left/right)."
        })
    }
}
