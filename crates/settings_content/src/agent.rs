use collections::{HashMap, IndexMap};
use schemars::{JsonSchema, json_schema};
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};
use std::sync::Arc;
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use crate::ExtendingVec;

use crate::DockPosition;

/// 会话侧边栏的停靠位置
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
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
pub enum SidebarDockPosition {
    /// 始终在左侧显示侧边栏。
    #[default]
    #[strum(serialize = "左")]
    Left,
    /// 始终在右侧显示侧边栏。
    #[strum(serialize = "右")]
    Right,
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, Default)]
pub struct ThreadsSidebarSettingsContent {
    /// 是否在现有窗口中打开文件夹时自动打开
    /// 会话侧边栏。当 `default_open_behavior` 或
    /// `cli_default_open_behavior` 设置为 `existing_window` 时生效。
    ///
    /// 默认值：true
    pub auto_open: Option<bool>,
    /// 会话侧边栏的停靠位置
    ///
    /// 默认值：left
    pub position: Option<SidebarDockPosition>,
    /// 会话侧边栏的默认宽度（像素）
    ///
    /// 取值范围为 200 到 800，与侧边栏可拖动到的宽度
    /// 一致。超出该范围的值将被限制在此范围内。
    ///
    /// 默认值：300
    #[schemars(range(min = 200, max = 800))]
    pub default_width: Option<crate::PixelSetting>,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum SidebarSide {
    #[default]
    Left,
    Right,
}

/// 思考块在智能体面板中的默认显示方式
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
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
pub enum ThinkingBlockDisplay {
    /// 思考块在流式传输期间完全展开，随后在
    /// 模型完成思考后自动折叠。用户可在折叠后重新展开。
    #[default]
    #[strum(serialize = "自动")]
    Auto,
    /// 思考块在流式传输期间自动展开并带有高度限制，
    /// 完成后保持受限状态。用户可以点击
    /// 完全展开或折叠
    #[strum(serialize = "预览")]
    Preview,
    /// 思考块默认始终完全展开（无高度限制）
    #[strum(serialize = "始终展开")]
    AlwaysExpanded,
    /// 思考块默认始终折叠
    #[strum(serialize = "始终折叠")]
    AlwaysCollapsed,
}

/// Threshold at which agent auto-compaction runs. See
/// [`AutoCompactSettingsContent::threshold`] for the accepted formats.
///
/// The canonical textual form is stored verbatim so it can round-trip through
/// the settings UI; it is serialized back as a JSON string for percentages and
/// as a JSON integer for token counts.
#[derive(Clone, Debug, PartialEq, Eq, MergeFrom)]
pub struct AutoCompactThreshold(pub String);

impl From<String> for AutoCompactThreshold {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<AutoCompactThreshold> for String {
    fn from(value: AutoCompactThreshold) -> Self {
        value.0
    }
}

impl AsRef<str> for AutoCompactThreshold {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Serialize for AutoCompactThreshold {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.0.ends_with('%') {
            serializer.serialize_str(&self.0)
        } else if let Ok(tokens) = self.0.parse::<i64>() {
            serializer.serialize_i64(tokens)
        } else {
            serializer.serialize_str(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AutoCompactThreshold {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ThresholdVisitor;

        impl serde::de::Visitor<'_> for ThresholdVisitor {
            type Value = AutoCompactThreshold;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter
                    .write_str("a percentage string like \"90%\" or an integer number of tokens")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(AutoCompactThreshold(value.to_owned()))
            }

            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(AutoCompactThreshold(value.to_string()))
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(AutoCompactThreshold(value.to_string()))
            }

            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(AutoCompactThreshold(value.to_string()))
            }
        }

        deserializer.deserialize_any(ThresholdVisitor)
    }
}

impl JsonSchema for AutoCompactThreshold {
    fn schema_name() -> Cow<'static, str> {
        "AutoCompactThreshold".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "oneOf": [
                {
                    "type": "string",
                    "pattern": "^\\d+(\\.\\d+)?%$"
                },
                {
                    "type": "integer"
                }
            ]
        })
    }
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, Default)]
pub struct AutoCompactSettingsContent {
    /// 是否在智能体的上下文增长过大时自动压缩，
    /// 通过汇总较早的消息来为模型的
    /// 上下文窗口腾出空间。
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 自动压缩运行的阈值。可为以下之一：
    ///
    /// - 以 `%` 结尾的百分比字符串，例如 `"90%"`，相对于
    ///   模型的上下文窗口计算。`"90%"` 表示上下文填充至 90% 时进行压缩。
    /// - 正整数：已使用的 Token 数达到该数值后运行压缩。
    ///   例如，`100000` 表示使用 100,000 个 Token 后压缩。
    /// - 负整数：上下文窗口中剩余的 Token 数低于该数值的绝对值时运行压缩。
    ///   例如，`-20000` 表示当上下文窗口中
    ///   剩余空间少于 20,000 个 Token 时压缩。
    ///
    /// `0` 不是有效的阈值。
    ///
    /// 默认值："90%"
    pub threshold: Option<AutoCompactThreshold>,
}

