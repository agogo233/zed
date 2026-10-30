use std::num::NonZeroUsize;

use collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

use crate::{
    CenteredPaddingSettings, CommandAliasTarget, DelayMs, DockPosition, DockSide, InactiveOpacity,
    ShowIndentGuides, ShowScrollbar, serialize_optional_f32_with_two_decimal_places,
};

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct WorkspaceSettingsContent {
    /// 活动窗格样式设置。
    pub active_pane_modifiers: Option<ActivePaneModifiers>,
    /// 要使用的文本渲染模式
    ///
    /// 默认值：platform_default
    pub text_rendering_mode: Option<TextRenderingMode>,
    /// 是否使用 Display P3 色彩空间渲染窗口（仅限 macOS）
    ///
    /// 默认值：false
    pub use_display_p3: Option<bool>,
    /// 底部停靠栏的布局模式
    ///
    /// 默认值：contained
    pub bottom_dock_layout: Option<BottomDockLayout>,
    /// 指定水平拆分方向
    ///
    /// 默认值："up"
    pub pane_split_direction_horizontal: Option<PaneSplitDirectionHorizontal>,
    /// 指定垂直拆分方向
    ///
    /// 默认值："left"
    pub pane_split_direction_vertical: Option<PaneSplitDirectionVertical>,
    /// 居中布局相关设置。
    pub centered_layout: Option<CenteredLayoutSettings>,
    /// 关闭应用程序前是否提示用户确认
    ///
    /// 默认值：false
    pub confirm_quit: Option<bool>,
    /// 是否在状态栏中显示通话状态图标
    ///
    /// 默认值：true
    pub show_call_status_icon: Option<bool>,
    /// 何时自动保存已编辑的缓冲区
    ///
    /// 默认值：off
    pub autosave: Option<AutosaveSetting>,
    /// 控制新启动的 Zed 实例对上次会话的恢复方式
    /// 可选值：empty_tab、last_workspace、last_session、launchpad
    /// 默认值：last_session
    pub restore_on_startup: Option<RestoreOnStartupBehavior>,
    /// 从 CLI 打开路径且未指定
    /// 显式的 `-e` 或 `-n` 标志时的默认行为。
    ///
    /// 默认值：existing_window
    pub cli_default_open_behavior: Option<CliDefaultOpenBehavior>,
    /// 从 UI 打开项目时的默认行为。
    ///
    /// 默认值：existing_window
    pub default_open_behavior: Option<DefaultOpenBehavior>,
    /// 再次打开文件时是否尝试恢复之前的文件状态
    /// 状态按窗格存储
    /// 禁用时，将应用默认值而非恢复状态
    ///
    /// 例如，对于编辑器，如果在同一窗格中关闭后再次打开同一文件，则会恢复选区、折叠和滚动位置
    /// 禁用时，默认使用文件开头的单个选区、零滚动位置和无折叠状态
    ///
    /// 默认值：true
    pub restore_on_file_reopen: Option<bool>,
    /// 是否在现有窗格中显示已打开的文件，而非在活动窗格中打开它
    ///
    /// 默认值：false
    pub reveal_if_open: Option<bool>,
    /// 工作区拆分时外边缘放置目标的大小
    /// 以分数形式给出，将乘以工作区较小的尺寸
    ///
    /// 默认值：`0.2`（工作区较小边长的 20%）
    #[serde(serialize_with = "serialize_optional_f32_with_two_decimal_places")]
    pub drop_target_size: Option<f32>,
    /// 在没有标签页的工作区中使用“关闭活动项”时是否关闭窗口
    ///
    /// 默认值：auto（macOS 上为 "on"，其他平台为 "off"）
    pub when_closing_with_no_tabs: Option<CloseWindowWhenNoItems>,
    /// 是否针对辅助技术优化 Zed 的界面，例如
    /// 屏幕阅读器
    ///
    /// 默认值：false
    pub accessible_mode: Option<bool>,
    /// 是否使用系统提供的对话框进行打开和另存为操作
    /// 设为 false 时，Zed 将使用内置的键盘优先选择器
    ///
    /// 默认值：true
    pub use_system_path_prompts: Option<bool>,
    /// 是否使用系统提供的对话框
    /// 设为 false 时，Zed 将使用内置对话框
    /// 注意：此设置在 Linux 上无效，Zed 将始终
    /// 使用内置对话框。
    ///
    /// 默认值：true
    pub use_system_prompts: Option<bool>,
    /// 命令面板的别名。当你在此映射中输入键名时，
    /// 将被视为等同于对应的值。
    ///
    /// 默认值：{}
    #[serde(default)]
    pub command_aliases: HashMap<String, CommandAliasTarget>,
    /// 窗格中的最大打开标签页数。不会关闭未保存的
    /// 标签页。设为 `None` 则不限制标签页数量。
    ///
    /// 默认值：none
    pub max_tabs: Option<NonZeroUsize>,
    /// 打开新窗口时显示的内容
    /// Values: empty_tab, launchpad, readme
    /// 默认值：launchpad
    pub on_new_window: Option<OnNewWindow>,
    /// 最后一个窗口关闭时执行的操作
    ///
    /// 默认值：auto（macOS 上不执行任何操作，其他平台为 "app quit"）
    pub on_last_window_closed: Option<OnLastWindowClosed>,
    /// 调整停靠区大小时是否同时调整其中所有面板的大小
    ///
    /// 默认值：["left"]
    pub resize_all_panels_in_dock: Option<Vec<DockPosition>>,
    /// 是否自动关闭磁盘上已被删除的文件
    ///
    /// 默认值：false
    pub close_on_file_delete: Option<bool>,
    /// 是否根据用户的标签页偏好将窗口合并为标签页（仅限 macOS）
    ///
    /// 默认值：false
    pub use_system_window_tabs: Option<bool>,
    /// `zed::ToggleFullScreen` 操作所进入的全屏模式（仅限 macOS）
    ///
    /// 默认值：native
    pub fullscreen_mode: Option<FullscreenMode>,
    /// 是否为缩放后的面板显示内边距
    /// 启用后，已缩放的底部面板顶部将有一些内边距，
    /// 已缩放的左/右面板则分别在右/左侧添加内边距
    ///
    /// 默认值：true
    pub zoomed_padding: Option<bool>,
    /// 面板已聚焦时调用其 `ToggleFocus` 操作是否
    /// 会关闭该面板，而非仅将焦点移回
    /// 编辑器。这仅适用于面板的焦点切换操作，而
    /// 不适用于其常规的可见性切换操作。
    ///
    /// 默认值：false
    pub close_panel_on_toggle: Option<bool>,
    /// 窗口标题模板。
    ///
    /// 可用变量为 `${projectName}`、`${fileName}`、
    /// `${filePath}`、`${relativePath}`、`${fileStem}`、`${remoteName}`、
    /// `${remoteHost}`、`${appName}`、`${branch}`
    /// 和 `${separator}`。
    /// 当相邻变量为空时会省略 `${separator}`，
    /// 但仍会保留字面文本。
    /// 协作指示器（如果存在）会附加在
    /// 渲染后的模板之后。
    /// 如果模板渲染结果为空，则改用默认模板。
    ///
    /// 默认值：`${projectName}${separator}${fileName}`
    pub window_title_format: Option<String>,
    /// 窗口标题格式中用于替换 `${separator}` 的字符串。
    /// 该值应包含前后的空白字符。
    ///
    /// 默认值：` — `
    pub window_title_separator: Option<String>,
    /// 控制在 Linux 上由 Zed 还是窗口管理器或合成器绘制窗口装饰
    ///
    /// 默认值：client
    pub window_decorations: Option<WindowDecorations>,
    /// 是否将焦点切换到鼠标所在的面板。
    /// 默认值：false
    pub focus_follows_mouse: Option<FocusFollowsMouse>,
}

