use std::{borrow::Cow, num::NonZeroU32};

use collections::{HashMap, HashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};
use std::sync::Arc;

use crate::{
    DelayMs, DocumentFoldingRanges, DocumentSymbols, ExtendingSet, SemanticTokens, SplicingVec,
    merge_from,
};

/// 某一时刻修饰键的状态
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ModifiersContent {
    /// Control 键
    #[serde(default)]
    pub control: bool,
    /// Alt 键
    /// ，有时也称为 'meta' 键
    #[serde(default)]
    pub alt: bool,
    /// Shift 键
    #[serde(default)]
    pub shift: bool,
    /// macOS 上的 command 键
    /// Windows 上的 Windows 键
    /// Linux 上的 super 键
    #[serde(default)]
    pub platform: bool,
    /// Fn 键
    #[serde(default)]
    pub function: bool,
}

#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, JsonSchema)]
pub struct AllLanguageSettingsContent {
    /// 编辑预测设置
    pub edit_predictions: Option<EditPredictionSettingsContent>,
    /// 默认语言设置
    #[serde(flatten)]
    pub defaults: LanguageSettingsContent,
    /// 各语言的设置
    #[serde(default)]
    pub languages: LanguageToSettingsMap,
    /// 用于将文件扩展名和文件名
    /// 与语言关联的设置
    pub file_types: Option<FileTypeMap>,
}

crate::fallible_options::flattened_deserialize!(AllLanguageSettingsContent {
    sections: { defaults },
    options: { edit_predictions, file_types },
    defaults: { languages },
});

impl merge_from::MergeFrom for AllLanguageSettingsContent {
    fn merge_from(&mut self, other: &Self) {
        self.file_types.merge_from(&other.file_types);
        self.edit_predictions.merge_from(&other.edit_predictions);

        // A user's global settings override the default global settings and
        // all default language-specific settings.
        self.defaults.merge_from(&other.defaults);
        for language_settings in self.languages.0.values_mut() {
            let language_servers = language_settings.language_servers.take();
            language_settings.merge_from(&other.defaults);
            language_settings.language_servers = language_servers;
        }

        // A user's language-specific settings override default language-specific settings.
        for (language_name, user_language_settings) in &other.languages.0 {
            if let Some(existing) = self.languages.0.get_mut(language_name) {
                existing.merge_from(&user_language_settings);
            } else {
                let mut new_settings = self.defaults.clone();
                new_settings.language_servers = None;
                new_settings.merge_from(&user_language_settings);

                self.languages.0.insert(language_name.clone(), new_settings);
            }
        }
    }
}

/// 提供编辑预测的提供商
#[derive(
    Copy, Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom,
)]
#[serde(rename_all = "snake_case")]
pub enum EditPredictionProvider {
    None,
    #[default]
    Copilot,
    Zed,
    Codestral,
    Ollama,
    OpenAiCompatibleApi,
    Mercury,
}

impl EditPredictionProvider {
    pub fn is_zed(&self) -> bool {
        match self {
            EditPredictionProvider::Zed => true,
            EditPredictionProvider::None
            | EditPredictionProvider::Copilot
            | EditPredictionProvider::Codestral
            | EditPredictionProvider::Ollama
            | EditPredictionProvider::OpenAiCompatibleApi
            | EditPredictionProvider::Mercury => false,
        }
    }

    pub fn display_name(&self) -> Option<&'static str> {
        match self {
            EditPredictionProvider::Zed => Some("Zed AI"),
            EditPredictionProvider::Copilot => Some("GitHub Copilot"),
            EditPredictionProvider::Codestral => Some("Codestral"),
            EditPredictionProvider::Mercury => Some("Mercury"),
            EditPredictionProvider::None => None,
            EditPredictionProvider::Ollama => Some("Ollama"),
            EditPredictionProvider::OpenAiCompatibleApi => Some("OpenAI-Compatible API"),
        }
    }
}

