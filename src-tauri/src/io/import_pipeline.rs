use std::collections::HashMap;

use crate::db::schema::{Contract, Customer, ProcessDifficulty, StrategyWeights};

use super::field_mapper::{FieldMapper, TargetFieldDef};
use super::generic_parser::{GenericParser, RawRow};
use super::row_converter::RowConverter;
use super::types::{ConflictRecord, FileFormat, ImportDataType, ImportPreview, ValidationError};
use super::value_transformer::{parse_transform_config, TransformConfig, ValueTransformer};
use super::ConflictHandler;

#[derive(Debug, Clone)]
pub enum PreparedRecords {
    Contracts(Vec<Contract>),
    Customers(Vec<Customer>),
    ProcessDifficulty(Vec<ProcessDifficulty>),
    StrategyWeights(Vec<StrategyWeights>),
}

#[derive(Debug, Clone)]
pub struct PreparedImportData {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub validation_errors: Vec<ValidationError>,
    pub conflicts: Vec<ConflictRecord>,
    pub sample_data: Vec<serde_json::Value>,
    pub records: PreparedRecords,
}

pub struct ImportPipeline;

impl ImportPipeline {
    pub fn preview(
        content: &[u8],
        format: FileFormat,
        data_type: ImportDataType,
        field_mapping: Option<HashMap<String, String>>,
        value_transforms: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<ImportPreview, String> {
        let prepared = Self::prepare(content, format, data_type, field_mapping, value_transforms)?;

        Ok(ImportPreview {
            total_rows: prepared.total_rows,
            valid_rows: prepared.valid_rows,
            error_rows: prepared.validation_errors.len(),
            conflicts: prepared.conflicts,
            validation_errors: prepared.validation_errors,
            sample_data: prepared.sample_data,
        })
    }

    pub fn prepare(
        content: &[u8],
        format: FileFormat,
        data_type: ImportDataType,
        field_mapping: Option<HashMap<String, String>>,
        value_transforms: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<PreparedImportData, String> {
        let parsed = GenericParser::parse(content, format)?;
        let mut all_errors = parsed.parse_errors;

        let target_fields = FieldMapper::get_target_fields(data_type);
        let mapping =
            Self::resolve_mapping(data_type, &parsed.headers, &target_fields, field_mapping);

        let missing_required = Self::find_missing_required_targets(&target_fields, &mapping);
        if !missing_required.is_empty() {
            for field_name in missing_required {
                all_errors.push(ValidationError {
                    row_number: 1,
                    field: field_name,
                    value: String::new(),
                    message: "必填字段未映射".to_string(),
                });
            }
        }

        let mut mapped_rows = FieldMapper::apply_mapping(&parsed.rows, &mapping);

        if let Some(config) = Self::resolve_transform_config(data_type, value_transforms)? {
            let warnings = ValueTransformer::transform(&mut mapped_rows, &config);
            all_errors.extend(warnings.into_iter().map(|w| ValidationError {
                row_number: w.row_number,
                field: w.field,
                value: String::new(),
                message: w.message,
            }));
        }

        let prepared = match data_type {
            ImportDataType::Contracts => {
                let (contracts, convert_errors) = RowConverter::to_contracts(&mapped_rows);
                let sample_data: Vec<serde_json::Value> = contracts
                    .iter()
                    .take(5)
                    .map(|c| serde_json::to_value(c).unwrap_or_default())
                    .collect();
                let conflicts = ConflictHandler::detect_contract_conflicts(&contracts)?;
                all_errors.extend(convert_errors);
                let total_rows = contracts.len() + all_errors.len();

                PreparedImportData {
                    total_rows,
                    valid_rows: contracts.len(),
                    validation_errors: all_errors,
                    conflicts,
                    sample_data,
                    records: PreparedRecords::Contracts(contracts),
                }
            }
            ImportDataType::Customers => {
                let (customers, convert_errors) = RowConverter::to_customers(&mapped_rows);
                let sample_data: Vec<serde_json::Value> = customers
                    .iter()
                    .take(5)
                    .map(|c| serde_json::to_value(c).unwrap_or_default())
                    .collect();
                let conflicts = ConflictHandler::detect_customer_conflicts(&customers)?;
                all_errors.extend(convert_errors);
                let total_rows = customers.len() + all_errors.len();

                PreparedImportData {
                    total_rows,
                    valid_rows: customers.len(),
                    validation_errors: all_errors,
                    conflicts,
                    sample_data,
                    records: PreparedRecords::Customers(customers),
                }
            }
            ImportDataType::ProcessDifficulty => {
                let (items, convert_errors) = RowConverter::to_process_difficulty(&mapped_rows);
                let sample_data: Vec<serde_json::Value> = items
                    .iter()
                    .take(5)
                    .map(|c| serde_json::to_value(c).unwrap_or_default())
                    .collect();
                let conflicts = ConflictHandler::detect_process_difficulty_conflicts(&items)?;
                all_errors.extend(convert_errors);
                let total_rows = items.len() + all_errors.len();

                PreparedImportData {
                    total_rows,
                    valid_rows: items.len(),
                    validation_errors: all_errors,
                    conflicts,
                    sample_data,
                    records: PreparedRecords::ProcessDifficulty(items),
                }
            }
            ImportDataType::StrategyWeights => {
                let (items, convert_errors) = RowConverter::to_strategy_weights(&mapped_rows);
                let sample_data: Vec<serde_json::Value> = items
                    .iter()
                    .take(5)
                    .map(|c| serde_json::to_value(c).unwrap_or_default())
                    .collect();
                let conflicts = ConflictHandler::detect_strategy_weight_conflicts(&items)?;
                all_errors.extend(convert_errors);
                let total_rows = items.len() + all_errors.len();

                PreparedImportData {
                    total_rows,
                    valid_rows: items.len(),
                    validation_errors: all_errors,
                    conflicts,
                    sample_data,
                    records: PreparedRecords::StrategyWeights(items),
                }
            }
        };

        Ok(prepared)
    }

    pub fn data_type_key(data_type: ImportDataType) -> &'static str {
        match data_type {
            ImportDataType::Contracts => "contracts",
            ImportDataType::Customers => "customers",
            ImportDataType::ProcessDifficulty => "process_difficulty",
            ImportDataType::StrategyWeights => "strategy_weights",
        }
    }

    fn resolve_mapping(
        data_type: ImportDataType,
        source_headers: &[String],
        target_fields: &[TargetFieldDef],
        user_mapping: Option<HashMap<String, String>>,
    ) -> HashMap<String, String> {
        if let Some(mapping) = user_mapping {
            return mapping;
        }

        let rules = FieldMapper::load_rules(Self::data_type_key(data_type)).unwrap_or_default();
        let auto_result = FieldMapper::auto_match(source_headers, target_fields, &rules);
        auto_result.mappings
    }

    fn find_missing_required_targets(
        target_fields: &[TargetFieldDef],
        mapping: &HashMap<String, String>,
    ) -> Vec<String> {
        target_fields
            .iter()
            .filter(|field| field.required && !mapping.contains_key(&field.name))
            .map(|field| field.name.clone())
            .collect()
    }

    fn resolve_transform_config(
        data_type: ImportDataType,
        value_transforms: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<Option<TransformConfig>, String> {
        if let Some(custom_config) = value_transforms {
            let value = serde_json::Value::Object(custom_config.into_iter().collect());

            if value.get("field_transforms").is_some() || value.get("default_values").is_some() {
                let config: TransformConfig = serde_json::from_value(value)
                    .map_err(|e| format!("转换配置格式错误: {}", e))?;
                return Ok(Some(config));
            }

            let transform_json =
                serde_json::to_string(&value).map_err(|e| format!("序列化转换配置失败: {}", e))?;
            return Ok(Some(parse_transform_config(Some(&transform_json), None)));
        }

        let rules = FieldMapper::load_rules(Self::data_type_key(data_type)).unwrap_or_default();
        if rules.is_empty() {
            return Ok(None);
        }

        let mut merged = TransformConfig {
            field_transforms: HashMap::new(),
            default_values: HashMap::new(),
        };

        for rule in rules {
            let parsed = parse_transform_config(
                rule.value_transform.as_deref(),
                rule.default_values.as_deref(),
            );

            for (field, steps) in parsed.field_transforms {
                merged
                    .field_transforms
                    .entry(field)
                    .or_default()
                    .extend(steps);
            }
            for (field, config) in parsed.default_values {
                merged.default_values.entry(field).or_insert(config);
            }
        }

        if merged.field_transforms.is_empty() && merged.default_values.is_empty() {
            Ok(None)
        } else {
            Ok(Some(merged))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::value_transformer::TransformStep;

    #[test]
    fn test_missing_required_targets() {
        let targets = FieldMapper::get_target_fields(ImportDataType::Customers);
        let mut mapping = HashMap::new();
        mapping.insert("customer_id".to_string(), "客户号".to_string());
        let missing = ImportPipeline::find_missing_required_targets(&targets, &mapping);
        assert!(missing.contains(&"customer_level".to_string()));
    }

    #[test]
    fn test_data_type_key() {
        assert_eq!(
            ImportPipeline::data_type_key(ImportDataType::Contracts),
            "contracts"
        );
        assert_eq!(
            ImportPipeline::data_type_key(ImportDataType::Customers),
            "customers"
        );
    }

    #[test]
    fn test_resolve_transform_config_custom() {
        let mut transform = HashMap::new();
        transform.insert(
            "customer_level".to_string(),
            serde_json::json!({"type":"mapping","values":{"VIP":"A"}}),
        );

        let config =
            ImportPipeline::resolve_transform_config(ImportDataType::Customers, Some(transform))
                .unwrap()
                .unwrap();

        assert!(config.field_transforms.contains_key("customer_level"));
    }

    #[test]
    fn test_resolve_transform_config_structured_payload() {
        let mut transform = HashMap::new();
        transform.insert(
            "field_transforms".to_string(),
            serde_json::json!({
                "customer_level": [
                    {"type":"trim"},
                    {"type":"mapping","values":{"VIP":"A"}}
                ]
            }),
        );
        transform.insert(
            "default_values".to_string(),
            serde_json::json!({
                "customer_group": {"value":"默认组","condition":"when_missing"}
            }),
        );

        let config =
            ImportPipeline::resolve_transform_config(ImportDataType::Customers, Some(transform))
                .unwrap()
                .unwrap();

        let steps = config
            .field_transforms
            .get("customer_level")
            .expect("缺少 customer_level 转换");
        assert!(matches!(steps.first(), Some(TransformStep::Trim)));
        assert!(config.default_values.contains_key("customer_group"));
    }
}
