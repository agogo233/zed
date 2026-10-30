use gpui::{Action, actions};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Arc};

// If the zed binary doesn't use anything in this crate, it will be optimized away
// and the actions won't initialize. So we just provide an empty initialization function
// to be called from main.
//
// These may provide relevant context:
// https://github.com/rust-lang/rust/issues/47384
// https://github.com/mmastrac/rust-ctor/issues/280
pub fn init() {}

/// 在系统默认浏览器中打开一个 URL
#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct OpenBrowser {
    pub url: Arc<str>,
}

/// 在应用程序内打开一个 zed:// URL
#[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct OpenZedUrl {
    pub url: Arc<str>,
}

/// Opens the keymap to either add a keybinding or change an existing one
#[derive(PartialEq, Clone, Default, Action, JsonSchema, Serialize, Deserialize)]
#[action(namespace = zed, no_json, no_register)]
pub struct ChangeKeybinding {
    pub action: String,
}

actions!(
    zed,
    [
        /// 打开设置编辑器
        #[action(deprecated_aliases = ["zed_actions::OpenSettingsEditor"])]
        OpenSettings,
        /// 打开设置 JSON 文件
        #[action(deprecated_aliases = ["zed_actions::OpenSettings"])]
        OpenSettingsFile,
        /// 打开项目专属设置
        #[action(deprecated_aliases = ["zed_actions::OpenProjectSettings"])]
        OpenProjectSettings,
        /// 打开项目任务配置
        OpenProjectTasks,
        /// 打开项目任务配置，并提供工作树设置指南
        OpenWorktreeSetupTasks,
        /// 打开默认键映射文件
        OpenDefaultKeymap,
        /// 打开用户键映射文件
        #[action(deprecated_aliases = ["zed_actions::OpenKeymap"])]
        OpenKeymapFile,
        /// 打开键映射编辑器
        #[action(deprecated_aliases = ["zed_actions::OpenKeymapEditor"])]
        OpenKeymap,
        /// 打开帐户设置
        OpenAccountSettings,
        /// 打开服务器设置
        OpenServerSettings,
        /// 退出应用程序
        Quit,
        /// 显示 Zed 的相关信息
        About,
        /// 打开文档网站
        OpenDocs,
        /// 查看开源许可证
        OpenLicenses,
        /// 打开 Zed 状态页面
        OpenStatusPage,
        /// 打开 Zed 周边商店
        GetMerch,
        /// 打开遥测日志
        OpenTelemetryLog,
        /// 打开性能分析器
        OpenPerformanceProfiler,
        /// 打开引导视图
        OpenOnboarding,
        /// 显示用于测试的自动更新通知
        ShowUpdateNotification,
    ]
);

#[derive(PartialEq, Clone, Copy, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionCategoryFilter {
    Themes,
    IconThemes,
    Languages,
    Grammars,
    LanguageServers,
    ContextServers,
    Snippets,
    DebugAdapters,
}

/// 打开扩展管理界面
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
    /// Filters the extensions page down to extensions that are in the specified category.
    #[serde(default)]
    pub category_filter: Option<ExtensionCategoryFilter>,
    /// Focuses just the extension with the specified ID.
    #[serde(default)]
    pub id: Option<String>,
}

/// 打开 ACP 注册表
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct AcpRegistry;

/// 显示通话诊断和连接质量统计信息
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = collab)]
#[serde(deny_unknown_fields)]
pub struct ShowCallStats;

/// 减小编辑器缓冲区的字体大小
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct DecreaseBufferFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 增大编辑器缓冲区的字体大小
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct IncreaseBufferFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 在指定路径处打开设置编辑器
#[derive(PartialEq, Clone, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct OpenSettingsAt {
    /// A path to a specific setting (e.g. `theme.mode`)
    pub path: String,
    /// The settings file to select before opening `path`. When omitted, the
    /// existing settings file selection is preserved.
    #[serde(default)]
    pub target: Option<OpenSettingsAtTarget>,
}

#[derive(PartialEq, Clone, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct OpenSettingsPage {
    /// A settings page title (e.g. `AI`).
    pub page: String,
    /// The settings file to select before opening `page`. When omitted, the
    /// existing settings file selection is preserved.
    #[serde(default)]
    pub target: Option<OpenSettingsAtTarget>,
}