/// 编辑预测设置的内容
#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct EditPredictionSettingsContent {
    /// 确定要使用的编辑预测提供商
    pub provider: Option<EditPredictionProvider>,
    /// 对匹配这些 glob 模式的文件禁用编辑预测
    ///
    /// 使用 `"..."` 可在不重复 Zed 默认值的情况下添加 glob 模式。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。省略
    /// `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "edit_predictions": {
    ///     "disabled_globs": ["**/build/**", "..."]
    ///   }
    /// }
    /// ```
    ///
    /// 继承的 glob 模式会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    ///
    /// 设为 `[]` 可清空继承的列表。省略此设置则会原样继承该列表。
    ///
    /// 相对路径模式会与相对于工作树根目录的路径进行匹配。
    /// 绝对路径模式会与绝对路径进行匹配。开头的 `~` 会
    /// 展开为你的主目录。
    pub disabled_globs: Option<SplicingVec>,
    /// 在缓冲区中显示编辑预测的模式
    /// 需要提供商支持
    pub mode: Option<EditPredictionsMode>,
    /// GitHub Copilot 的专项设置
    pub copilot: Option<CopilotEditPredictionSettingsContent>,
    /// Codestral 的专项设置
    pub codestral: Option<CodestralSettingsContent>,
    /// Ollama 的专项设置
    pub ollama: Option<OllamaEditPredictionSettingsContent>,
    /// 使用自定义 OpenAI 兼容服务器进行编辑预测的专项设置
    pub open_ai_compatible_api: Option<CustomEditPredictionProviderSettingsContent>,
    /// Zed 编辑预测提供商的专项设置
    pub zed: Option<ZedEditPredictionSettingsContent>,
    /// Mercury 编辑预测提供商的专项设置
    pub mercury: Option<MercuryEditPredictionSettingsContent>,
    /// 控制 Zed 是否可在使用编辑预测时收集训练数据
    /// 仅为检测为开源的项目中的文件收集数据
    ///
    /// - `"default"`：使用之前通过状态栏开关设置的偏好，
    ///   若未存储任何偏好，则为 false。
    /// - `"yes"`：允许对开源项目中的文件进行数据收集。
    /// - `"no"`：始终不允许数据收集。
    pub allow_data_collection: Option<EditPredictionDataCollectionChoice>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct CustomEditPredictionProviderSettingsContent {
    /// 用于补全的 API URL。
    ///
    /// 默认值：""
    pub api_url: Option<String>,
    /// 用于补全的提示词格式。设为 `""` 可根据模型名称推导格式。
    ///
    /// 默认值：""
    pub prompt_format: Option<EditPredictionPromptFormatContent>,
    /// 模型名称
    ///
    /// 默认值：""
    pub model: Option<String>,
    /// FIM 模型和自托管 Sweep 重写响应的最大生成 Token 数
    ///
    /// 默认值：256
    pub max_output_tokens: Option<u32>,
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// Default: 0
    pub prediction_debounce: Option<DelayMs>,
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
pub enum EditPredictionPromptFormatContent {
    #[default]
    #[strum(serialize = "推断")]
    Infer,
    #[strum(serialize = "Zeta")]
    Zeta,
    #[strum(serialize = "Zeta2")]
    Zeta2,
    #[strum(serialize = "Zeta2 1")]
    Zeta2_1,
    #[strum(serialize = "Code Llama")]
    CodeLlama,
    #[strum(serialize = "Star Coder")]
    StarCoder,
    #[strum(serialize = "Deepseek Coder")]
    DeepseekCoder,
    #[strum(serialize = "Qwen")]
    Qwen,
    #[strum(serialize = "Code Gemma")]
    CodeGemma,
    #[strum(serialize = "Codestral")]
    Codestral,
    #[strum(serialize = "Glm")]
    Glm,
    Sweep,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct CopilotSettingsContent {
    /// Copilot Chat 和编辑预测共用的企业 URI
    ///
    /// 默认值：none
    pub enterprise_uri: Option<String>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct CopilotEditPredictionSettingsContent {
    /// Copilot 使用的 HTTP/HTTPS 代理
    ///
    /// 默认值：none
    pub proxy: Option<String>,
    /// 禁用代理证书验证（不推荐）
    ///
    /// 默认值：false
    pub proxy_no_verify: Option<bool>,
    /// 是否启用 Copilot 下一处编辑建议功能
    ///
    /// 默认值：true
    pub enable_next_edit_suggestions: Option<bool>,
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// 默认值：75
    pub prediction_debounce: Option<DelayMs>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct CodestralSettingsContent {
    /// 用于补全的模型
    ///
    /// 默认值："codestral-latest"
    pub model: Option<String>,
    /// 最大生成 Token 数
    ///
    /// 默认值：150
    pub max_tokens: Option<u32>,
    /// 用于补全的 API URL。
    ///
    /// 默认值："https://codestral.mistral.ai"
    pub api_url: Option<String>,
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// 默认值：150
    pub prediction_debounce: Option<DelayMs>,
}

/// Zed 编辑预测提供商的专项设置
#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct ZedEditPredictionSettingsContent {
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// Default: 0
    pub prediction_debounce: Option<DelayMs>,
}

/// Mercury 编辑预测提供商的专项设置
#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct MercuryEditPredictionSettingsContent {
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// Default: 0
    pub prediction_debounce: Option<DelayMs>,
}

/// 用于编辑预测的 Ollama 模型名称。
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
#[serde(transparent)]
pub struct OllamaModelName(pub String);

impl AsRef<str> for OllamaModelName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<String> for OllamaModelName {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl From<OllamaModelName> for String {
    fn from(value: OllamaModelName) -> Self {
        value.0
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct OllamaEditPredictionSettingsContent {
    /// 用于补全的模型
    ///
    /// 默认值：none
    pub model: Option<OllamaModelName>,
    /// FIM 模型和自托管 Sweep 重写响应的最大生成 Token 数
    ///
    /// 默认值：256
    pub max_output_tokens: Option<u32>,
    /// 用于补全的 API URL。
    ///
    /// 默认值："http://localhost:11434"
    pub api_url: Option<String>,

    /// 用于补全的提示词格式。设为 `""` 可根据模型名称推导格式。
    ///
    /// 默认值：""
    pub prompt_format: Option<EditPredictionPromptFormatContent>,
    /// 停止输入后自动请求预测前的防抖延迟（毫秒）。
    /// 设置为 0 可立即请求预测。
    ///
    /// Default: 0
    pub prediction_debounce: Option<DelayMs>,
}

/// 控制使用 Zed 编辑预测时是否收集训练数据
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum EditPredictionDataCollectionChoice {
    /// 使用之前通过状态栏开关设置的偏好或 false，
    /// 未存储偏好时使用后者。
    #[default]
    #[strum(serialize = "默认")]
    Default,
    /// 允许 Zed 从开源项目收集训练数据
    #[strum(serialize = "是")]
    Yes,
    /// 从不允许收集训练数据
    #[strum(serialize = "否")]
    No,
}

/// 显示编辑预测的模式
#[derive(
    Copy,
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    JsonSchema,
    MergeFrom,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum EditPredictionsMode {
    /// 如果提供商支持，在按住修饰键（如 alt）时内联显示
    /// 否则，使用即时预览。
    #[serde(alias = "auto")]
    #[strum(serialize = "低调")]
    Subtle,
    /// 当没有可用的语言服务器补全时内联显示
    #[default]
    #[serde(alias = "eager_preview")]
    #[strum(serialize = "预测优先")]
    Eager,
}

/// 控制编辑器中的软换行行为
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
pub enum AutoIndentMode {
    /// 输入时根据语法上下文调整缩进
    /// 使用 tree-sitter 分析代码结构并相应地调整缩进
    #[strum(serialize = "语法感知")]
    SyntaxAware,
    /// 创建新行时保留当前行的缩进，
    /// 但不根据语法上下文进行调整。
    #[strum(serialize = "保留缩进")]
    PreserveIndent,
    /// 无自动缩进。新行从第 0 列开始。
    #[strum(serialize = "无")]
    None,
}

/// 控制编辑器中的软换行行为
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
pub enum SoftWrap {
    /// 通常保持单行，除非遇到过长的行
    #[strum(serialize = "无")]
    None,
    /// 已弃用：改用 None。保留此项是为了避免破坏现有用户的配置
    /// 通常保持单行，除非遇到过长的行
    #[strum(serialize = "优先单行")]
    PreferLine,
    /// 对超出编辑器宽度的行进行自动换行
    #[strum(serialize = "编辑器宽度")]
    EditorWidth,
    /// 在首选行长度或编辑器宽度（取较小者）处自动换行
    #[serde(alias = "preferred_line_length")]
    #[strum(serialize = "限定宽度")]
    Bounded,
}

pub const REST_OF_LANGUAGE_SERVERS: &str = "...";

#[derive(Clone, Debug, PartialEq, Eq, JsonSchema)]
#[schemars(with = "String")]
pub struct ConfiguredLanguageServer {
    pub name: Arc<str>,
    pub disabled: bool,
}

impl ConfiguredLanguageServer {
    const DISABLED_CHAR: char = '!';

    pub fn new(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            disabled: false,
        }
    }

    pub fn new_disabled(name: impl Into<Arc<str>>) -> Self {
        Self {
            name: name.into(),
            disabled: true,
        }
    }
}

impl From<&str> for ConfiguredLanguageServer {
    fn from(value: &str) -> Self {
        match value.strip_prefix(Self::DISABLED_CHAR) {
            Some(name) => Self::new_disabled(name),
            None => Self::new(value),
        }
    }
}

impl Serialize for ConfiguredLanguageServer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if self.disabled {
            serializer.collect_str(&format_args!("{}{}", Self::DISABLED_CHAR, self.name))
        } else {
            serializer.serialize_str(&self.name)
        }
    }
}

impl<'de> Deserialize<'de> for ConfiguredLanguageServer {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Cow::<'de, str>::deserialize(deserializer)?;
        Ok(Self::from(value.as_ref()))
    }
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
    strum::EnumString,
    strum::VariantArray,
    strum::VariantNames,
)]
#[serde(rename_all = "snake_case")]
pub enum SoftWrapIndent {
    /// 续行从第 0 列开始
    #[strum(serialize = "无")]
    None,
    /// 续行使用与原始行相同的缩进
    #[default]
    #[strum(serialize = "相同")]
    Same,
    /// 续行在原始行的基础上增加 1 级缩进
    #[strum(serialize = "增加一级")]
    ExtraOne,
    /// 续行在原始行的基础上增加 2 级缩进
    #[strum(serialize = "增加两级")]
    ExtraTwo,
}

