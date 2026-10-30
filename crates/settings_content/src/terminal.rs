use std::path::PathBuf;

use collections::{HashMap, HashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

use crate::{FontFamilyName, FontFeaturesContent, FontSize, FontWeightContent};

#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ProjectTerminalSettingsContent {
    /// 打开终端时使用的 Shell
    ///
    /// 默认值：system
    pub shell: Option<Shell>,
    /// 启动终端时使用的工作目录
    ///
    /// 默认值：current_project_directory
    pub working_directory: Option<WorkingDirectory>,
    /// 添加到此列表的任何键值对都将添加到终端的
    /// 环境变量。使用 `:` 分隔多个值。
    ///
    /// 默认值：{}
    pub env: Option<HashMap<String, String>>,
    /// 如果找到 Python 虚拟环境，则在
    /// 终端工作目录中激活它（由 working_directory
    /// 设置解析）。将此项设为 "off" 可禁用该行为。
    ///
    /// 默认值：on
    pub detect_venv: Option<VenvSettings>,
    /// 用于识别超链接导航路径的正则表达式
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "terminal": {
    ///     "path_hyperlink_regexes": [
    ///       // Python 风格的诊断信息
    ///       "File \"(?<path>[^\"]+)\", line (?<line>[0-9]+)",
    ///       // 通用路径语法，包含可选的行号、列号、描述、末尾标点符号，或
    ///       // 周围的符号或引号
    ///       [
    ///         "(?x)",
    ///         "(?<path>",
    ///         "    (",
    ///         "        # multi-char path: first char (not opening delimiter, space, or box drawing char)",
    ///         "        [^({\\[<\"'`\\ \\u2500-\\u257F]",
    ///         "        # middle chars: non-space, and colon/paren only if not followed by digit/paren/space",
    ///         "        ([^\\ :(]|[:(][^0-9()\\ ])*",
    ///         "        # last char: not closing delimiter or colon",
    ///         "        [^()}\\]>\"'`.,;:\\ ]",
    ///         "    |",
    ///         "        # single-char path: not delimiter, punctuation, space, or box drawing char",
    ///         "        [^(){}\\[\\]<>\"'`.,;:\\ \\u2500-\\u257F]",
    ///         "    )",
    ///         "    # optional line/column suffix (included in path for PathWithPosition::parse_str)",
    ///         "    (:+[0-9]+(:[0-9]+)?|:?\\([0-9]+([,:]?[0-9]+)?\\))?",
    ///         ")"
    ///       ]
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加正则表达式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。设为 `[]` 可将其清空。省略此
    /// 设置会保留继承的列表。
    ///
    /// ```json
    /// {
    ///   "terminal": {
    ///     "path_hyperlink_regexes": [
    ///       "\\s+(-->|:::|at) (?<link>(?<path>.+?))(:$|$)",
    ///       "\\s+(Compiling|Checking|Documenting) [^(]+\\((?<link>(?<path>.+))\\)",
    ///       "..."
    ///     ]
    ///   }
    /// }
    /// ```
    ///
    /// 继承的正则表达式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。当正则表达式的文本在
    /// 将多行条目用换行符连接后相同时，即视为重复项。
    ///
    /// 每个正则表达式可以是单个字符串，也可以是用
    /// 换行符连接的字符串数组。该标记只有作为顶层字符串时才会被识别。要将
    /// `...` 用作匹配三个字符的正则表达式，应将 `["..."]` 写为一个条目。
    /// 多行条目内的 `"..."` 行始终视为正则表达式文本。
    ///
    /// 可选的命名捕获组 `path` 用于选择超链接目标。若没有
    /// 该捕获组，则整个匹配内容即为目标。使用 `path` 时，`line` 和 `column`
    /// 用于指定位置。若未捕获到 `line`，内置的后缀
    /// 处理会解析 `line:column` 和 `(line,column)` 形式。可选的
    /// `link` 捕获组用于选择可点击的文本，否则整个匹配内容均
    /// 可点击。
    ///
    /// 处理在第一个匹配终端行的正则表达式处停止，即使
    /// 光标不在其可点击文本内也是如此。将更具体的正则表达式放在
    /// 更宽泛的表达式之前。正则表达式使用 Rust 的 `regex` 语法。无效的正则表达式
    /// 会被记录到日志中并忽略。`path_hyperlink_timeout_ms` 设置控制
    /// 发现超时时间。将其设为 `0` 可禁用路径超链接。
    pub path_hyperlink_regexes: Option<PathHyperlinkRegexes>,
    /// 悬停及 Cmd 点击路径超链接发现的超时时间（毫秒）。
    ///
    /// Default: 1
    pub path_hyperlink_timeout_ms: Option<u64>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Default, Serialize, JsonSchema, MergeFrom)]