/// `OpenSettingsAt` path of the agent skills page in the settings UI.
pub const AGENT_SKILLS_SETTINGS_PATH: &str = "agent.skills";

/// `OpenSettingsAt` path of the agent sandbox permissions page in the settings
/// UI.
pub const AGENT_SANDBOX_SETTINGS_PATH: &str = "agent.sandbox_permissions";

#[derive(PartialEq, Clone, Debug, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OpenSettingsAtTarget {
    User,
    Project { worktree_id: usize },
}

/// 将缓冲区字体大小重置为默认值
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct ResetBufferFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 减小用户界面的字体大小
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct DecreaseUiFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 增大用户界面的字体大小
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct IncreaseUiFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 将 UI 字体大小重置为默认值
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct ResetUiFontSize {
    #[serde(default)]
    pub persist: bool,
}

/// 将所有缩放级别（UI 和缓冲区字体大小，包括智能体面板）重置为默认值
#[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
#[action(namespace = zed)]
#[serde(deny_unknown_fields)]
pub struct ResetAllZoom {
    #[serde(default)]
    pub persist: bool,
}

pub mod editor {
    use gpui::actions;
    actions!(
        editor,
        [
            /// 向上移动光标
            MoveUp,
            /// 向下移动光标
            MoveDown,
            /// 在系统文件管理器中显示当前文件
            RevealInFileManager,
        ]
    );
}

pub mod dev {
    use gpui::actions;

    actions!(
        dev,
        [
            /// 切换用于调试 UI 元素的开发者检查器
            ToggleInspector,
            /// 将调试帧时间叠加层在隐藏、当前
            /// 帧时间和详细帧时间统计信息之间循环切换
            ToggleFpsOverlay,
            /// 重置调试帧时间叠加层的统计信息，但
            /// 总帧数除外
            ResetFrameOverlayStats,
            /// 打开键上下文视图以调试键绑定
            OpenKeyContextView
        ]
    );
}

pub mod remote_debug {
    use gpui::actions;

    actions!(
        remote_debug,
        [
            /// 模拟断开远程服务器连接以进行测试
            /// 这将触发重连逻辑
            SimulateDisconnect,
            /// 模拟远程服务器超时/慢速连接以进行测试
            /// 这将导致心跳失败并触发重连
            SimulateTimeout,
            /// 模拟远程服务器超时/慢速连接以进行测试
            /// 这将导致心跳失败，并在耗尽所有重连尝试后继续尝试重连
            SimulateTimeoutExhausted,
        ]
    );
}

pub mod workspace {
    use gpui::actions;

    actions!(
        workspace,
        [
            #[action(deprecated_aliases = ["editor::CopyPath", "outline_panel::CopyPath", "project_panel::CopyPath"])]
            CopyPath,
            #[action(deprecated_aliases = ["editor::CopyRelativePath", "outline_panel::CopyRelativePath", "project_panel::CopyRelativePath"])]
            CopyRelativePath,
            /// 用系统默认应用程序打开所选文件
            #[action(deprecated_aliases = ["project_panel::OpenWithSystem"])]
            OpenWithSystem,
        ]
    );
}

/// Describes which ref to base a new git worktree on. The worktree is
/// always created in a detached HEAD state; users can opt into creating
/// a branch afterwards from the worktree itself.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum NewWorktreeBranchTarget {
    /// Create a detached worktree from the current HEAD.
    #[default]
    CurrentBranch,
    /// Create a detached worktree at the tip of an existing branch.
    ExistingBranch { name: String },
    /// Create a detached worktree at the tip of a remote-tracking branch.
    RemoteBranch {
        remote_name: String,
        branch_name: String,
    },
}

/// 创建新的 Git 工作树并将工作区切换至该工作树
/// 当用户在统一工作树选择器中选择“新建工作树”条目时触发
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Action)]
#[action(namespace = git)]
#[serde(deny_unknown_fields)]
pub struct CreateWorktree {
    /// When this is None, Zed will randomly generate a worktree name.
    pub worktree_name: Option<String>,
    pub branch_target: NewWorktreeBranchTarget,
}

/// 将工作区切换到已有的关联工作树
/// 当用户在统一工作树选择器中选择现有工作树时触发
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Action)]
#[action(namespace = git)]
#[serde(deny_unknown_fields)]
pub struct SwitchWorktree {
    pub path: PathBuf,
    pub display_name: String,
}

