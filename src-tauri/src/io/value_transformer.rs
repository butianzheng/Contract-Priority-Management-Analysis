use super::generic_parser::RawRow;
use crate::db::init::get_connection;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 转换警告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformWarning {
    pub row_number: usize,
    pub field: String,
    pub message: String,
}

/// 转换配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformConfig {
    /// 字段级转换规则：field_name → Vec<TransformStep>
    pub field_transforms: HashMap<String, Vec<TransformStep>>,
    /// 默认值配置
    #[serde(default)]
    pub default_values: HashMap<String, DefaultValueConfig>,
}

/// 单个转换步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TransformStep {
    /// 值映射：{"VIP": "A", "重点": "A", "普通": "B"}
    Mapping {
        values: HashMap<String, String>,
        fallback: Option<String>,
    },
    /// 正则替换
    Regex {
        pattern: String,
        replacement: String,
    },
    /// 日期格式统一
    DateFormat {
        input_formats: Vec<String>,
        output_format: String,
    },
    /// 去空格
    Trim,
    /// 大小写转换
    Case { mode: CaseMode },
    /// 数学公式
    Formula { expression: String },
    /// 条件表达式
    Condition {
        rules: Vec<ConditionRule>,
        default: Option<String>,
    },
    /// 多字段拼接
    Concat {
        template: String,
        separator: Option<String>,
    },
    /// 字段拆分
    Split {
        separator: String,
        target_fields: Vec<String>,
    },
    /// 查找替换表（从 DB 表中查找映射值）
    LookupTable {
        table_name: String,
        key_field: String,
        value_field: String,
    },
    /// 自定义脚本表达式
    Script { expression: String },
}

/// 条件规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionRule {
    pub condition: String,
    pub result: String,
}

/// 大小写模式
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaseMode {
    Upper,
    Lower,
    Title,
}

/// 默认值配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultValueConfig {
    pub value: String,
    pub condition: DefaultCondition,
}

/// 默认值条件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefaultCondition {
    WhenEmpty,
    WhenMissing,
    Always,
}

pub struct ValueTransformer;

impl ValueTransformer {
    /// 应用所有转换规则
    pub fn transform(rows: &mut [RawRow], config: &TransformConfig) -> Vec<TransformWarning> {
        let mut warnings = Vec::new();

        // 1. 应用默认值
        for (row_idx, row) in rows.iter_mut().enumerate() {
            for (field, default_config) in &config.default_values {
                Self::apply_default(row, field, default_config);
            }

            // 2. 应用字段转换
            for (field, steps) in &config.field_transforms {
                for step in steps {
                    if let Err(msg) = Self::apply_step(row, field, step) {
                        warnings.push(TransformWarning {
                            row_number: row_idx + 2,
                            field: field.clone(),
                            message: msg,
                        });
                    }
                }
            }
        }

        warnings
    }

