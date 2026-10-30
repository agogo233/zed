mod action;
mod agent;
mod editor;
mod extension;
mod fallible_options;
mod language;
mod language_model;
pub mod merge_from;
mod project;
mod serde_helper;
mod terminal;
mod theme;
mod title_bar;
mod workspace;

pub use action::{ActionName, ActionWithArguments, CommandAliasTarget};
pub use agent::*;
use anyhow::Context;
pub use editor::*;
pub use extension::*;
pub use fallible_options::*;
pub use language::*;
pub use language_model::*;
pub use merge_from::MergeFrom as MergeFromTrait;
pub use project::*;
use serde::de::DeserializeOwned;
pub use serde_helper::{
    serialize_f32_with_two_decimal_places, serialize_optional_f32_with_two_decimal_places,
};
use settings_json::parse_json_with_comments;
pub use terminal::*;
pub use theme::*;
pub use title_bar::*;
pub use workspace::*;

use collections::{HashMap, IndexMap, IndexSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

/// 以像素为单位的非负尺寸
///
/// 有效范围：0.0 及以上
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
    PartialOrd,
    derive_more::FromStr,
    derive_more::Deref,
    derive_more::From,
)]
#[serde(transparent)]
pub struct PixelSetting(
    #[serde(serialize_with = "crate::serialize_f32_with_two_decimal_places")] pub f32,
);

impl std::fmt::Display for PixelSetting {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let rounded = (self.0 * 100.0).round() / 100.0;
        write!(f, "{rounded}")
    }
}

