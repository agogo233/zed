use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result};

use collections::HashMap;
use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use settings_macros::MergeFrom;

/// `command_aliases` 中条目的值，可以是已注册的操作名称
/// 或任意字符串，例如 `"w"` 或 `"wq"` 这样的 vim 命令。
///
/// This is a separate type from `ActionName` on purpose, as `ActionName` is
/// also used by keymap bindings, which only accept registered action names, and
/// we don't want to weaken those guarantees and accept any arbitrary string.
#[derive(Serialize, Deserialize, Default, MergeFrom, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct CommandAliasTarget(String);

impl CommandAliasTarget {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Build the runtime schema, an `anyOf` whose entries are the same as the
    /// ones produced by [`ActionName::build_schema`] plus a final fallback for
    /// arbitrary strings that aren't registered actions.
    pub fn build_schema(
        action_names: &[&str],
        action_documentation: &HashMap<&str, &str>,
        deprecations: &HashMap<&str, &str>,
        deprecation_messages: &HashMap<&str, &str>,
    ) -> Schema {
        let action_schema = ActionName::build_schema(
            action_names.iter().copied(),
            action_documentation,
            deprecations,
            deprecation_messages,
        );

        let mut alternatives = action_schema
            .to_value()
            .get("anyOf")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        let mut exclude = action_names
            .iter()
            .map(|name| serde_json::json!({ "const": *name }))
            .collect::<Vec<Value>>();

        // Since we accept any arbitrary string, besides the list of registered
        // action names, we'll add any string that uses a `::` pattern to the
        // list of exclusions, ensuring we display a warning in that case,
        // otherwise the user might be led into thinking they've typed a valid
        // action name when that's not the case.
        exclude.push(serde_json::json!({ "pattern": "::" }));

        alternatives.push(serde_json::json!({
            "type": "string",
            "not": { "anyOf": exclude }
        }));

        json_schema!({ "anyOf": alternatives })
    }
}

impl Display for CommandAliasTarget {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0)
    }
}

impl AsRef<str> for CommandAliasTarget {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl JsonSchema for CommandAliasTarget {
    fn schema_name() -> Cow<'static, str> {
        "CommandAliasTarget".into()
    }

    /// 作为占位符返回 `true`
    ///
    /// 真实 schema 是所有已注册操作名称的 `anyOf`，并附带操作文档，
    /// documentation and deprecation metadata, as well as any arbitrary string,
    /// 无法在此处生成，因为 `JsonSchema::json_schema` 不接收任何
    /// runtime context. It is instead built by call sites that do have access
    /// to the GPUI action registry using [`CommandAliasTarget::build_schema`].
    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!(true)
    }
}

/// 已注册 GPUI 操作的名称，序列化为普通 JSON 字符串，
/// 例如，"editor::Cancel"` 或 `"workspace::CloseActiveItem"`。
///
/// 此新类型的存在是为了使 `command_aliases` 等设置字段或
/// 键映射文件绑定，能够针对操作集合请求 JSON schema 自动补全，
/// 该集合包含运行时已知的操作。
#[derive(Serialize, Deserialize, Default, MergeFrom, Clone, Debug, PartialEq)]
#[serde(transparent)]
pub struct ActionName(String);

/// 使用所提供的弃用消息填充架构 `deprecationMessage` 字段的
/// 小型辅助函数
fn add_deprecation(schema: &mut Schema, message: String) {
    schema.insert("deprecationMessage".into(), Value::String(message));
}

/// 使用所提供的描述填充架构 `description` 字段的
/// 小型辅助函数
fn add_description(schema: &mut Schema, description: &str) {
    schema.insert("description".into(), Value::String(description.to_string()));
}

impl ActionName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// 构建用于 `$defs/ActionName` 的 JSON 架构，基本上是一个
    /// 所有可用操作的 `anyOf`，并附带每个操作的文档
    /// 并附加弃用元数据。
    pub fn build_schema<'a>(
        action_names: impl IntoIterator<Item = &'a str>,
        action_documentation: &HashMap<&str, &str>,
        deprecations: &HashMap<&str, &str>,
        deprecation_messages: &HashMap<&str, &str>,
    ) -> Schema {
        let mut alternatives = Vec::new();

        for action_name in action_names {
            let mut entry = json_schema!({
                "type": "string",
                "const": action_name
            });

            if let Some(message) = deprecation_messages.get(action_name) {
                add_deprecation(&mut entry, message.to_string());
            } else if let Some(new_name) = deprecations.get(action_name) {
                add_deprecation(&mut entry, format!("Deprecated, use {new_name}"));
            }

            if let Some(description) = action_documentation.get(action_name) {
                add_description(&mut entry, description);
            }

            alternatives.push(entry);
        }

        json_schema!({ "anyOf": alternatives })
    }
}