    /// 应用单个转换步骤到一行数据
    fn apply_step(row: &mut RawRow, field: &str, step: &TransformStep) -> Result<(), String> {
        match step {
            TransformStep::Mapping { values, fallback } => {
                if let Some(current) = row.get(field).cloned() {
                    let trimmed = current.trim().to_string();
                    if let Some(mapped) = values.get(&trimmed) {
                        row.insert(field.to_string(), mapped.clone());
                    } else if let Some(fb) = fallback {
                        row.insert(field.to_string(), fb.clone());
                    }
                }
                Ok(())
            }

            TransformStep::Regex {
                pattern,
                replacement,
            } => {
                if let Some(current) = row.get(field).cloned() {
                    match regex::Regex::new(pattern) {
                        Ok(re) => {
                            let result = re.replace_all(&current, replacement.as_str()).to_string();
                            row.insert(field.to_string(), result);
                            Ok(())
                        }
                        Err(e) => Err(format!("正则表达式错误: {}", e)),
                    }
                } else {
                    Ok(())
                }
            }

            TransformStep::DateFormat {
                input_formats,
                output_format,
            } => {
                if let Some(current) = row.get(field).cloned() {
                    let trimmed = current.trim();
                    if trimmed.is_empty() {
                        return Ok(());
                    }

                    for fmt in input_formats {
                        if let Some(result) = Self::try_parse_date(trimmed, fmt, output_format) {
                            row.insert(field.to_string(), result);
                            return Ok(());
                        }
                    }
                    // 如果所有格式都不匹配，保留原值
                    Err(format!("日期格式不匹配: {}", current))
                } else {
                    Ok(())
                }
            }

            TransformStep::Trim => {
                if let Some(current) = row.get(field).cloned() {
                    row.insert(field.to_string(), current.trim().to_string());
                }
                Ok(())
            }

            TransformStep::Case { mode } => {
                if let Some(current) = row.get(field).cloned() {
                    let result = match mode {
                        CaseMode::Upper => current.to_uppercase(),
                        CaseMode::Lower => current.to_lowercase(),
                        CaseMode::Title => Self::to_title_case(&current),
                    };
                    row.insert(field.to_string(), result);
                }
                Ok(())
            }

            TransformStep::Formula { expression } => {
                if let Some(current) = row.get(field).cloned() {
                    if let Ok(value) = current.trim().parse::<f64>() {
                        match Self::eval_simple_formula(expression, value) {
                            Ok(result) => {
                                row.insert(field.to_string(), result.to_string());
                                Ok(())
                            }
                            Err(e) => Err(format!("公式计算错误: {}", e)),
                        }
                    } else {
                        Err(format!("公式转换需要数字值，当前值: {}", current))
                    }
                } else {
                    Ok(())
                }
            }

            TransformStep::Condition { rules, default } => {
                if let Some(current) = row.get(field).cloned() {
                    for rule in rules {
                        if Self::eval_condition(&rule.condition, &current, row) {
                            row.insert(field.to_string(), rule.result.clone());
                            return Ok(());
                        }
                    }
                    if let Some(def) = default {
                        row.insert(field.to_string(), def.clone());
                    }
                }
                Ok(())
            }

            TransformStep::Concat {
                template,
                separator: _,
            } => {
                let result = Self::expand_template(template, row);
                row.insert(field.to_string(), result);
                Ok(())
            }

            TransformStep::Split {
                separator,
                target_fields,
            } => {
                if let Some(current) = row.get(field).cloned() {
                    let parts: Vec<&str> = current.split(separator.as_str()).collect();
                    for (i, target) in target_fields.iter().enumerate() {
                        if let Some(part) = parts.get(i) {
                            row.insert(target.clone(), part.trim().to_string());
                        }
                    }
                }
                Ok(())
            }

            TransformStep::LookupTable {
                table_name,
                key_field,
                value_field,
            } => {
                if let Some(current) = row.get(field).cloned() {
                    let key = current.trim();
                    if key.is_empty() {
                        return Ok(());
                    }
                    match Self::lookup_table_value(table_name, key_field, value_field, key)? {
                        Some(mapped) => {
                            row.insert(field.to_string(), mapped);
                            Ok(())
                        }
                        None => Err(format!(
                            "查找表未命中: 表={} 字段={} 值={}",
                            table_name, key_field, key
                        )),
                    }
                } else {
                    Ok(())
                }
            }

            TransformStep::Script { expression } => {
                // 脚本化转换委托给 ExpressionEngine
                // 这里做简单的变量替换和基础运算
                match super::expression_engine::ExpressionEngine::evaluate_str(expression, row) {
                    Ok(result) => {
                        row.insert(field.to_string(), result);
                        Ok(())
                    }
                    Err(e) => Err(format!("脚本执行错误: {}", e)),
                }
            }
        }
    }

    /// 应用默认值
    fn apply_default(row: &mut RawRow, field: &str, config: &DefaultValueConfig) {
        match config.condition {
            DefaultCondition::WhenEmpty => {
                if let Some(v) = row.get(field) {
                    if v.trim().is_empty() {
                        row.insert(field.to_string(), config.value.clone());
                    }
                }
            }
            DefaultCondition::WhenMissing => {
                if !row.contains_key(field) {
                    row.insert(field.to_string(), config.value.clone());
                }
            }
            DefaultCondition::Always => {
                row.insert(field.to_string(), config.value.clone());
            }
        }
    }