/// 定义一个设置覆盖结构体，其中每个字段为
/// `Option<Box<SettingsContent>>`，以及：
/// - `OVERRIDE_KEYS`：字段名称（JSON 键）的 `&[&str]`
/// - `get_by_key(&self, key) -> Option<&SettingsContent>`：按键访问的访问器
///
/// 字段列表是覆盖键字符串的唯一可信来源
macro_rules! settings_overrides {
    (
        $(#[$attr:meta])*
        pub struct $name:ident { $($field:ident),* $(,)? }
    ) => {
        $(#[$attr])*
        pub struct $name {
            $(pub $field: Option<Box<SettingsContent>>,)*
        }

        impl $name {
            /// JSON 覆盖键，从此结构体的字段名派生
            pub const OVERRIDE_KEYS: &[&str] = &[$(stringify!($field)),*];

            /// 按 JSON 键名查找覆盖项
            pub fn get_by_key(&self, key: &str) -> Option<&SettingsContent> {
                match key {
                    $(stringify!($field) => self.$field.as_deref(),)*
                    _ => None,
                }
            }
        }
    }
}
use std::collections::{BTreeMap, BTreeSet};
use std::hash::Hash;
use std::sync::Arc;
pub use util::serde::default_true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseStatus {
    /// 设置解析成功
    Success,
    /// 设置文件未更改，因此未执行解析
    Unchanged,
    /// 设置解析失败
    Failed { error: String },
}

/// 确定在响应键盘
/// 输入时隐藏鼠标光标的时机。
///
/// 默认值：on_typing_and_action
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
pub enum HideMouseMode {
    /// 从不隐藏鼠标光标
    #[strum(serialize = "从不")]
    Never,
    /// 仅在输入时隐藏
    #[strum(serialize = "输入时")]
    OnTyping,
    /// 在输入时以及触发操作的快捷键激活时隐藏
    #[default]
    #[strum(serialize = "输入和操作时")]
    OnTypingAndAction,
}

/// 确定是否通过静态渲染来减少界面中不必要的动态效果，例如
/// 加载指示器和脉动标签。
///
/// 默认值：off
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
pub enum ReduceMotionMode {
    /// 始终减少动态效果
    #[strum(serialize = "开启")]
    On,
    /// 从不减少动态效果
    #[default]
    #[strum(serialize = "关闭")]
    Off,
}

#[with_fallible_options]
#[derive(Debug, PartialEq, Default, Clone, Serialize, JsonSchema, MergeFrom)]
pub struct SettingsContent {
    #[serde(flatten)]
    pub project: ProjectSettingsContent,

    #[serde(flatten)]
    pub theme: Box<ThemeSettingsContent>,

    #[serde(flatten)]
    pub extension: ExtensionSettingsContent,

    #[serde(flatten)]
    pub workspace: WorkspaceSettingsContent,

    #[serde(flatten)]
    pub editor: EditorSettingsContent,

    #[serde(flatten)]
    pub remote: RemoteSettingsContent,

    /// 与命令面板相关的设置
    pub command_palette: Option<CommandPaletteSettingsContent>,

    /// 与文件查找器相关的设置
    pub file_finder: Option<FileFinderSettingsContent>,

    pub call_hierarchy: Option<CallHierarchySettingsContent>,

    pub git_panel: Option<GitPanelSettingsContent>,

    pub tabs: Option<ItemSettingsContent>,
    pub tab_bar: Option<TabBarSettingsContent>,
    pub tab_switcher: Option<TabSwitcherSettingsContent>,
    pub status_bar: Option<StatusBarSettingsContent>,

    pub preview_tabs: Option<PreviewTabsSettingsContent>,

    pub agent: Option<AgentSettingsContent>,
    pub agent_servers: Option<AllAgentServersSettings>,

    /// Zed 音频的配置
    pub audio: Option<AudioSettingsContent>,

    /// 是否自动检查更新
    ///
    /// 默认值：false
    pub auto_update: Option<bool>,

    /// 此基础键映射设置将 Zed 的默认键绑定调整为类似于
    /// 其他常见代码编辑器的键绑定。默认情况下，Zed 的键位映射基本沿用 VSCode 的
    /// 键位映射并做了少量调整，对应的是 "VSCode" 设置。
    ///
    /// 默认值：VSCode
    pub base_keymap: Option<BaseKeymapContent>,

    /// 协作面板外观设置的配置
    pub collaboration_panel: Option<PanelSettingsContent>,

    pub debugger: Option<DebuggerSettingsContent>,

    /// 诊断相关功能的配置
    pub diagnostics: Option<DiagnosticsSettingsContent>,

    /// Git 相关功能的配置
    pub git: Option<GitSettings>,

    /// 通用语言服务器设置
    pub global_lsp_settings: Option<GlobalLspSettingsContent>,

    /// 图像查看器的设置
    pub image_viewer: Option<ImageViewerSettingsContent>,

    /// Markdown 预览的设置
    pub markdown_preview: Option<MarkdownPreviewSettingsContent>,

    pub repl: Option<ReplSettingsContent>,

    /// 是否启用 Helix 模式
    ///
    /// 默认值：false
    pub helix_mode: Option<bool>,

    /// 确定在响应
    /// 键盘输入时隐藏鼠标光标的时机。全局适用于所有输入界面（编辑器、
    /// 终端、命令面板等）。
    ///
    /// 默认值：on_typing_and_action
    pub hide_mouse: Option<HideMouseMode>,

    pub journal: Option<JournalSettingsContent>,

    /// 日志范围到目标日志级别的映射
    /// 可用于过滤嘈杂日志或启用更详细的日志记录
    ///
    /// 示例：{"log": {"client": "warn"}}
    pub log: Option<HashMap<String, String>>,

    pub line_indicator_format: Option<LineIndicatorFormat>,

    pub language_models: Option<AllLanguageModelSettingsContent>,

    pub copilot: Option<CopilotSettingsContent>,

    pub outline_panel: Option<OutlinePanelSettingsContent>,

    pub project_panel: Option<ProjectPanelSettingsContent>,

    /// Node 相关功能的配置
    pub node: Option<NodeBinarySettings>,

    pub proxy: Option<String>,

    /// 是否通过静态渲染来减少界面中不必要的动态效果，例如
    /// 加载指示器和脉动标签
    ///
    /// 默认值：off
    pub reduce_motion: Option<ReduceMotionMode>,

    /// 要连接的 Zed 服务器 URL
    pub server_url: Option<String>,

    /// 用作凭据存储键的 URL
    ///
    /// 设置后，凭据将存储在此 URL 下，而非 `server_url`。
    /// 这允许并排运行多个 Zed 实例而不会
    /// 覆盖彼此的密钥链条目。
    pub credentials_url: Option<String>,

    /// 会话相关功能的配置
    pub session: Option<SessionSettingsContent>,
    /// 控制 Zed 收集哪些信息
    pub telemetry: Option<TelemetrySettingsContent>,

    /// Zed 终端的配置
    pub terminal: Option<TerminalSettingsContent>,

    pub title_bar: Option<TitleBarSettingsContent>,

    /// 是否启用 Vim 模式
    ///
    /// 默认值：false
    pub vim_mode: Option<bool>,

    // Settings related to calls in Zed
    pub calls: Option<CallSettingsContent>,

    /// which-key 弹出窗口的设置
    pub which_key: Option<WhichKeySettingsContent>,

    /// Zed 中与 Vim 模式相关的设置
    pub vim: Option<VimSettingsContent>,

    /// 在文件头尾搜索模式行的行数。
    /// 模式行包含编辑器指令（例如 vim/emacs 设置），用于配置
    /// 特定文件的编辑器行为。
    ///
    /// 默认值：5
    pub modeline_lines: Option<usize>,

    /// 按标志名称索引的功能标志本地覆盖项
    pub feature_flags: Option<FeatureFlagsMap>,

    /// 面向开发者的检测工具设置（性能分析器、
    /// 追踪器等）可在运行时切换
    pub instrumentation: Option<InstrumentationSettingsContent>,
}

/// 面向开发者的检测工具配置，用于收集
/// 运行中的 Zed 实例的诊断数据。
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct InstrumentationSettingsContent {
    /// 性能分析器的配置，可通过以下方式访问
    /// `zed: open performance profiler` 操作。
    pub performance_profiler: Option<PerformanceProfilerSettingsContent>,
}

/// 性能分析器的配置，用于收集
/// 前台和后台执行器任务的时序数据。
#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct PerformanceProfilerSettingsContent {
    /// 是否收集前台和后台执行器
    /// 任务的计时数据。启用此功能可能会增加内存占用，因此
    /// 在常规构建中默认禁用。
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, MergeFrom)]
#[serde(transparent)]
pub struct FeatureFlagsMap(pub HashMap<String, String>);

// A manual `JsonSchema` impl keeps this type's schema registered under a
// unique name. The derived impl on a `#[serde(transparent)]` newtype around
// `HashMap<String, String>` would inline to the map's own schema name (`Map_of_string`),
// which is shared with every other `HashMap<String, String>` setting field in
// `SettingsContent`. A named placeholder lets `json_schema_store` find and
// replace just this field's schema at runtime without clobbering the others.
impl JsonSchema for FeatureFlagsMap {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "FeatureFlagsMap".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "additionalProperties": { "type": "string" }
        })
    }
}