impl Display for ActionName {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> Result {
        write!(formatter, "{}", self.0)
    }
}

impl AsRef<str> for ActionName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl JsonSchema for ActionName {
    /// 此类型在生成器 `$defs` 映射中存储时使用的名称，
    /// 在 schema 生成过程中 schemars 遇到该类型时使用。
    /// 将其保持为 `"ActionName"` 使消费者可通过以下方式引用
    /// `#/$defs/ActionName`，并让 [`util::schemars::replace_subschema`] 能够在运行时
    /// 查找它，以替换为真实 schema。
    fn schema_name() -> Cow<'static, str> {
        "ActionName".into()
    }

    /// 作为占位符返回 `true`
    ///
    /// 真实 schema 是所有已注册操作名称的 `anyOf`，并附带操作文档，
    /// 以及弃用元数据；它无法在此处生成，因为
    /// `JsonSchema::json_schema` 不接收运行时上下文。它改为
    /// 由能够访问 GPUI 操作注册表的调用位置
    /// 使用 [`ActionName::build_schema`] 构建。
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!(true)
    }
}

/// 一个 GPUI 操作及其输入数据，序列化为二元素 JSON 数组；
/// 其格式为 `["namespace::Name", { ... }]`，例如
/// `["pane::ActivateItem", { "index": 0 }]`
#[derive(Deserialize, Default)]
#[serde(transparent)]
pub struct ActionWithArguments(pub Value);

impl JsonSchema for ActionWithArguments {
    /// 此类型在生成器 `$defs` 映射中存储时使用的名称，
    /// 在 schema 生成过程中 schemars 遇到该类型时使用。
    /// 将其保持为 `"ActionWithArguments"` 后，消费者可
    /// 通过 `#/$defs/ActionWithArguments` 引用，并允许
    /// [`util::schemars::replace_subschema`] 在运行时查找它以替换为
    /// 真实的模式定义。
    fn schema_name() -> Cow<'static, str> {
        "ActionWithArguments".into()
    }

    /// 作为占位符返回 `true`
    ///
    /// 真实 schema 是由所有支持参数的已注册操作名称组成的 `anyOf`，
    /// 并附带操作文档和弃用元数据；它
    /// 无法在此处生成，因为 `JsonSchema::json_schema` 不接收任何
    /// 运行时上下文。在撰写本文时，它改由以下方式构建：
    /// [`KeymapFile::generate_json_schema`]，其中所有运行时
    /// 信息可用。
    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_schema_produces_anyof_of_consts_per_name() {
        let mut action_documentation = HashMap::default();
        let mut deprecations = HashMap::default();
        let mut deprecation_messages = HashMap::default();
        action_documentation.insert("editor::Cancel", "Cancel the current operation.");
        deprecations.insert("workspace::CloseCurrentItem", "workspace::CloseActiveItem");
        deprecation_messages.insert("editor::Explode", "DO NOT USE!");

        let schema = ActionName::build_schema(
            [
                "editor::Cancel",
                "editor::Explode",
                "workspace::CloseCurrentItem",
                "workspace::CloseActiveItem",
            ],
            &action_documentation,
            &deprecations,
            &deprecation_messages,
        );

        let value = schema.to_value();
        let values = value
            .pointer("/anyOf")
            .and_then(|v| v.as_array())
            .expect("anyOf should be present");
        assert_eq!(values.len(), 4);

        let (name, schema_type, description) = (
            values[0].get("const").and_then(Value::as_str),
            values[0].get("type").and_then(Value::as_str),
            values[0].get("description").and_then(Value::as_str),
        );
        assert_eq!(name, Some("editor::Cancel"));
        assert_eq!(schema_type, Some("string"));
        assert_eq!(description, Some("Cancel the current operation."));

        let (name, schema_type, message) = (
            values[1].get("const").and_then(Value::as_str),
            values[1].get("type").and_then(Value::as_str),
            values[1].get("deprecationMessage").and_then(Value::as_str),
        );
        assert_eq!(name, Some("editor::Explode"));
        assert_eq!(schema_type, Some("string"));
        assert_eq!(message, Some("DO NOT USE!"));

        let (name, schema_type, message) = (
            values[2].get("const").and_then(Value::as_str),
            values[2].get("type").and_then(Value::as_str),
            values[2].get("deprecationMessage").and_then(Value::as_str),
        );
        assert_eq!(name, Some("workspace::CloseCurrentItem"));
        assert_eq!(schema_type, Some("string"));
        assert_eq!(message, Some("Deprecated, use workspace::CloseActiveItem"));

        let (name, schema_type) = (
            values[3].get("const").and_then(Value::as_str),
            values[3].get("type").and_then(Value::as_str),
        );
        assert_eq!(name, Some("workspace::CloseActiveItem"));
        assert_eq!(schema_type, Some("string"));
    }
}