pub struct TerminalSettingsContent {
    #[serde(flatten)]
    pub project: ProjectTerminalSettingsContent,
    /// 设置终端的字体大小
    ///
    /// 如果未包含此选项，
    /// 终端将默认使用与缓冲区相同的字体大小
    pub font_size: Option<FontSize>,
    /// 设置终端的字体族
    ///
    /// 如果未包含此选项，
    /// 终端将默认使用与缓冲区相同的字体族
    pub font_family: Option<FontFamilyName>,

    /// 设置终端的字体回退
    ///
    /// 如果未包含此选项，
    /// 终端将默认使用与缓冲区相同的字体回退
    #[schemars(extend("uniqueItems" = true))]
    pub font_fallbacks: Option<Vec<FontFamilyName>>,

    /// 设置终端的行高
    ///
    /// 默认值：comfortable
    pub line_height: Option<TerminalLineHeight>,
    pub font_features: Option<FontFeaturesContent>,
    /// 以 CSS 粗细单位 0-900 设置终端的字体粗细
    pub font_weight: Option<FontWeightContent>,
    /// 终端的默认光标形状
    /// 可以是 "bar"、"block"、"underline" 或 "hollow"。
    ///
    /// 默认值："block"
    pub cursor_shape: Option<CursorShapeContent>,
    /// 设置终端中光标的闪烁行为
    ///
    /// 默认值：terminal_controlled
    pub blinking: Option<TerminalBlink>,
    /// 设置交替滚动模式（代码：?1007）是否默认启用。
    /// 交替滚动模式会在备用屏幕中转换鼠标滚动事件。
    /// 转换结果为上 / 下方向键按下操作，例如运行
    /// vim 或 less 时。终端仍可自行开启或关闭此模式。
    ///
    /// 默认值：on
    pub alternate_scroll: Option<AlternateScroll>,
    /// 设置 Option 键是否作为 Meta 键使用
    ///
    /// 默认值：false
    pub option_as_meta: Option<bool>,
    /// 在终端中选中文本是否会自动
    /// 复制到系统剪贴板。
    ///
    /// 默认值：false
    pub copy_on_select: Option<bool>,
    /// 是否在复制到剪贴板后保留文本选区
    ///
    /// 默认值：true
    pub keep_selection_on_copy: Option<bool>,
    /// cmd-click（在 Linux 和 Windows 上为 ctrl-click）是否会打开超链接，
    /// 即使终端应用程序已启用鼠标报告（例如在 vim 中设置
    /// mouse=a，或使用 htop）。若为 false，这些点击会转发给应用程序，
    /// 此时仍可通过 shift-cmd-click
    /// （shift-ctrl-click）打开超链接。
    ///
    /// 默认值：true
    pub open_links_in_mouse_mode: Option<bool>,
    /// 是否在状态栏中显示终端按钮
    ///
    /// 默认值：true
    pub button: Option<bool>,
    pub dock: Option<TerminalDockPosition>,
    /// 启动时是否打开终端面板
    ///
    /// 默认值：false
    pub starts_open: Option<bool>,
    /// 终端面板是否使用弹性（比例）尺寸
    ///
    /// 默认值：true
    pub flexible: Option<bool>,
    /// 终端停靠到左侧或右侧时的默认宽度
    ///
    /// 默认值：640
    pub default_width: Option<crate::PixelSetting>,
    /// 终端停靠到底部时的默认高度
    ///
    /// 默认值：320
    pub default_height: Option<crate::PixelSetting>,
    /// 滚动历史记录中保留的最大行数
    /// 最大允许值为 100_000，超过该值的所有值将被视为 100_000
    /// 0 表示禁用滚动
    /// 现有终端需要重新创建后才能应用此更改
    /// 有关更多信息，请参阅 <a href="https://github.com/alacritty/alacritty/blob/cb3a79dbf6472740daca8440d5166c1d4af5029e/extra/man/alacritty.5.scd?plain=1#L207-L213">Alacritty documentation</a>
    ///
    /// 默认值：10_000
    pub max_scroll_history_lines: Option<usize>,
    /// 鼠标滚轮滚动的倍数
    ///
    /// 默认值：1.0
    pub scroll_multiplier: Option<f32>,
    /// 工具栏相关设置
    pub toolbar: Option<TerminalToolbarContent>,
    /// 滚动条相关设置
    pub scrollbar: Option<ScrollbarSettingsContent>,
    /// 前景色与背景色之间的最小 APCA 感知对比度
    ///
    /// APCA（无障碍感知对比度算法）比 WCAG 2.x 更精确，
    /// 尤其适用于深色模式。取值范围为 0 到 106。
    ///
    /// 基于 APCA 可读性准则 (ARC) 铜级简单模式：
    /// https://readtech.org/ARC/tests/bronze-simple-mode/
    /// - 0：不调整对比度
    /// - 45：大号流畅可读文本（36px+）的最低要求
    /// - 60：其他内容文本的最低要求
    /// - 75：正文文本的最低要求
    /// - 90：正文文本的推荐值
    ///
    /// 默认值：45
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub minimum_contrast: Option<f32>,
    /// 是否在终端面板图标上显示已打开终端数量的徽标
    ///
    /// 默认值：false
    pub show_count_badge: Option<bool>,
    /// 当 `BEL` 字符（`\a`）输出到终端时执行的操作
    ///
    /// 默认值："system"
    pub bell: Option<TerminalBell>,
}