/// 在新窗口中打开现有工作树
/// 当用户点击工作树选择器的“在新窗口中打开”按钮时触发
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Action)]
#[action(namespace = git)]
#[serde(deny_unknown_fields)]
pub struct OpenWorktreeInNewWindow {
    pub path: PathBuf,
}

pub mod git {
    use gpui::actions;

    actions!(
        git,
        [
            /// 签出不同的 Git 分支
            CheckoutBranch,
            /// 切换到不同的 Git 分支
            Switch,
            /// 选择其他仓库
            SelectRepo,
            /// 筛选远程仓库
            FilterRemotes,
            /// 创建 Git 远程仓库
            CreateRemote,
            /// 打开 Git 分支选择器
            #[action(deprecated_aliases = ["branches::OpenRecent"])]
            Branch,
            /// 显示整个项目中未提交的更改
            ViewUncommittedChanges,
            /// 显示整个项目中未暂存的更改
            ViewUnstagedChanges,
            /// 显示整个项目中已暂存的更改
            ViewStagedChanges,
            /// 打开 Git 贮藏选择器
            ViewStash,
            /// 打开 Git 工作树选择器
            Worktree,
            /// 为当前分支创建拉取请求
            CreatePullRequest
        ]
    );
}

pub mod toast {
    use gpui::actions;

    actions!(
        toast,
        [
            /// 执行与提示通知关联的操作
            RunAction
        ]
    );
}

pub mod command_palette {
    use gpui::actions;

    actions!(
        command_palette,
        [
            /// 切换命令面板
            Toggle,
        ]
    );
}

pub mod text_finder {
    use gpui::actions;

    actions!(
        text_finder,
        [
            /// 打开项目搜索选择器
            Toggle,
        ]
    );
}

pub mod project_panel {
    use gpui::actions;

    actions!(
        project_panel,
        [
            /// 切换项目面板
            Toggle,
            /// 切换项目面板的焦点
            ToggleFocus
        ]
    );
}
pub mod feedback {
    use gpui::actions;

    actions!(
        feedback,
        [
            /// 打开邮件客户端，向 Zed 支持团队发送反馈
            EmailZed,
            /// 打开错误报告表单
            FileBugReport,
            /// 打开功能请求表单
            RequestFeature
        ]
    );
}

pub mod theme {
    use gpui::actions;

    actions!(theme, [ToggleMode]);
}

pub mod theme_selector {
    use gpui::Action;
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// 切换主题选择器界面
    #[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
    #[action(namespace = theme_selector)]
    #[serde(deny_unknown_fields)]
    pub struct Toggle {
        /// A list of theme names to filter the theme selector down to.
        pub themes_filter: Option<Vec<String>>,
    }
}

pub mod icon_theme_selector {
    use gpui::Action;
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// 切换图标主题选择器界面
    #[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
    #[action(namespace = icon_theme_selector)]
    #[serde(deny_unknown_fields)]
    pub struct Toggle {
        /// A list of icon theme names to filter the theme selector down to.
        pub themes_filter: Option<Vec<String>>,
    }
}

pub mod search {
    use gpui::{Action, actions};

    /// Opens a new project search filtered down to the given directory.
    ///
    /// An internal forwarding action: the user-facing, keybindable entry
    /// point is `project_panel::NewSearchInDirectory`, which resolves the
    /// selected directory and dispatches this action with it.
    #[derive(Clone, Debug, Default, PartialEq, Action)]
    #[action(namespace = search, no_json, no_register)]
    pub struct NewSearchInDirectory {
        pub directory: String,
    }

    actions!(
        search,
        [
            /// 聚焦到搜索输入框
            FocusSearch,
            /// 选中下一个搜索匹配项
            SelectNextMatch,
            /// 选中上一个搜索匹配项
            SelectPreviousMatch,
            /// 切换区分大小写搜索
            ToggleCaseSensitive,
            /// 切换在忽略文件中搜索
            ToggleIncludeIgnored
        ]
    );
}
pub mod buffer_search {
    use gpui::{Action, actions};
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// 以指定配置打开缓冲区搜索界面
    #[derive(PartialEq, Clone, Deserialize, JsonSchema, Action)]
    #[action(namespace = buffer_search)]
    #[serde(deny_unknown_fields)]
    pub struct Deploy {
        #[serde(default = "util::serde::default_true")]
        pub focus: bool,
        #[serde(default)]
        pub replace_enabled: bool,
        #[serde(default)]
        pub selection_search_enabled: bool,
    }