/// 特定语言的设置
#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LanguageSettingsContent {
    /// 制表符应占用多少列
    ///
    /// 默认值：4
    #[schemars(range(min = 1, max = 128))]
    pub tab_size: Option<NonZeroU32>,
    /// 是否使用制表符而不是多个
    /// 空格来缩进行。
    ///
    /// 默认值：false
    pub hard_tabs: Option<bool>,
    /// 长文本行的软换行方式
    ///
    /// 默认值：none
    pub soft_wrap: Option<SoftWrap>,
    /// 软换行续行的缩进方式
    ///
    /// 默认值：same
    pub soft_wrap_indent: Option<SoftWrapIndent>,
    /// 对行进行软换行的列位置，适用于
    /// 已启用软换行的缓冲区。
    ///
    /// 默认值：80
    pub preferred_line_length: Option<u32>,
    /// 是否在编辑器中显示换行参考线。设置为 true 后，若
    /// softwrap 设为 'preferred_line_length'，会显示一条参考线；
    /// 该参考线位于 'preferred_line_length' 处，还会显示额外参考线。
    /// 这些参考线由 'wrap_guides' 设置指定。
    ///
    /// 默认值：true
    pub show_wrap_guides: Option<bool>,
    /// 在编辑器中显示换行参考线的字符数
    ///
    /// 默认值：[]
    pub wrap_guides: Option<Vec<usize>>,
    /// 缩进参考线相关设置。
    pub indent_guides: Option<IndentGuideSettingsContent>,
    /// 保存前是否对缓冲区执行格式化
    ///
    /// 默认值：off
    pub format_on_save: Option<FormatOnSave>,
    /// 是否移除缓冲区各行末尾的空白字符
    /// （保存之前）。
    ///
    /// 默认值：true
    pub remove_trailing_whitespace_on_save: Option<bool>,
    /// 是否确保缓冲区末尾有且仅有一个换行符
    /// （保存时）。
    ///
    /// 默认值：true
    pub ensure_final_newline_on_save: Option<bool>,
    /// 新文件以及在格式化和
    /// 保存操作时行尾的处理方式。
    ///
    /// - `detect`：检测现有行尾符，否则使用平台
    ///   默认值（Unix 上为 `lf`，Windows 上为 `crlf`）。
    /// - `prefer_lf`：对新文件和没有现有行尾符的文件优先使用 LF
    ///   行尾符。
    /// - `prefer_crlf`：对新文件和无现有行尾符的文件优先使用 CRLF
    ///   行尾符。
    /// - `enforce_lf`：在格式化和保存时强制使用 LF。
    /// - `enforce_crlf`：在格式化和保存时强制使用 CRLF。
    ///
    /// EditorConfig 的 `end_of_line` 属性会覆盖此设置，且
    /// 其行为类似于 `enforce_lf` 或 `enforce_crlf`。
    ///
    /// 默认值：detect
    pub line_ending: Option<LineEndingSetting>,
    /// 缓冲区格式化的执行方式
    ///
    /// 默认值：auto
    pub formatter: Option<FormatterList>,
    /// Zed 的 Prettier 集成设置。
    /// 允许启用/禁用 Prettier 格式化
    /// 并配置默认的 Prettier，当项目级 Prettier 未安装时使用
    ///
    /// 默认值：off
    pub prettier: Option<PrettierSettingsContent>,
    /// 是否自动关闭 JSX 标签
    pub jsx_tag_auto_close: Option<JsxTagAutoCloseSettingsContent>,
    /// 是否使用语言服务器提供代码智能功能
    ///
    /// 默认值：true
    pub enable_language_server: Option<bool>,
    /// 为此语言使用（或禁用）的语言服务器列表
    ///
    /// 此数组应包含语言服务器 ID 以及以下
    /// 特殊标记：
    /// - `"!<language_server_id>"` — 以 `!` 为前缀的语言服务器 ID 将被禁用。
    /// - `"..."` — 用于引用该语言**其余**已注册语言服务器的占位符。
    ///
    /// 默认值：["..."]
    pub language_servers: Option<Vec<ConfiguredLanguageServer>>,
    /// 控制语言服务器的语义标记在语法高亮中的使用方式
    ///
    /// 选项：
    /// - "off"：不向语言服务器请求语义标记。
    /// - "combined"：将 LSP 语义标记与 tree-sitter 高亮结合使用。
    /// - "full"：仅使用 LSP 语义标记，替换 tree-sitter 高亮。
    ///
    /// 默认值："off"
    pub semantic_tokens: Option<SemanticTokens>,
    /// 控制是否使用来自语言服务器的折叠范围来替代
    /// 基于 tree-sitter 和缩进的代码折叠
    ///
    /// 选项：
    /// - "off"：使用 tree-sitter 和基于缩进的折叠（默认）。
    /// - "on"：尽可能使用 LSP 折叠，当服务器未返回结果时回退到 tree-sitter 和基于缩进的折叠。
    ///
    /// 默认值："off"
    pub document_folding_ranges: Option<DocumentFoldingRanges>,
    /// 控制用于大纲和面包屑的文档符号来源
    ///
    /// 选项：
    /// - "off"：使用 tree-sitter 查询计算文档符号（默认）。
    /// - "on"：使用语言服务器的 `textDocument/documentSymbol` LSP 响应。启用后，不使用 tree-sitter 计算文档符号。
    ///
    /// 默认值："off"
    pub document_symbols: Option<DocumentSymbols>,
    /// 控制此语言中 `editor::Rewrap` 操作的允许范围
    ///
    /// 注意：此设置在 Vim 模式下无效，因为重排
    /// 已允许在所有位置使用。
    ///
    /// 默认值："in_comments"
    pub allow_rewrap: Option<RewrapBehavior>,
    /// 控制编辑预测是否立即显示（true）
    /// 或通过触发 `editor::ShowEditPrediction` 手动显示（false）。
    ///
    /// 默认值：true
    pub show_edit_predictions: Option<bool>,
    /// 在这些语言作用域中禁用编辑预测，例如 "comment" 和
    /// "string".
    ///
    /// 默认值：
    ///
    /// ```json
    /// {
    ///   "edit_predictions_disabled_in": []
    /// }
    /// ```
    ///
    /// 使用 `"..."` 可在不重复继承列表的情况下添加作用域。在项目
    /// 设置中，它会扩展用户配置或父级配置的值。在
    /// 特定语言的设置中，它会扩展该
    /// 语言所继承的作用域。省略 `"..."` 则会替换继承的列表。
    ///
    /// ```json
    /// {
    ///   "edit_predictions_disabled_in": ["comment"],
    ///   "languages": {
    ///     "Go": {
    ///       "edit_predictions_disabled_in": ["string", "..."]
    ///     }
    ///   }
    /// }
    /// ```
    ///
    /// 继承的作用域会插入到 `"..."` 所在位置，重复项保留
    /// 首次出现的条目。
    ///
    /// 设为 `[]` 可清空继承的列表。省略此设置则会原样继承该列表。
    pub edit_predictions_disabled_in: Option<SplicingVec>,
    /// 是否在编辑器中显示制表符和空格
    pub show_whitespaces: Option<ShowWhitespaceSetting>,
    /// 启用 show_whitespaces 时用于渲染空白字符的可见字符。
    ///
    /// 默认值：空格用 "•"，制表符用 "→"
    pub whitespace_map: Option<WhitespaceMapContent>,
    /// 当上一行也是注释时，是否在新行开头自动添加注释
    ///
    /// 默认值：true
    pub extend_comment_on_newline: Option<bool>,
    /// 是否在按下 Enter 键时继续 Markdown 列表
    ///
    /// 默认值：true
    pub extend_list_on_newline: Option<bool>,
    /// 是否在列表标记后按 Tab 键时缩进列表项
    ///
    /// 默认值：true
    pub indent_list_on_tab: Option<bool>,
    /// 内嵌提示相关设置。
    pub inlay_hints: Option<InlayHintSettingsContent>,
    /// 是否自动为你输入闭合字符。例如，
    /// 输入 '(' 时，Zed 会自动在正确位置添加右括号 ')'
    ///
    /// 默认值：true
    pub use_autoclose: Option<bool>,
    /// 是否自动为你用字符包围文本。例如，
    /// 选中文本并输入 '(' 时，Zed 会自动用 () 包围所选文本
    ///
    /// 默认值：true
    pub use_auto_surround: Option<bool>,
    /// 控制编辑器对自动闭合字符的处理方式
    /// 设为 `false`（默认）时，跳过并自动删除闭合字符
    /// 仅对自动插入的字符生效。
    /// 否则（当值为 `true` 时），闭合字符将始终被跳过并自动删除
    /// 无论它们是以何种方式插入的。
    ///
    /// 默认值：false
    pub always_treat_brackets_as_autoclosed: Option<bool>,
    /// 是否使用额外的 LSP 查询，以便在
    /// 每次输入由 LSP 服务器能力定义的“触发”符号后格式化（并修正）代码。
    ///
    /// 默认值：true
    pub use_on_type_format: Option<bool>,
    /// 保存时在格式化器之前运行哪些代码操作
    /// 若格式化已关闭，这些操作不会运行
    ///
    /// 默认值：{}（对于 Go 为 {"source.organizeImports": true}）
    pub code_actions_on_format: Option<HashMap<String, bool>>,
    /// 是否执行关联范围的联动编辑（若语言服务器支持）
    /// 例如，编辑 <html> 开始标签时，对应的 </html> 闭合标签内容也会同步编辑
    ///
    /// 默认值：true
    pub linked_edits: Option<bool>,
    /// 控制输入时的自动缩进行为
    ///
    /// - "syntax_aware"：根据语法上下文调整缩进（默认）
    /// - "preserve_indent"：在新行上保留当前行的缩进
    /// - "none"：不自动缩进
    ///
    /// 默认值：syntax_aware
    pub auto_indent: Option<AutoIndentMode>,
    /// 粘贴内容的缩进是否应根据上下文进行调整
    ///
    /// 默认值：true
    pub auto_indent_on_paste: Option<bool>,
    /// 此语言的任务配置
    ///
    /// 默认值：{}
    pub tasks: Option<LanguageTaskSettingsContent>,
    /// 是否在编辑器中输入时自动弹出补全菜单
    /// （无需显式触发）。
    ///
    /// 默认值：true
    pub show_completions_on_input: Option<bool>,
    /// 是否为补全菜单中的项显示内联及并排
    /// 文档。
    ///
    /// 默认值：true
    pub show_completion_documentation: Option<bool>,
    /// 控制此语言的补全处理方式
    pub completions: Option<CompletionSettingsContent>,
    /// 该语言的首选调试器
    ///
    /// 默认值：[]
    pub debuggers: Option<Vec<String>>,
    /// 是否在编辑器中启用单词级差异高亮
    ///
    /// 启用后，已修改行中发生变化的单词将被高亮显示
    /// 精确显示变更内容
    ///
    /// 默认值：true
    pub word_diff_enabled: Option<bool>,
    /// 是否使用 tree-sitter 括号查询来检测并为编辑器中的括号着色
    ///
    /// 默认值：false
    pub colorize_brackets: Option<bool>,
}

