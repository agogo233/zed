use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use collections::{BTreeMap, HashMap};
use gpui::Rgba;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_json::parse_json_with_comments;
use settings_macros::{MergeFrom, with_fallible_options};
use util::serde::default_true;

use crate::{
    AllLanguageSettingsContent, DelayMs, ExtendingVec, ParseStatus, ProjectTerminalSettingsContent,
    RootUserSettings, SaturatingBool, SplicingVec, fallible_options,
};

#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LspSettingsMap(pub HashMap<Arc<str>, LspSettings>);

impl IntoIterator for LspSettingsMap {
    type Item = (Arc<str>, LspSettings);
    type IntoIter = std::collections::hash_map::IntoIter<Arc<str>, LspSettings>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl RootUserSettings for ProjectSettingsContent {
    fn parse_json(json: &str) -> (Option<Self>, ParseStatus) {
        fallible_options::parse_json(json)
    }
    fn parse_json_with_comments(json: &str) -> anyhow::Result<Self> {
        parse_json_with_comments(json)
    }
}

#[with_fallible_options]
#[derive(Debug, PartialEq, Clone, Default, Serialize, JsonSchema, MergeFrom)]
pub struct ProjectSettingsContent {
    #[serde(flatten)]
    pub all_languages: AllLanguageSettingsContent,

    #[serde(flatten)]
    pub worktree: WorktreeSettingsContent,

    /// 语言服务器的配置
    ///
    /// 以下设置可针对特定语言服务器进行覆盖：
    /// - initialization_options
    ///
    /// 若要覆盖某种语言的设置，将该语言服务器
    /// 名称作为条目添加到 lsp 值中。
    /// 默认值：null
    #[serde(default)]
    pub lsp: LspSettingsMap,

    pub terminal: Option<ProjectTerminalSettingsContent>,

    /// 调试器相关功能的配置
    #[serde(default)]
    pub dap: HashMap<Arc<str>, DapSettingsContent>,

    /// 用于 AI 相关功能的上下文服务器设置
    #[serde(default)]
    pub context_servers: HashMap<Arc<str>, ContextServerSettingsContent>,

    /// 上下文服务器工具调用的默认超时时间（秒）
    /// 可在 context_servers 配置中按服务器覆盖
    ///
    /// Default: 60
    pub context_server_timeout: Option<u64>,

    /// direnv 配置的加载方式
    pub load_direnv: Option<DirenvSettings>,

    /// 自定义 Git 托管提供商列表
    pub git_hosting_providers: Option<ExtendingVec<GitHostingProviderConfig>>,