    impl Deploy {
        pub fn find() -> Self {
            Self {
                focus: true,
                replace_enabled: false,
                selection_search_enabled: false,
            }
        }

        pub fn replace() -> Self {
            Self {
                focus: true,
                replace_enabled: true,
                selection_search_enabled: false,
            }
        }
    }

    actions!(
        buffer_search,
        [
            /// 打开搜索和替换界面
            DeployReplace,
            /// 关闭搜索栏
            Dismiss,
            /// 将焦点切换回编辑器
            FocusEditor,
            /// 从选区或光标下的单词设置搜索查询
            UseSelectionForFind,
        ]
    );
}
pub mod settings_profile_selector {
    use gpui::Action;
    use schemars::JsonSchema;
    use serde::Deserialize;

    #[derive(PartialEq, Clone, Default, Debug, Deserialize, JsonSchema, Action)]
    #[action(namespace = settings_profile_selector)]
    pub struct Toggle;
}

pub mod agent {
    use gpui::{Action, SharedString, actions};
    use schemars::JsonSchema;
    use serde::Deserialize;

    actions!(
        agent,
        [
            /// 打开智能体设置界面
            #[action(deprecated_aliases = ["agent::OpenConfiguration"])]
            OpenSettings,
            /// 打开智能体引导对话框
            OpenOnboardingModal,
            /// 重置智能体引导状态
            ResetOnboarding,
            /// 与智能体开始聊天会话
            Chat,
            /// 切换语言模型选择器下拉菜单
            #[action(deprecated_aliases = ["assistant::ToggleModelSelector", "assistant2::ToggleModelSelector"])]
            ToggleModelSelector,
            /// 触发 Gemini 重新身份验证
            ReauthenticateAgent,
            /// 注销当前外部智能体
            LogoutAgent,
            /// 将当前选区作为上下文添加到智能体面板的会话中
            #[action(deprecated_aliases = ["assistant::QuoteSelection", "agent::QuoteSelection"])]
            AddSelectionToThread,
            /// 重置智能体面板的缩放级别（智能体 UI 和缓冲区字体大小）
            ResetAgentZoom,
            /// 粘贴剪贴板内容，不保留任何格式
            PasteRaw,
        ]
    );

    /// 选择智能体面板中新会话使用的智能体，无需
    /// 打开面板。下次打开面板时将启动所选
    /// 智能体。
    #[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
    #[action(namespace = agent)]
    #[serde(deny_unknown_fields)]
    pub struct SelectAgent {
        /// The id of the agent to select.
        pub agent: String,
    }

    /// 打开一个新的智能体会话，以审查所提供的分支差异
    #[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
    #[action(namespace = agent)]
    #[serde(deny_unknown_fields)]
    pub struct ReviewBranchDiff {
        /// The full text of the diff to review.
        pub diff_text: SharedString,
        /// The base ref that the diff was computed against (e.g. "main").
        pub base_ref: SharedString,
    }

    /// A single merge conflict region extracted from a file.
    #[derive(Clone, Debug, PartialEq, Deserialize, JsonSchema)]
    pub struct ConflictContent {
        pub file_path: String,
        pub conflict_text: String,
        pub ours_branch_name: String,
        pub theirs_branch_name: String,
    }

    /// 打开一个新的智能体会话，以解决特定的合并冲突
    #[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
    #[action(namespace = agent)]
    #[serde(deny_unknown_fields)]
    pub struct ResolveConflictsWithAgent {
        /// Individual conflicts with their full text.
        pub conflicts: Vec<ConflictContent>,
    }

    /// 打开一个新的智能体会话，以解决指定文件路径中的合并冲突
    #[derive(Clone, PartialEq, Deserialize, JsonSchema, Action)]
    #[action(namespace = agent)]
    #[serde(deny_unknown_fields)]
    pub struct ResolveConflictedFilesWithAgent {
        /// File paths with unresolved conflicts (for project-wide resolution).
        pub conflicted_file_paths: Vec<String>,
    }
}

pub mod assistant {
    use gpui::{Action, actions};
    use schemars::JsonSchema;
    use serde::Deserialize;