/// 控制编辑器中空白字符的显示方式
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
pub enum ShowWhitespaceSetting {
    /// 仅为所选文本绘制空白符
    #[strum(serialize = "选择")]
    Selection,
    /// 不绘制任何制表符或空格
    #[strum(serialize = "无")]
    None,
    /// 绘制所有不可见符号
    #[strum(serialize = "全部")]
    All,
    /// 仅在边界处绘制空白符
    ///
    /// 空白字符处于边界位置需满足以下任一条件：
    /// - 是制表符
    /// - 紧邻边界（开头或结尾）
    /// - 紧邻空白字符（左侧或右侧）
    #[strum(serialize = "边界")]
    Boundary,
    /// 仅在非空白字符后绘制空白符
    #[strum(serialize = "尾随")]
    Trailing,
}

#[with_fallible_options]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq)]
pub struct WhitespaceMapContent {
    pub space: Option<char>,
    pub tab: Option<char>,
}

/// `editor::Rewrap` 的行为
#[derive(
    Debug,
    PartialEq,
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
pub enum RewrapBehavior {
    /// 仅在注释内重新换行
    #[default]
    #[strum(serialize = "在注释中")]
    InComments,
    /// 仅在当前选区内重新换行
    #[strum(serialize = "在选区中")]
    InSelections,
    /// 允许在任意位置重新换行
    #[strum(serialize = "任意位置")]
    Anywhere,
}

