use std::sync::Arc;

use collections::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use settings_macros::{MergeFrom, with_fallible_options};
use util::serde::default_true;

#[with_fallible_options]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, JsonSchema, MergeFrom)]
pub struct ExtensionSettingsContent {
    /// Zed 应自动安装的扩展
    ///
    /// 此项用于使扩展提供的功能（例如语言支持）
    /// 开箱即用。
    ///
    /// 默认值：{ "html": true }
    #[serde(default)]
    pub auto_install_extensions: HashMap<Arc<str>, bool>,
    #[serde(default)]
    pub auto_update_extensions: HashMap<Arc<str>, bool>,
    /// 是否根据打开的文件建议安装扩展
    ///
    /// 默认值：true
    #[serde(default = "default_true")]
    pub suggest_extensions: bool,
    /// 授予扩展的权限
    pub granted_extension_capabilities: Option<Vec<ExtensionCapabilityContent>>,
}

impl Default for ExtensionSettingsContent {
    fn default() -> Self {
        Self {
            auto_install_extensions: HashMap::default(),
            auto_update_extensions: HashMap::default(),
            suggest_extensions: true,
            granted_extension_capabilities: None,
        }
    }
}

/// 扩展的权限
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ExtensionCapabilityContent {
    #[serde(rename = "process:exec")]
    ProcessExec {
        /// 要执行的命令
        command: String,
        /// 传递给命令的参数。使用 `*` 作为单个通配符参数。
        /// 如果最后一个元素是 `**`，则允许任何尾随参数
        args: Vec<String>,
    },
    DownloadFile {
        host: String,
        path: Vec<String>,
    },
    #[serde(rename = "npm:install")]
    NpmInstallPackage {
        package: String,
    },
}
