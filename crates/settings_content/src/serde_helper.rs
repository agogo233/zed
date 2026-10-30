use serde::Serializer;

/// 以 2 位小数精度序列化 f32 值
///
/// 此函数将值四舍五入到 2 位小数并格式化为字符串，
/// 然后在序列化之前将其解析回 f64，以确保生成干净的 JSON 输出
/// 不含 IEEE 754 浮点数精度问题
///
/// # 参数
///
/// * `value` — 要序列化的 f32 值
/// * `serializer` — 要使用的 serde 序列化器
///
/// # 返回值
///
/// 序列化操作的结果
///
/// # 用法
///
/// 此函数可与 Serde 的 `serialize_with` 属性一起使用：
/// ```
/// use serde::Serialize;
/// use settings_content::serialize_f32_with_two_decimal_places;
///
/// #[derive(Serialize)]
/// struct ExampleStruct(#[serde(serialize_with = "serialize_f32_with_two_decimal_places")] f32);
/// ```
pub fn serialize_f32_with_two_decimal_places<S>(
    value: &f32,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let rounded = (value * 100.0).round() / 100.0;
    let formatted = format!("{:.2}", rounded);
    let clean_value: f64 = formatted.parse().unwrap_or(rounded as f64);
    serializer.serialize_f64(clean_value)
}

/// 以 2 位小数精度序列化可选的 f32 值
///
/// 此函数处理 `Option<f32>` 类型，将 `Some` 值序列化为 2 位小数精度
/// 并将 `None` 值序列化为 null。对于 `Some` 值，会四舍五入到 2 位小数
/// 并格式化为字符串，再解析回 f64 后进行序列化。这可确保
/// 输出干净的 JSON，不含 IEEE 754 浮点数精度误差。
///
/// # 参数
///
/// * `value` — 要序列化的可选 f32 值
/// * `serializer` — 要使用的 serde 序列化器
///
/// # 返回值
///
/// 序列化操作的结果
///
/// # 行为
///
/// * `Some(v)` — 将值四舍五入到 2 位小数后序列化
/// * `None` — 序列化为 JSON null
///
/// # 用法
///
/// 此函数可与 Serde 的 `serialize_with` 属性一起使用：
/// ```
/// use serde::Serialize;
/// use settings_content::serialize_optional_f32_with_two_decimal_places;
///
/// #[derive(Serialize)]
/// struct ExampleStruct {
///     #[serde(serialize_with = "serialize_optional_f32_with_two_decimal_places")]
///     optional_value: Option<f32>,
/// }
/// ```
pub fn serialize_optional_f32_with_two_decimal_places<S>(
    value: &Option<f32>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(v) => {
            let rounded = (v * 100.0).round() / 100.0;
            let formatted = format!("{:.2}", rounded);
            let clean_value: f64 = formatted.parse().unwrap_or(rounded as f64);
            serializer.serialize_some(&clean_value)
        }
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct TestOptional {
        #[serde(serialize_with = "serialize_optional_f32_with_two_decimal_places")]
        value: Option<f32>,
    }

    #[derive(Serialize, Deserialize)]
    struct TestNonOptional {
        #[serde(serialize_with = "serialize_f32_with_two_decimal_places")]
        value: f32,
    }

    #[test]
    fn test_serialize_optional_f32_with_two_decimal_places() {
        let cases = [
            (Some(123.456789), r#"{"value":123.46}"#),
            (Some(1.2), r#"{"value":1.2}"#),
            (Some(300.00000), r#"{"value":300.0}"#),
        ];
        for (value, expected) in cases {
            let value = TestOptional { value };
            assert_eq!(serde_json::to_string(&value).unwrap(), expected);
        }
    }

    #[test]
    fn test_serialize_f32_with_two_decimal_places() {
        let cases = [
            (123.456789, r#"{"value":123.46}"#),
            (1.200, r#"{"value":1.2}"#),
            (300.00000, r#"{"value":300.0}"#),
        ];
        for (value, expected) in cases {
            let value = TestNonOptional { value };
            assert_eq!(serde_json::to_string(&value).unwrap(), expected);
        }
    }
}