#[with_fallible_options]
#[derive(Clone, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom, Debug, Default)]
pub struct AgentSettingsContent {
    /// 是否启用智能体
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 是否在状态栏中显示智能体面板按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    /// 智能体面板停靠的位置
    ///
    /// 默认值：left（Agentic 布局），right（Classic 布局）
    pub dock: Option<DockPosition>,
    /// 智能体面板停靠到左侧或
    /// 右侧时是否使用弹性（比例）尺寸。
    ///
    /// 启用后，`default_width` 不会控制面板宽度，重置面板
    /// 会恢复默认比例。
    ///
    /// 默认值：true
    pub flexible: Option<bool>,
    /// The deprecated version of `threads_sidebar.position`.
    ///
    /// 不要使用此字段
    #[schemars(skip)]
    pub sidebar_side: Option<SidebarDockPosition>,
    /// The deprecated version of `threads_sidebar.default_width`.
    ///
    /// 不要使用此字段
    #[schemars(skip)]
    pub threads_sidebar_default_width: Option<crate::PixelSetting>,
    /// The deprecated version of `threads_sidebar.auto_open`.
    ///
    /// 不要使用此字段
    #[schemars(skip)]
    pub threads_sidebar_auto_open: Option<bool>,
    /// 在智能体面板中保留的、可重新加载的空闲会话的最大数量。
    /// 设置为 0 时，每个符合条件的空闲会话
    /// 在不再活动时都会被卸载。
    ///
    /// 默认值：5
    pub max_idle_retained_threads: Option<usize>,
    /// 会话侧边栏的设置
    pub threads_sidebar: Option<ThreadsSidebarSettingsContent>,
    /// 智能体面板停靠到左侧或右侧且
    /// `flexible` 为 false 时的默认固定宽度（像素）
    ///
    /// 默认值：640
    pub default_width: Option<crate::PixelSetting>,
    /// 智能体面板停靠到底部时的默认高度（像素）
    ///
    /// 默认值：320
    pub default_height: Option<crate::PixelSetting>,
    /// 是否限制智能体面板的内容宽度。启用后，
    /// 内容将被限制在 `max_content_width` 范围内，并在
    /// 面板宽度超过该值时居中显示，以获得最佳可读性。
    ///
    /// 默认值：true
    pub limit_content_width: Option<bool>,
    /// 智能体面板的最大内容宽度（像素）。内容会在
    /// 面板宽度超过此值时居中显示。
    ///
    /// 默认值：850
    pub max_content_width: Option<crate::PixelSetting>,
    /// 创建新会话及其他功能未指定模型时使用的默认模型
    pub default_model: Option<LanguageModelSelection>,
    /// 子智能体使用的模型。未指定时默认使用父会话的模型，
    /// 并沿用其思考模式、思考力度和快速模式设置。如果配置的模型不可用，
    /// 则改用父会话的模型。
    pub subagent_model: Option<LanguageModelSelection>,
    /// 在模型选择器顶部显示的收藏模型
    #[serde(default)]
    pub favorite_models: Vec<LanguageModelSelection>,
    /// 内联助手使用的模型，未指定时默认使用 default_model
    pub inline_assistant_model: Option<LanguageModelSelection>,
    /// 启用流式工具时内联助手使用的模型
    ///
    /// 默认值：true
    pub inline_assistant_use_streaming_tools: Option<bool>,
    /// 用于生成 git 提交消息的模型，未指定时默认使用 default_model
    pub commit_message_model: Option<LanguageModelSelection>,
    /// 是否将项目规则文件（AGENTS.md、CLAUDE.md、.rules 等）包含
    /// 在生成 git 提交消息时使用的提示词中
    ///
    /// 默认值：true
    pub commit_message_include_project_rules: Option<bool>,
    /// 生成 git 提交消息时要包含在提示词中的自定义指令
    /// 在任何项目规则文件（如 `.rules` 或 `AGENTS.md`）之外额外应用
    pub commit_message_instructions: Option<String>,
    /// 用于生成会话摘要的模型，未指定时默认使用 default_model
    pub thread_summary_model: Option<LanguageModelSelection>,
    /// 用于上下文压缩（`/compact` 与自动压缩）的模型。
    /// 未指定时，回退到会话当前选择的模型。
    /// 如果配置的模型不可用（提供商未注册或找不到模型），
    /// 则改用会话的当前模型。
    pub compaction_model: Option<LanguageModelSelection>,
    /// 在执行内联辅助时用于生成备选方案的附加模型
    pub inline_alternatives: Option<Vec<LanguageModelSelection>>,
    /// 在智能体中使用的默认配置文件
    ///
    /// 默认值：write
    pub default_profile: Option<Arc<str>>,
    /// 可用的智能体配置文件
    pub profiles: Option<IndexMap<Arc<str>, AgentProfileContent>>,
    /// 智能体等待用户输入时显示弹出通知的位置
    ///
    /// 默认值："primary_screen"
    pub notify_when_agent_waiting: Option<NotifyWhenAgentWaiting>,
    /// 智能体完成响应或需要用户输入时播放提示音的时机
    ///
    /// 默认值：never
    pub play_sound_when_agent_done: Option<PlaySoundWhenAgentDone>,
    /// 是否在智能体会话运行期间保持系统唤醒
    ///
    /// 默认值：true
    pub prevent_idle_sleep: Option<bool>,
    /// 是否在审查多缓冲区窗格之外，也在单文件编辑器中显示智能体编辑内容
    ///
    /// 默认值：false
    pub single_file_review: Option<bool>,
    /// 语言模型请求的附加参数。向模型发起请求时，
    /// 参数将取自此列表中最后一个
    /// 与该模型的提供商和名称匹配的条目。在每个条目中，
    /// 提供商和模型均为可选项，因此你可以为其中任一项
    /// 指定参数。
    ///
    /// 默认值：[]
    #[serde(default)]
    pub model_parameters: Vec<LanguageModelParameters>,
    /// 智能体上下文自动压缩的设置，用于汇总
    /// 较早的消息，并在上下文变得过大时释放模型
    /// 上下文窗口中的空间。
    pub auto_compact: Option<AutoCompactSettingsContent>,
    /// 是否在智能体面板中显示用于反馈的点赞/点踩按钮
    ///
    /// 默认值：true
    pub enable_feedback: Option<bool>,
    /// 是否在智能体面板中展开编辑卡片，显示完整差异预览
    ///
    /// 默认值：true
    pub expand_edit_card: Option<bool>,
    /// 是否在智能体面板中展开终端卡片，显示完整命令输出
    ///
    /// 默认值：true
    pub expand_terminal_card: Option<bool>,
    /// Zed 在智能体面板中创建终端会话 shell 时自动运行的命令。
    /// 该命令会像手动输入一样发送到 shell，因此会由你
    /// 配置的 shell 解释（包括 Windows 和远程/WSL 项目）。
    /// 空字符串会禁用此行为。
    ///
    /// 默认值：""
    pub terminal_init_command: Option<String>,
    /// 思考块在智能体面板中的默认显示方式
    ///
    /// 默认值：automatic
    pub thinking_display: Option<ThinkingBlockDisplay>,
    /// 点击正在运行的终端工具的停止按钮是否也应取消智能体的生成。
    /// 注意：这仅适用于停止按钮，不适用于终端内的 ctrl+c。
    ///
    /// 默认值：true
    pub cancel_generation_on_terminal_stop: Option<bool>,
    /// 是否始终使用 cmd-enter（Linux 或 Windows 上为 ctrl-enter）在智能体面板中发送消息
    ///
    /// 默认值：false
    pub use_modifier_to_send: Option<bool>,
    /// 智能体消息编辑器的最小高度（以行数计）
    ///
    /// 默认值：4
    pub message_editor_min_lines: Option<usize>,
    /// 是否显示轮次统计信息（生成期间的耗时、最终轮次时长）
    ///
    /// 默认值：false
    pub show_turn_stats: Option<bool>,
    /// 是否在状态栏中显示合并冲突指示器，
    /// 该指示器提供使用智能体解决冲突的选项。
    ///
    /// 默认值：true
    pub show_merge_conflict_indicator: Option<bool>,
    /// 逐工具权限规则，用于精细控制哪些工具操作
    /// 需要确认。
    ///
    /// 当没有工具特定规则匹配时，全局 `default` 将生效。
    /// 对于定义了自身权限模式的外部智能体服务器（如 Claude Agent），
    /// "deny" 和 "confirm" 仍优先生效——
    /// 外部智能体的权限系统仅在 Zed 允许
    /// 该操作时才会使用。每工具正则表达式模式（`always_allow`、`always_deny`、
    /// `always_confirm`）会针对工具的文本输入（命令、路径、
    /// URL 等）进行匹配。
    pub tool_permissions: Option<ToolPermissionsContent>,