    actions!(
        agent,
        [
            /// 切换智能体面板
            Toggle,
            #[action(deprecated_aliases = ["assistant::ToggleFocus"])]
            ToggleFocus,
            FocusAgent,
            /// 打开技能创建器窗口以创建新技能
            OpenSkillCreator,
            /// 打开技能创建器窗口以从 GitHub URL 导入技能
            CreateSkillFromUrl,
            /// 打开用户全局 AGENTS.md 规则文件
            #[action(name = "OpenGlobalAGENTS.mdRules")]
            OpenGlobalAgentsMdRules,
            /// 打开项目 AGENTS.md 规则文件
            #[action(name = "OpenProjectAGENTS.mdRules")]
            OpenProjectAgentsMdRules,
            /// 在设置窗口中打开技能管理器
            #[action(deprecated_aliases = ["agent::OpenRulesLibrary", "assistant::OpenRulesLibrary", "assistant::DeployPromptLibrary"])]
            ManageSkills,
        ]
    );

    /// 使用指定配置打开助手界面
    #[derive(Clone, Default, Deserialize, PartialEq, JsonSchema, Action)]
    #[action(namespace = assistant)]
    #[serde(deny_unknown_fields)]
    pub struct InlineAssist {
        pub prompt: Option<String>,
    }
}

/// 打开最近项目界面
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = projects)]
#[serde(deny_unknown_fields)]
pub struct OpenRecent {
    #[serde(default)]
    pub create_new_window: Option<bool>,
}

/// 从所选模板创建项目
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = projects)]
#[serde(deny_unknown_fields)]
pub struct OpenRemote {
    #[serde(default)]
    pub from_existing_connection: bool,
    #[serde(default)]
    pub create_new_window: Option<bool>,
}

/// 打开 Dev Container 连接对话框
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = projects)]
#[serde(deny_unknown_fields)]
pub struct OpenDevContainer;

/// Where to spawn the task in the UI.
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RevealTarget {
    /// In the central pane group, "main" editor area.
    Center,
    /// In the terminal dock, "regular" terminal items' place.
    #[default]
    Dock,
}

/// 按名称运行任务或打开任务对话框
#[derive(Debug, PartialEq, Clone, Deserialize, JsonSchema, Action)]
#[action(namespace = task)]
#[serde(untagged)]
pub enum Spawn {
    /// Spawns a task by the name given.
    ByName {
        task_name: String,
        #[serde(default)]
        reveal_target: Option<RevealTarget>,
    },
    /// Spawns a task by the tag given.
    ByTag {
        task_tag: String,
        #[serde(default)]
        reveal_target: Option<RevealTarget>,
    },
    /// Spawns a task via modal's selection.
    ViaModal {
        /// Selected task's `reveal_target` property override.
        #[serde(default)]
        reveal_target: Option<RevealTarget>,
    },
}

impl Spawn {
    pub fn modal() -> Self {
        Self::ViaModal {
            reveal_target: None,
        }
    }
}

/// 重新运行上一个任务
#[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
#[action(namespace = task)]
#[serde(deny_unknown_fields)]
pub struct Rerun {
    /// Controls whether the task context is reevaluated prior to execution of a task.
    /// If it is not, environment variables such as ZED_COLUMN, ZED_FILE are gonna be the same as in the last execution of a task
    /// If it is, these variables will be updated to reflect current state of editor at the time task::Rerun is executed.
    /// default: false
    #[serde(default)]
    pub reevaluate_context: bool,
    /// Overrides `allow_concurrent_runs` property of the task being reran.
    /// Default: null
    #[serde(default)]
    pub allow_concurrent_runs: Option<bool>,
    /// Overrides `use_new_terminal` property of the task being reran.
    /// Default: null
    #[serde(default)]
    pub use_new_terminal: Option<bool>,

    /// If present, rerun the task with this ID, otherwise rerun the last task.
    #[serde(skip)]
    pub task_id: Option<String>,
}

pub mod outline {
    use std::sync::OnceLock;

    use gpui::{AnyView, App, Window, actions};

    actions!(
        outline,
        [
            #[action(name = "Toggle")]
            ToggleOutline
        ]
    );
    /// A pointer to outline::toggle function, exposed here to sewer the breadcrumbs <-> outline dependency.
    pub static TOGGLE_OUTLINE: OnceLock<fn(AnyView, &mut Window, &mut App)> = OnceLock::new();
}