crate::fallible_options::flattened_deserialize!(TerminalSettingsContent {
    sections: { project },
    options: {
        font_size, font_family, font_fallbacks, line_height, font_features, font_weight,
        cursor_shape, blinking, alternate_scroll, option_as_meta, copy_on_select,
        keep_selection_on_copy, open_links_in_mouse_mode, button, dock, starts_open, flexible,
        default_width, default_height, max_scroll_history_lines, scroll_multiplier, toolbar,
        scrollbar, minimum_contrast, show_count_badge, bell,
    },
    defaults: {},
});

/// 打开终端时使用的 Shell 配置
#[derive(
    Clone,
    Debug,
    Default,
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
pub enum Shell {
    /// 使用 /etc/passwd 中的系统默认终端配置
    #[default]
    #[strum_discriminants(strum(serialize = "系统"))]
    System,
    /// 使用不带参数的特定程序
    #[strum_discriminants(strum(serialize = "程序"))]
    Program(String),
    /// 使用带参数的特定程序
    #[strum_discriminants(strum(serialize = "带参数"))]
    WithArguments {
        /// 要运行的程序
        program: String,
        /// 传递给程序的参数
        args: Vec<String>,
        /// 用于覆盖终端标签页标题的可选字符串
        title_override: Option<String>,
    },
}

#[derive(
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
pub enum WorkingDirectory {
    /// 使用当前文件所在目录，回退至项目目录，
    /// 然后回退到工作区中的第一个项目
    #[strum_discriminants(strum(serialize = "当前文件目录"))]
    CurrentFileDirectory,
    /// 使用当前文件所在的项目目录。若不成功，
    /// 则回退到首个项目目录策略。
    #[strum_discriminants(strum(serialize = "当前项目目录"))]
    CurrentProjectDirectory,
    /// 使用此工作区中第一个项目的目录，回退至
    /// 此平台的主目录。
    #[strum_discriminants(strum(serialize = "第一个项目目录"))]
    FirstProjectDirectory,
    /// 始终使用此平台的主目录（如果可以找到）。
    #[strum_discriminants(strum(serialize = "始终使用主目录"))]
    AlwaysHome,
    /// 始终使用指定目录，此值将由 shell 展开。
    /// 如果此路径不是有效目录，终端将默认使用
    /// 此平台的主目录（如果可以找到）
    #[strum_discriminants(strum(serialize = "始终"))]
    Always { directory: String },
}