    /// 针对智能体运行的终端命令持久授予的沙盒权限。
    /// 在沙盒权限提升提示中选择“始终允许”时会填充
    /// 这些权限。
    pub sandbox_permissions: Option<SandboxPermissionsContent>,
}

impl AgentSettingsContent {
    pub fn set_dock(&mut self, dock: DockPosition) {
        self.dock = Some(dock);
    }

    pub fn set_threads_sidebar_position(&mut self, position: Option<SidebarDockPosition>) {
        self.sidebar_side = None;
        self.threads_sidebar.get_or_insert_default().position = position;
    }

    pub fn set_threads_sidebar_default_width(
        &mut self,
        default_width: Option<crate::PixelSetting>,
    ) {
        self.threads_sidebar_default_width = None;
        self.threads_sidebar.get_or_insert_default().default_width = default_width;
    }

    pub fn set_threads_sidebar_auto_open(&mut self, auto_open: Option<bool>) {
        self.threads_sidebar_auto_open = None;
        self.threads_sidebar.get_or_insert_default().auto_open = auto_open;
    }

    pub fn set_flexible_size(&mut self, flexible: bool) {
        self.flexible = Some(flexible);
    }

    pub fn set_model(&mut self, language_model: LanguageModelSelection) {
        self.default_model = Some(language_model)
    }

    pub fn set_inline_assistant_model(&mut self, provider: String, model: String) {
        self.inline_assistant_model = Some(LanguageModelSelection {
            provider: provider.into(),
            model,
            enable_thinking: false,
            effort: None,
            speed: None,
        });
    }

    pub fn set_profile(&mut self, profile_id: Arc<str>) {
        self.default_profile = Some(profile_id);
    }

    pub fn add_favorite_model(&mut self, model: LanguageModelSelection) {
        // Note: this is intentional to not compare using `PartialEq`here.
        // Full equality would treat entries that differ just in thinking/effort/speed
        // as distinct and silently produce duplicates.
        if !self
            .favorite_models
            .iter()
            .any(|m| m.provider == model.provider && m.model == model.model)
        {
            self.favorite_models.push(model);
        }
    }

    pub fn remove_favorite_model(&mut self, model: &LanguageModelSelection) {
        self.favorite_models
            .retain(|m| !(m.provider == model.provider && m.model == model.model));
    }

    pub fn update_favorite_model<F>(&mut self, provider: &str, model: &str, f: F)
    where
        F: FnOnce(&mut LanguageModelSelection),
    {
        if let Some(entry) = self
            .favorite_models
            .iter_mut()
            .find(|m| m.provider.0 == provider && m.model == model)
        {
            f(entry);
        }
    }

    pub fn set_tool_default_permission(&mut self, tool_id: &str, mode: ToolPermissionMode) {
        let tool_permissions = self.tool_permissions.get_or_insert_default();
        let tool_rules = tool_permissions
            .tools
            .entry(Arc::from(tool_id))
            .or_default();
        tool_rules.default = Some(mode);
    }

    pub fn add_tool_allow_pattern(&mut self, tool_name: &str, pattern: String) {
        let tool_permissions = self.tool_permissions.get_or_insert_default();
        let tool_rules = tool_permissions
            .tools
            .entry(Arc::from(tool_name))
            .or_default();
        let always_allow = tool_rules.always_allow.get_or_insert_default();
        if !always_allow.0.iter().any(|r| r.pattern == pattern) {
            always_allow.0.push(ToolRegexRule {
                pattern,
                case_sensitive: None,
            });
        }
    }