actions!(
    zed_predict_onboarding,
    [
        /// 打开 Zed Predict 引导对话框
        OpenZedPredictOnboarding
    ]
);
actions!(
    git_onboarding,
    [
        /// 打开 Git 集成引导对话框
        OpenGitIntegrationOnboarding
    ]
);
actions!(
    call_hierarchy,
    [ShowIncomingCalls, ShowOutgoingCalls, ToggleDirection]
);

pub mod debug_panel {
    use gpui::actions;
    actions!(
        debug_panel,
        [
            /// 切换调试面板
            Toggle,
            /// 切换调试面板的焦点
            ToggleFocus
        ]
    );
}

actions!(
    debugger,
    [
        /// 切换断点的启用状态
        ToggleEnableBreakpoint,
        /// 移除一个断点
        UnsetBreakpoint,
        /// 打开项目调试任务配置
        OpenProjectDebugTasks,
    ]
);

pub mod vim {
    use gpui::actions;

    actions!(
        vim,
        [
            /// 打开默认键映射文件
            OpenDefaultKeymap
        ]
    );
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WslConnectionOptions {
    pub distro_name: String,
    pub user: Option<String>,
}

// `debug_assertions` makes the actions visible for the docs preprocessor
#[cfg(any(debug_assertions, target_os = "windows"))]
pub mod wsl_actions {
    use gpui::Action;
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// 在 WSL 内打开一个文件夹
    #[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
    #[action(namespace = projects)]
    #[serde(deny_unknown_fields)]
    pub struct OpenFolderInWsl {
        #[serde(default)]
        pub create_new_window: Option<bool>,
    }

    /// 打开一个 WSL 发行版
    #[derive(PartialEq, Clone, Deserialize, Default, JsonSchema, Action)]
    #[action(namespace = projects)]
    #[serde(deny_unknown_fields)]
    pub struct OpenWsl {
        #[serde(default)]
        pub create_new_window: Option<bool>,
    }
}

pub mod preview {
    pub mod markdown {
        use gpui::actions;

        actions!(
            markdown,
            [
                /// 为当前文件打开 Markdown 预览
                OpenPreview,
                /// 在分割窗格中打开 Markdown 预览
                OpenPreviewToTheSide,
            ]
        );
    }

    pub mod svg {
        use gpui::actions;

        actions!(
            svg,
            [
                /// 为当前文件打开 SVG 预览
                OpenPreview,
                /// 在分割窗格中打开 SVG 预览
                OpenPreviewToTheSide,
            ]
        );
    }
}

pub mod agents_sidebar {
    use gpui::{Action, actions};
    use schemars::JsonSchema;
    use serde::Deserialize;

    /// 切换侧边栏聚焦时的会话切换弹出窗口
    #[derive(PartialEq, Clone, Deserialize, JsonSchema, Default, Action)]
    #[action(namespace = agents_sidebar)]
    #[serde(deny_unknown_fields)]
    pub struct ToggleThreadSwitcher {
        #[serde(default)]
        pub select_last: bool,
    }

    actions!(
        agents_sidebar,
        [
            /// 将焦点移至侧边栏的搜索/筛选编辑器
            FocusSidebarFilter,
        ]
    );
}

pub mod notebook {
    use gpui::actions;

    actions!(
        notebook,
        [
            /// 打开一个 Jupyter 笔记本文件
            OpenNotebook,
            /// 运行笔记本中的所有单元格
            RunAll,
            /// 运行当前单元格并停留在此处
            Run,
            /// 运行当前单元格并移至下一个单元格
            RunAndAdvance,
            /// 清除所有单元格输出
            ClearOutputs,
            /// 将当前单元格向上移动
            MoveCellUp,
            /// 将当前单元格向下移动
            MoveCellDown,
            /// 添加新 Markdown 单元格
            AddMarkdownBlock,
            /// 添加新代码单元格
            AddCodeBlock,
            /// 删除当前单元格
            DeleteCell,
            /// 重启内核
            RestartKernel,
            /// 中断当前执行
            InterruptKernel,
            /// 在单元格间向下移动
            NotebookMoveDown,
            /// 在单元格间向上移动
            NotebookMoveUp,
            /// 进入当前单元格的编辑器（编辑模式）
            EnterEditMode,
            /// 退出单元格编辑器并返回单元格命令模式
            EnterCommandMode,
        ]
    );
}

pub mod git_panel {
    use gpui::actions;

    actions!(
        git_panel,
        [
            /// 切换 Git 面板的焦点
            ToggleFocus,
        ]
    );
}