impl std::ops::Deref for FeatureFlagsMap {
    type Target = HashMap<String, String>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for FeatureFlagsMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl SettingsContent {
    pub fn languages_mut(&mut self) -> &mut HashMap<String, LanguageSettingsContent> {
        &mut self.project.all_languages.languages.0
    }
}

fallible_options::flattened_deserialize!(SettingsContent {
    sections: { project, theme, extension, workspace, editor, remote },
    options: {
        call_hierarchy, command_palette, file_finder, git_panel, tabs, tab_bar, tab_switcher, status_bar, preview_tabs, agent,
        agent_servers, audio, auto_update, base_keymap, collaboration_panel, debugger, diagnostics,
        git,
        global_lsp_settings, image_viewer, markdown_preview, repl, helix_mode, hide_mouse,
        journal, log, line_indicator_format, language_models, copilot, outline_panel, project_panel,
        node, proxy, reduce_motion, server_url, credentials_url, session, telemetry, terminal,
        title_bar, vim_mode, calls, which_key, vim, modeline_lines, feature_flags,
        instrumentation,
    },
    defaults: {},
});

fallible_options::flattened_deserialize!(UserSettingsContent {
    sections: { content, release_channel_overrides, platform_overrides },
    options: {},
    defaults: { profiles },
});

// These impls are there to optimize builds by avoiding monomorphization downstream. Yes, they're repetitive, but using default impls
// break the optimization, for whatever reason.
pub trait RootUserSettings: Sized + DeserializeOwned {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus);
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self>;
}

impl RootUserSettings for SettingsContent {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}
// Explicit opt-in instead of blanket impl to avoid monomorphizing downstream. Just a hunch though.
impl RootUserSettings for Option<SettingsContent> {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}
impl RootUserSettings for UserSettingsContent {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}

settings_overrides! {
    #[with_fallible_options]
    #[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
    pub struct ReleaseChannelOverrides { dev, nightly, preview, stable }
}

settings_overrides! {
    #[with_fallible_options]
    #[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
    pub struct PlatformOverrides { macos, linux, windows }
}

/// 确定配置文件在应用覆盖前的基础设置
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum ProfileBase {
    /// 在用户当前设置之上应用配置文件设置。
    #[default]
    User,
    /// 在 Zed 默认设置之上应用配置文件设置，忽略用户自定义。
    Default,
}

/// 可临时覆盖设置的命名配置文件
#[with_fallible_options]
#[derive(Debug, Default, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct SettingsProfile {
    /// 应用此配置文件覆盖项之前的基础设置起点
    ///
    /// - `user`：在用户设置的基础上应用（默认）
    /// - `default`：在 Zed 默认设置之上应用，忽略用户自定义
    #[serde(default)]
    pub base: ProfileBase,

    /// 此配置文件的设置覆盖项
    #[serde(default)]
    pub settings: Box<SettingsContent>,
}

#[with_fallible_options]
#[derive(Debug, Default, PartialEq, Clone, Serialize, JsonSchema, MergeFrom)]
pub struct UserSettingsContent {
    #[serde(flatten)]
    pub content: Box<SettingsContent>,

    #[serde(flatten)]
    pub release_channel_overrides: ReleaseChannelOverrides,

    #[serde(flatten)]
    pub platform_overrides: PlatformOverrides,

    #[serde(default)]
    pub profiles: IndexMap<String, SettingsProfile>,
}

pub struct ExtensionsSettingsContent {
    pub all_languages: AllLanguageSettingsContent,
}

/// 基础键绑定方案。基础键映射可被用户键映射覆盖。
///
/// 默认值：Zed
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
    Default,
    strum::VariantArray,
)]
pub enum BaseKeymapContent {
    #[default]
    Zed,
    VSCode,
    JetBrains,
    SublimeText,
    Atom,
    TextMate,
    Emacs,
    Cursor,
    None,
}

impl strum::VariantNames for BaseKeymapContent {
    const VARIANTS: &'static [&'static str] = &[
        "Zed",
        "VSCode",
        "JetBrains",
        "Sublime Text",
        "Atom",
        "TextMate",
        "Emacs",
        "Cursor",
        "None",
    ];
}

/// Zed 音频的配置
#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct AudioSettingsContent {
    /// 选择特定的音频输出设备
    #[serde(rename = "experimental.output_audio_device")]
    pub output_audio_device: Option<AudioOutputDeviceName>,
    /// 选择特定的音频输入设备
    #[serde(rename = "experimental.input_audio_device")]
    pub input_audio_device: Option<AudioInputDeviceName>,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct AudioOutputDeviceName(pub Option<String>);

impl AsRef<Option<String>> for AudioInputDeviceName {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}