#[with_fallible_options]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct JsxTagAutoCloseSettingsContent {
    /// 启用或禁用 JSX 标签的自动关闭
    pub enabled: Option<bool>,
}

/// 内嵌提示的设置
#[with_fallible_options]
#[derive(Clone, Default, Debug, Serialize, Deserialize, JsonSchema, MergeFrom, PartialEq, Eq)]
pub struct InlayHintSettingsContent {
    /// 用于开启或关闭提示的全局开关
    ///
    /// 默认值：false
    pub enabled: Option<bool>,
    /// 调试时用于开关内联值显示的全局开关
    ///
    /// 默认值：true
    pub show_value_hints: Option<bool>,
    /// 是否显示类型提示
    ///
    /// 默认值：true
    pub show_type_hints: Option<bool>,
    /// 是否显示参数提示
    ///
    /// 默认值：true
    pub show_parameter_hints: Option<bool>,
    /// 是否显示其他提示
    ///
    /// 默认值：true
    pub show_other_hints: Option<bool>,
    /// 是否为内嵌提示显示背景
    ///
    /// 如果设置为 `true`，背景将使用 `hint.background` 颜色
    /// （来自当前主题）。
    ///
    /// 默认值：false
    pub show_background: Option<bool>,
    /// 是否对缓冲区编辑后的内嵌提示更新进行防抖
    ///
    /// 设为 0 可禁用防抖
    ///
    /// 默认值：700
    pub edit_debounce_ms: Option<u64>,
    /// 是否对缓冲区滚动后的内嵌提示更新进行防抖
    ///
    /// 设为 0 可禁用防抖
    ///
    /// 默认值：50
    pub scroll_debounce_ms: Option<u64>,
    /// 当用户按下指定修饰键时切换内嵌提示（隐藏或显示）
    /// 如果仅按下指定修饰键的子集，则不会切换提示
    /// 如果未指定修饰键，则等同于 `null`
    ///
    /// 默认值：null
    pub toggle_on_modifiers_press: Option<ModifiersContent>,
}

/// 内嵌提示的种类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlayHintKind {
    /// 类型的内嵌提示。
    Type,
    /// 参数的内嵌提示。
    Parameter,
}

impl InlayHintKind {
    /// 根据给定名称返回 [`InlayHintKind`]
    ///
    /// 若 `name` 与任何预期值均不匹配，则返回 `None`；预期值均为
    /// 字符串表示形式。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "type" => Some(InlayHintKind::Type),
            "parameter" => Some(InlayHintKind::Parameter),
            _ => None,
        }
    }

    /// 返回此 [`InlayHintKind`] 的名称
    pub fn name(&self) -> &'static str {
        match self {
            InlayHintKind::Type => "type",
            InlayHintKind::Parameter => "parameter",
        }
    }
}