#[with_fallible_options]
#[derive(
    Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq, Default,
)]
pub struct ScrollbarSettingsContent {
    /// 在终端中显示滚动条的时机
    ///
    /// 默认值：继承编辑器滚动条设置
    pub show: Option<ShowScrollbar>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom, Default)]
#[serde(rename_all = "snake_case")]
pub enum TerminalLineHeight {
    /// 使用舒适的阅读行高 1.618
    #[default]
    Comfortable,
    /// 使用标准行高 1.3，适用于 TUI，
    /// 尤其是使用框线字符的 TUI 应用
    Standard,
    /// 使用自定义行高。
    Custom(#[serde(serialize_with = "crate::serialize_f32_with_two_decimal_places")] f32),
}

impl TerminalLineHeight {
    pub fn value(&self) -> f32 {
        match self {
            TerminalLineHeight::Comfortable => 1.618,
            TerminalLineHeight::Standard => 1.3,
            TerminalLineHeight::Custom(line_height) => f32::max(*line_height, 1.),
        }
    }
}

/// 显示滚动条的时机
///
/// 默认值：auto
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
pub enum ShowScrollbar {
    /// 当存在重要信息时显示滚动条，或
    /// 跟随系统配置的行为。
    #[default]
    #[strum(serialize = "自动")]
    Auto,
    /// 匹配系统配置的行为
    #[strum(serialize = "系统")]
    System,
    /// 始终显示滚动条。
    #[strum(serialize = "始终")]
    Always,
    /// 从不显示滚动条
    #[strum(serialize = "从不")]
    Never,
}

#[derive(
    Clone,
    Copy,
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
// todo() -> combine with CursorShape
pub enum CursorShapeContent {
    /// 光标为方块形，如 `█`
    #[default]
    Block,
    /// 光标为下划线形，如 `_`
    Underline,
    /// 光标为竖线形，如 `⎸`
    Bar,
    /// 光标为空心方块形，如 `▯`
    Hollow,
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
pub enum TerminalBlink {
    /// 从不闪烁光标，忽略终端模式
    #[strum(serialize = "关闭")]
    Off,
    /// 默认关闭光标闪烁，但允许终端
    /// 设置光标闪烁。
    #[strum(serialize = "由终端控制")]
    TerminalControlled,
    /// 始终闪烁光标，忽略终端模式
    #[strum(serialize = "开启")]
    On,
}

#[derive(
    Clone,
    Copy,
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
pub enum AlternateScroll {
    #[strum(serialize = "开启")]
    On,
    #[strum(serialize = "关闭")]
    Off,
}

// Toolbar related settings
#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
pub struct TerminalToolbarContent {
    /// 是否在终端窗格内的面包屑中显示终端标题
    /// 仅当终端标题不为空时显示。
    ///
    /// 终端中运行的 Shell 需要配置为发出标题
    /// 示例：`echo -e "\e]2;New Title\007";`
    ///
    /// 默认值：true
    pub breadcrumbs: Option<bool>,
}

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
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum TerminalBell {
    /// 播放操作系统特定的提示音
    #[default]
    #[strum(serialize = "系统")]
    System,
    /// 不播放任何声音
    #[strum(serialize = "关闭")]
    Off,
}

#[derive(
    Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum CondaManager {
    /// 自动检测 conda 管理器
    #[default]
    Auto,
    /// 使用 conda
    Conda,
    /// 使用 mamba
    Mamba,
    /// 使用 micromamba
    Micromamba,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum VenvSettings {
    #[default]
    Off,
    On {
        /// 搜索虚拟环境的默认目录，
        /// 相对于当前工作目录。建议
        /// 在项目设置中进行配置，而非全局配置。
        activate_script: Option<ActivateScript>,
        venv_name: Option<String>,
        directories: Option<Vec<PathBuf>>,
        /// 激活 Conda 环境时使用的首选 Conda 管理器
        ///
        /// 默认值：auto
        conda_manager: Option<CondaManager>,
    },
}
#[with_fallible_options]
pub struct VenvSettingsContent<'a> {
    pub activate_script: ActivateScript,
    pub venv_name: &'a str,
    pub directories: &'a [PathBuf],
    pub conda_manager: CondaManager,
}

impl VenvSettings {
    pub fn as_option(&self) -> Option<VenvSettingsContent<'_>> {
        match self {
            VenvSettings::Off => None,
            VenvSettings::On {
                activate_script,
                venv_name,
                directories,
                conda_manager,
            } => Some(VenvSettingsContent {
                activate_script: activate_script.unwrap_or(ActivateScript::Default),
                venv_name: venv_name.as_deref().unwrap_or(""),
                directories: directories.as_deref().unwrap_or(&[]),
                conda_manager: conda_manager.unwrap_or(CondaManager::Auto),
            }),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(transparent)]
pub struct PathHyperlinkRegexes(pub Vec<PathHyperlinkRegex>);

impl crate::merge_from::MergeFrom for PathHyperlinkRegexes {
    fn merge_from(&mut self, other: &Self) {
        let inherited = std::mem::take(&mut self.0);
        let mut seen = HashSet::default();
        self.0 = other
            .0
            .iter()
            .flat_map(|entry| {
                if entry.is_rest() {
                    inherited.as_slice()
                } else {
                    std::slice::from_ref(entry)
                }
            })
            .filter(|entry| {
                let regex = match entry {
                    PathHyperlinkRegex::SingleLine(regex) => regex.clone(),
                    PathHyperlinkRegex::MultiLine(lines) => lines.join("\n"),
                };
                // Project layers can still contain an unresolved marker, which must
                // not collapse with the literal regex written as ["..."]
                seen.insert((entry.is_rest(), regex))
            })
            .cloned()
            .collect();
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
#[serde(untagged)]
pub enum PathHyperlinkRegex {
    SingleLine(String),
    MultiLine(Vec<String>),
}

impl PathHyperlinkRegex {
    fn is_rest(&self) -> bool {
        matches!(self, Self::SingleLine(regex) if regex == crate::SplicingVec::REST)
    }
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
pub enum TerminalDockPosition {
    #[strum(serialize = "左")]
    Left,
    #[strum(serialize = "底部")]
    Bottom,
    #[strum(serialize = "右")]
    Right,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum ActivateScript {
    #[default]
    Default,
    Csh,
    Fish,
    Nushell,
    PowerShell,
    Pyenv,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::{PathHyperlinkRegexes, ProjectSettingsContent, Shell, merge_from::MergeFrom};

    #[test]
    fn test_path_hyperlink_regexes_preserve_entry_forms() -> anyhow::Result<()> {
        let entries = json!([
            "...",
            ["..."],
            ["(?x)", "# comment", "...", "(?<path>file)"],
            "first\nsecond",
            ["first", "second"]
        ]);
        let regexes: PathHyperlinkRegexes = serde_json::from_value(entries.clone())?;
        assert_eq!(serde_json::to_value(regexes)?, entries);
        Ok(())
    }

    #[test]
    fn test_path_hyperlink_regexes_merge_preserves_first_representation() -> anyhow::Result<()> {
        let mut regexes: PathHyperlinkRegexes = serde_json::from_value(json!([
            "...",
            ["..."],
            "first\nsecond",
            ["(?x)", "# comment", "(?<path>file)"]
        ]))?;
        regexes.merge_from(&serde_json::from_value(json!([
            ["first", "second"],
            "...",
            "(?x)\n# comment\n(?<path>file)",
            "..."
        ]))?);
        assert_eq!(
            serde_json::to_value(regexes)?,
            json!([
                ["first", "second"],
                "...",
                ["..."],
                ["(?x)", "# comment", "(?<path>file)"]
            ])
        );
        Ok(())
    }

    #[test]
    #[ignore]
    fn test_project_settings() {
        let project_content =
            json!({"terminal": {"shell": {"program": "/bin/project"}}, "option_as_meta": true});

        let _user_content =
            json!({"terminal": {"shell": {"program": "/bin/user"}}, "option_as_meta": false});

        let project_settings =
            serde_json::from_value::<ProjectSettingsContent>(project_content).unwrap();

        assert_eq!(
            project_settings.terminal.unwrap().shell,
            Some(Shell::Program("/bin/project".to_owned()))
        );
    }
}