#[with_fallible_options]
#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ItemSettingsContent {
    /// 是否在标签页上显示 Git 文件状态
    ///
    /// 默认值：false
    pub git_status: Option<bool>,
    /// 标签页关闭按钮的位置
    ///
    /// 默认值：right
    pub close_position: Option<ClosePosition>,
    /// 是否在标签页中显示文件图标
    ///
    /// 默认值：false
    pub file_icons: Option<bool>,
    /// 关闭当前标签页后执行的操作
    ///
    /// 默认值：history
    pub activate_on_close: Option<ActivateOnClose>,
    /// 在标签页中标记哪些包含诊断错误/警告的文件
    /// 此设置可取以下三个值：
    ///
    /// 默认值：off
    pub show_diagnostics: Option<ShowDiagnostics>,
    /// 是否始终在标签页上显示关闭按钮
    ///
    /// 默认值：false
    pub show_close_button: Option<ShowCloseButton>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct PreviewTabsSettingsContent {
    /// 是否将已打开的编辑器显示为预览标签页
    /// 预览标签页不会持久保留，会被复用，直到显式设置为保持打开状态（通过双击或编辑），文件名以斜体显示
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 在项目面板中
    /// 通过单击或 `project_panel::Open` 操作打开时，是否以预览模式打开标签页
    ///
    /// 默认值：true
    pub enable_preview_from_project_panel: Option<bool>,
    /// 从文件查找器中选择时，是否以预览模式打开标签页
    ///
    /// 默认值：false
    pub enable_preview_from_file_finder: Option<bool>,
    /// 从多缓冲区打开时，是否以预览模式打开标签页
    ///
    /// 默认值：true
    pub enable_preview_from_multibuffer: Option<bool>,
    /// 在使用代码导航打开多缓冲区时，是否以预览模式打开标签页
    ///
    /// 默认值：false
    pub enable_preview_multibuffer_from_code_navigation: Option<bool>,
    /// 在使用代码导航打开单个文件时，是否以预览模式打开标签页
    ///
    /// 默认值：true
    pub enable_preview_file_from_code_navigation: Option<bool>,
    /// 在使用代码导航离开标签页时，是否保持标签页处于预览模式
    /// 如果 `enable_preview_file_from_code_navigation` 或 `enable_preview_multibuffer_from_code_navigation` 也为 true，新标签页可能会替换现有标签页
    ///
    /// 默认值：false
    pub enable_keep_preview_on_code_navigation: Option<bool>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
pub enum ClosePosition {
    #[strum(serialize = "左")]
    Left,
    #[default]
    #[strum(serialize = "右")]
    Right,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "lowercase")]