    /// 是否禁用 Zed 中的所有 AI 功能
    ///
    /// 默认值：false
    pub disable_ai: Option<SaturatingBool>,
}

crate::fallible_options::flattened_deserialize!(ProjectSettingsContent {
    sections: { all_languages, worktree },
    options: {
        terminal, context_server_timeout, load_direnv, git_hosting_providers, disable_ai,
    },
    defaults: { lsp, dap, context_servers },
});

/// 何时扫描符号链接目录的内容
#[derive(
    Copy,
    Clone,
    Default,
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
pub enum ScanSymlinksSetting {
    /// 始终扫描符号链接目录
    #[strum(serialize = "始终")]
    Always,
    /// 仅在符号链接目录已在工作区中展开时扫描
    #[default]
    #[strum(serialize = "已展开")]
    Expanded,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct WorktreeSettingsContent {
    /// 是否阻止此项目在公开频道中共享
    ///
    /// 默认值：false
    #[serde(default)]
    pub prevent_sharing_in_public_channels: bool,

    /// 将匹配这些 glob 模式的文件排除在文件扫描、文件搜索
    /// 以及项目文件树之外。优先级高于 `file_scan_inclusions`。
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "file_scan_exclusions": [
    ///     "**/.git",
    ///     "**/.svn",
    ///     "**/.hg",
    ///     "**/.jj",
    ///     "**/.sl",
    ///     "**/.repo",
    ///     "**/CVS",
    ///     "**/.DS_Store",
    ///     "**/Thumbs.db",
    ///     "**/.classpath",
    ///     "**/.settings"
    ///   ]
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加 glob 模式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "file_scan_exclusions": ["**/node_modules", "..."]
    /// }
    /// ```
    ///
    /// 继承的 glob 模式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    pub file_scan_exclusions: Option<SplicingVec>,

    /// 扫描时包含匹配这些 glob 模式的文件，即使它们被
    /// Git 忽略。注意，匹配范围过广的模式可能会降低文件扫描速度。
    /// `file_scan_exclusions` 的优先级更高。
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "file_scan_inclusions": [".env*"]
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加 glob 模式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "file_scan_inclusions": ["**/build/**", "..."]
    /// }
    /// ```
    ///
    /// 继承的 glob 模式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    pub file_scan_inclusions: Option<SplicingVec>,

    /// 何时扫描符号链接目录的内容
    ///
    /// 默认值：expanded
    pub scan_symlinks: Option<ScanSymlinksSetting>,

    /// Maximum directory depth to eagerly index outside of git repositories;
    /// contents of directories at this depth or deeper are indexed on demand.
    /// Repositories rooted shallower than this depth are always indexed fully.
    /// In projects that are not rooted at a git repository, repositories directly
    /// inside a root folder activate their git features immediately; deeper ones
    /// activate on first use.
    /// `0` means no limit and activates all git repositories immediately.
    ///
    /// 默认值：5
    pub file_scan_depth: Option<u32>,

    /// 将匹配这些 glob 的文件视为 `.env` 文件
    /// Default: ["**/.env*", "**/*.pem", "**/*.key", "**/*.cert", "**/*.crt", "**/secrets.yml"]
    pub private_files: Option<ExtendingVec<String>>,

    /// 将匹配这些 glob 模式的文件和文件夹视为隐藏项，包括
    /// 匹配文件夹内的文件。要隐藏这些条目，可在命令面板中运行
    /// `project panel: toggle hide hidden`，或将
    /// `project_panel.hide_hidden` 设为 `true`。
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "hidden_files": ["**/.*"]
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加 glob 模式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "hidden_files": ["**/*.log", "..."],
    ///   "project_panel": {
    ///     "hide_hidden": true
    ///   }
    /// }
    /// ```
    ///
    /// 继承的 glob 模式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    pub hidden_files: Option<SplicingVec>,

    /// 打开时将匹配这些 glob 模式的文件视为只读。你可以
    /// 查看但无法编辑这些文件，这对于构建产物、外部
    /// 依赖项或生成的文件很有用。
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "read_only_files": []
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加 glob 模式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "read_only_files": ["**/build/**", "..."]
    /// }
    /// ```
    ///
    /// 继承的 glob 模式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    pub read_only_files: Option<SplicingVec>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom, Hash)]
#[serde(rename_all = "snake_case")]
pub struct LspSettings {
    pub binary: Option<BinarySettings>,
    /// 启动时传递给语言服务器的选项
    ///
    /// Ref: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
    ///
    /// 查阅对应语言服务器的文档以了解支持的设置。
    pub initialization_options: Option<serde_json::Value>,
    /// 语言服务器设置
    ///
    /// Ref: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#workspace_configuration
    ///
    /// 查阅对应语言服务器的文档以了解支持的设置。
    pub settings: Option<serde_json::Value>,
    /// 如果服务器支持通过 LSP 扩展发送任务，
    /// 此设置可用于在 Zed 中启用或禁用它们
    /// 默认值：true
    #[serde(default = "default_true")]
    pub enable_lsp_tasks: bool,
    pub fetch: Option<FetchSettings>,
}

impl Default for LspSettings {
    fn default() -> Self {
        Self {
            binary: None,
            initialization_options: None,
            settings: None,
            enable_lsp_tasks: true,
            fetch: None,
        }
    }
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom, Hash,
)]
pub struct BinarySettings {
    pub path: Option<String>,
    pub arguments: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, String>>,
    pub ignore_system_version: Option<bool>,
}

#[with_fallible_options]
#[derive(
    Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom, Hash,
)]
pub struct FetchSettings {
    // Whether to consider pre-releases for fetching
    pub pre_release: Option<bool>,
}

/// 通用语言服务器设置
#[with_fallible_options]
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct GlobalLspSettingsContent {
    /// 是否在状态栏中显示 LSP 服务器按钮
    ///
    /// 默认值：`true`
    pub button: Option<bool>,
    /// 等待语言服务器响应的最长时间（秒）
    /// `0` 表示不应用超时（所有 LSP 响应将无限等待直到完成）
    ///
    /// 默认值：`120`
    pub request_timeout: Option<u64>,
    /// 缓冲区可包含的最大行长度，超过该长度会为整个缓冲区禁用语言服务器功能
    ///
    /// 默认值：`20000`
    #[schemars(range(min = 1))]
    pub max_buffer_line_length: Option<u32>,
    /// 语言服务器通知的设置
    pub notifications: Option<LspNotificationSettingsContent>,
    /// 用于渲染 LSP 语义标记的规则
    pub semantic_token_rules: Option<SemanticTokenRules>,
}

#[with_fallible_options]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LspNotificationSettingsContent {
    /// 自动关闭语言服务器通知的超时时间（毫秒）。
    /// 设为 0 可禁用自动关闭
    ///
    /// 默认值：5000
    pub dismiss_timeout_ms: Option<u64>,
}

/// 用于渲染 LSP 语义标记的自定义规则
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(transparent)]
pub struct SemanticTokenRules {
    pub rules: Vec<SemanticTokenRule>,
}

impl SemanticTokenRules {
    pub const FILE_NAME: &'static str = "semantic_token_rules.json";