    pub fn add_tool_deny_pattern(&mut self, tool_name: &str, pattern: String) {
        let tool_permissions = self.tool_permissions.get_or_insert_default();
        let tool_rules = tool_permissions
            .tools
            .entry(Arc::from(tool_name))
            .or_default();
        let always_deny = tool_rules.always_deny.get_or_insert_default();
        if !always_deny.0.iter().any(|r| r.pattern == pattern) {
            always_deny.0.push(ToolRegexRule {
                pattern,
                case_sensitive: None,
            });
        }
    }

    pub fn allow_sandbox_all_hosts(&mut self) {
        self.sandbox_permissions
            .get_or_insert_default()
            .allow_all_hosts = Some(true);
    }

    /// 持久化的沙盒网络主机模式，按原样保存（调用方负责
    /// 解析/验证）。
    pub fn sandbox_network_hosts(&self) -> &[String] {
        self.sandbox_permissions
            .as_ref()
            .and_then(|permissions| permissions.network_hosts.as_ref())
            .map(|hosts| hosts.0.as_slice())
            .unwrap_or_default()
    }

    /// 替换持久化的沙盒网络主机模式。调用方计算
    /// 新列表（通常为旧列表加上新授予的主机，并移除
    /// 已被通配符涵盖的条目），而不是盲目追加。
    pub fn set_sandbox_network_hosts(&mut self, hosts: Vec<String>) {
        self.sandbox_permissions
            .get_or_insert_default()
            .network_hosts = Some(ExtendingVec(hosts));
    }

    pub fn allow_sandbox_fs_write_all(&mut self) {
        self.sandbox_permissions
            .get_or_insert_default()
            .allow_fs_write_all = Some(true);
    }

    pub fn allow_sandbox_unsandboxed(&mut self) {
        self.sandbox_permissions
            .get_or_insert_default()
            .allow_unsandboxed = Some(true);
    }

    pub fn add_sandbox_write_path(&mut self, granted: GrantedWritePathContent) {
        let write_paths = &mut self
            .sandbox_permissions
            .get_or_insert_default()
            .write_paths
            .get_or_insert_default()
            .0;

        // Mirror `util::paths::insert_subtree`, keeping the grant set minimal,
        // but compare by each entry's canonical (resolved) grant path.
        let canonical = granted.canonical_or_requested().to_path_buf();
        if write_paths
            .iter()
            .any(|existing| canonical.starts_with(existing.canonical_or_requested()))
        {
            return;
        }
        write_paths.retain(|existing| !existing.canonical_or_requested().starts_with(&canonical));
        write_paths.push(granted);
    }
}

#[with_fallible_options]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct AgentProfileContent {
    pub name: Arc<str>,
    #[serde(default)]
    #[schemars(schema_with = "agent_profile_tools_schema")]
    pub tools: IndexMap<Arc<str>, bool>,
    /// 是否默认启用所有上下文服务器
    pub enable_all_context_servers: Option<bool>,
    #[serde(default)]
    pub context_servers: IndexMap<Arc<str>, ContextServerPresetContent>,
    /// 使用此配置文件时选用的默认语言模型
    pub default_model: Option<LanguageModelSelection>,
}

#[with_fallible_options]
#[derive(Debug, PartialEq, Clone, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ContextServerPresetContent {
    pub tools: IndexMap<Arc<str>, bool>,
}

fn agent_profile_tools_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
    let tool_names = [
        "apply_code_action",
        "ask_user",
        "copy_path",
        "create_directory",
        "create_thread",
        "delete_path",
        "diagnostics",
        "edit_file",
        "fetch",
        "find_path",
        "find_references",
        "get_code_actions",
        "go_to_definition",
        "grep",
        "list_agents_and_models",
        "list_directory",
        "move_path",
        "read_file",
        "rename_symbol",
        "search_web",
        "skill",
        "spawn_agent",
        "terminal",
        "write_file",
    ];
    let properties = tool_names
        .into_iter()
        .map(|name| (name.to_string(), serde_json::json!({ "type": "boolean" })))
        .collect::<serde_json::Map<_, _>>();
    json_schema!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false
    })
}

#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum NotifyWhenAgentWaiting {
    #[default]
    #[strum(serialize = "主屏幕")]
    PrimaryScreen,
    #[strum(serialize = "所有屏幕")]
    AllScreens,
    #[strum(serialize = "从不")]
    Never,
}

#[derive(
    Copy,
    Clone,
    Default,
    Debug,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    PartialEq,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum PlaySoundWhenAgentDone {
    #[default]
    #[strum(serialize = "从不")]
    Never,
    #[strum(serialize = "隐藏时")]
    WhenHidden,
    #[strum(serialize = "始终")]
    Always,
}