pub enum ShowCloseButton {
    #[strum(serialize = "始终")]
    Always,
    #[default]
    #[strum(serialize = "悬停")]
    Hover,
    #[strum(serialize = "隐藏")]
    Hidden,
}

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
pub enum ShowDiagnostics {
    #[default]
    #[strum(serialize = "关闭")]
    Off,
    #[strum(serialize = "错误")]
    Errors,
    #[strum(serialize = "全部")]
    All,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ActivateOnClose {
    #[default]
    #[strum(serialize = "历史记录")]
    History,
    #[strum(serialize = "相邻标签页")]
    Neighbour,
    #[strum(serialize = "左侧相邻标签页")]
    LeftNeighbour,
}

#[with_fallible_options]
#[derive(Copy, Clone, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct ActivePaneModifiers {
    /// 活动窗格周围边框的大小
    /// 设为 0 时，活动窗格没有任何边框
    /// 边框以内嵌方式绘制
    ///
    /// 默认值：`0.0`
    pub border_size: Option<crate::PixelSetting>,
    /// 非活动窗格的不透明度。
    /// 设为 1.0 时，非活动窗格与活动窗格的不透明度相同
    /// 如果设置为 0，非活动窗格的内容将完全不可见
    /// 值被限制在 [0.0, 1.0] 范围内
    ///
    /// 默认值：`1.0`
    #[schemars(range(min = 0.0, max = 1.0))]
    pub inactive_opacity: Option<InactiveOpacity>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Serialize,
    Deserialize,
    PartialEq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum BottomDockLayout {
    /// 位于左右停靠栏之间
    #[default]
    #[strum(serialize = "两栏之间")]
    Contained,
    /// 占据窗口的完整宽度
    #[strum(serialize = "完整")]
    Full,
    /// 延伸至左侧停靠栏下方，同时贴靠右侧停靠栏
    #[strum(serialize = "左对齐")]
    LeftAligned,
    /// 延伸至右侧停靠栏下方，同时贴靠左侧停靠栏
    #[strum(serialize = "右对齐")]
    RightAligned,
}

#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum FullscreenMode {
    /// 使用 macOS 原生全屏模式，将窗口移入其专属的
    /// Mission Control 空间
    #[default]
    #[strum(serialize = "原生")]
    Native,
    /// 将窗口调整为覆盖整个屏幕（包括菜单栏），并
    /// 在带刘海的显示器上覆盖刘海周围的区域
    #[strum(serialize = "简单")]
    Simple,
}