/// 控制此语言的补全处理方式
#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom, Default)]
#[serde(rename_all = "snake_case")]
pub struct CompletionSettingsContent {
    /// 控制单词的补全方式
    /// 对于大型文档，可能无法获取所有词语用于补全
    ///
    /// Default: `fallback`
    pub words: Option<WordsCompletionMode>,
    /// 补全查询中需要多少个字符才能自动显示基于单词的补全
    /// 在此值以下，仍可通过相应的编辑器命令手动触发基于单词的补全。
    ///
    /// 默认值：3
    pub words_min_length: Option<u32>,
    /// 是否获取 LSP 补全
    ///
    /// 默认值：true
    pub lsp: Option<bool>,
    /// 获取 LSP 补全时，确定等待特定服务器响应的时长
    /// 设为 0 时，无限等待
    ///
    /// Default: 0
    pub lsp_fetch_timeout_ms: Option<u64>,
    /// 控制 LSP 补全的插入方式
    ///
    /// Default: "replace_suffix"
    pub lsp_insert_mode: Option<LspInsertMode>,
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
pub enum LspInsertMode {
    /// 使用 LSP 规范中的 `insert` 范围替换光标前的文本
    #[strum(serialize = "插入")]
    Insert,
    /// 使用 LSP 规范中的 `replace` 范围替换光标前后的文本
    #[strum(serialize = "替换")]
    Replace,
    /// 若将被替换的文本是补全内容的子序列，则行为类似 `"replace"`，
    /// 否则类似于 `"insert"`。
    #[strum(serialize = "替换子序列")]
    ReplaceSubsequence,
    /// 若光标后的文本是补全的后缀，则行为类似 `"replace"`，否则类似
    /// `"insert"`。
    #[strum(serialize = "替换后缀")]
    ReplaceSuffix,
}

/// 控制文档中单词的补全方式
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
pub enum WordsCompletionMode {
    /// 始终获取文档中的单词以与 LSP 补全一同提供补全。
    #[strum(serialize = "已启用")]
    Enabled,
    /// 仅当 LSP 响应出错或超时时，
    /// 使用文档中的词汇提供补全
    #[strum(serialize = "回退")]
    Fallback,
    /// 从不获取或补全文档中的词语
    /// （基于单词的补全仍可通过独立操作查询）
    #[strum(serialize = "已禁用")]
    Disabled,
}

/// 允许启用/禁用 Prettier 格式化
/// 并配置默认的 Prettier，当项目级 Prettier 未安装时使用
/// Prettier 格式化默认禁用
#[with_fallible_options]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct PrettierSettingsContent {
    /// 为指定语言启用或禁用 Prettier 格式化
    pub allowed: Option<bool>,

    /// 强制 Prettier 集成在格式化该语言文件时使用指定的解析器名称
    pub parser: Option<String>,

    /// 强制 Prettier 集成在格式化该语言文件时使用指定插件
    /// 默认 Prettier 将与这些插件一起安装
    pub plugins: Option<HashSet<String>>,

    /// 默认 Prettier 选项，格式与 package.json 中 Prettier 部分相同
    /// 如果项目通过 package.json 安装了 Prettier，这些选项将被忽略
    #[serde(flatten)]
    pub options: Option<HashMap<String, serde_json::Value>>,
}

/// TODO: this should just be a bool
/// 控制保存文件时的格式化行为
#[derive(
    Debug,
    Clone,
    Copy,
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
pub enum FormatOnSave {
    /// 文件应在保存时格式化
    #[strum(serialize = "开启")]
    On,
    /// 文件不应在保存时格式化
    #[strum(serialize = "关闭")]
    Off,
    /// 保存时仅格式化包含未暂存更改的行。
    /// 需要版本控制和 LSP 范围格式化支持。
    /// 如果没有可用的 Git 差异，或 LSP 不支持
    /// 范围格式化，则跳过格式化。
    #[strum(serialize = "修改内容")]
    Modifications,
    /// 保存时仅格式化包含未暂存更改的行。
    /// 如果没有可用的 Git 差异（例如版本控制
    /// 不可用），或 LSP 不支持范围格式化，
    /// 则回退到格式化整个文件。
    #[strum(serialize = "修改内容（若可用）")]
    ModificationsIfAvailable,
}

/// 控制保存缓冲区时行尾的规范化方式
#[derive(
    Debug,
    Clone,
    Copy,
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
pub enum LineEndingSetting {
    /// 保留文件现有的行尾符。新文件使用
    /// 平台默认行尾符。
    #[strum(serialize = "自动检测")]
    Detect,
    /// 对新文件及没有既定行尾约定的文件使用 LF
    /// 规范，同时保留现有的 LF 或 CRLF 文件。
    #[strum(serialize = "首选 LF")]
    PreferLf,
    /// 对新文件及没有既定行尾约定的文件使用 CRLF
    /// 规范，同时保留现有的 LF 或 CRLF 文件。
    #[strum(serialize = "首选 CRLF")]
    PreferCrlf,
    /// 在格式化和保存时将行尾规范化为 LF（`\n`）。
    #[strum(serialize = "强制 LF")]
    EnforceLf,
    /// 在格式化和保存时将行尾规范化为 CRLF（`\r\n`）。
    #[strum(serialize = "强制 CRLF")]
    EnforceCrlf,
}

/// 控制格式化代码时使用的格式化工具
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(untagged)]
pub enum FormatterList {
    Single(Formatter),
    Vec(Vec<Formatter>),
}

impl Default for FormatterList {
    fn default() -> Self {
        Self::Single(Formatter::default())
    }
}

impl AsRef<[Formatter]> for FormatterList {
    fn as_ref(&self) -> &[Formatter] {
        match &self {
            Self::Single(single) => std::slice::from_ref(single),
            Self::Vec(v) => v,
        }
    }
}

/// 控制格式化代码时使用的格式化工具；若有多个格式化工具，则按声明顺序依次执行
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum Formatter {
    /// 使用 Zed 的 Prettier 集成格式化文件（如适用），
    /// 或回退为通过语言服务器进行格式化。
    #[default]
    Auto,
    /// 不格式化代码
    None,
    /// 使用 Zed 的 Prettier 集成来格式化代码
    Prettier,
    /// 使用外部命令格式化代码
    External {
        /// 要运行的外部程序
        command: String,
        /// 传递给程序的参数
        arguments: Option<Vec<String>>,
    },
    /// 文件应使用语言服务器执行的代码操作进行格式化
    CodeAction(String),
    /// 使用语言服务器格式化代码
    #[serde(untagged)]
    LanguageServer(LanguageServerFormatterSpecifier),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(
    rename_all = "snake_case",
    // allow specifying language servers as "language_server" or {"language_server": {"name": ...}}
    from = "LanguageServerVariantContent",
    into = "LanguageServerVariantContent"
)]
pub enum LanguageServerFormatterSpecifier {
    Specific {
        name: String,
    },
    #[default]
    Current,
}