impl PlaySoundWhenAgentDone {
    pub fn should_play(&self, visible: bool) -> bool {
        match self {
            PlaySoundWhenAgentDone::Never => false,
            PlaySoundWhenAgentDone::WhenHidden => !visible,
            PlaySoundWhenAgentDone::Always => true,
        }
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct LanguageModelSelection {
    pub provider: LanguageModelProviderSetting,
    pub model: String,
    #[serde(default)]
    pub enable_thinking: bool,
    pub effort: Option<String>,
    pub speed: Option<language_model_core::Speed>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct LanguageModelParameters {
    pub provider: Option<LanguageModelProviderSetting>,
    pub model: Option<String>,
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, MergeFrom)]
pub struct LanguageModelProviderSetting(pub String);

impl JsonSchema for LanguageModelProviderSetting {
    fn schema_name() -> Cow<'static, str> {
        "LanguageModelProviderSetting".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // list the builtin providers as a subset so that we still auto complete them in the settings
        json_schema!({
            "anyOf": [
                {
                    "type": "string",
                    "enum": [
                        "amazon-bedrock",
                        "anthropic",
                        "copilot_chat",
                        "deepseek",
                        "google",
                        "lmstudio",
                        "mistral",
                        "ollama",
                        "openai",
                        "openai-subscribed",
                        "opencode",
                        "openrouter",
                        "vercel_ai_gateway",
                        "x_ai",
                        "x_ai_subscribed",
                        "zed.dev"
                    ]
                },
                {
                    "type": "string",
                }
            ]
        })
    }
}

impl From<String> for LanguageModelProviderSetting {
    fn from(provider: String) -> Self {
        Self(provider)
    }
}

impl From<&str> for LanguageModelProviderSetting {
    fn from(provider: &str) -> Self {
        Self(provider.to_string())
    }
}

#[with_fallible_options]
#[derive(Default, PartialEq, Deserialize, Serialize, Clone, JsonSchema, MergeFrom, Debug)]
#[serde(transparent)]
pub struct AllAgentServersSettings(pub HashMap<String, CustomAgentServerSettings>);

impl std::ops::Deref for AllAgentServersSettings {
    type Target = HashMap<String, CustomAgentServerSettings>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for AllAgentServersSettings {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Deserialize, Serialize, Clone, JsonSchema, MergeFrom, Debug, PartialEq, Eq)]
#[serde(untagged)]
pub enum AgentConfigOptionValue {
    ValueId(String),
    Boolean(bool),
}

impl AgentConfigOptionValue {
    pub fn as_value_id(&self) -> Option<&str> {
        match self {
            Self::ValueId(value) => Some(value),
            Self::Boolean(_) => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(value) => Some(*value),
            Self::ValueId(_) => None,
        }
    }
}

impl std::fmt::Display for AgentConfigOptionValue {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValueId(value) => formatter.write_str(value),
            Self::Boolean(value) => value.fmt(formatter),
        }
    }
}

impl From<String> for AgentConfigOptionValue {
    fn from(value: String) -> Self {
        Self::ValueId(value)
    }
}

impl From<&str> for AgentConfigOptionValue {
    fn from(value: &str) -> Self {
        Self::ValueId(value.to_string())
    }
}

impl From<bool> for AgentConfigOptionValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

#[with_fallible_options]
#[derive(Deserialize, Serialize, Clone, JsonSchema, MergeFrom, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomAgentServerSettings {
    Custom {
        #[serde(rename = "command")]
        path: PathBuf,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<String>,
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        /// 此智能体使用的默认模式
        ///
        /// 注意：并非所有智能体都支持模式。
        ///
        /// 默认值：None
        default_mode: Option<String>,
        /// 会话配置选项的默认值
        ///
        /// 这是从配置选项 ID 到该选项默认值的映射。
        ///
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        default_config_options: HashMap<String, AgentConfigOptionValue>,
        /// 会话配置选项的已收藏值
        ///
        /// 这是从配置选项 ID 到收藏值 ID 列表的映射。
        ///
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        favorite_config_option_values: HashMap<String, Vec<String>>,
    },
    // Used for the ACP extension migration
    #[serde(alias = "extension")]
    Registry {
        /// 传递给智能体的附加环境变量
        ///
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        env: HashMap<String, String>,
        /// 此智能体使用的默认模式
        ///
        /// 注意：并非所有智能体都支持模式。
        ///
        /// 默认值：None
        default_mode: Option<String>,
        /// 会话配置选项的默认值
        ///
        /// 这是从配置选项 ID 到该选项默认值的映射。
        ///
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        default_config_options: HashMap<String, AgentConfigOptionValue>,
        /// 会话配置选项的已收藏值
        ///
        /// 这是从配置选项 ID 到收藏值 ID 列表的映射。
        ///
        /// 默认值：{}
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        favorite_config_option_values: HashMap<String, Vec<String>>,
    },
}

/// A persisted sandbox writable-path grant. Deserializes from either a bare
/// path string (`"/tmp/x"`, a legacy/hand-authored entry with no resolved
/// target) or an object (`{ "requested": "/tmp/x", "resolved": "/tmp/real" }`).
/// Serializes back as a bare string when `resolved` is `None` and as an object
/// otherwise, so hand-authored bare strings round-trip and Zed-written grants
/// are objects.
#[derive(Clone, Debug, Default, PartialEq, MergeFrom)]
pub struct GrantedWritePathContent {
    /// The path exactly as the user/model requested it.
    pub requested: PathBuf,
    /// The canonical, symlink-resolved target established when the grant was
    /// approved. Absent for a bare-string entry.
    pub resolved: Option<PathBuf>,
    /// Windows/WSL only: whether the canonical target lives on a Windows-hosted
    /// (DrvFs) filesystem, whose sandbox-integrity guarantees are weaker. Absent
    /// (false) on other platforms and for bare-string entries.
    pub on_windows_fs: bool,
}

impl GrantedWritePathContent {
    /// The path used for lexical subtree/coverage/dedup logic: the resolved
    /// canonical target when known, otherwise the requested path.
    fn canonical_or_requested(&self) -> &Path {
        self.resolved.as_deref().unwrap_or(&self.requested)
    }
}