impl From<Option<String>> for AudioInputDeviceName {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct AudioInputDeviceName(pub Option<String>);

impl AsRef<Option<String>> for AudioOutputDeviceName {
    fn as_ref(&self) -> &Option<String> {
        &self.0
    }
}

impl From<Option<String>> for AudioOutputDeviceName {
    fn from(value: Option<String>) -> Self {
        Self(value)
    }
}

/// 控制 Zed 收集哪些信息
#[with_fallible_options]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Debug, MergeFrom)]
pub struct TelemetrySettingsContent {
    /// 发送调试信息（如崩溃报告）
    ///
    /// 默认值：false
    pub diagnostics: Option<bool>,
    /// 发送匿名使用数据，例如你在 Zed 中使用的编程语言
    ///
    /// 默认值：false
    pub metrics: Option<bool>,
    /// 允许向无法在以下模式下提供的 Anthropic 模型发送请求：
    /// Zero Data Retention
    ///
    /// 默认值：false
    pub anthropic_retention: Option<bool>,
}

impl Default for TelemetrySettingsContent {
    fn default() -> Self {
        Self {
            diagnostics: Some(false),
            metrics: Some(false),
            anthropic_retention: Some(false),
        }
    }
}

#[with_fallible_options]
#[derive(Default, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Clone, MergeFrom)]
pub struct DebuggerSettingsContent {
    /// 确定步进粒度
    ///
    /// 默认值：line
    pub stepping_granularity: Option<SteppingGranularity>,
    /// 是否在 Zed 会话之间复用断点
    ///
    /// 默认值：true
    pub save_breakpoints: Option<bool>,
    /// 是否在状态栏中显示调试按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 连接 TCP 调试适配器时触发超时错误的毫秒数
    ///
    /// 默认值：2000 ms
    pub timeout: Option<u64>,
    /// 是否记录活动调试适配器与 Zed 之间的消息
    ///
    /// 默认值：true
    pub log_dap_communications: Option<bool>,
    /// 是否在将 DAP 消息添加到调试适配器日志时进行格式化
    ///
    /// 默认值：true
    pub format_dap_log_messages: Option<bool>,
    /// 调试面板的停靠位置
    ///
    /// 默认值：Bottom
    pub dock: Option<DockPosition>,
}

/// 步进请求 `next`、`stepIn`、`stepOut` 和 `stepBack` 中一次“步进”的粒度
#[derive(
    PartialEq,
    Eq,
    Debug,
    Hash,
    Clone,
    Copy,
    Deserialize,
    Serialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum SteppingGranularity {
    /// 单步应允许程序运行至当前语句执行完毕
    /// 语句的含义由调试适配器决定，可等同于一行代码
    /// 例如，'for(int i = 0; i < 10; i++)' 可被视为包含 3 条语句：'int i = 0'、'i < 10' 和 'i++'
    #[strum(serialize = "语句")]
    Statement,
    /// 单步应允许程序运行至当前源代码行执行完毕
    #[strum(serialize = "行")]
    Line,
    /// 单步应允许执行一条指令（例如一条 x86 指令）
    #[strum(serialize = "指令")]
    Instruction,
}

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
pub enum DockPosition {
    #[strum(serialize = "左")]
    Left,
    #[strum(serialize = "底部")]
    Bottom,
    #[strum(serialize = "右")]
    Right,
}

/// Zed 语音通话的配置
#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct CallSettingsContent {
    /// 加入频道或通话时是否静音麦克风
    ///
    /// 默认值：false
    pub mute_on_join: Option<bool>,

    /// 加入空频道时是否共享当前项目
    ///
    /// 默认值：false
    pub share_on_join: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug)]
pub struct GitPanelSettingsContent {
    /// 是否在状态栏中显示面板按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 面板停靠的位置
    ///
    /// 默认值：right（Agentic 布局），left（Classic 布局）
    pub dock: Option<DockPosition>,
    /// 面板的默认宽度（像素）
    ///
    /// 默认值：360
    pub default_width: Option<PixelSetting>,
    /// 条目状态的显示方式
    ///
    /// 默认值：icon
    pub status_style: Option<StatusStyle>,

    /// 是否在 Git 面板中显示文件图标
    ///
    /// 默认值：false
    pub file_icons: Option<bool>,

    /// Git 面板中目录的显示方式
    ///
    /// 默认值：icon
    pub folder_indicator: Option<FolderIndicator>,

    /// 滚动条的显示方式和时机
    ///
    /// 默认值：继承编辑器滚动条设置
    pub scrollbar: Option<ScrollbarSettings>,

    /// 以下情况使用的默认分支名称：
    /// git 中未设置 `init.defaultBranch`
    ///
    /// 默认值：main
    pub fallback_branch_name: Option<String>,

    /// 控制 Git 面板中条目的排序方式
    ///
    /// 默认值：path
    pub sort_by: Option<GitPanelSortBy>,

    /// 控制 Git 面板中条目的分组方式
    ///
    /// 默认值：status
    pub group_by: Option<GitPanelGroupBy>,

    /// 是否在差异面板中折叠未跟踪的文件
    ///
    /// 默认值：false
    pub collapse_untracked_diff: Option<bool>,

    /// 是否在面板中以树形或平铺视图显示条目
    ///
    /// 默认值：false
    pub tree_view: Option<bool>,

    /// 是否在 Git 面板中每个文件旁显示增删行数统计
    ///
    /// 默认值：true
    pub diff_stats: Option<bool>,

    /// 是否在 Git 面板图标上显示未提交更改数量的徽章
    ///
    /// 默认值：false
    pub show_count_badge: Option<bool>,

    /// 启动时是否打开 Git 面板
    ///
    /// 默认值：false
    pub starts_open: Option<bool>,