/// 指定在 Linux 上由哪个组件绘制 Zed 的窗口装饰。
/// 此设置对其他平台无效。
#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    Serialize,
    Deserialize,
    PartialEq,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum WindowDecorations {
    /// Zed 自行绘制窗口装饰/标题栏（客户端装饰）
    #[default]
    #[strum(serialize = "客户端")]
    Client,
    /// 服务器端窗口装饰由窗口管理器或合成器绘制。
    /// GNOME Wayland 不支持此模式。
    #[strum(serialize = "服务器端")]
    Server,
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CloseWindowWhenNoItems {
    /// 默认遵循平台惯例，因此 macOS 上为“开启”，其他平台上为“关闭”
    #[default]
    #[strum(serialize = "平台默认")]
    PlatformDefault,
    /// 没有标签页时关闭窗口
    #[strum(serialize = "关闭窗口")]
    CloseWindow,
    /// 没有标签页时保持窗口打开
    #[strum(serialize = "保持窗口打开")]
    KeepWindowOpen,
}

impl CloseWindowWhenNoItems {
    pub fn should_close(&self) -> bool {
        match self {
            CloseWindowWhenNoItems::PlatformDefault => cfg!(target_os = "macos"),
            CloseWindowWhenNoItems::CloseWindow => true,
            CloseWindowWhenNoItems::KeepWindowOpen => false,
        }
    }
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum CliDefaultOpenBehavior {
    /// 将目录作为新工作区在当前 Zed 窗口的侧边栏中打开。
    #[default]
    #[strum(serialize = "添加到现有窗口")]
    ExistingWindow,
    /// 在新窗口中打开路径，除非它们是现有项目的子路径
    #[strum(serialize = "打开新窗口")]
    NewWindow,
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DefaultOpenBehavior {
    /// 在当前 Zed 窗口中打开项目。
    #[default]
    #[strum(serialize = "添加到现有窗口")]
    ExistingWindow,
    /// 在新窗口中打开项目。
    #[strum(serialize = "打开新窗口")]
    NewWindow,
}

#[derive(
    Copy,
    Clone,
    PartialEq,
    Eq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum RestoreOnStartupBehavior {
    /// 始终以空编辑器标签页启动
    #[serde(alias = "none")]
    #[strum(serialize = "空标签页")]
    EmptyTab,
    /// 还原上次关闭的工作区
    #[strum(serialize = "上次工作区")]
    LastWorkspace,
    /// 还原退出 Zed 时打开的所有工作区
    #[default]
    #[strum(serialize = "上次会话")]
    LastSession,
    /// 显示含有最近项目的启动台（无标签页）
    #[strum(serialize = "启动台")]
    Launchpad,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct TabBarSettingsContent {
    /// 是否在编辑器中显示标签栏
    ///
    /// 默认值：true
    pub show: Option<bool>,
    /// 是否在标签栏中显示导航历史按钮
    ///
    /// 默认值：true
    pub show_nav_history_buttons: Option<bool>,
    /// 是否显示标签栏按钮
    ///
    /// 默认值：true
    pub show_tab_bar_buttons: Option<bool>,
    /// 是否在单独一行中显示固定标签页
    /// 启用后，已固定标签页显示在顶行，未固定标签页显示在底行
    ///
    /// 默认值：false
    pub show_pinned_tabs_in_separate_row: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq, Eq)]
pub struct StatusBarSettingsContent {
    /// 是否显示状态栏
    ///
    /// 默认值：true
    #[serde(rename = "experimental.show")]
    pub show: Option<bool>,
    /// 是否在状态栏中显示活动文件的名称
    ///
    /// 默认值：false
    pub show_active_file: Option<bool>,
    /// 是否在状态栏中显示当前语言按钮
    ///
    /// 默认值：true
    pub active_language_button: Option<bool>,
    /// 是否在状态栏中显示光标位置按钮
    ///
    /// 默认值：true
    pub cursor_position_button: Option<bool>,
    /// 是否在状态栏中显示当前行尾按钮
    ///
    /// 默认值：false
    pub line_endings_button: Option<bool>,
    /// 是否在状态栏中显示当前编码按钮
    ///
    /// 默认值：non_utf8
    pub active_encoding_button: Option<EncodingDisplayOptions>,
    /// 是否在多键输入等待期间显示指示器。
    /// 如果输入设有超时，则会显示倒计时，将鼠标悬停在该指示器上可暂停倒计时。
    /// 启用 which-key 弹出窗口时，其绑定预览弹出框会被禁用。
    ///
    /// 默认值：true
    pub pending_keystrokes_indicator: Option<bool>,
}

#[derive(
    Copy,
    Clone,
    Debug,
    Eq,
    PartialEq,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantNames,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
pub enum EncodingDisplayOptions {
    #[strum(serialize = "已启用")]
    Enabled,
    #[strum(serialize = "已禁用")]
    Disabled,
    #[default]
    #[strum(serialize = "非 UTF-8")]
    NonUtf8,
}
impl EncodingDisplayOptions {
    pub fn should_show(&self, is_utf8: bool, has_bom: bool) -> bool {
        match self {
            Self::Disabled => false,
            Self::Enabled => true,
            Self::NonUtf8 => {
                let is_standard_utf8 = is_utf8 && !has_bom;
                !is_standard_utf8
            }
        }
    }
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
    strum::EnumDiscriminants,
)]
#[strum_discriminants(derive(strum::VariantArray, strum::VariantNames, strum::FromRepr))]
#[serde(rename_all = "snake_case")]
pub enum AutosaveSetting {
    /// 禁用自动保存
    #[strum_discriminants(strum(serialize = "关闭"))]
    Off,
    /// 在 `milliseconds` 毫秒无活动后自动保存
    #[strum_discriminants(strum(serialize = "延迟后"))]
    AfterDelay { milliseconds: DelayMs },
    /// 焦点改变时自动保存。
    #[strum_discriminants(strum(serialize = "焦点切换时"))]
    OnFocusChange,
    /// 活动窗口改变时自动保存。
    #[strum_discriminants(strum(serialize = "窗口切换时"))]
    OnWindowChange,
}

impl AutosaveSetting {
    pub fn should_save_on_close(&self) -> bool {
        matches!(
            &self,
            AutosaveSetting::OnFocusChange
                | AutosaveSetting::OnWindowChange
                | AutosaveSetting::AfterDelay { .. }
        )
    }
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
pub enum PaneSplitDirectionHorizontal {
    #[strum(serialize = "向上")]
    Up,
    #[strum(serialize = "向下")]
    Down,
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
pub enum PaneSplitDirectionVertical {
    #[strum(serialize = "左")]
    Left,
    #[strum(serialize = "右")]
    Right,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
#[with_fallible_options]
pub struct CenteredLayoutSettings {
    /// 使用居中布局时，中央窗格左侧内边距相对于
    /// 工作区的相对宽度。
    ///
    /// 默认值：0.2
    pub left_padding: Option<CenteredPaddingSettings>,
    // The relative width of the right padding of the central pane from the
    // workspace when the centered layout is used.
    ///
    /// 默认值：0.2
    pub right_padding: Option<CenteredPaddingSettings>,
}

#[derive(
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum OnNewWindow {
    /// 打开新窗口时显示一个空的未命名缓冲区
    #[strum(serialize = "空标签页")]
    EmptyTab,
    /// 打开新窗口时显示含有最近项目的启动台
    #[default]
    #[strum(serialize = "启动台")]
    Launchpad,
    /// Open a root-level README in a preview tab when opening a project with no open items.
    /// Leave the pane empty if no README is found.
    Readme,
}

#[derive(
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum OnLastWindowClosed {
    /// 默认遵循平台惯例，在 macOS 上不退出，在其他平台上退出
    #[default]
    #[strum(serialize = "平台默认")]
    PlatformDefault,
    /// 关闭最后一个窗口时退出应用程序
    #[strum(serialize = "退出应用")]
    QuitApp,
}

#[derive(
    Copy,
    Clone,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    Eq,
    Debug,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum TextRenderingMode {
    /// 使用平台默认行为。
    #[default]
    #[strum(serialize = "平台默认")]
    PlatformDefault,
    /// 使用亚像素（ClearType 风格）文本渲染。
    #[strum(serialize = "亚像素")]
    Subpixel,
    /// 使用灰度文本渲染。
    #[strum(serialize = "灰度")]
    Grayscale,
}

impl OnLastWindowClosed {
    pub fn is_quit_app(&self) -> bool {
        match self {
            OnLastWindowClosed::PlatformDefault => false,
            OnLastWindowClosed::QuitApp => true,
        }
    }
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct ProjectPanelAutoOpenSettings {
    /// 是否在编辑器中自动打开新创建的文件
    ///
    /// 默认值：true
    pub on_create: Option<bool>,
    /// 是否在粘贴或复制文件后自动打开这些文件
    ///
    /// 默认值：true
    pub on_paste: Option<bool>,
    /// 是否自动打开从外部来源拖放的文件
    ///
    /// 默认值：true
    pub on_drop: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct ProjectPanelSettingsContent {
    /// 是否在状态栏中显示项目面板按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 是否在项目面板中隐藏被 .gitignore 忽略的文件
    ///
    /// 默认值：false
    pub hide_gitignore: Option<bool>,
    /// 自定义项目面板的默认宽度（像素）
    ///
    /// 默认值：240
    pub default_width: Option<crate::PixelSetting>,
    /// 项目面板的位置
    ///
    /// 默认值：right（Agentic 布局），left（Classic 布局）
    pub dock: Option<DockSide>,
    // TODO
    pub title_tooltip_delay: Option<ProjectPanelTitleTooltipDelay>,
    /// 项目面板中工作树条目之间的间距
    ///
    /// 默认值：comfortable
    pub entry_spacing: Option<ProjectPanelEntrySpacing>,
    /// 是否在项目面板中显示文件图标
    ///
    /// 默认值：true
    pub file_icons: Option<bool>,
    /// 项目面板中目录的显示方式
    ///
    /// 默认值：icon
    pub folder_indicator: Option<FolderIndicator>,
    /// 是否在项目面板中显示 Git 状态
    ///
    /// 默认值：true
    pub git_status: Option<bool>,
    /// 嵌套项的缩进量（以像素为单位）。
    ///
    /// 默认值：20
    pub indent_size: Option<crate::PixelSetting>,
    /// 是否在项目面板中自动显示，
    /// 当对应的项目条目变为活动状态时。
    /// 被 Git 忽略的条目永远不会自动显示。
    ///
    /// 默认值：true
    pub auto_reveal_entries: Option<bool>,
    /// 是否自动折叠目录。
    /// 仅当目录中只有一个子目录时生效。
    ///
    /// 默认值：true
    pub auto_fold_dirs: Option<bool>,
    /// 是否在项目面板中以粗体显示文件夹名称
    ///
    /// 默认值：false
    pub bold_folder_labels: Option<bool>,
    /// 启动时是否打开项目面板
    ///
    /// 默认值：true
    pub starts_open: Option<bool>,
    /// 滚动条相关设置
    pub scrollbar: Option<ProjectPanelScrollbarSettingsContent>,
    /// 在项目面板中标记哪些包含诊断错误/警告的文件
    ///
    /// 默认值：all
    pub show_diagnostics: Option<ShowDiagnostics>,
    /// 项目面板中与缩进参考线相关的设置
    pub indent_guides: Option<ProjectPanelIndentGuidesSettings>,
    /// 当窗口中只有一个文件夹时是否隐藏根条目
    ///
    /// 默认值：false
    pub hide_root: Option<bool>,
    /// 是否在项目面板中隐藏隐藏文件条目
    ///
    /// 默认值：false
    pub hide_hidden: Option<bool>,
    /// 是否将父目录固定在项目面板顶部
    ///
    /// 默认值：true
    pub sticky_scroll: Option<bool>,
    /// 是否在项目面板中启用拖放操作
    ///
    /// 默认值：true
    pub drag_and_drop: Option<bool>,
    /// 自动打开文件的设置
    pub auto_open: Option<ProjectPanelAutoOpenSettings>,
    /// 项目面板中同级条目的排序方式
    ///
    /// 默认值：directories_first
    pub sort_mode: Option<ProjectPanelSortMode>,
    /// 是否在项目面板中区分大小写地排序文件和文件夹名称。
    /// 此项与 `sort_mode` 配合使用。`sort_mode` 控制文件和目录的分组方式。
    /// 此设置则控制名称的比较方式。
    ///
    /// 默认值：default
    pub sort_order: Option<ProjectPanelSortOrder>,
    /// 是否在项目面板中的文件名旁显示错误和警告计数徽标
    ///
    /// 默认值：false
    pub diagnostic_badges: Option<bool>,
    /// 是否在项目面板中的文件名旁显示 Git 状态指示器
    ///
    /// 默认值：false
    pub git_status_indicator: Option<bool>,
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
pub enum ProjectPanelTitleTooltipDelay {
    /// 默认值为 1500 ms
    #[default]
    #[strum_discriminants(strum(serialize = "默认"))]
    Default,
    /// 工具提示显示前的自定义延迟（毫秒）
    #[strum_discriminants(strum(serialize = "自定义"))]
    Custom(crate::DelayMs),
    /// 禁用工具提示
    #[strum_discriminants(strum(serialize = "已禁用"))]
    Disabled,
}

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
pub enum ProjectPanelEntrySpacing {
    /// 条目的舒适间距
    #[default]
    #[strum(serialize = "舒适")]
    Comfortable,
    /// 条目的标准间距
    #[strum(serialize = "标准")]
    Standard,
}

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
pub enum FolderIndicator {
    /// 显示文件夹图标
    #[default]
    #[strum(serialize = "图标")]
    Icon,
    /// 显示展开箭头
    #[strum(serialize = "箭头")]
    Chevron,
    /// 先显示展开箭头，再显示文件夹图标
    #[strum(serialize = "两者")]
    Both,
}

impl FolderIndicator {
    pub fn shows_chevron(self) -> bool {
        matches!(self, Self::Chevron | Self::Both)
    }

    pub fn shows_icon(self) -> bool {
        matches!(self, Self::Icon | Self::Both)
    }
}

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
pub enum ProjectPanelSortMode {
    /// 先显示目录，后显示文件
    #[default]
    #[strum(serialize = "目录优先")]
    DirectoriesFirst,
    /// 文件夹与文件混合显示
    #[strum(serialize = "混合")]
    Mixed,
    /// 先显示文件，再显示目录
    #[strum(serialize = "文件优先")]
    FilesFirst,
}

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
pub enum ProjectPanelSortOrder {
    /// 不区分大小写的自然排序，排序结果相同时小写名称优先。
    /// 文件名中的数字按数值比较（例如，`file2` 在 `file10` 之前）。
    #[default]
    #[strum(serialize = "默认")]
    Default,
    /// 大写名称排在小写名称之前，在各组内使用不区分大小写的
    /// 自然排序。以点开头的名称排在两组之前。
    #[strum(serialize = "大写优先")]
    Upper,
    /// 小写名称排在大写名称之前，在各组内使用不区分大小写的
    /// 自然排序。以点开头的名称排在两组之前。
    #[strum(serialize = "小写优先")]
    Lower,
    /// 纯 Unicode 码位比较，不进行大小写折叠，不进行自然数排序
    /// ASCII 大写字母排在小写字母之前，带重音字符排在 ASCII 字符之后
    #[strum(serialize = "Unicode")]
    Unicode,
    /// 文件按其最后一个扩展名（不区分大小写）升序分组，
    /// 没有扩展名的文件（包括 `.gitignore` 等点文件）排在最前。
    /// 带有额外后缀的点文件（如 `.config.json`）保留该扩展名。
    /// 组内名称使用默认的自然排序。
    #[strum(serialize = "文件扩展名")]
    Extension,
}

impl From<ProjectPanelSortMode> for util::paths::SortMode {
    fn from(mode: ProjectPanelSortMode) -> Self {
        match mode {
            ProjectPanelSortMode::DirectoriesFirst => Self::DirectoriesFirst,
            ProjectPanelSortMode::Mixed => Self::Mixed,
            ProjectPanelSortMode::FilesFirst => Self::FilesFirst,
        }
    }
}

impl From<ProjectPanelSortOrder> for util::paths::SortOrder {
    fn from(order: ProjectPanelSortOrder) -> Self {
        match order {
            ProjectPanelSortOrder::Default => Self::Default,
            ProjectPanelSortOrder::Upper => Self::Upper,
            ProjectPanelSortOrder::Lower => Self::Lower,
            ProjectPanelSortOrder::Unicode => Self::Unicode,
            ProjectPanelSortOrder::Extension => Self::Extension,
        }
    }
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq, Default,
)]
pub struct ProjectPanelScrollbarSettingsContent {
    /// 在项目面板中显示滚动条的时机
    ///
    /// 默认值：继承编辑器滚动条设置
    pub show: Option<ShowScrollbar>,
    /// 是否允许在项目面板中水平滚动
    /// 为 false 时，视图将锁定到最左侧位置，
    /// 过长的文件名将被截断。
    ///
    /// 默认值：true
    pub horizontal_scroll: Option<bool>,
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq, Default,
)]
pub struct ProjectPanelIndentGuidesSettings {
    pub show: Option<ShowIndentGuides>,
}

/// 控制语言服务器的语义标记在语法高亮中的使用方式
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
    strum::EnumMessage,
)]
#[serde(rename_all = "snake_case")]
pub enum SemanticTokens {
    /// 不从语言服务器请求语义标记
    #[default]
    #[strum(serialize = "关闭")]
    Off,
    /// 将 LSP 语义标记与 tree-sitter 高亮结合使用
    #[strum(serialize = "组合")]
    Combined,
    /// 完全使用 LSP 语义标记，替换 tree-sitter 高亮
    #[strum(serialize = "完整")]
    Full,
}

impl SemanticTokens {
    /// 如果应从语言服务器请求语义标记，则返回 true
    pub fn enabled(&self) -> bool {
        self != &Self::Off
    }

    /// 如果应使用 tree-sitter 语法高亮，则返回 true
    /// 在 `full` 模式下，tree-sitter 将被禁用，改用 LSP 语义标记。
    pub fn use_tree_sitter(&self) -> bool {
        self != &Self::Full
    }
}

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
pub enum DocumentFoldingRanges {
    /// 不从语言服务器请求折叠范围，改用 tree-sitter 和基于缩进的折叠
    #[default]
    #[strum(serialize = "关闭")]
    Off,
    /// 尽可能使用 LSP 折叠，当服务器未返回结果时回退到 tree-sitter 和基于缩进的折叠
    #[strum(serialize = "开启")]
    On,
}

impl DocumentFoldingRanges {
    /// 如果应从语言服务器请求 LSP 折叠范围，则返回 true
    pub fn enabled(&self) -> bool {
        self != &Self::Off
    }
}

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
pub enum DocumentSymbols {
    /// 使用 tree-sitter 查询计算用于大纲和面包屑的文档符号（默认）
    #[default]
    #[serde(alias = "tree_sitter")]
    #[strum(serialize = "关闭")]
    Off,
    /// 使用语言服务器的 `textDocument/documentSymbol` LSP 响应生成大纲和
    /// 面包屑。启用后，将不使用 tree-sitter 计算文档符号。
    #[serde(alias = "language_server")]
    #[strum(serialize = "开启")]
    On,
}

impl DocumentSymbols {
    /// 如果应使用 LSP 文档符号而非 tree-sitter，则返回 true
    pub fn lsp_enabled(&self) -> bool {
        self == &Self::On
    }
}

#[with_fallible_options]
#[derive(Copy, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct FocusFollowsMouse {
    pub enabled: Option<bool>,
    pub debounce_ms: Option<u64>,
}