impl Serialize for GrantedWritePathContent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.resolved {
            None => self.requested.serialize(serializer),
            Some(resolved) => {
                use serde::ser::SerializeStruct as _;
                let field_count = if self.on_windows_fs { 3 } else { 2 };
                let mut state =
                    serializer.serialize_struct("GrantedWritePathContent", field_count)?;
                state.serialize_field("requested", &self.requested)?;
                state.serialize_field("resolved", resolved)?;
                if self.on_windows_fs {
                    state.serialize_field("on_windows_fs", &self.on_windows_fs)?;
                }
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for GrantedWritePathContent {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Object {
            requested: PathBuf,
            #[serde(default)]
            resolved: Option<PathBuf>,
            #[serde(default)]
            on_windows_fs: bool,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrObject {
            String(PathBuf),
            Object(Object),
        }

        Ok(match StringOrObject::deserialize(deserializer)? {
            StringOrObject::String(requested) => Self {
                requested,
                resolved: None,
                on_windows_fs: false,
            },
            StringOrObject::Object(Object {
                requested,
                resolved,
                on_windows_fs,
            }) => Self {
                requested,
                resolved,
                on_windows_fs,
            },
        })
    }
}

impl JsonSchema for GrantedWritePathContent {
    fn schema_name() -> Cow<'static, str> {
        "GrantedWritePathContent".into()
    }

    fn json_schema(_: &mut schemars::SchemaGenerator) -> schemars::Schema {
        json_schema!({
            "oneOf": [
                { "type": "string" },
                {
                    "type": "object",
                        "properties": {
                        "requested": { "type": "string" },
                        "resolved": { "type": ["string", "null"] },
                        "on_windows_fs": { "type": "boolean" }
                    },
                    "required": ["requested"]
                }
            ]
        })
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct SandboxPermissionsContent {
    /// 沙盒中的终端命令是否始终可以通过网络访问任意主机
    /// 且无需提示确认。
    /// 默认值：false
    pub allow_all_hosts: Option<bool>,

    /// 沙盒中的终端命令始终可通过网络访问且
    /// 无需提示确认的主机。每个条目都是精确主机名
    /// （`github.com`）或以 `*.` 开头的子域通配符（`*.npmjs.org`）。
    /// 默认值：[]
    pub network_hosts: Option<ExtendingVec<String>>,

    /// 沙盒中的终端命令是否始终可以写入
    /// 文件系统中的任意位置且无需提示确认
    /// 默认值：false
    pub allow_fs_write_all: Option<bool>,

    /// 是否持久地在 OS
    /// 沙盒外运行智能体终端命令。这是面向模型的“关闭开关”：为 true 时，沙盒中的
    /// 终端工具不会暴露，系统提示词会省略沙盒
    /// 部分，因此模型会使用普通的 `terminal` 工具。在 Windows 上，WSL
    /// 沙盒设置会被跳过。这不同于模型请求的
    /// `unsandboxed: true` 逃逸（已批准“一次”或“针对此会话”）。
    /// 默认值：false
    pub allow_unsandboxed: Option<bool>,

    /// 沙盒中的终端命令始终可写入且
    /// 无需提示确认的目录子树。每个条目可以是纯路径字符串，也可以是
    /// 对象 `{requested, resolved}`；Zed 写入的是对象（批准时确定的规范路径，
    /// 即解析符号链接后的目标），而
    /// 手动编写的条目可以是纯路径字符串。Zed 写入的路径
    /// 均为绝对路径。
    /// 默认值：[]
    pub write_paths: Option<ExtendingVec<GrantedWritePathContent>>,

    /// 是否在沙盒权限提升提示请求的域名或
    /// 写入路径包含可能造成混淆的 Unicode 字符
    /// （同形字符、不可见字符或双向控制符）时发出警告。
    /// 启用后，此类提示会显示一条警告，必须确认
    /// 后才能允许该请求。
    /// 默认值：true
    pub warn_confusable_unicode: Option<bool>,

    /// 是否在以下情况下发出警告（仅限 Windows/WSL）：沙盒授权的目标是
    /// Windows 主机文件系统（DrvFs）中的文件。此类授权在 WSL 内
    /// 通过转换后的路径强制执行，其沙盒完整性保证弱于
    /// 发行版原生文件系统。启用后，此类授权会显示警告，
    /// 必须先确认该警告才能运行命令。
    /// 默认值：true
    pub warn_ntfs_grants: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ToolPermissionsContent {
    /// 当没有工具特定规则匹配时的全局默认权限。
    /// 单个工具可以用自己的默认值覆盖此设置。
    /// 默认值：confirm
    #[serde(alias = "default_mode")]
    pub default: Option<ToolPermissionMode>,

    /// 逐工具权限规则。
    /// 键为工具名称（如 terminal、edit_file、fetch），包括 MCP
    /// 工具（例如 mcp:server_name:tool_name）。任何工具名称均可接受；
    /// 即使工具没有有意义的文本输入，也可以设置 `default`。
    #[serde(default)]
    pub tools: HashMap<Arc<str>, ToolRulesContent>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ToolRulesContent {
    /// 没有正则表达式规则匹配时的默认模式
    /// 未设置时，从全局 `tool_permissions.default` 继承
    #[serde(alias = "default_mode")]
    pub default: Option<ToolPermissionMode>,

    /// 自动批准输入的正则表达式
    /// 终端：匹配命令。文件工具：匹配路径。fetch：匹配 URL
    /// 对于 `copy_path` 和 `move_path`，模式分别独立匹配各个
    /// 路径（源路径和目标路径）。
    /// 模式在各设置层（用户、项目、配置文件）中累积，无法被
    /// 高优先级层移除——只能添加新模式。
    /// 默认值：[]
    pub always_allow: Option<ExtendingVec<ToolRegexRule>>,

    /// 自动拒绝输入的正则表达式
    /// **安全**：这些规则优先于所有其他规则，跨所有设置层级生效。
    /// 对于 `copy_path` 和 `move_path`，模式分别独立匹配各个
    /// 路径（源路径和目标路径）。
    /// 模式在各设置层（用户、项目、配置文件）中累积，无法被
    /// 高优先级层移除——只能添加新模式。
    /// 默认值：[]
    pub always_deny: Option<ExtendingVec<ToolRegexRule>>,

    /// 必须始终提示确认的输入的正则表达式
    /// 优先于 always_allow，但不优先于 always_deny
    /// 对于 `copy_path` 和 `move_path`，模式分别独立匹配各个
    /// 路径（源路径和目标路径）。
    /// 模式在各设置层（用户、项目、配置文件）中累积，无法被
    /// 高优先级层移除——只能添加新模式。
    /// 默认值：[]
    pub always_confirm: Option<ExtendingVec<ToolRegexRule>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ToolRegexRule {
    /// 要匹配的正则表达式模式
    #[serde(default)]
    pub pattern: String,

    /// 正则表达式是否区分大小写
    /// 默认值：false（不区分大小写）
    pub case_sensitive: Option<bool>,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionMode {
    /// 无需提示确认，自动批准。
    Allow,
    /// 自动拒绝并返回错误。
    Deny,
    /// 始终提示确认（默认行为）。
    #[default]
    Confirm,
}

impl std::fmt::Display for ToolPermissionMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToolPermissionMode::Allow => write!(f, "允许"),
            ToolPermissionMode::Deny => write!(f, "拒绝"),
            ToolPermissionMode::Confirm => write!(f, "确认"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_config_option_value_serializes_value_id_as_string() {
        let value = AgentConfigOptionValue::from("manual");

        assert_eq!(
            serde_json::to_value(&value).expect("serialize value id"),
            serde_json::json!("manual")
        );
        assert_eq!(
            serde_json::from_value::<AgentConfigOptionValue>(serde_json::json!("manual"))
                .expect("deserialize value id"),
            AgentConfigOptionValue::ValueId("manual".to_string())
        );
    }

    #[test]
    fn agent_config_option_value_serializes_boolean_as_boolean() {
        let value = AgentConfigOptionValue::Boolean(true);

        assert_eq!(
            serde_json::to_value(&value).expect("serialize boolean"),
            serde_json::json!(true)
        );
        assert_eq!(
            serde_json::from_value::<AgentConfigOptionValue>(serde_json::json!(true))
                .expect("deserialize boolean"),
            AgentConfigOptionValue::Boolean(true)
        );
    }

    #[test]
    fn agent_config_option_value_merge_replaces_existing_value() {
        use crate::merge_from::MergeFrom as _;

        let mut value = AgentConfigOptionValue::ValueId("manual".to_string());
        value.merge_from(&AgentConfigOptionValue::Boolean(true));

        assert_eq!(value, AgentConfigOptionValue::Boolean(true));
    }

    #[test]
    fn test_set_tool_default_permission_creates_structure() {
        let mut settings = AgentSettingsContent::default();
        assert!(settings.tool_permissions.is_none());

        settings.set_tool_default_permission("terminal", ToolPermissionMode::Allow);

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        assert_eq!(terminal_rules.default, Some(ToolPermissionMode::Allow));
    }

    #[test]
    fn test_set_tool_default_permission_updates_existing() {
        let mut settings = AgentSettingsContent::default();

        settings.set_tool_default_permission("terminal", ToolPermissionMode::Confirm);
        settings.set_tool_default_permission("terminal", ToolPermissionMode::Allow);

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        assert_eq!(terminal_rules.default, Some(ToolPermissionMode::Allow));
    }

    #[test]
    fn test_set_tool_default_permission_for_mcp_tool() {
        let mut settings = AgentSettingsContent::default();

        settings.set_tool_default_permission("mcp:github:create_issue", ToolPermissionMode::Allow);

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let mcp_rules = tool_permissions
            .tools
            .get("mcp:github:create_issue")
            .unwrap();
        assert_eq!(mcp_rules.default, Some(ToolPermissionMode::Allow));
    }

    #[test]
    fn test_add_tool_allow_pattern_creates_structure() {
        let mut settings = AgentSettingsContent::default();
        assert!(settings.tool_permissions.is_none());

        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_allow = terminal_rules.always_allow.as_ref().unwrap();
        assert_eq!(always_allow.0.len(), 1);
        assert_eq!(always_allow.0[0].pattern, "^cargo\\s");
    }

    #[test]
    fn test_add_tool_allow_pattern_appends_to_existing() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());
        settings.add_tool_allow_pattern("terminal", "^npm\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_allow = terminal_rules.always_allow.as_ref().unwrap();
        assert_eq!(always_allow.0.len(), 2);
        assert_eq!(always_allow.0[0].pattern, "^cargo\\s");
        assert_eq!(always_allow.0[1].pattern, "^npm\\s");
    }

    #[test]
    fn test_add_tool_allow_pattern_does_not_duplicate() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());
        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());
        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_allow = terminal_rules.always_allow.as_ref().unwrap();
        assert_eq!(
            always_allow.0.len(),
            1,
            "Duplicate patterns should not be added"
        );
    }

    #[test]
    fn test_add_tool_allow_pattern_for_different_tools() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());
        settings.add_tool_allow_pattern("fetch", "^https?://github\\.com".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();

        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        assert_eq!(
            terminal_rules.always_allow.as_ref().unwrap().0[0].pattern,
            "^cargo\\s"
        );

        let fetch_rules = tool_permissions.tools.get("fetch").unwrap();
        assert_eq!(
            fetch_rules.always_allow.as_ref().unwrap().0[0].pattern,
            "^https?://github\\.com"
        );
    }

    #[test]
    fn test_add_tool_deny_pattern_creates_structure() {
        let mut settings = AgentSettingsContent::default();
        assert!(settings.tool_permissions.is_none());

        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_deny = terminal_rules.always_deny.as_ref().unwrap();
        assert_eq!(always_deny.0.len(), 1);
        assert_eq!(always_deny.0[0].pattern, "^rm\\s");
    }

    #[test]
    fn test_add_tool_deny_pattern_appends_to_existing() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());
        settings.add_tool_deny_pattern("terminal", "^sudo\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_deny = terminal_rules.always_deny.as_ref().unwrap();
        assert_eq!(always_deny.0.len(), 2);
        assert_eq!(always_deny.0[0].pattern, "^rm\\s");
        assert_eq!(always_deny.0[1].pattern, "^sudo\\s");
    }

    #[test]
    fn test_add_tool_deny_pattern_does_not_duplicate() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());
        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());
        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();
        let always_deny = terminal_rules.always_deny.as_ref().unwrap();
        assert_eq!(
            always_deny.0.len(),
            1,
            "Duplicate patterns should not be added"
        );
    }

    #[test]
    fn test_add_tool_deny_and_allow_patterns_separate() {
        let mut settings = AgentSettingsContent::default();

        settings.add_tool_allow_pattern("terminal", "^cargo\\s".to_string());
        settings.add_tool_deny_pattern("terminal", "^rm\\s".to_string());

        let tool_permissions = settings.tool_permissions.as_ref().unwrap();
        let terminal_rules = tool_permissions.tools.get("terminal").unwrap();

        let always_allow = terminal_rules.always_allow.as_ref().unwrap();
        assert_eq!(always_allow.0.len(), 1);
        assert_eq!(always_allow.0[0].pattern, "^cargo\\s");

        let always_deny = terminal_rules.always_deny.as_ref().unwrap();
        assert_eq!(always_deny.0.len(), 1);
        assert_eq!(always_deny.0[0].pattern, "^rm\\s");
    }

    #[test]
    fn test_allow_sandbox_permissions_create_structure() {
        let mut settings = AgentSettingsContent::default();
        assert!(settings.sandbox_permissions.is_none());

        settings.allow_sandbox_all_hosts();
        assert_eq!(settings.sandbox_network_hosts(), &[] as &[String]);
        settings
            .set_sandbox_network_hosts(vec!["github.com".to_string(), "*.npmjs.org".to_string()]);
        assert_eq!(
            settings.sandbox_network_hosts(),
            &["github.com".to_string(), "*.npmjs.org".to_string()]
        );
        settings.allow_sandbox_fs_write_all();
        settings.allow_sandbox_unsandboxed();
        settings.add_sandbox_write_path(GrantedWritePathContent {
            requested: PathBuf::from("/tmp/build"),
            resolved: None,
            on_windows_fs: false,
        });

        let sandbox_permissions = settings.sandbox_permissions.as_ref().unwrap();
        assert_eq!(sandbox_permissions.allow_all_hosts, Some(true));
        assert_eq!(
            sandbox_permissions
                .network_hosts
                .as_ref()
                .unwrap()
                .0
                .as_slice(),
            &["github.com".to_string(), "*.npmjs.org".to_string()]
        );
        assert_eq!(sandbox_permissions.allow_fs_write_all, Some(true));
        assert_eq!(sandbox_permissions.allow_unsandboxed, Some(true));
        assert_eq!(
            sandbox_permissions
                .write_paths
                .as_ref()
                .unwrap()
                .0
                .as_slice(),
            &[GrantedWritePathContent {
                requested: PathBuf::from("/tmp/build"),
                resolved: None,
                on_windows_fs: false,
            }]
        );
    }

    #[test]
    fn test_add_sandbox_write_path_prunes_redundant_paths() {
        let mut settings = AgentSettingsContent::default();

        settings.add_sandbox_write_path(GrantedWritePathContent {
            requested: PathBuf::from("/tmp/build/cache"),
            resolved: None,
            on_windows_fs: false,
        });
        settings.add_sandbox_write_path(GrantedWritePathContent {
            requested: PathBuf::from("/tmp/build"),
            resolved: None,
            on_windows_fs: false,
        });
        settings.add_sandbox_write_path(GrantedWritePathContent {
            requested: PathBuf::from("/tmp/build/output"),
            resolved: None,
            on_windows_fs: false,
        });

        let write_paths = settings
            .sandbox_permissions
            .as_ref()
            .unwrap()
            .write_paths
            .as_ref()
            .unwrap()
            .0
            .as_slice();
        assert_eq!(
            write_paths,
            &[GrantedWritePathContent {
                requested: PathBuf::from("/tmp/build"),
                resolved: None,
                on_windows_fs: false,
            }]
        );
    }

    #[test]
    fn test_granted_write_path_content_deserializes_string_or_object() {
        let from_string: GrantedWritePathContent =
            serde_json::from_value(serde_json::json!("/tmp/x")).unwrap();
        assert_eq!(
            from_string,
            GrantedWritePathContent {
                requested: PathBuf::from("/tmp/x"),
                resolved: None,
                on_windows_fs: false,
            }
        );

        let from_object: GrantedWritePathContent = serde_json::from_value(
            serde_json::json!({ "requested": "/tmp/x", "resolved": "/tmp/real" }),
        )
        .unwrap();
        assert_eq!(
            from_object,
            GrantedWritePathContent {
                requested: PathBuf::from("/tmp/x"),
                resolved: Some(PathBuf::from("/tmp/real")),
                on_windows_fs: false,
            }
        );

        // `resolved` is optional in the object form.
        let object_without_resolved: GrantedWritePathContent =
            serde_json::from_value(serde_json::json!({ "requested": "/tmp/x" })).unwrap();
        assert_eq!(
            object_without_resolved,
            GrantedWritePathContent {
                requested: PathBuf::from("/tmp/x"),
                resolved: None,
                on_windows_fs: false,
            }
        );
    }

    #[test]
    fn test_granted_write_path_content_serializes_string_or_object() {
        // No resolved target serializes as a bare string, so hand-authored
        // bare strings round-trip.
        let bare = GrantedWritePathContent {
            requested: PathBuf::from("/tmp/x"),
            resolved: None,
            on_windows_fs: false,
        };
        assert_eq!(
            serde_json::to_value(&bare).unwrap(),
            serde_json::json!("/tmp/x")
        );

        // A resolved target serializes as an object.
        let resolved = GrantedWritePathContent {
            requested: PathBuf::from("/tmp/x"),
            resolved: Some(PathBuf::from("/tmp/real")),
            on_windows_fs: false,
        };
        assert_eq!(
            serde_json::to_value(&resolved).unwrap(),
            serde_json::json!({ "requested": "/tmp/x", "resolved": "/tmp/real" })
        );
    }
}
