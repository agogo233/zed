use crate::merge_from::MergeFrom;
use collections::HashMap;
use language_model_core::ReasoningEffort;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};

use std::sync::Arc;

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct AllLanguageModelSettingsContent {
    pub anthropic: Option<AnthropicSettingsContent>,
    pub anthropic_compatible: Option<HashMap<Arc<str>, AnthropicCompatibleSettingsContent>>,
    pub bedrock: Option<AmazonBedrockSettingsContent>,
    pub deepseek: Option<DeepseekSettingsContent>,
    pub google: Option<GoogleSettingsContent>,
    #[serde(rename = "llama.cpp")]
    pub llama_cpp: Option<LlamaCppSettingsContent>,
    pub lmstudio: Option<LmStudioSettingsContent>,
    pub mistral: Option<MistralSettingsContent>,
    pub ollama: Option<OllamaSettingsContent>,
    pub opencode: Option<OpenCodeSettingsContent>,
    pub open_router: Option<OpenRouterSettingsContent>,
    pub openai: Option<OpenAiSettingsContent>,
    pub openai_compatible: Option<HashMap<Arc<str>, OpenAiCompatibleSettingsContent>>,
    pub vercel_ai_gateway: Option<VercelAiGatewaySettingsContent>,
    pub x_ai: Option<XAiSettingsContent>,
    #[serde(rename = "zed.dev")]
    pub zed_dot_dev: Option<ZedDotDevSettingsContent>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct AnthropicSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<AnthropicAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct AnthropicCompatibleSettingsContent {
    pub api_url: String,
    pub available_models: Vec<AnthropicCompatibleAvailableModel>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct AnthropicCompatibleAvailableModel {
    /// 模型在提供商 API 中的名称。例如 claude-3-5-sonnet-latest
    pub name: String,
    /// 模型在 Zed UI 中的名称，例如助手面板的模型选择器下拉菜单中显示的名称。
    pub display_name: Option<String>,
    /// 模型的上下文窗口大小
    pub max_tokens: u64,
    /// 调用工具时用于替代的模型 `name`，适用于主模型不支持工具调用的情况
    pub tool_override: Option<String>,
    pub max_output_tokens: Option<u64>,
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_temperature: Option<f32>,
    #[serde(default)]
    pub extra_beta_headers: Vec<String>,
    /// 模型的模式（例如思考模式）
    pub mode: Option<ModelMode>,
    #[serde(default)]
    pub capabilities: AnthropicCompatibleModelCapabilities,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct AnthropicCompatibleModelCapabilities {
    pub tools: bool,
    pub images: bool,
    /// 是否为提示词缓存发送显式的 `cache_control` 断点。
    /// 如果提供商拒绝包含这些断点的请求，则保持禁用。
    pub prompt_caching: bool,
}

impl Default for AnthropicCompatibleModelCapabilities {
    fn default() -> Self {
        Self {
            tools: true,
            images: false,
            prompt_caching: false,
        }
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct AnthropicAvailableModel {
    /// 模型在 Anthropic API 中的名称，例如 claude-3-5-sonnet-latest、claude-3-opus-20240229 等
    pub name: String,
    /// 模型在 Zed 界面中显示的名称，例如智能体面板中的模型选择下拉菜单
    pub display_name: Option<String>,
    /// 模型的上下文窗口大小
    pub max_tokens: u64,
    /// 调用工具时用于替代的模型 `name`，适用于主模型不支持工具调用的情况
    pub tool_override: Option<String>,
    /// Anthropic 缓存 API 的配置
    pub cache_configuration: Option<LanguageModelCacheConfiguration>,
    pub max_output_tokens: Option<u64>,
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_temperature: Option<f32>,
    #[serde(default)]
    pub extra_beta_headers: Vec<String>,
    /// 此模型是否可使用 Anthropic 的快速模式
    pub supports_fast_mode: Option<bool>,
    /// 模型的模式（例如思考模式）
    pub mode: Option<ModelMode>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct AmazonBedrockSettingsContent {
    pub available_models: Option<Vec<BedrockAvailableModel>>,
    /// 通过 `bedrock-mantle` 端点的
    /// OpenAI 兼容 API 提供的自定义模型，以及内置的 Mantle 模型
    /// （GPT-5.6 Sol、GPT-5.6 Terra、GPT-5.6 Luna、GPT-5.5、GPT-5.4、Grok 4.3）。
    pub mantle_available_models: Option<Vec<BedrockMantleAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
    pub endpoint_url: Option<String>,
    pub region: Option<String>,
    pub profile: Option<String>,
    pub authentication_method: Option<BedrockAuthMethodContent>,
    pub allow_global: Option<bool>,
    /// 应用于 Bedrock API 请求的护栏标识符（ARN 或 ID）
    pub guardrail_identifier: Option<String>,
    /// 要使用的护栏版本。未指定时默认为 "DRAFT"
    pub guardrail_version: Option<String>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct BedrockAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub cache_configuration: Option<LanguageModelCacheConfiguration>,
    pub max_output_tokens: Option<u64>,
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_temperature: Option<f32>,
    pub supports_tools: Option<bool>,
    pub supports_images: Option<bool>,
    pub thinking: Option<BedrockThinkingSettings>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(untagged)]
pub enum BedrockThinkingSettings {
    Enabled(bool),
    Config(BedrockThinkingConfigSettings),
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct BedrockThinkingConfigSettings {
    pub adaptive: Option<bool>,
    pub has_xhigh: Option<bool>,
    pub budget_tokens: Option<u64>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct BedrockMantleAvailableModel {
    /// Bedrock Mantle 请求体中预期的模型 ID，例如 `openai.gpt-5.5`
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    /// 调用此模型时必须使用的 OpenAI 兼容 API
    pub protocol: BedrockMantleProtocolContent,
    pub supports_tools: Option<bool>,
    pub supports_images: Option<bool>,
    /// 此自定义 Mantle 模型是否支持 OpenAI 推理力度参数
    pub supports_thinking: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub enum BedrockMantleProtocolContent {
    /// OpenAI Chat Completions API（`/chat/completions`）
    #[serde(rename = "chat_completions")]
    ChatCompletions,
    /// OpenAI Responses API（`/responses`）
    #[serde(rename = "responses")]
    Responses,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub enum BedrockAuthMethodContent {
    #[serde(rename = "named_profile")]
    NamedProfile,
    #[serde(rename = "sso")]
    SingleSignOn,
    #[serde(rename = "api_key")]
    ApiKey,
    /// IMDSv2, PodIdentity, env vars, etc.
    #[serde(rename = "default")]
    Automatic,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct OllamaSettingsContent {
    pub api_url: Option<String>,
    pub auto_discover: Option<bool>,
    pub available_models: Option<Vec<OllamaAvailableModel>>,
    pub context_window: Option<u64>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OllamaAvailableModel {
    /// Ollama API 中的模型名称（例如 "llama3.2:latest"）
    pub name: String,
    /// 模型在 Zed 界面中显示的名称，例如智能体面板中的模型选择下拉菜单
    pub display_name: Option<String>,
    /// 模型的上下文长度参数（即 num_ctx 或 n_ctx）
    pub max_tokens: u64,
    /// 最后一次请求后保持连接的秒数
    pub keep_alive: Option<KeepAlive>,
    /// 模型是否支持工具
    pub supports_tools: Option<bool>,
    /// 模型是否支持视觉
    pub supports_images: Option<bool>,
    /// 是否启用思考模式
    pub supports_thinking: Option<bool>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq, JsonSchema, MergeFrom)]
#[serde(untagged)]
pub enum KeepAlive {
    /// 保持模型存活 N 秒
    Seconds(isize),
    /// 保持模型存活固定时长。接受如 "5m"、"10m"、"1h"、"1d" 等格式的时长。
    Duration(String),
}

impl KeepAlive {
    /// 保持模型存活，直到加载新模型或 Ollama 关闭
    pub fn indefinite() -> Self {
        Self::Seconds(-1)
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self::indefinite()
    }
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct OpenCodeSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<OpenCodeAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
    /// 是否显示 OpenCode Zen 模型，默认为 true
    pub show_zen_models: Option<bool>,
    /// 是否显示 OpenCode Go 模型，默认为 true
    pub show_go_models: Option<bool>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub enum OpenCodeApiProtocol {
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "openai_responses", alias = "open_ai_responses")]
    OpenAiResponses,
    #[serde(rename = "openai_chat", alias = "open_ai_chat")]
    OpenAiChat,
    #[serde(rename = "google")]
    Google,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "snake_case")]
pub enum OpenCodeModelSubscription {
    Zen,
    Go,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenCodeAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    /// 此模型使用的 API 协议："anthropic"、"openai_responses"、"openai_chat" 或 "google"。默认为 "openai_chat"。
    pub protocol: Option<OpenCodeApiProtocol>,
    /// 此模型的订阅方案："zen" 或 "go"，默认为 Zen。
    pub subscription: Option<OpenCodeModelSubscription>,
    /// 此模型使用的自定义模型 API URL
    pub custom_model_api_url: Option<String>,
    /// 支持的推理力度级别，例如 `["low", "medium", "high"].
    pub reasoning_effort_levels: Option<Vec<ReasoningEffort>>,
    /// 使用 OpenAiChat 协议时，思考 token 是作为专用 `reasoning_content` 字段发送，还是内联在消息文本中
    #[serde(default)]
    pub interleaved_reasoning: bool,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct LmStudioSettingsContent {
    pub api_url: Option<String>,
    pub api_key: Option<String>,
    pub available_models: Option<Vec<LmStudioAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LmStudioAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub supports_tool_calls: bool,
    pub supports_images: bool,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct LlamaCppSettingsContent {
    pub api_url: Option<String>,
    /// 是否自动发现 llama.cpp 服务器提供的模型。
    /// 默认为 true。
    pub auto_discover: Option<bool>,
    pub available_models: Option<Vec<LlamaCppAvailableModel>>,
    /// 覆盖每个 llama.cpp 模型所报告的上下文长度
    pub context_window: Option<u64>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LlamaCppAvailableModel {
    /// llama.cpp 服务器报告的模型 ID（其 `--alias` 或模型文件路径）
    pub name: String,
    /// 模型在 Zed 界面中显示的名称，例如智能体面板中的模型选择下拉菜单
    pub display_name: Option<String>,
    /// 传递给模型的上下文长度参数（即 n_ctx）
    pub max_tokens: u64,
    /// 模型是否支持工具
    pub supports_tools: Option<bool>,
    /// 模型是否支持视觉
    pub supports_images: Option<bool>,
    /// 模型是否输出推理/思考内容
    pub supports_thinking: Option<bool>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct DeepseekSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<DeepseekAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct DeepseekAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct MistralSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<MistralAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct MistralAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub supports_tools: Option<bool>,
    pub supports_images: Option<bool>,
    pub supports_thinking: Option<bool>,
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct OpenAiSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<OpenAiAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenAiAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub reasoning_effort: Option<OpenAiReasoningEffort>,
    #[serde(default)]
    pub capabilities: OpenAiModelCapabilities,
}

pub use language_model_core::ReasoningEffort as OpenAiReasoningEffort;

impl MergeFrom for OpenAiReasoningEffort {
    fn merge_from(&mut self, other: &Self) {
        *self = *other;
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct OpenAiCompatibleSettingsContent {
    pub api_url: String,
    pub available_models: Vec<OpenAiCompatibleAvailableModel>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenAiModelCapabilities {
    #[serde(default = "default_true")]
    pub chat_completions: bool,
    #[serde(default = "default_true")]
    pub images: bool,
}

impl Default for OpenAiModelCapabilities {
    fn default() -> Self {
        Self {
            chat_completions: default_true(),
            images: default_true(),
        }
    }
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenAiCompatibleAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub reasoning_effort: Option<OpenAiReasoningEffort>,
    #[serde(default)]
    pub capabilities: OpenAiCompatibleModelCapabilities,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenAiCompatibleModelCapabilities {
    pub tools: bool,
    pub images: bool,
    pub parallel_tool_calls: bool,
    pub prompt_cache_key: bool,
    #[serde(default = "default_true")]
    pub chat_completions: bool,
    #[serde(default)]
    pub interleaved_reasoning: bool,
    #[serde(default)]
    pub max_tokens_parameter: bool,
}

impl Default for OpenAiCompatibleModelCapabilities {
    fn default() -> Self {
        Self {
            tools: true,
            images: false,
            parallel_tool_calls: false,
            prompt_cache_key: false,
            chat_completions: default_true(),
            interleaved_reasoning: false,
            max_tokens_parameter: false,
        }
    }
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct VercelAiGatewaySettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<VercelAiGatewayAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct VercelAiGatewayAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    #[serde(default)]
    pub capabilities: OpenAiCompatibleModelCapabilities,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct GoogleSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<GoogleAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct GoogleAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub mode: Option<ModelMode>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct XAiSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<XaiAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct XaiAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub supports_images: Option<bool>,
    pub supports_tools: Option<bool>,
    pub parallel_tool_calls: Option<bool>,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct ZedDotDevSettingsContent {
    pub available_models: Option<Vec<ZedDotDevAvailableModel>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ZedDotDevAvailableModel {
    /// 语言模型的提供商
    pub provider: ZedDotDevAvailableProvider,
    /// 模型在提供商 API 中的名称，例如 claude-3-5-sonnet-20240620
    pub name: String,
    /// 在界面中显示的名称，例如智能体面板的模型下拉菜单
    pub display_name: Option<String>,
    /// 上下文窗口的大小，表示模型可处理的最大 token 数量
    pub max_tokens: usize,
    /// 模型允许的最大输出 token 数
    pub max_output_tokens: Option<u64>,
    /// 模型允许的最大生成 Token 数（仅限 o1-*）
    pub max_completion_tokens: Option<u64>,
    /// 使用其他 Anthropic 模型覆盖此模型以进行工具调用。
    pub tool_override: Option<String>,
    /// 指示此自定义模型是否支持缓存。
    pub cache_configuration: Option<LanguageModelCacheConfiguration>,
    /// 此模型使用的默认温度
    #[serde(serialize_with = "crate::serialize_optional_f32_with_two_decimal_places")]
    pub default_temperature: Option<f32>,
    /// 使用模型时提供的额外 beta 标头。
    #[serde(default)]
    pub extra_beta_headers: Vec<String>,
    /// 模型的模式（例如思考模式）
    pub mode: Option<ModelMode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "lowercase")]
pub enum ZedDotDevAvailableProvider {
    Anthropic,
    OpenAi,
    Google,
}

#[with_fallible_options]
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema, MergeFrom)]
pub struct OpenRouterSettingsContent {
    pub api_url: Option<String>,
    pub available_models: Option<Vec<OpenRouterAvailableModel>>,
    pub custom_headers: Option<HashMap<String, String>>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenRouterAvailableModel {
    pub name: String,
    pub display_name: Option<String>,
    pub max_tokens: u64,
    pub max_output_tokens: Option<u64>,
    pub max_completion_tokens: Option<u64>,
    pub supports_tools: Option<bool>,
    pub supports_images: Option<bool>,
    pub mode: Option<ModelMode>,
    pub reasoning_effort: Option<ReasoningEffort>,
    pub provider: Option<OpenRouterProvider>,
}

#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct OpenRouterProvider {
    order: Option<Vec<String>>,
    #[serde(default = "default_true")]
    allow_fallbacks: bool,
    #[serde(default)]
    require_parameters: bool,
    #[serde(default)]
    data_collection: DataCollection,
    only: Option<Vec<String>>,
    ignore: Option<Vec<String>>,
    quantizations: Option<Vec<String>>,
    sort: Option<String>,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
#[serde(rename_all = "lowercase")]
pub enum DataCollection {
    #[default]
    Allow,
    Disallow,
}

fn default_true() -> bool {
    true
}

/// 缓存语言模型消息的配置
#[with_fallible_options]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct LanguageModelCacheConfiguration {
    pub max_cache_anchors: usize,
    pub should_speculate: bool,
    pub min_total_token: u64,
}

pub use language_model_core::ModelMode;

impl MergeFrom for ModelMode {
    fn merge_from(&mut self, other: &Self) {
        *self = *other;
    }
}