    /// 尝试解析日期
    fn try_parse_date(input: &str, input_format: &str, output_format: &str) -> Option<String> {
        // 简单的日期格式转换
        // 支持常见格式：YYYY-MM-DD, YYYY/MM/DD, DD-MM-YYYY, DD/MM/YYYY, YYYYMMDD
        let (year, month, day) = Self::extract_date_parts(input, input_format)?;

        // 验证日期有效性
        if year < 1900 || year > 2100 || month < 1 || month > 12 || day < 1 || day > 31 {
            return None;
        }

        // 格式化输出
        let result = output_format
            .replace("YYYY", &format!("{:04}", year))
            .replace("MM", &format!("{:02}", month))
            .replace("DD", &format!("{:02}", day));

        Some(result)
    }

    fn extract_date_parts(input: &str, format: &str) -> Option<(u32, u32, u32)> {
        let input = input.trim();

        match format {
            "YYYY-MM-DD" => {
                let parts: Vec<&str> = input.split('-').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].parse().ok()?,
                        parts[1].parse().ok()?,
                        parts[2].parse().ok()?,
                    ))
                } else {
                    None
                }
            }
            "YYYY/MM/DD" => {
                let parts: Vec<&str> = input.split('/').collect();
                if parts.len() == 3 {
                    Some((
                        parts[0].parse().ok()?,
                        parts[1].parse().ok()?,
                        parts[2].parse().ok()?,
                    ))
                } else {
                    None
                }
            }
            "DD-MM-YYYY" => {
                let parts: Vec<&str> = input.split('-').collect();
                if parts.len() == 3 {
                    Some((
                        parts[2].parse().ok()?,
                        parts[1].parse().ok()?,
                        parts[0].parse().ok()?,
                    ))
                } else {
                    None
                }
            }
            "DD/MM/YYYY" => {
                let parts: Vec<&str> = input.split('/').collect();
                if parts.len() == 3 {
                    Some((
                        parts[2].parse().ok()?,
                        parts[1].parse().ok()?,
                        parts[0].parse().ok()?,
                    ))
                } else {
                    None
                }
            }
            "YYYYMMDD" => {
                if input.len() == 8 {
                    Some((
                        input[0..4].parse().ok()?,
                        input[4..6].parse().ok()?,
                        input[6..8].parse().ok()?,
                    ))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Title Case 转换
    fn to_title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 简单公式求值（支持 value * N, value / N, value + N, value - N）
    fn eval_simple_formula(expression: &str, value: f64) -> Result<f64, String> {
        let expr = expression.trim().replace("value", &value.to_string());

        // 简单的四则运算解析
        // 支持格式：number op number
        let expr = expr.trim();

        // 尝试直接解析为数字
        if let Ok(n) = expr.parse::<f64>() {
            return Ok(n);
        }

        // 查找运算符（从右到左，先处理加减）
        for op in ['+', '-'] {
            if let Some(pos) = expr.rfind(op) {
                if pos > 0 {
                    let left = expr[..pos].trim();
                    let right = expr[pos + 1..].trim();
                    if let (Ok(l), Ok(r)) = (
                        Self::eval_simple_formula(left, value),
                        Self::eval_simple_formula(right, value),
                    ) {
                        return Ok(match op {
                            '+' => l + r,
                            '-' => l - r,
                            _ => unreachable!(),
                        });
                    }
                }
            }
        }

        for op in ['*', '/'] {
            if let Some(pos) = expr.rfind(op) {
                if pos > 0 {
                    let left = expr[..pos].trim();
                    let right = expr[pos + 1..].trim();
                    if let (Ok(l), Ok(r)) = (
                        Self::eval_simple_formula(left, value),
                        Self::eval_simple_formula(right, value),
                    ) {
                        return Ok(match op {
                            '*' => l * r,
                            '/' => {
                                if r == 0.0 {
                                    return Err("除数不能为零".to_string());
                                }
                                l / r
                            }
                            _ => unreachable!(),
                        });
                    }
                }
            }
        }

        Err(format!("无法解析公式: {}", expression))
    }

    /// 模板展开：将 {field_name} 替换为行中对应字段的值
    fn expand_template(template: &str, row: &RawRow) -> String {
        let mut result = template.to_string();
        for (key, value) in row {
            result = result.replace(&format!("{{{}}}", key), value);
        }
        result
    }

    /// 简单条件求值
    fn eval_condition(condition: &str, current_value: &str, row: &RawRow) -> bool {
        let condition = condition.trim();

        // 支持格式：value op literal
        // op: =, !=, >, <, >=, <=, contains, starts_with, ends_with

        // 替换 value 为当前值
        let condition = condition.replace("value", &format!("\"{}\"", current_value));

        // 解析比较表达式
        for (op, op_str) in [
            (">=", ">="),
            ("<=", "<="),
            ("!=", "!="),
            ("=", "="),
            (">", ">"),
            ("<", "<"),
        ] {
            if let Some(pos) = condition.find(op_str) {
                let left = condition[..pos].trim().trim_matches('"');
                let right = condition[pos + op.len()..].trim().trim_matches('"');

                // 尝试数值比较
                if let (Ok(l), Ok(r)) = (left.parse::<f64>(), right.parse::<f64>()) {
                    return match op {
                        ">=" => l >= r,
                        "<=" => l <= r,
                        "!=" => (l - r).abs() > f64::EPSILON,
                        "=" => (l - r).abs() < f64::EPSILON,
                        ">" => l > r,
                        "<" => l < r,
                        _ => false,
                    };
                }

                // 字符串比较
                return match op {
                    "=" => left == right,
                    "!=" => left != right,
                    _ => false,
                };
            }
        }

        // contains 操作
        if condition.contains("contains") {
            let parts: Vec<&str> = condition.splitn(2, "contains").collect();
            if parts.len() == 2 {
                let left = parts[0].trim().trim_matches('"');
                let right = parts[1].trim().trim_matches('"');
                return left.contains(right);
            }
        }

        false
    }

    fn lookup_table_value(
        table_name: &str,
        key_field: &str,
        value_field: &str,
        key: &str,
    ) -> Result<Option<String>, String> {
        if !Self::is_safe_identifier(table_name)
            || !Self::is_safe_identifier(key_field)
            || !Self::is_safe_identifier(value_field)
        {
            return Err("查找表配置包含非法标识符".to_string());
        }

        let conn = get_connection().map_err(|e| format!("获取数据库连接失败: {}", e))?;
        let sql = format!("SELECT {value_field} FROM {table_name} WHERE {key_field} = ?1 LIMIT 1");

        conn.query_row(&sql, params![key], |row| row.get::<_, String>(0))
            .optional()
            .map_err(|e| format!("查找表查询失败: {}", e))
    }

    fn is_safe_identifier(value: &str) -> bool {
        let mut chars = value.chars();
        match chars.next() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
            _ => return false,
        }

        chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

/// 从 DB 的 value_transform JSON 解析为 TransformConfig
pub fn parse_transform_config(
    value_transform: Option<&str>,
    default_values: Option<&str>,
) -> TransformConfig {
    let field_transforms = if let Some(vt) = value_transform {
        parse_db_value_transform(vt)
    } else {
        HashMap::new()
    };

    let defaults = if let Some(dv) = default_values {
        serde_json::from_str(dv).unwrap_or_default()
    } else {
        HashMap::new()
    };

    TransformConfig {
        field_transforms,
        default_values: defaults,
    }
}

/// 解析 DB 中的 value_transform JSON 为 TransformStep
fn parse_db_value_transform(json_str: &str) -> HashMap<String, Vec<TransformStep>> {
    let mut result = HashMap::new();

    let parsed: HashMap<String, serde_json::Value> = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return result,
    };

    for (field, config) in parsed {
        let steps = parse_transform_steps_for_field(&config);
        if !steps.is_empty() {
            result.insert(field, steps);
        }
    }

    result
}

fn parse_transform_steps_for_field(config: &serde_json::Value) -> Vec<TransformStep> {
    match config {
        serde_json::Value::Array(arr) => arr.iter().filter_map(parse_transform_step).collect(),
        serde_json::Value::Object(obj) => {
            if let Some(steps) = obj.get("steps").and_then(|v| v.as_array()) {
                let parsed: Vec<TransformStep> =
                    steps.iter().filter_map(parse_transform_step).collect();
                if !parsed.is_empty() {
                    return parsed;
                }
            }

            parse_transform_step(config).into_iter().collect()
        }
        _ => Vec::new(),
    }
}

fn parse_transform_step(config: &serde_json::Value) -> Option<TransformStep> {
    let obj = config.as_object()?;
    let transform_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_lowercase();

    if transform_type.is_empty() {
        return None;
    }

    let scoped = obj.get("config").unwrap_or(config);
    let get_field = |name: &str| scoped.get(name).or_else(|| obj.get(name));

    match transform_type.as_str() {
        "mapping" => {
            let values = get_field("values")
                .and_then(json_to_string_map)
                .unwrap_or_default();
            Some(TransformStep::Mapping {
                values,
                fallback: get_field("fallback")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        }
        "regex" => Some(TransformStep::Regex {
            pattern: get_field("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            replacement: get_field("replacement")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "date_format" | "dateformat" => {
            let input_formats = parse_string_array(get_field("input_formats"));
            let output_format = get_field("output_format")
                .and_then(|v| v.as_str())
                .unwrap_or("YYYY-MM-DD")
                .to_string();
            Some(TransformStep::DateFormat {
                input_formats,
                output_format,
            })
        }
        "formula" => Some(TransformStep::Formula {
            expression: get_field("expression")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "condition" => {
            let rules = get_field("rules")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|item| {
                            let obj = item.as_object()?;
                            let condition = obj.get("condition")?.as_str()?.to_string();
                            let result = obj.get("result")?.as_str()?.to_string();
                            Some(ConditionRule { condition, result })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(TransformStep::Condition {
                rules,
                default: get_field("default")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            })
        }
        "concat" => Some(TransformStep::Concat {
            template: get_field("template")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            separator: get_field("separator")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        }),
        "split" => Some(TransformStep::Split {
            separator: get_field("separator")
                .and_then(|v| v.as_str())
                .unwrap_or(",")
                .to_string(),
            target_fields: parse_string_array(get_field("target_fields")),
        }),
        "script" => Some(TransformStep::Script {
            expression: get_field("expression")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        "trim" => Some(TransformStep::Trim),
        "case" => {
            let mode = match get_field("mode")
                .and_then(|v| v.as_str())
                .unwrap_or("lower")
                .to_lowercase()
                .as_str()
            {
                "upper" => CaseMode::Upper,
                "title" => CaseMode::Title,
                _ => CaseMode::Lower,
            };
            Some(TransformStep::Case { mode })
        }
        "lookup_table" | "lookup" => Some(TransformStep::LookupTable {
            table_name: get_field("table_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            key_field: get_field("key_field")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            value_field: get_field("value_field")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        }),
        _ => None,
    }
}

fn parse_string_array(value: Option<&serde_json::Value>) -> Vec<String> {
    match value {
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        Some(serde_json::Value::String(s)) => s
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect(),
        _ => Vec::new(),
    }
}

fn json_to_string_map(value: &serde_json::Value) -> Option<HashMap<String, String>> {
    let obj = value.as_object()?;
    let mut out = HashMap::new();
    for (k, v) in obj {
        let mapped = match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => String::new(),
            other => other.to_string(),
        };
        out.insert(k.clone(), mapped);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapping_transform() {
        let mut rows = vec![{
            let mut m = RawRow::new();
            m.insert("level".into(), "VIP".into());
            m
        }];

        let mut field_transforms = HashMap::new();
        field_transforms.insert(
            "level".into(),
            vec![TransformStep::Mapping {
                values: {
                    let mut m = HashMap::new();
                    m.insert("VIP".into(), "A".into());
                    m.insert("普通".into(), "B".into());
                    m
                },
                fallback: Some("C".into()),
            }],
        );

        let config = TransformConfig {
            field_transforms,
            default_values: HashMap::new(),
        };

        let warnings = ValueTransformer::transform(&mut rows, &config);
        assert!(warnings.is_empty());
        assert_eq!(rows[0].get("level").unwrap(), "A");
    }

    #[test]
    fn test_trim_transform() {
        let mut rows = vec![{
            let mut m = RawRow::new();
            m.insert("name".into(), "  Alice  ".into());
            m
        }];

        let mut field_transforms = HashMap::new();
        field_transforms.insert("name".into(), vec![TransformStep::Trim]);

        let config = TransformConfig {
            field_transforms,
            default_values: HashMap::new(),
        };

        ValueTransformer::transform(&mut rows, &config);
        assert_eq!(rows[0].get("name").unwrap(), "Alice");
    }

    #[test]
    fn test_date_format_transform() {
        let mut rows = vec![{
            let mut m = RawRow::new();
            m.insert("date".into(), "2026/03/15".into());
            m
        }];

        let mut field_transforms = HashMap::new();
        field_transforms.insert(
            "date".into(),
            vec![TransformStep::DateFormat {
                input_formats: vec!["YYYY/MM/DD".into()],
                output_format: "YYYY-MM-DD".into(),
            }],
        );

        let config = TransformConfig {
            field_transforms,
            default_values: HashMap::new(),
        };

        ValueTransformer::transform(&mut rows, &config);
        assert_eq!(rows[0].get("date").unwrap(), "2026-03-15");
    }

    #[test]
    fn test_default_values() {
        let mut rows = vec![{
            let mut m = RawRow::new();
            m.insert("name".into(), "Alice".into());
            m.insert("level".into(), "".into());
            m
        }];

        let mut default_values = HashMap::new();
        default_values.insert(
            "level".into(),
            DefaultValueConfig {
                value: "C".into(),
                condition: DefaultCondition::WhenEmpty,
            },
        );
        default_values.insert(
            "group".into(),
            DefaultValueConfig {
                value: "default".into(),
                condition: DefaultCondition::WhenMissing,
            },
        );

        let config = TransformConfig {
            field_transforms: HashMap::new(),
            default_values,
        };

        ValueTransformer::transform(&mut rows, &config);
        assert_eq!(rows[0].get("level").unwrap(), "C");
        assert_eq!(rows[0].get("group").unwrap(), "default");
    }

    #[test]
    fn test_parse_db_value_transform() {
        let json =
            r#"{"customer_level": {"type": "mapping", "values": {"VIP": "A", "普通": "B"}}}"#;
        let result = parse_db_value_transform(json);
        assert!(result.contains_key("customer_level"));
    }

    #[test]
    fn test_parse_db_value_transform_multi_steps() {
        let json = r#"{
            "customer_level": {
                "steps": [
                    {"type":"trim"},
                    {"type":"mapping","values":{"VIP":"A"}}
                ]
            }
        }"#;
        let result = parse_db_value_transform(json);
        let steps = result
            .get("customer_level")
            .expect("未解析到 customer_level");
        assert_eq!(steps.len(), 2);
        assert!(matches!(steps[0], TransformStep::Trim));
        assert!(matches!(steps[1], TransformStep::Mapping { .. }));
    }

    #[test]
    fn test_parse_db_value_transform_extended_types() {
        let json = r#"{
            "grade_bucket": {
                "type":"condition",
                "rules":[{"condition":"value>100","result":"高"}],
                "default":"低"
            },
            "spec_family": {
                "type":"concat",
                "template":"{steel_grade}-{thickness}"
            },
            "parts": {
                "type":"split",
                "separator":"-",
                "target_fields":["left","right"]
            },
            "calc": {
                "type":"script",
                "expression":"ROUND(value * 100, 0)"
            },
            "lookuped": {
                "type":"lookup_table",
                "table_name":"customer_master",
                "key_field":"customer_id",
                "value_field":"customer_level"
            }
        }"#;

        let result = parse_db_value_transform(json);
        assert!(matches!(
            result.get("grade_bucket").and_then(|v| v.first()),
            Some(TransformStep::Condition { .. })
        ));
        assert!(matches!(
            result.get("spec_family").and_then(|v| v.first()),
            Some(TransformStep::Concat { .. })
        ));
        assert!(matches!(
            result.get("parts").and_then(|v| v.first()),
            Some(TransformStep::Split { .. })
        ));
        assert!(matches!(
            result.get("calc").and_then(|v| v.first()),
            Some(TransformStep::Script { .. })
        ));
        assert!(matches!(
            result.get("lookuped").and_then(|v| v.first()),
            Some(TransformStep::LookupTable { .. })
        ));
    }

    #[test]
    fn test_lookup_table_rejects_unsafe_identifier() {
        let mut rows = vec![{
            let mut m = RawRow::new();
            m.insert("customer_id".into(), "C001".into());
            m
        }];

        let mut field_transforms = HashMap::new();
        field_transforms.insert(
            "customer_id".into(),
            vec![TransformStep::LookupTable {
                table_name: "customer_master;drop".into(),
                key_field: "customer_id".into(),
                value_field: "customer_level".into(),
            }],
        );

        let config = TransformConfig {
            field_transforms,
            default_values: HashMap::new(),
        };

        let warnings = ValueTransformer::transform(&mut rows, &config);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].message.contains("非法标识符"));
        assert_eq!(rows[0].get("customer_id").map(String::as_str), Some("C001"));
    }
}