impl From<LanguageServerVariantContent> for LanguageServerFormatterSpecifier {
    fn from(value: LanguageServerVariantContent) -> Self {
        match value {
            LanguageServerVariantContent::Specific {
                language_server: LanguageServerSpecifierContent { name: Some(name) },
            } => Self::Specific { name },
            _ => Self::Current,
        }
    }
}

impl From<LanguageServerFormatterSpecifier> for LanguageServerVariantContent {
    fn from(value: LanguageServerFormatterSpecifier) -> Self {
        match value {
            LanguageServerFormatterSpecifier::Specific { name } => Self::Specific {
                language_server: LanguageServerSpecifierContent { name: Some(name) },
            },
            LanguageServerFormatterSpecifier::Current => {
                Self::Current(CurrentLanguageServerContent::LanguageServer)
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case", untagged)]
enum LanguageServerVariantContent {
    /// 使用指定的语言服务器格式化代码
    Specific {
        language_server: LanguageServerSpecifierContent,
    },
    /// 使用当前语言服务器格式化代码
    Current(CurrentLanguageServerContent),
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
enum CurrentLanguageServerContent {
    #[default]
    LanguageServer,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq, Eq, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
struct LanguageServerSpecifierContent {
    /// 用于格式化的语言服务器名称
    name: Option<String>,
}

/// 缩进参考线的设置
#[with_fallible_options]
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct IndentGuideSettingsContent {
    /// 是否在编辑器中显示缩进参考线
    ///
    /// 默认值：true
    pub enabled: Option<bool>,
    /// 缩进参考线的宽度（像素），范围为 1 到 10
    ///
    /// Default: 1
    pub line_width: Option<u32>,
    /// 活动缩进参考线的宽度（像素），范围为 1 到 10
    ///
    /// Default: 1
    pub active_line_width: Option<u32>,
    /// 确定缩进参考线的着色方式
    ///
    /// Default: Fixed
    pub coloring: Option<IndentGuideColoring>,
    /// 确定缩进参考线背景的着色方式
    ///
    /// Default: Disabled
    pub background_coloring: Option<IndentGuideBackgroundColoring>,
}

/// 特定语言的任务设置
#[with_fallible_options]
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Serialize, JsonSchema, MergeFrom)]
pub struct LanguageTaskSettingsContent {
    /// 为特定语言设置额外的任务变量
    pub variables: Option<HashMap<String, String>>,
    pub enabled: Option<bool>,
    /// 优先使用 LSP 任务而非 Zed 语言扩展任务
    /// 如果由于错误/超时或正常执行未返回 LSP 任务，
    /// 将改用 Zed 语言扩展任务。
    ///
    /// 其他 Zed 任务仍会显示：
    /// * 来自任意任务配置文件的 Zed 任务
    /// * 来自历史记录的 Zed 任务（如之前曾运行过的一次性任务）
    pub prefer_lsp: Option<bool>,
}

/// 从语言名称到设置的映射
#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LanguageToSettingsMap(pub HashMap<String, LanguageSettingsContent>);

/// 从语言名称到文件模式的映射
#[with_fallible_options]
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct FileTypeMap(pub HashMap<Arc<str>, ExtendingSet<String>>);

impl<'a> IntoIterator for &'a FileTypeMap {
    type Item = (&'a Arc<str>, &'a ExtendingSet<String>);
    type IntoIter = std::collections::hash_map::Iter<'a, Arc<str>, ExtendingSet<String>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// 确定缩进参考线的着色方式
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
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
pub enum IndentGuideColoring {
    /// 不为缩进参考线渲染任何线条
    #[strum(serialize = "已禁用")]
    Disabled,
    /// 对所有缩进级别使用相同颜色。
    #[default]
    #[strum(serialize = "固定颜色")]
    Fixed,
    /// 为每个缩进级别使用不同的颜色
    #[strum(serialize = "缩进感知")]
    IndentAware,
}

/// 确定缩进参考线背景的着色方式
#[derive(
    Default,
    Debug,
    Copy,
    Clone,
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
pub enum IndentGuideBackgroundColoring {
    /// 不为缩进参考线渲染任何背景
    #[default]
    #[strum(serialize = "已禁用")]
    Disabled,
    /// 为每个缩进级别使用不同的颜色
    #[strum(serialize = "缩进感知")]
    IndentAware,
}

#[cfg(test)]
mod test {

    use crate::{ParseStatus, fallible_options, merge_from::MergeFrom};

    use super::*;

    #[test]
    fn test_formatter_deserialization() {
        let raw_auto = "{\"formatter\": \"auto\"}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw_auto).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Single(Formatter::Auto))
        );
        let raw_none = "{\"formatter\": \"none\"}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw_none).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Single(Formatter::None))
        );
        let raw = "{\"formatter\": \"language_server\"}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Single(Formatter::LanguageServer(
                LanguageServerFormatterSpecifier::Current
            )))
        );

