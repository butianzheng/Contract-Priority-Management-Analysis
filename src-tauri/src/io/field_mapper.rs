use super::generic_parser::RawRow;
use super::types::ImportDataType;
use crate::db;
use crate::db::schema::FieldAlignmentRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 字段类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FieldType {
    String,
    Float,
    Integer,
    Date,
}

/// 目标字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetFieldDef {
    pub name: String,
    pub display_name: String,
    pub required: bool,
    pub field_type: FieldType,
    pub default_value: Option<String>,
}

/// 字段映射结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// target_field → source_column
    pub mappings: HashMap<String, String>,
    /// 未映射的目标字段
    pub unmapped_targets: Vec<String>,
    /// 未映射的源列
    pub unmapped_sources: Vec<String>,
    /// 每个映射的置信度 (0.0 - 1.0)
    pub confidence: HashMap<String, f64>,
}

pub struct FieldMapper;

impl FieldMapper {
    /// 获取目标数据类型的字段定义
    pub fn get_target_fields(data_type: ImportDataType) -> Vec<TargetFieldDef> {
        match data_type {
            ImportDataType::Contracts => vec![
                TargetFieldDef {
                    name: "contract_id".into(),
                    display_name: "合同编号".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "customer_id".into(),
                    display_name: "客户编号".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "steel_grade".into(),
                    display_name: "钢种".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "thickness".into(),
                    display_name: "厚度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "width".into(),
                    display_name: "宽度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "spec_family".into(),
                    display_name: "规格族".into(),
                    required: false,
                    field_type: FieldType::String,
                    default_value: Some("常规".into()),
                },
                TargetFieldDef {
                    name: "pdd".into(),
                    display_name: "交期".into(),
                    required: true,
                    field_type: FieldType::Date,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "days_to_pdd".into(),
                    display_name: "剩余天数".into(),
                    required: false,
                    field_type: FieldType::Integer,
                    default_value: Some("0".into()),
                },
                TargetFieldDef {
                    name: "margin".into(),
                    display_name: "毛利".into(),
                    required: false,
                    field_type: FieldType::Float,
                    default_value: Some("0".into()),
                },
            ],
            ImportDataType::Customers => vec![
                TargetFieldDef {
                    name: "customer_id".into(),
                    display_name: "客户编号".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "customer_name".into(),
                    display_name: "客户名称".into(),
                    required: false,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "customer_level".into(),
                    display_name: "客户等级".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "credit_level".into(),
                    display_name: "信用等级".into(),
                    required: false,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "customer_group".into(),
                    display_name: "客户分组".into(),
                    required: false,
                    field_type: FieldType::String,
                    default_value: None,
                },
            ],
            ImportDataType::ProcessDifficulty => vec![
                TargetFieldDef {
                    name: "id".into(),
                    display_name: "ID".into(),
                    required: false,
                    field_type: FieldType::Integer,
                    default_value: Some("0".into()),
                },
                TargetFieldDef {
                    name: "steel_grade".into(),
                    display_name: "钢种".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "thickness_min".into(),
                    display_name: "最小厚度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "thickness_max".into(),
                    display_name: "最大厚度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "width_min".into(),
                    display_name: "最小宽度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "width_max".into(),
                    display_name: "最大宽度".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "difficulty_level".into(),
                    display_name: "难度等级".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "difficulty_score".into(),
                    display_name: "难度分数".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
            ],
            ImportDataType::StrategyWeights => vec![
                TargetFieldDef {
                    name: "strategy_name".into(),
                    display_name: "策略名称".into(),
                    required: true,
                    field_type: FieldType::String,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "ws".into(),
                    display_name: "S权重".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "wp".into(),
                    display_name: "P权重".into(),
                    required: true,
                    field_type: FieldType::Float,
                    default_value: None,
                },
                TargetFieldDef {
                    name: "description".into(),
                    display_name: "描述".into(),
                    required: false,
                    field_type: FieldType::String,
                    default_value: None,
                },
            ],
        }
    }

    /// 从 DB 加载字段对齐规则
    pub fn load_rules(data_type: &str) -> Result<Vec<FieldAlignmentRule>, String> {
        db::list_field_alignment_rules(Some(data_type), false)
    }

    /// 自动推断映射关系
    /// 匹配策略优先级：精确匹配 → 别名匹配（field_alignment_rule） → 模糊匹配（编辑距离）
    pub fn auto_match(
        source_headers: &[String],
        target_fields: &[TargetFieldDef],
        rules: &[FieldAlignmentRule],
    ) -> FieldMappingResult {
        let mut mappings = HashMap::new();
        let mut confidence = HashMap::new();
        let mut used_sources: std::collections::HashSet<String> = std::collections::HashSet::new();

        // 构建别名映射表：target_field → Vec<alias>
        let alias_map = Self::build_alias_map(rules);

        for target in target_fields {
            let mut best_match: Option<(String, f64)> = None;

            // 1. 精确匹配（源列名 == 目标字段名，不区分大小写）
            for source in source_headers {
                if used_sources.contains(source) {
                    continue;
                }
                if source.to_lowercase() == target.name.to_lowercase() {
                    best_match = Some((source.clone(), 1.0));
                    break;
                }
            }

            // 2. 别名匹配（从 field_alignment_rule 加载的别名列表）
            if best_match.is_none() {
                if let Some(aliases) = alias_map.get(&target.name) {
                    for source in source_headers {
                        if used_sources.contains(source) {
                            continue;
                        }
                        let source_lower = source.to_lowercase().trim().to_string();
                        for alias in aliases {
                            if alias.to_lowercase().trim() == source_lower {
                                best_match = Some((source.clone(), 0.95));
                                break;
                            }
                        }
                        if best_match.is_some() {
                            break;
                        }
                    }
                }
            }

            // 3. 中文显示名匹配
            if best_match.is_none() {
                for source in source_headers {
                    if used_sources.contains(source) {
                        continue;
                    }
                    if source.trim() == target.display_name {
                        best_match = Some((source.clone(), 0.90));
                        break;
                    }
                }
            }

            // 4. 模糊匹配（编辑距离）
            if best_match.is_none() {
                let mut best_score = 0.0f64;
                let mut best_source = None;

                for source in source_headers {
                    if used_sources.contains(source) {
                        continue;
                    }
                    // 计算与目标字段名的相似度
                    let sim1 =
                        Self::similarity(&source.to_lowercase(), &target.name.to_lowercase());
                    // 计算与中文显示名的相似度
                    let sim2 = Self::similarity(source, &target.display_name);
                    // 检查包含关系
                    let sim3 = Self::containment_score(source, &target.name, &target.display_name);

                    let sim = sim1.max(sim2).max(sim3);
                    if sim > best_score && sim >= 0.5 {
                        best_score = sim;
                        best_source = Some(source.clone());
                    }
                }

                if let Some(source) = best_source {
                    best_match = Some((source, best_score * 0.8)); // 模糊匹配降低置信度
                }
            }

            if let Some((source, conf)) = best_match {
                used_sources.insert(source.clone());
                mappings.insert(target.name.clone(), source);
                confidence.insert(target.name.clone(), conf);
            }
        }

        let unmapped_targets: Vec<String> = target_fields
            .iter()
            .filter(|t| !mappings.contains_key(&t.name))
            .map(|t| t.name.clone())
            .collect();

        let unmapped_sources: Vec<String> = source_headers
            .iter()
            .filter(|s| !used_sources.contains(*s))
            .cloned()
            .collect();

        FieldMappingResult {
            mappings,
            unmapped_targets,
            unmapped_sources,
            confidence,
        }
    }

    /// 应用映射：将通用行的 key 从源列名转换为目标字段名
    pub fn apply_mapping(rows: &[RawRow], mapping: &HashMap<String, String>) -> Vec<RawRow> {
        // 反转映射：source_column → target_field
        let reverse_map: HashMap<&str, &str> = mapping
            .iter()
            .map(|(target, source)| (source.as_str(), target.as_str()))
            .collect();

        rows.iter()
            .map(|row| {
                let mut new_row = RawRow::new();
                for (key, value) in row {
                    if let Some(&target) = reverse_map.get(key.as_str()) {
                        new_row.insert(target.to_string(), value.clone());
                    }
                    // 也保留原始 key（以防转换规则需要引用原始字段名）
                    // 但目标字段名优先
                    if !reverse_map.contains_key(key.as_str()) {
                        new_row.insert(key.clone(), value.clone());
                    }
                }
                new_row
            })
            .collect()
    }

    /// 从 field_alignment_rule 构建别名映射表
    fn build_alias_map(rules: &[FieldAlignmentRule]) -> HashMap<String, Vec<String>> {
        let mut alias_map: HashMap<String, Vec<String>> = HashMap::new();

        for rule in rules {
            if rule.enabled != 1 {
                continue;
            }
            if let Ok(mapping) =
                serde_json::from_str::<HashMap<String, Vec<String>>>(&rule.field_mapping)
            {
                for (target, aliases) in mapping {
                    alias_map.entry(target).or_default().extend(aliases);
                }
            }
        }

        alias_map
    }

    /// 计算两个字符串的相似度 (0.0 - 1.0)，基于 Levenshtein 距离
    fn similarity(a: &str, b: &str) -> f64 {
        if a == b {
            return 1.0;
        }
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let len_a = a_chars.len();
        let len_b = b_chars.len();

        let mut matrix = vec![vec![0usize; len_b + 1]; len_a + 1];

        for i in 0..=len_a {
            matrix[i][0] = i;
        }
        for j in 0..=len_b {
            matrix[0][j] = j;
        }

        for i in 1..=len_a {
            for j in 1..=len_b {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        let max_len = len_a.max(len_b);
        1.0 - (matrix[len_a][len_b] as f64 / max_len as f64)
    }

    /// 包含关系评分：如果源列名包含目标字段名或显示名的一部分
    fn containment_score(source: &str, target_name: &str, display_name: &str) -> f64 {
        let source_lower = source.to_lowercase();
        let target_lower = target_name.to_lowercase();

        // 源包含目标
        if source_lower.contains(&target_lower) || target_lower.contains(&source_lower) {
            return 0.7;
        }

        // 源包含中文显示名
        if source.contains(display_name) || display_name.contains(source) {
            return 0.7;
        }

        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let sources = vec![
            "contract_id".into(),
            "customer_id".into(),
            "steel_grade".into(),
        ];
        let targets = FieldMapper::get_target_fields(ImportDataType::Contracts);
        let result = FieldMapper::auto_match(&sources, &targets, &[]);

        assert_eq!(result.mappings.get("contract_id").unwrap(), "contract_id");
        assert_eq!(*result.confidence.get("contract_id").unwrap(), 1.0);
    }

    #[test]
    fn test_alias_match() {
        let sources = vec!["合同号".into(), "客户号".into(), "钢种".into()];
        let targets = FieldMapper::get_target_fields(ImportDataType::Contracts);
        let rules = vec![FieldAlignmentRule {
            rule_id: Some(1),
            rule_name: "test".into(),
            data_type: "contracts".into(),
            source_type: None,
            description: None,
            enabled: 1,
            priority: 1,
            field_mapping:
                r#"{"contract_id":["合同号"],"customer_id":["客户号"],"steel_grade":["钢种"]}"#
                    .into(),
            value_transform: None,
            default_values: None,
            created_by: "test".into(),
            created_at: None,
            updated_at: None,
        }];

        let result = FieldMapper::auto_match(&sources, &targets, &rules);
        assert_eq!(result.mappings.get("contract_id").unwrap(), "合同号");
        assert!(*result.confidence.get("contract_id").unwrap() >= 0.9);
    }

    #[test]
    fn test_display_name_match() {
        let sources = vec!["合同编号".into(), "客户编号".into(), "钢种".into()];
        let targets = FieldMapper::get_target_fields(ImportDataType::Contracts);
        let result = FieldMapper::auto_match(&sources, &targets, &[]);

        assert_eq!(result.mappings.get("contract_id").unwrap(), "合同编号");
    }

    #[test]
    fn test_apply_mapping() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("合同号".into(), "C001".into());
            m.insert("客户号".into(), "K001".into());
            m
        }];
        let mut mapping = HashMap::new();
        mapping.insert("contract_id".into(), "合同号".into());
        mapping.insert("customer_id".into(), "客户号".into());

        let result = FieldMapper::apply_mapping(&rows, &mapping);
        assert_eq!(result[0].get("contract_id").unwrap(), "C001");
        assert_eq!(result[0].get("customer_id").unwrap(), "K001");
    }
}