    /// 显示警告前提交消息标题的最大长度
    /// 设为 0 可禁用
    ///
    /// Default: 0
    pub commit_title_max_length: Option<usize>,

    /// 点击 Git 面板中已更改文件时的默认操作
    ///
    /// 默认值：project_diff
    pub entry_primary_click_action: Option<GitPanelClickBehavior>,

    /// 是否默认在 Git 面板中显示提交消息编辑器
    ///
    /// 默认值：expanded
    pub commit_editor: Option<GitPanelCommitEditor>,
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
pub enum GitPanelCommitEditor {
    /// 显示提交消息编辑器
    #[default]
    #[strum(serialize = "已展开")]
    Expanded,
    /// 隐藏提交消息编辑器，仅保留提交按钮所在行
    #[strum(serialize = "已折叠")]
    Collapsed,
}

#[derive(
    Default,
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
pub enum GitPanelClickBehavior {
    /// 打开项目差异，显示所有已更改文件
    #[default]
    #[strum(serialize = "项目差异")]
    ProjectDiff,
    /// 打开单文件差异视图
    #[strum(serialize = "文件差异")]
    FileDiff,
    /// 在编辑器中打开文件，不显示差异视图
    #[strum(serialize = "查看文件")]
    ViewFile,
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
pub enum GitPanelSortBy {
    #[default]
    #[strum(serialize = "路径")]
    Path,
    #[strum(serialize = "名称")]
    Name,
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
pub enum GitPanelGroupBy {
    #[strum(serialize = "无")]
    None,
    #[default]
    #[strum(serialize = "状态")]
    Status,
    #[strum(serialize = "暂存状态")]
    Staging,
}

#[derive(
    Default,
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
pub enum StatusStyle {
    #[default]
    #[strum(serialize = "图标")]
    Icon,
    #[strum(serialize = "标签颜色")]
    LabelColor,
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq,
)]
pub struct ScrollbarSettings {
    pub show: Option<ShowScrollbar>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct PanelSettingsContent {
    /// 是否在状态栏中显示面板按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 面板停靠的位置
    ///
    /// 默认值：right（Agentic 布局），left（Classic 布局）
    pub dock: Option<DockPosition>,
    /// 面板的默认宽度（像素）
    ///
    /// 默认值：240
    pub default_width: Option<PixelSetting>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct CommandPaletteSettingsContent {
    /// 是否在命令面板中使用命令历史记录进行排序
    ///
    /// 默认值：true
    pub use_command_history: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct FileFinderSettingsContent {
    /// 是否在文件查找器中显示文件图标
    ///
    /// 默认值：true
    pub file_icons: Option<bool>,
    /// 确定文件查找器相对于可用窗口宽度可占用的空间
    ///
    /// 默认值：small
    pub modal_max_width: Option<ModalWidthContent>,
    /// 确定文件查找器是否应避免在搜索结果中聚焦当前文件
    ///
    /// 默认值：true
    pub skip_focus_for_active_in_search: Option<bool>,
    /// 搜索时是否使用被 Git 忽略的文件。
    /// 仅使用 Zed 已索引的文件，不一定包含所有被 Git 忽略的文件。
    ///
    /// 默认值：Smart
    pub include_ignored: Option<IncludeIgnoredContent>,
    /// 是否在文件查找器结果中包含文本频道
    ///
    /// 默认值：false
    pub include_channels: Option<bool>,
    /// 是否在打开文件查找器时，
    /// 使用当前聚焦项（例如编辑器或终端）中选中的文本预填其查询
    ///
    /// 默认值：true
    pub prefill_query_from_selection: Option<bool>,
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
pub enum IncludeIgnoredContent {
    /// 使用所有被 Git 忽略的文件
    #[strum(serialize = "全部")]
    All,
    /// 仅使用 Zed 已索引的文件
    #[strum(serialize = "已索引")]
    Indexed,
    /// 从被 Git 忽略的工作树调用时，智能搜索被忽略的文件
    #[default]
    #[strum(serialize = "智能")]
    Smart,
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
#[serde(rename_all = "lowercase")]
pub enum ModalWidthContent {
    #[default]
    #[strum(serialize = "小")]
    Small,
    #[strum(serialize = "中等")]
    Medium,
    #[strum(serialize = "大")]
    Large,
    #[strum(serialize = "超大")]
    XLarge,
    #[strum(serialize = "完整")]
    Full,
}

impl ModalWidthContent {
    pub fn to_pixels(self, base_width: gpui::Pixels, window_width: gpui::Pixels) -> gpui::Pixels {
        match self {
            ModalWidthContent::Small => base_width,
            ModalWidthContent::Full => window_width,
            ModalWidthContent::XLarge => (window_width - gpui::px(512.)).max(base_width),
            ModalWidthContent::Large => (window_width - gpui::px(768.)).max(base_width),
            ModalWidthContent::Medium => (window_width - gpui::px(1024.)).max(base_width),
        }
    }
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct CallHierarchySettingsContent {
    /// 确定调用层次结构选择器相对于可用窗口宽度可占用的空间
    ///
    /// 默认值：medium
    pub modal_max_width: Option<ModalWidthContent>,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug, JsonSchema, MergeFrom)]
pub struct VimSettingsContent {
    pub default_mode: Option<ModeContent>,
    pub toggle_relative_line_numbers: Option<bool>,
    pub use_system_clipboard: Option<UseSystemClipboard>,
    pub use_smartcase_find: Option<bool>,
    pub use_regex_search: Option<bool>,
    /// 启用后，`:substitute` 命令会替换一行中的所有匹配项
    /// （默认行为）。此时 'g' 标志会切换此行为。
    pub gdefault: Option<bool>,
    pub custom_digraphs: Option<HashMap<String, Arc<str>>>,
    pub highlight_on_yank_duration: Option<u64>,
    pub cursor_shape: Option<CursorShapeSettings>,
    /// 启用后，编辑预测将在 Vim 普通模式下显示
    /// 默认情况下，编辑预测仅在插入和替换模式下显示
    pub show_edit_predictions_in_normal_mode: Option<bool>,
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
pub enum ModeContent {
    #[default]
    #[strum(serialize = "常规")]
    Normal,
    #[strum(serialize = "插入")]
    Insert,
}

/// 控制何时使用系统剪贴板
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
pub enum UseSystemClipboard {
    /// 不使用系统剪贴板
    #[strum(serialize = "从不")]
    Never,
    /// 使用系统剪贴板。
    #[strum(serialize = "始终")]
    Always,
    /// 在复制操作中使用系统剪贴板
    #[strum(serialize = "复制时")]
    OnYank,
}

/// Vim 插入模式的光标形状配置
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
pub enum VimInsertModeCursorShape {
    /// 继承编辑器基础 cursor_shape 设置中的光标形状。
    #[strum(serialize = "继承")]
    Inherit,
    /// 竖线光标。
    #[strum(serialize = "竖线")]
    Bar,
    /// 包围字符的块状光标。
    #[strum(serialize = "块状")]
    Block,
    /// 下划线光标。
    #[strum(serialize = "下划线")]
    Underline,
    /// 空心方块光标
    #[strum(serialize = "空心框")]
    Hollow,
}

/// 光标形状的设置
#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom,
)]
pub struct CursorShapeSettings {
    /// 普通模式的光标形状
    ///
    /// 默认值：block
    pub normal: Option<CursorShape>,
    /// 替换模式的光标形状
    ///
    /// 默认值：underline
    pub replace: Option<CursorShape>,
    /// 可视模式的光标形状
    ///
    /// 默认值：block
    pub visual: Option<CursorShape>,
    /// 插入模式的光标形状
    ///
    /// 默认值遵循主 cursor_shape。
    pub insert: Option<VimInsertModeCursorShape>,
}

/// 日志记录的专项设置
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct JournalSettingsContent {
    /// 日志条目存储目录的路径
    ///
    /// 默认值：`~`
    pub path: Option<String>,
    /// 显示小时的格式
    ///
    /// 默认值：hour12
    pub hour_format: Option<HourFormat>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HourFormat {
    #[default]
    Hour12,
    Hour24,
}

#[with_fallible_options]
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, PartialEq)]
pub struct OutlinePanelSettingsContent {
    /// 是否在状态栏中显示大纲面板按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 自定义大纲面板的默认宽度（像素）
    ///
    /// 默认值：240
    pub default_width: Option<PixelSetting>,
    /// 大纲面板的位置
    ///
    /// 默认值：right（Agentic 布局），left（Classic 布局）
    pub dock: Option<DockSide>,
    /// 是否在大纲面板中显示文件图标
    ///
    /// 默认值：true
    pub file_icons: Option<bool>,
    /// 大纲面板中目录的显示方式
    ///
    /// 默认值：icon
    pub folder_indicator: Option<FolderIndicator>,
    /// 是否在大纲面板中显示 Git 状态
    ///
    /// 默认值：true
    pub git_status: Option<bool>,
    /// 嵌套项的缩进量（以像素为单位）。
    ///
    /// 默认值：20
    pub indent_size: Option<PixelSetting>,
    /// 是否在大纲面板中自动显示它，
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
    /// 大纲面板中与缩进参考线相关的设置
    pub indent_guides: Option<IndentGuidesSettingsContent>,
    /// 滚动条相关设置
    pub scrollbar: Option<ScrollbarSettingsContent>,
    /// 展开当前文件大纲条目的默认深度
    /// 大纲条目在显示时默认展开到的深度
    /// - 设为 0 以折叠所有包含子项的条目
    /// - 设为 1 或更大以折叠该深度及更深处的条目
    ///
    /// 默认值：100
    pub expand_outlines_with_depth: Option<usize>,
    /// 是否在大纲面板中隐藏符号、摘录和搜索匹配项。
    /// 仅当多缓冲区视图（例如差异或搜索结果）处于活动状态时生效，
    /// 此时仅显示文件和目录。
    /// 不影响单文件视图。
    ///
    /// 默认值：false
    pub multi_buffer_hide_symbols: Option<bool>,
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DockSide {
    #[strum(serialize = "左")]
    Left,
    #[strum(serialize = "右")]
    Right,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ShowIndentGuides {
    #[strum(serialize = "始终")]
    Always,
    #[strum(serialize = "从不")]
    Never,
}

#[with_fallible_options]
#[derive(
    Copy, Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq, Default,
)]
pub struct IndentGuidesSettingsContent {
    /// 在大纲面板中显示滚动条的时机
    pub show: Option<ShowIndentGuides>,
}

#[derive(Clone, Copy, Default, PartialEq, Debug, JsonSchema, MergeFrom, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LineIndicatorFormat {
    Short,
    #[default]
    Long,
}

/// Markdown 预览的设置
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, Default, PartialEq)]
pub struct MarkdownPreviewSettingsContent {
    /// 用于在 Markdown 预览中渲染的字体名称
    /// 未设置时回退到 UI 字体
    pub font_family: Option<FontFamilyName>,
    /// 代码（代码块和内联代码）在
    /// Markdown 预览中使用的字体名称。未设置时回退到缓冲区字体。
    pub code_font_family: Option<FontFamilyName>,
    /// 用于在 Markdown 预览中渲染的字体大小
    /// 未设置时回退到 UI 字体大小
    pub font_size: Option<FontSize>,
    /// Markdown 预览使用的主题
    /// 未设置时回退到主编辑器主题
    pub theme: Option<ThemeSelection>,
    /// Markdown 预览中标题（H1 至 H6）的字体粗细，以 CSS
    /// 单位表示，范围为 100 到 900。同样适用于笔记本中渲染的
    /// Markdown 单元格，它们与预览共用排版样式。
    ///
    /// 默认值：600
    pub heading_font_weight: Option<FontWeightContent>,
    /// 是否自动在 Markdown 预览中打开 Markdown 文件
    ///
    /// 默认值：false
    pub open_markdown_files_in_preview: Option<bool>,
    /// 是否限制渲染后的 Markdown 内容宽度。启用后
    /// 内容会受 `max_width` 限制并在预览窗格中
    /// 水平居中，以获得最佳可读性。
    ///
    /// 默认值：true
    pub limit_content_width: Option<bool>,
    /// 渲染后的 Markdown 内容的最大宽度（像素），当
    /// 启用 `limit_content_width` 时。
    ///
    /// 默认值：800
    pub max_width: Option<PixelSetting>,
}

/// 图像查看器的设置
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, Default, PartialEq)]
pub struct ImageViewerSettingsContent {
    /// 显示图像文件大小所使用的单位
    ///
    /// 默认值："binary"
    pub unit: Option<ImageFileSizeUnit>,
}