    pub fn load(file_path: &Path) -> anyhow::Result<Self> {
        let rules_content = std::fs::read_to_string(file_path).with_context(|| {
            anyhow::anyhow!(
                "Could not read semantic token rules from {}",
                file_path.display()
            )
        })?;

        Self::parse(&rules_content).with_context(|| {
            anyhow::anyhow!(
                "Failed to parse semantic token rules from {}",
                file_path.display()
            )
        })
    }

    pub fn parse(file_content: &str) -> anyhow::Result<Self> {
        serde_json_lenient::from_str::<SemanticTokenRules>(file_content)
            .context("failed to parse semantic token rules")
    }
}

impl crate::merge_from::MergeFrom for SemanticTokenRules {
    fn merge_from(&mut self, other: &Self) {
        self.rules.splice(0..0, other.rules.iter().cloned());
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct SemanticTokenRule {
    pub token_type: Option<String>,
    #[serde(default)]
    pub token_modifiers: Vec<String>,
    #[serde(default)]
    pub style: Vec<String>,
    pub foreground_color: Option<Rgba>,
    pub background_color: Option<Rgba>,
    pub underline: Option<SemanticTokenColorOverride>,
    pub strikethrough: Option<SemanticTokenColorOverride>,
    pub font_weight: Option<SemanticTokenFontWeight>,
    pub font_style: Option<SemanticTokenFontStyle>,
}

impl SemanticTokenRule {
    pub fn no_style_defined(&self) -> bool {
        self.style.is_empty()
            && self.foreground_color.is_none()
            && self.background_color.is_none()
            && self.underline.is_none()
            && self.strikethrough.is_none()
            && self.font_weight.is_none()
            && self.font_style.is_none()
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
#[serde(untagged)]
pub enum SemanticTokenColorOverride {
    InheritForeground(bool),
    Replace(Rgba),
}

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
pub enum SemanticTokenFontWeight {
    #[default]
    #[strum(serialize = "常规")]
    Normal,
    #[strum(serialize = "粗体")]
    Bold,
}

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
pub enum SemanticTokenFontStyle {
    #[default]
    #[strum(serialize = "常规")]
    Normal,
    #[strum(serialize = "斜体")]
    Italic,
}

#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct DapSettingsContent {
    pub binary: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(
    Default, Copy, Clone, PartialEq, Eq, Debug, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
pub struct SessionSettingsContent {
    /// 重启时是否恢复未保存的缓冲区
    ///
    /// 如果为 true，关闭应用时不会提示用户是否保存或丢弃
    /// 未保存文件。
    ///
    /// 默认值：true
    pub restore_unsaved_buffers: Option<bool>,
    /// 是否跳过工作树信任检查
    /// 信任后，项目设置将自动同步，
    /// 语言服务器和 MCP 服务器将自动下载并启动。
    ///
    /// 默认值：false
    pub trust_all_worktrees: Option<bool>,
}

#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema, MergeFrom, Debug)]
#[serde(untagged, rename_all = "snake_case")]
pub enum ContextServerSettingsContent {
    Stdio {
        /// 是否启用上下文服务器
        #[serde(default = "default_true")]
        enabled: bool,
        /// 使用远程开发时，是否在远程服务器上运行上下文服务器
        ///
        /// 如果为 false，上下文服务器将始终在本地计算机上运行
        ///
        /// 默认值：false
        #[serde(default)]
        remote: bool,
        #[serde(flatten)]
        command: ContextServerCommand,
    },
    Http {
        /// 是否启用上下文服务器
        #[serde(default = "default_true")]
        enabled: bool,
        /// 远程上下文服务器的 URL
        url: String,
        /// 要发送的可选请求头。
        #[serde(skip_serializing_if = "HashMap::is_empty", default)]
        headers: HashMap<String, String>,
        /// 工具调用的超时时间（秒），未指定时默认使用全局 context_server_timeout。
        timeout: Option<u64>,
        /// 为授权服务器提供预注册 OAuth 客户端凭据；这些服务器
        /// 需要带外客户端注册。
        #[serde(default, skip_serializing_if = "Option::is_none")]
        oauth: Option<OAuthClientSettings>,
    },
    Extension {
        /// 是否启用上下文服务器
        #[serde(default = "default_true")]
        enabled: bool,
        /// 使用远程开发时，是否在远程服务器上运行上下文服务器
        ///
        /// 如果为 false，上下文服务器将始终在本地计算机上运行
        ///
        /// 默认值：false
        #[serde(default)]
        remote: bool,
        /// 扩展所指定的上下文服务器设置
        ///
        /// 查阅上下文服务器的文档以了解哪些设置
        /// 受支持。
        #[serde(default)]
        settings: serde_json::Value,
    },
}

impl ContextServerSettingsContent {
    pub fn set_enabled(&mut self, enabled: bool) {
        match self {
            ContextServerSettingsContent::Stdio {
                enabled: custom_enabled,
                ..
            } => {
                *custom_enabled = enabled;
            }
            ContextServerSettingsContent::Extension {
                enabled: ext_enabled,
                ..
            } => *ext_enabled = enabled,
            ContextServerSettingsContent::Http {
                enabled: remote_enabled,
                ..
            } => *remote_enabled = enabled,
        }
    }
}

/// 为不支持以下协议的 MCP 服务器提供预注册 OAuth 客户端凭据：
/// Dynamic Client Registration。
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema, MergeFrom, Debug)]
pub struct OAuthClientSettings {
    /// OAuth 客户端 ID 是通过带外注册从
    /// 授权服务器获取的。
    pub client_id: String,
    /// OAuth 客户端密钥（如果这是一个机密客户端）。出于安全考虑，
    /// 建议以交互方式提供；我们会提示输入并将其存储在
    /// 系统钥匙串中
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

#[with_fallible_options]
#[derive(Deserialize, Serialize, Clone, PartialEq, Eq, JsonSchema, MergeFrom)]
pub struct ContextServerCommand {
    #[serde(rename = "command")]
    pub path: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
    pub env: Option<HashMap<String, String>>,
    /// 工具调用的超时时间（秒），未指定时默认为 60。
    pub timeout: Option<u64>,
}

impl std::fmt::Debug for ContextServerCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let filtered_env = self.env.as_ref().map(|env| {
            env.iter()
                .map(|(k, v)| {
                    (
                        k,
                        if util::redact::should_redact(k) {
                            "[REDACTED]"
                        } else {
                            v
                        },
                    )
                })
                .collect::<Vec<_>>()
        });

        f.debug_struct("ContextServerCommand")
            .field("path", &self.path)
            .field("args", &self.args)
            .field("env", &filtered_env)
            .finish()
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct GitSettings {
    /// 是否启用 Git 集成
    ///
    /// 默认值：true
    #[serde(flatten)]
    pub enabled: Option<GitEnabledSettings>,
    /// 是否显示 Git 装订线
    ///
    /// 默认值：tracked_files
    pub git_gutter: Option<GitGutterSetting>,
    /// 设置防抖阈值（毫秒），超过此时间后更改才会反映在 Git 装订线中
    ///
    /// Default: 0
    pub gutter_debounce: Option<u64>,
    /// 是否将 Git 追溯数据内联显示于
    /// 当前聚焦的行。
    ///
    /// 默认值：on
    pub inline_blame: Option<InlineBlameSettings>,
    /// Git 追溯设置
    pub blame: Option<BlameSettings>,
    /// 在分支选择器中显示哪些信息
    ///
    /// 默认值：on
    pub branch_picker: Option<BranchPickerSettingsContent>,
    /// 文件差异设置
    pub file_diff: Option<FileDiffSettingsContent>,
    /// 区块在编辑器中的视觉显示方式
    ///
    /// 默认值：staged_hollow
    pub hunk_style: Option<GitHunkStyleSetting>,
    /// 装订线、文件颜色、git::Diff 等 Git 功能所依据的差异基准
    ///
    /// 默认值：head
    pub diff_base: Option<GitDiffBaseSetting>,
    /// 文件路径在 Git 装订线中的显示方式
    ///
    /// 默认值：file_name_first
    pub path_style: Option<GitPathStyle>,
    /// 是否在差异区块上显示暂存和还原按钮
    ///
    /// 默认值：true
    pub show_stage_restore_buttons: Option<bool>,
    /// 创建 git 工作树的目录，相对于仓库
    /// 工作目录。
    ///
    /// 当解析出的目录位于项目根目录外时，
    /// 项目目录名会自动附加，以避免
    /// 同级仓库冲突。例如，使用默认值
    /// `"../worktrees"` 和位于 `~/code/zed` 的项目，工作树将
    /// 创建在 `~/code/worktrees/zed/` 目录下。
    ///
    /// 当解析出的目录位于项目根目录内时，不会
    /// 添加额外的路径组件（它已限定在项目范围内）。
    ///
    /// 示例：
    /// - `"../worktrees"` — `~/code/worktrees/<project>/`（默认）
    /// - `".git/zed-worktrees"` — `<project>/.git/zed-worktrees/`
    /// - `"my-worktrees"` — `<project>/my-worktrees/`
    ///
    /// 忽略尾部斜杠
    ///
    /// 默认值： ../worktrees
    pub worktree_directory: Option<String>,
    /// Settings for automatically fetching the default branch's remote and the current branch's upstream remote.
    pub auto_fetch: Option<AutoFetchSettingsContent>,
}

#[with_fallible_options]
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct AutoFetchSettingsContent {
    /// Whether to automatically fetch in the background.
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
    /// How long to wait between automatic fetches, in seconds.
    /// Only used when `enabled` is true. (min: 15)
    ///
    /// Default: 60
    pub interval_secs: Option<u64>,
}

#[with_fallible_options]
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct GitEnabledSettings {
    pub disable_git: Option<bool>,
    pub enable_status: Option<bool>,
    pub enable_diff: Option<bool>,
}

impl GitEnabledSettings {
    pub fn is_git_status_enabled(&self) -> bool {
        !self.disable_git.unwrap_or(false) && self.enable_status.unwrap_or(true)
    }

    pub fn is_git_diff_enabled(&self) -> bool {
        !self.disable_git.unwrap_or(false) && self.enable_diff.unwrap_or(true)
    }
}

#[derive(
    Clone,
    Copy,
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
pub enum GitGutterSetting {
    /// 在已跟踪文件中显示 Git 装订线
    #[default]
    #[strum(serialize = "已跟踪文件")]
    TrackedFiles,
    /// 隐藏 Git 装订线
    #[strum(serialize = "隐藏")]
    Hide,
}

#[derive(
    Clone,
    Copy,
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
pub enum InlineBlameLocation {
    /// 在当前行内联显示 Git 追溯
    #[default]
    #[strum(serialize = "内联")]
    Inline,
    /// 在窗口底部的状态栏中显示 Git 追溯
    #[strum(serialize = "状态栏")]
    StatusBar,
}

#[with_fallible_options]
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct InlineBlameSettings {
    /// 是否将 Git 追溯数据内联显示于
    /// 当前聚焦的行。
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 是否仅在光标停止移动后
    /// 延迟一段时间才显示内联追溯信息。
    ///
    /// Default: 0
    pub delay_ms: Option<DelayMs>,
    /// 启用后渲染追溯信息的位置
    ///
    /// 默认值：inline
    pub location: Option<InlineBlameLocation>,
    /// 源代码行末尾与
    /// 内联追溯起始之间的间距（以列为单位）。
    ///
    /// 默认值：7
    pub padding: Option<u32>,
    /// 显示内联追溯信息的最小列号
    ///
    /// Default: 0
    pub min_column: Option<u32>,
    /// 是否将提交摘要显示为内联追溯的一部分
    ///
    /// 默认值：false
    pub show_commit_summary: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct BlameSettings {
    /// 是否显示提交作者的头像
    ///
    /// 默认值：true
    pub show_avatar: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct BranchPickerSettingsContent {
    /// 是否将作者名称显示为提交信息的一部分
    ///
    /// 默认值：false
    pub show_author_name: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Copy, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub struct FileDiffSettingsContent {
    /// 新打开的文件差异是否显示完整文件，而非仅显示更改
    ///
    /// 默认值：true
    pub show_full_file: Option<bool>,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum GitHunkStyleSetting {
    /// 未暂存区块显示为实心背景，已暂存区块显示为空心
    #[default]
    #[strum(serialize = "已暂存空心")]
    StagedHollow,
    /// 未暂存区块显示为空心，已暂存区块显示为实心背景
    #[strum(serialize = "未暂存空心")]
    UnstagedHollow,
}

#[derive(
    Clone,
    Copy,
    PartialEq,
    Debug,
    Default,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum GitDiffBaseSetting {
    /// 与 HEAD 进行差异比较：显示工作区（未提交）更改
    #[default]
    #[strum(serialize = "HEAD")]
    Head,
    /// 以 HEAD 与仓库默认分支的合并基准进行差异比较。
    /// 显示当前分支上的所有更改。
    ///
    /// 无法解析默认分支时，仓库将回退。
    /// 此时采用 `head` 行为。
    #[strum(serialize = "默认分支")]
    DefaultBranch,
}

#[with_fallible_options]
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
pub enum GitPathStyle {
    /// 先显示文件名，后显示路径
    #[default]
    #[strum(serialize = "文件名优先")]
    FileNameFirst,
    /// 先显示完整路径
    #[strum(serialize = "文件路径优先")]
    FilePathFirst,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct DiagnosticsSettingsContent {
    /// 是否在状态栏中显示项目诊断按钮
    pub button: Option<bool>,

    /// 是否包含警告级别的诊断信息
    ///
    /// 默认值：true
    pub include_warnings: Option<bool>,

    /// 在 Zed 中使用 LSP 拉取诊断机制的设置
    pub lsp_pull_diagnostics: Option<LspPullDiagnosticsSettingsContent>,

    /// 显示内联诊断的设置
    pub inline: Option<InlineDiagnosticsSettingsContent>,
}

#[with_fallible_options]
#[derive(
    Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq,
)]
pub struct LspPullDiagnosticsSettingsContent {
    /// 是否拉取诊断信息
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 从语言服务器拉取诊断信息前等待的最短时间
    /// 0 表示关闭防抖
    ///
    /// 默认值：50
    pub debounce_ms: Option<DelayMs>,
}

#[with_fallible_options]
#[derive(
    Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom, Eq,
)]
pub struct InlineDiagnosticsSettingsContent {
    /// 是否显示内联诊断
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
    /// 是否仅在上一次编辑器事件后延迟一段时间
    /// 才显示内联诊断信息。
    ///
    /// 默认值：150
    pub update_debounce_ms: Option<DelayMs>,
    /// 源代码行末尾与
    /// 内联诊断起始之间的间距（以列为单位）。
    ///
    /// 默认值：4
    pub padding: Option<u32>,
    /// 显示内联诊断信息的最小列号。此设置可
    /// 用于在某个位置水平对齐内联诊断信息。行长度
    /// 超过此值时，诊断信息仍会继续向右推移。
    ///
    /// Default: 0
    pub min_column: Option<u32>,

    pub max_severity: Option<DiagnosticSeverityContent>,
}

#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct NodeBinarySettings {
    /// Node 二进制文件的路径
    pub path: Option<String>,
    /// Zed 应使用的 npm 二进制文件路径（默认为 `.path/../npm`）
    pub npm_path: Option<String>,
    /// 如果启用，Zed 将下载自己的 Node 副本
    pub ignore_system_version: Option<bool>,
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum DirenvSettings {
    /// 通过 shell 钩子加载 direnv 配置
    ShellHook,
    /// 直接使用 `direnv export json` 加载 direnv 配置
    #[default]
    Direct,
    /// 不加载 direnv 配置
    Disabled,
}

#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverityContent {
    // No diagnostics are shown.
    #[strum(serialize = "关闭")]
    Off,
    #[strum(serialize = "错误")]
    Error,
    #[strum(serialize = "警告")]
    Warning,
    #[strum(serialize = "信息")]
    Info,
    #[strum(serialize = "提示")]
    Hint,
    #[strum(serialize = "全部")]
    All,
}

/// 自定义 Git 托管提供商
#[with_fallible_options]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct GitHostingProviderConfig {
    /// 提供商的类型
    ///
    /// 必须为 `github`、`gitlab`、`bitbucket`、`gitea`、`forgejo`、`gerrit`、`sourcehut`、`tangled` 或 `gitiles` 之一
    pub provider: GitHostingProviderKind,

    /// 提供商的基础 URL（例如，"https://code.corp.big.com"）
    pub base_url: String,

    /// 提供商的显示名称（例如，"BigCorp GitHub"）
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum GitHostingProviderKind {
    Github,
    Gitlab,
    Bitbucket,
    Gitea,
    Forgejo,
    Gerrit,
    SourceHut,
    Tangled,
    Gitiles,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merge_from::MergeFrom;

    fn exclusions(globs: &[&str]) -> WorktreeSettingsContent {
        WorktreeSettingsContent {
            file_scan_exclusions: Some(SplicingVec::from(
                globs
                    .iter()
                    .map(|glob| glob.to_string())
                    .collect::<Vec<_>>(),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn test_file_scan_exclusions_splice_rest_of_list() {
        let defaults = exclusions(&["**/.git", "**/.DS_Store"]);

        let mut extended = defaults.clone();
        extended.merge_from(&exclusions(&["**/node_modules", SplicingVec::REST]));
        assert_eq!(
            extended.file_scan_exclusions.unwrap().0,
            vec!["**/node_modules", "**/.git", "**/.DS_Store"]
        );

        let mut replaced = defaults;
        replaced.merge_from(&exclusions(&["**/node_modules"]));
        assert_eq!(
            replaced.file_scan_exclusions.unwrap().0,
            vec!["**/node_modules"]
        );
    }

    #[test]
    fn test_file_scan_exclusions_splice_each_layer() {
        let mut settings = exclusions(&["**/.git"]);
        settings.merge_from(&exclusions(&[SplicingVec::REST, "**/target"]));
        settings.merge_from(&exclusions(&[SplicingVec::REST, "**/dist"]));

        assert_eq!(
            settings.file_scan_exclusions.unwrap().0,
            vec!["**/.git", "**/target", "**/dist"]
        );
    }

    #[test]
    fn test_file_scan_exclusions_splice_edge_cases() {
        let mut repeated = exclusions(&["**/.git"]);
        repeated.merge_from(&exclusions(&[SplicingVec::REST, SplicingVec::REST]));
        assert_eq!(repeated.file_scan_exclusions.unwrap().0, vec!["**/.git"]);

        let mut relisted = exclusions(&["**/.git", "**/.DS_Store"]);
        relisted.merge_from(&exclusions(&["**/.git", SplicingVec::REST]));
        assert_eq!(
            relisted.file_scan_exclusions.unwrap().0,
            vec!["**/.git", "**/.DS_Store"]
        );

        let mut cleared = exclusions(&["**/.git"]);
        cleared.merge_from(&exclusions(&[]));
        assert!(cleared.file_scan_exclusions.unwrap().0.is_empty());

        let mut unchanged = exclusions(&["**/.git", "**/.DS_Store"]);
        unchanged.merge_from(&exclusions(&[SplicingVec::REST]));
        assert_eq!(
            unchanged.file_scan_exclusions.unwrap().0,
            vec!["**/.git", "**/.DS_Store"]
        );
    }

    #[test]
    fn test_file_scan_exclusions_splice_without_a_base_layer() {
        let mut settings = WorktreeSettingsContent::default();
        settings.merge_from(&exclusions(&[SplicingVec::REST, "**/target"]));

        // `Option::merge_from` replaces a `None` base outright rather than
        // splicing, so the sentinel survives here. `assets/settings/default.json`
        // always populates this field, and `WorktreeSettings::from_settings`
        // unwraps it, so no glob is ever compiled from this state.
        assert_eq!(
            settings.file_scan_exclusions.unwrap().0,
            vec![SplicingVec::REST, "**/target"]
        );
    }

    fn inclusions(globs: &[&str]) -> WorktreeSettingsContent {
        WorktreeSettingsContent {
            file_scan_inclusions: Some(SplicingVec::from(
                globs
                    .iter()
                    .map(|glob| glob.to_string())
                    .collect::<Vec<_>>(),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn test_file_scan_inclusions_splice_each_layer() {
        let mut settings = inclusions(&[".env*"]);
        settings.merge_from(&inclusions(&[SplicingVec::REST, "**/*.user"]));
        assert_eq!(settings, inclusions(&[".env*", "**/*.user"]));

        settings.merge_from(&inclusions(&["**/*.project", SplicingVec::REST]));
        assert_eq!(
            settings,
            inclusions(&["**/*.project", ".env*", "**/*.user"])
        );
    }

    #[test]
    fn test_file_scan_inclusions_replace_and_clear() {
        let mut settings = inclusions(&[".env*"]);
        settings.merge_from(&inclusions(&["**/*.user"]));
        assert_eq!(settings, inclusions(&["**/*.user"]));

        settings.merge_from(&inclusions(&["**/*.project"]));
        assert_eq!(settings, inclusions(&["**/*.project"]));

        settings.merge_from(&inclusions(&[]));
        assert_eq!(settings, inclusions(&[]));

        settings.merge_from(&inclusions(&[SplicingVec::REST, "**/*.next"]));
        assert_eq!(settings, inclusions(&["**/*.next"]));
    }

    #[test]
    fn test_file_scan_inclusions_splice_preserves_first_occurrence() {
        let inherited = inclusions(&[".env*", "**/*.user"]);
        let mut settings = inherited.clone();
        settings.merge_from(&WorktreeSettingsContent::default());
        assert_eq!(settings, inherited);

        settings.merge_from(&inclusions(&[SplicingVec::REST, SplicingVec::REST]));
        assert_eq!(settings, inherited);

        settings.merge_from(&inclusions(&[
            "**/*.user",
            SplicingVec::REST,
            ".env*",
            "**/*.project",
            SplicingVec::REST,
            "**/*.project",
        ]));
        assert_eq!(
            settings,
            inclusions(&["**/*.user", ".env*", "**/*.project"])
        );

        settings.merge_from(&inclusions(&["**/*.project", "**/*.user", "**/*.project"]));
        assert_eq!(settings, inclusions(&["**/*.project", "**/*.user"]));
    }

    fn hidden_files(globs: &[&str]) -> WorktreeSettingsContent {
        WorktreeSettingsContent {
            hidden_files: Some(SplicingVec::from(
                globs
                    .iter()
                    .map(|glob| glob.to_string())
                    .collect::<Vec<_>>(),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn test_hidden_files_splice_each_layer() {
        let mut settings = hidden_files(&["**/.*"]);
        settings.merge_from(&hidden_files(&[SplicingVec::REST, "**/*.user"]));
        assert_eq!(settings, hidden_files(&["**/.*", "**/*.user"]));

        settings.merge_from(&hidden_files(&["**/*.project", SplicingVec::REST]));
        assert_eq!(
            settings,
            hidden_files(&["**/*.project", "**/.*", "**/*.user"])
        );

        settings.merge_from(&hidden_files(&[
            "**/*.before",
            SplicingVec::REST,
            "**/*.after",
        ]));
        assert_eq!(
            settings,
            hidden_files(&[
                "**/*.before",
                "**/*.project",
                "**/.*",
                "**/*.user",
                "**/*.after",
            ])
        );
    }

    #[test]
    fn test_hidden_files_replace_and_clear() {
        let mut settings = hidden_files(&["**/.*"]);
        settings.merge_from(&hidden_files(&["**/*.user"]));
        assert_eq!(settings, hidden_files(&["**/*.user"]));

        settings.merge_from(&hidden_files(&["**/*.project"]));
        assert_eq!(settings, hidden_files(&["**/*.project"]));

        settings.merge_from(&hidden_files(&[]));
        assert_eq!(settings, hidden_files(&[]));

        settings.merge_from(&hidden_files(&[SplicingVec::REST, "**/*.next"]));
        assert_eq!(settings, hidden_files(&["**/*.next"]));
    }

    #[test]
    fn test_hidden_files_splice_preserves_first_occurrence() {
        let inherited = hidden_files(&["**/.*", "**/*.user"]);
        let mut settings = inherited.clone();
        settings.merge_from(&WorktreeSettingsContent::default());
        assert_eq!(settings, inherited);

        settings.merge_from(&hidden_files(&[SplicingVec::REST]));
        assert_eq!(settings, inherited);

        settings.merge_from(&hidden_files(&[SplicingVec::REST, SplicingVec::REST]));
        assert_eq!(settings, inherited);

        settings.merge_from(&hidden_files(&[
            "**/*.user",
            SplicingVec::REST,
            "**/.*",
            "**/*.project",
            SplicingVec::REST,
            "**/*.project",
        ]));
        assert_eq!(
            settings,
            hidden_files(&["**/*.user", "**/.*", "**/*.project"])
        );

        settings.merge_from(&hidden_files(&[
            "**/*.project",
            "**/*.user",
            "**/*.project",
        ]));
        assert_eq!(settings, hidden_files(&["**/*.project", "**/*.user"]));
    }

    fn read_only_files(globs: &[&str]) -> WorktreeSettingsContent {
        WorktreeSettingsContent {
            read_only_files: Some(SplicingVec::from(
                globs
                    .iter()
                    .map(|glob| glob.to_string())
                    .collect::<Vec<_>>(),
            )),
            ..Default::default()
        }
    }

    #[test]
    fn test_read_only_files_splice_rest_of_list() {
        let inherited = read_only_files(&["**/*.lock"]);
        let mut extended = inherited.clone();
        extended.merge_from(&read_only_files(&["**/generated/**", SplicingVec::REST]));
        assert_eq!(extended, read_only_files(&["**/generated/**", "**/*.lock"]));

        let mut replaced = inherited;
        replaced.merge_from(&read_only_files(&["**/generated/**"]));
        assert_eq!(replaced, read_only_files(&["**/generated/**"]));
    }

    #[test]
    fn test_read_only_files_splice_each_layer() {
        let mut settings = read_only_files(&[]);
        settings.merge_from(&read_only_files(&[SplicingVec::REST, "**/*.lock"]));
        settings.merge_from(&read_only_files(&[SplicingVec::REST, "**/generated/**"]));
        settings.merge_from(&read_only_files(&[SplicingVec::REST, "**/vendor/**"]));
        assert_eq!(
            settings,
            read_only_files(&["**/*.lock", "**/generated/**", "**/vendor/**"])
        );
    }

    #[test]
    fn test_read_only_files_splice_edge_cases() {
        let inherited = read_only_files(&["**/*.gen.rs", "**/*.lock"]);
        let mut repeated = inherited.clone();
        repeated.merge_from(&read_only_files(&[SplicingVec::REST, SplicingVec::REST]));
        assert_eq!(repeated, inherited);

        let mut relisted = inherited.clone();
        relisted.merge_from(&read_only_files(&["**/*.lock", SplicingVec::REST]));
        assert_eq!(relisted, read_only_files(&["**/*.lock", "**/*.gen.rs"]));

        let mut cleared = inherited.clone();
        cleared.merge_from(&read_only_files(&[]));
        assert_eq!(cleared, read_only_files(&[]));

        let mut unchanged = inherited.clone();
        unchanged.merge_from(&read_only_files(&[SplicingVec::REST]));
        assert_eq!(unchanged, inherited);
        unchanged.merge_from(&WorktreeSettingsContent::default());
        assert_eq!(unchanged, inherited);
    }

    #[test]
    fn test_read_only_files_splice_without_a_base_layer() {
        let mut settings = WorktreeSettingsContent::default();
        settings.merge_from(&read_only_files(&[SplicingVec::REST, "**/generated/**"]));
        assert_eq!(
            settings,
            read_only_files(&[SplicingVec::REST, "**/generated/**"])
        );
    }

    #[test]
    fn test_stdio_context_server_without_args() {
        let settings: ContextServerSettingsContent =
            serde_json::from_str(r#"{ "command": "echo" }"#)
                .expect("stdio context server without `args` should parse");
        let ContextServerSettingsContent::Stdio { command, .. } = settings else {
            panic!("expected Stdio variant, got {settings:?}");
        };
        assert_eq!(command.path, PathBuf::from("echo"));
        assert!(command.args.is_empty());

        let settings: ContextServerSettingsContent =
            serde_json::from_str(r#"{ "command": "echo", "args": ["hello"] }"#).unwrap();
        let ContextServerSettingsContent::Stdio { command, .. } = settings else {
            panic!("expected Stdio variant, got {settings:?}");
        };
        assert_eq!(command.args, vec!["hello".to_string()]);
    }
}