        let raw = "{\"formatter\": [{\"language_server\": {\"name\": null}}]}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Vec(vec![Formatter::LanguageServer(
                LanguageServerFormatterSpecifier::Current
            )]))
        );
        let raw = "{\"formatter\": [{\"language_server\": {\"name\": null}}, \"language_server\", \"prettier\"]}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Vec(vec![
                Formatter::LanguageServer(LanguageServerFormatterSpecifier::Current),
                Formatter::LanguageServer(LanguageServerFormatterSpecifier::Current),
                Formatter::Prettier
            ]))
        );

        let raw = "{\"formatter\": [{\"language_server\": {\"name\": \"ruff\"}}, \"prettier\"]}";
        let settings: LanguageSettingsContent = serde_json::from_str(raw).unwrap();
        assert_eq!(
            settings.formatter,
            Some(FormatterList::Vec(vec![
                Formatter::LanguageServer(LanguageServerFormatterSpecifier::Specific {
                    name: "ruff".to_string()
                }),
                Formatter::Prettier
            ]))
        );

        assert_eq!(
            serde_json::to_string(&LanguageServerFormatterSpecifier::Current).unwrap(),
            "\"language_server\"",
        );
    }

    #[test]
    fn test_formatter_deserialization_invalid() {
        let raw_auto = "{\"formatter\": {}}";
        let (_, result) = fallible_options::parse_json::<LanguageSettingsContent>(raw_auto);
        assert!(matches!(result, ParseStatus::Failed { .. }));
    }

    #[test]
    fn test_configured_language_server_serialization() {
        let enabled = ConfiguredLanguageServer::new("rust-analyzer");
        let disabled = ConfiguredLanguageServer::new_disabled("rust-analyzer");

        assert_eq!(
            serde_json::to_string(&enabled).expect("enabled server should serialize"),
            "\"rust-analyzer\""
        );
        assert_eq!(
            serde_json::to_string(&disabled).expect("disabled server should serialize"),
            "\"!rust-analyzer\""
        );
        assert_eq!(
            serde_json::from_str::<ConfiguredLanguageServer>("\"rust-analyzer\"")
                .expect("enabled server should deserialize"),
            enabled
        );
        assert_eq!(
            serde_json::from_str::<ConfiguredLanguageServer>("\"!rust-analyzer\"")
                .expect("disabled server should deserialize"),
            disabled
        );
    }

    #[test]
    fn test_language_servers_merge_keeps_per_language_lists_pure() {
        let default_typescript_servers = vec![
            ConfiguredLanguageServer::new_disabled("typescript-language-server"),
            ConfiguredLanguageServer::new("vtsls"),
            ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
        ];
        let mut base = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec![REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(default_typescript_servers.clone()),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        let user = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec!["!vtsls".into(), REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            ..AllLanguageSettingsContent::default()
        };
        base.merge_from(&user);
        assert_eq!(
            base.languages.0["TypeScript"].language_servers.as_ref(),
            Some(&default_typescript_servers),
            "a global list must not rewrite per-language lists"
        );
        assert_eq!(
            base.defaults.language_servers.as_ref(),
            Some(&vec![
                ConfiguredLanguageServer::new_disabled("vtsls"),
                ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
            ])
        );

        let project = AllLanguageSettingsContent {
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(vec![
                            "vtsls".into(),
                            REST_OF_LANGUAGE_SERVERS.into(),
                        ]),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };
        base.merge_from(&project);
        assert_eq!(
            base.languages.0["TypeScript"].language_servers.as_ref(),
            Some(&vec![
                ConfiguredLanguageServer::new("vtsls"),
                ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
            ]),
            "a per-language list must replace the older one wholesale"
        );
    }

    #[test]
    fn test_language_servers_merge_no_per_language_config_stays_unset() {
        let mut base = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec![REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            languages: LanguageToSettingsMap(
                [(
                    "Rust".into(),
                    LanguageSettingsContent {
                        tab_size: Some(std::num::NonZeroU32::new(4).unwrap()),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        let user = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec!["!eslint".into(), REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            ..AllLanguageSettingsContent::default()
        };

        base.merge_from(&user);

        assert_eq!(
            base.languages.0["Rust"].language_servers, None,
            "languages without their own list must not get a stale copy of the global one"
        );
        assert_eq!(
            base.defaults.language_servers.as_ref(),
            Some(&vec![
                ConfiguredLanguageServer::new_disabled("eslint"),
                ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
            ])
        );
    }

    #[test]
    fn test_language_servers_merge_user_per_language_overrides() {
        let mut base = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec![REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(vec![
                            "!typescript-language-server".into(),
                            "vtsls".into(),
                            REST_OF_LANGUAGE_SERVERS.into(),
                        ]),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        let user = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec!["!eslint".into(), REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(vec![
                            "deno".into(),
                            "!vtsls".into(),
                            REST_OF_LANGUAGE_SERVERS.into(),
                        ]),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        base.merge_from(&user);

        let ts_servers = base.languages.0["TypeScript"]
            .language_servers
            .as_ref()
            .unwrap();
        assert_eq!(
            ts_servers,
            &vec![
                ConfiguredLanguageServer::new("deno"),
                ConfiguredLanguageServer::new_disabled("vtsls"),
                ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
            ]
        );
    }

    #[test]
    fn test_user_per_language_config_overrides_default_disables() {
        let mut base = AllLanguageSettingsContent {
            defaults: LanguageSettingsContent {
                language_servers: Some(vec![REST_OF_LANGUAGE_SERVERS.into()]),
                ..LanguageSettingsContent::default()
            },
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(vec![
                            "!typescript-language-server".into(),
                            "vtsls".into(),
                            REST_OF_LANGUAGE_SERVERS.into(),
                        ]),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        let user = AllLanguageSettingsContent {
            languages: LanguageToSettingsMap(
                [(
                    "TypeScript".into(),
                    LanguageSettingsContent {
                        language_servers: Some(vec![
                            "typescript-language-server".into(),
                            REST_OF_LANGUAGE_SERVERS.into(),
                        ]),
                        ..LanguageSettingsContent::default()
                    },
                )]
                .into_iter()
                .collect(),
            ),
            ..AllLanguageSettingsContent::default()
        };

        base.merge_from(&user);

        let ts_servers = base.languages.0["TypeScript"]
            .language_servers
            .as_ref()
            .unwrap();
        assert_eq!(
            ts_servers,
            &vec![
                ConfiguredLanguageServer::new("typescript-language-server"),
                ConfiguredLanguageServer::new(REST_OF_LANGUAGE_SERVERS),
            ]
        );
    }

    #[test]
    fn test_prettier_options() {
        let raw_prettier = r#"{"allowed": false, "tabWidth": 4, "semi": false}"#;
        let result = serde_json::from_str::<PrettierSettingsContent>(raw_prettier)
            .expect("Failed to parse prettier options");
        assert!(
            result
                .options
                .as_ref()
                .expect("options were flattened")
                .contains_key("semi")
        );
        assert!(
            result
                .options
                .as_ref()
                .expect("options were flattened")
                .contains_key("tabWidth")
        );
    }
}