#[with_fallible_options]
#[derive(
    Clone,
    Copy,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    Default,
    PartialEq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum ImageFileSizeUnit {
    /// 以二进制单位显示文件大小（例如 KiB、MiB）
    #[default]
    #[strum(serialize = "二进制")]
    Binary,
    /// 以十进制单位显示文件大小（例如 KB、MB）
    #[strum(serialize = "十进制")]
    Decimal,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct RemoteSettingsContent {
    pub ssh_connections: Option<Vec<SshConnection>>,
    pub wsl_connections: Option<Vec<WslConnection>>,
    pub dev_container_connections: Option<Vec<DevContainerConnection>>,
    pub read_ssh_config: Option<bool>,
    pub use_podman: Option<bool>,
    /// 是否使用 BuildKit 构建开发容器镜像。
    ///
    /// 未设置时，Zed 会通过探测 `buildx` CLI
    /// 插件来自动检测 BuildKit。设为 `false` 可强制使用经典 Docker 构建器，
    /// 这对于缺少集成 BuildKit 的 Docker 兼容引擎是必需的
    /// （例如通过 Docker-API 桥接的 Apple Container），在这些环境中 BuildKit 构建
    /// 无法解析本地构建的镜像。
    ///
    /// Default: null (auto-detect)
    pub dev_container_use_buildkit: Option<bool>,
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom, Hash,
)]
pub struct DevContainerConnection {
    pub name: String,
    pub remote_user: String,
    pub container_id: String,
    pub use_podman: bool,
    pub extension_ids: Vec<String>,
    pub remote_env: BTreeMap<String, String>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct SshConnection {
    pub host: String,
    pub username: Option<String>,
    pub port: Option<u16>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub projects: collections::BTreeSet<RemoteProject>,
    /// 此服务器在界面中显示的名称
    pub nickname: Option<String>,
    // By default Zed will download the binary to the host directly.
    // If this is set to true, Zed will download the binary to your local machine,
    // and then upload it over the SSH connection. Useful if your SSH server has
    // limited outbound internet access.
    pub upload_binary_over_ssh: Option<bool>,

    pub port_forwards: Option<Vec<SshPortForwardOption>>,
    /// SSH 连接及下载远程服务器二进制文件的超时时间（秒）。
    /// 如未指定，默认为 10 秒
    pub connection_timeout: Option<u16>,
}

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom, Debug)]
pub struct WslConnection {
    pub distro_name: String,
    pub user: Option<String>,
    #[serde(default)]
    pub projects: BTreeSet<RemoteProject>,
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, PartialEq, Eq, PartialOrd, Ord, Deserialize, JsonSchema,
)]
pub struct RemoteProject {
    pub paths: Vec<String>,
}

#[with_fallible_options]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, Serialize, JsonSchema, MergeFrom)]
pub struct SshPortForwardOption {
    pub local_host: Option<String>,
    pub local_port: u16,
    pub remote_host: Option<String>,
    pub remote_port: u16,
}

/// 配置 REPL 显示和行为的设置
#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ReplSettingsContent {
    /// REPL 滚动缓冲区中保留的最大行数
    /// 限制在 [4, 256] 范围内。
    ///
    /// 默认值：32
    pub max_lines: Option<usize>,
    /// REPL 滚动缓冲区中保留的最大列数
    /// 限制在 [20, 512] 范围内。
    ///
    /// 默认值：128
    pub max_columns: Option<usize>,
    /// 是否将简短的单行输出内联显示，而非以块形式显示
    ///
    /// 默认值：true
    pub inline_output: Option<bool>,
    /// 内联显示输出的最大字符数
    /// 仅在 `inline_output` 为 true 时生效。
    ///
    /// 默认值：50
    pub inline_output_max_length: Option<usize>,
    /// 滚动前显示的最大输出行数
    /// 设为 0 可禁用输出高度限制
    ///
    /// Default: 0
    pub output_max_height_lines: Option<usize>,
}

/// 配置标签页切换器行为的设置
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct TabSwitcherSettingsContent {
    /// 为 false 时，在标签页切换器中循环切换时不显示文件。
    /// 只有已确认的文件（松开修饰键时）才会显示，
    /// 并记录到 MRU 历史记录中。
    ///
    /// 默认值：true
    pub show_file_preview: Option<bool>,
}

/// 配置 which-key 弹出窗口行为的设置
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct WhichKeySettingsContent {
    /// 是否在按住组合键时显示 which-key 弹出窗口。
    /// 启用后，待处理按键指示器保持可见，但其绑定预览
    /// 弹出框会被禁用。
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
    /// 显示 which-key 弹出窗口前的延迟（毫秒）
    ///
    /// 默认值：1000
    pub delay_ms: Option<u64>,
}

// An ExtendingVec in the settings can only accumulate new values.
//
// This is useful for things like private files where you only want
// to allow new values to be added.
//
// Consider using a HashMap<String, bool> instead of this type
// (like auto_install_extensions) so that user settings files can both add
// and remove values from the set.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtendingVec<T>(pub Vec<T>);

impl<T> Into<Vec<T>> for ExtendingVec<T> {
    fn into(self) -> Vec<T> {
        self.0
    }
}
impl<T> From<Vec<T>> for ExtendingVec<T> {
    fn from(vec: Vec<T>) -> Self {
        ExtendingVec(vec)
    }
}

impl<T: Clone> merge_from::MergeFrom for ExtendingVec<T> {
    fn merge_from(&mut self, other: &Self) {
        self.0.extend_from_slice(other.0.as_slice());
    }
}

// A SplicingVec in the settings replaces the value it merges over, except that
// a `...` entry expands to that previous value.
//
// This lets a settings file add to a list without restating what it inherits,
// while omitting `...` still replaces the list outright. Unlike ExtendingVec,
// entries can be dropped by leaving `...` out and listing what to keep.
//
// Entries collapse to their first occurrence, so naming a value that `...`
// already covers keeps it at the position it was written in rather than
// repeating it.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SplicingVec(pub Vec<String>);

impl SplicingVec {
    pub const REST: &str = "...";
}

impl From<Vec<String>> for SplicingVec {
    fn from(vec: Vec<String>) -> Self {
        SplicingVec(vec)
    }
}

impl merge_from::MergeFrom for SplicingVec {
    fn merge_from(&mut self, other: &Self) {
        let inherited = std::mem::take(&mut self.0);
        self.0 = other
            .0
            .iter()
            .flat_map(|entry| {
                if entry == Self::REST {
                    inherited.clone()
                } else {
                    vec![entry.clone()]
                }
            })
            .collect::<IndexSet<_>>()
            .into_iter()
            .collect();
    }
}

// An ExtendingSet in the settings can only accumulate new values, and ignores
// values that are already present, so merging the same source more than once
// (e.g. re-importing VS Code settings) is idempotent.
//
// Insertion order is preserved, so it round-trips through the user's settings
// file without reordering their entries.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExtendingSet<T: std::hash::Hash + Eq>(pub IndexSet<T>);

impl<T: std::hash::Hash + Eq> From<Vec<T>> for ExtendingSet<T> {
    fn from(vec: Vec<T>) -> Self {
        ExtendingSet(vec.into_iter().collect())
    }
}

impl<T: Clone + std::hash::Hash + Eq> merge_from::MergeFrom for ExtendingSet<T> {
    fn merge_from(&mut self, other: &Self) {
        self.0.extend(other.0.iter().cloned());
    }
}

// A SaturatingBool in the settings can only ever be set to true,
// later attempts to set it to false will be ignored.
//
// Used by `disable_ai`.
#[derive(Debug, Default, Copy, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SaturatingBool(pub bool);

impl From<bool> for SaturatingBool {
    fn from(value: bool) -> Self {
        SaturatingBool(value)
    }
}

impl From<SaturatingBool> for bool {
    fn from(value: SaturatingBool) -> bool {
        value.0
    }
}

impl merge_from::MergeFrom for SaturatingBool {
    fn merge_from(&mut self, other: &Self) {
        self.0 |= other.0
    }
}

#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    MergeFrom,
    JsonSchema,
)]
#[serde(transparent)]
pub struct DelayMs(pub u64);

impl From<u64> for DelayMs {
    fn from(n: u64) -> Self {
        Self(n)
    }
}

impl std::fmt::Display for DelayMs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}ms", self.0)
    }
}

impl std::str::FromStr for DelayMs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.trim()
            .strip_suffix("ms")
            .unwrap_or(s.trim())
            .parse::<u64>()
            .map(DelayMs)
            .with_context(|| format!("failed to parse delay duration: {s}"))
    }
}
