use super::generic_parser::RawRow;
use super::types::ValidationError;
use super::validator::Validator;
use crate::db::schema::{Contract, Customer, ProcessDifficulty, StrategyWeights};

pub struct RowConverter;

impl RowConverter {
    /// 通用行 → Contract（含校验）
    pub fn to_contracts(rows: &[RawRow]) -> (Vec<Contract>, Vec<ValidationError>) {
        let mut contracts = Vec::new();
        let mut errors = Vec::new();

        for (idx, row) in rows.iter().enumerate() {
            let row_num = idx + 2; // +1 for 0-index, +1 for header
            match Self::row_to_contract(row, row_num) {
                Ok(contract) => match Validator::validate_contract(&contract, row_num) {
                    Ok(_) => contracts.push(contract),
                    Err(e) => errors.push(e),
                },
                Err(e) => errors.push(e),
            }
        }

        (contracts, errors)
    }

    /// 通用行 → Customer（含校验）
    pub fn to_customers(rows: &[RawRow]) -> (Vec<Customer>, Vec<ValidationError>) {
        let mut customers = Vec::new();
        let mut errors = Vec::new();

        for (idx, row) in rows.iter().enumerate() {
            let row_num = idx + 2;
            match Self::row_to_customer(row, row_num) {
                Ok(customer) => match Validator::validate_customer(&customer, row_num) {
                    Ok(_) => customers.push(customer),
                    Err(e) => errors.push(e),
                },
                Err(e) => errors.push(e),
            }
        }

        (customers, errors)
    }

    /// 通用行 → ProcessDifficulty（含校验）
    pub fn to_process_difficulty(
        rows: &[RawRow],
    ) -> (Vec<ProcessDifficulty>, Vec<ValidationError>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for (idx, row) in rows.iter().enumerate() {
            let row_num = idx + 2;
            match Self::row_to_process_difficulty(row, row_num) {
                Ok(item) => match Validator::validate_process_difficulty(&item, row_num) {
                    Ok(_) => items.push(item),
                    Err(e) => errors.push(e),
                },
                Err(e) => errors.push(e),
            }
        }

        (items, errors)
    }

    /// 通用行 → StrategyWeights（含校验）
    pub fn to_strategy_weights(rows: &[RawRow]) -> (Vec<StrategyWeights>, Vec<ValidationError>) {
        let mut items = Vec::new();
        let mut errors = Vec::new();

        for (idx, row) in rows.iter().enumerate() {
            let row_num = idx + 2;
            match Self::row_to_strategy_weight(row, row_num) {
                Ok(item) => items.push(item),
                Err(e) => errors.push(e),
            }
        }

        (items, errors)
    }

    // === 内部转换函数 ===

    fn row_to_contract(row: &RawRow, row_num: usize) -> Result<Contract, ValidationError> {
        let contract_id = Self::get_string(row, "contract_id", row_num)?;
        let customer_id = Self::get_string(row, "customer_id", row_num)?;
        let steel_grade = Self::get_string(row, "steel_grade", row_num)?;
        let thickness = Self::get_float(row, "thickness", row_num)?;
        let width = Self::get_float(row, "width", row_num)?;
        let spec_family = Self::get_string_or_default(row, "spec_family", "常规");
        let pdd = Self::get_string(row, "pdd", row_num)?;
        let days_to_pdd = Self::get_int_or_default(row, "days_to_pdd", 0);
        let margin = Self::get_float_or_default(row, "margin", 0.0);

        Ok(Contract {
            contract_id,
            customer_id,
            steel_grade,
            thickness,
            width,
            spec_family,
            pdd,
            days_to_pdd,
            margin,
        })
    }

    fn row_to_customer(row: &RawRow, row_num: usize) -> Result<Customer, ValidationError> {
        let customer_id = Self::get_string(row, "customer_id", row_num)?;
        let customer_name = Self::get_optional_string(row, "customer_name");
        let customer_level = Self::get_string(row, "customer_level", row_num)?;
        let credit_level = Self::get_optional_string(row, "credit_level");
        let customer_group = Self::get_optional_string(row, "customer_group");

        Ok(Customer {
            customer_id,
            customer_name,
            customer_level,
            credit_level,
            customer_group,
        })
    }

    fn row_to_process_difficulty(
        row: &RawRow,
        row_num: usize,
    ) -> Result<ProcessDifficulty, ValidationError> {
        let id = Self::get_int_or_default(row, "id", 0);
        let steel_grade = Self::get_string(row, "steel_grade", row_num)?;
        let thickness_min = Self::get_float(row, "thickness_min", row_num)?;
        let thickness_max = Self::get_float(row, "thickness_max", row_num)?;
        let width_min = Self::get_float(row, "width_min", row_num)?;
        let width_max = Self::get_float(row, "width_max", row_num)?;
        let difficulty_level = Self::get_string(row, "difficulty_level", row_num)?;
        let difficulty_score = Self::get_float(row, "difficulty_score", row_num)?;

        Ok(ProcessDifficulty {
            id,
            steel_grade,
            thickness_min,
            thickness_max,
            width_min,
            width_max,
            difficulty_level,
            difficulty_score,
        })
    }

    fn row_to_strategy_weight(
        row: &RawRow,
        row_num: usize,
    ) -> Result<StrategyWeights, ValidationError> {
        let strategy_name = Self::get_string(row, "strategy_name", row_num)?;
        let ws = Self::get_float(row, "ws", row_num)?;
        let wp = Self::get_float(row, "wp", row_num)?;
        let description = Self::get_optional_string(row, "description");

        if ws < 0.0 {
            return Err(ValidationError {
                row_number: row_num,
                field: "ws".to_string(),
                value: ws.to_string(),
                message: "ws 不能小于 0".to_string(),
            });
        }

        if wp < 0.0 {
            return Err(ValidationError {
                row_number: row_num,
                field: "wp".to_string(),
                value: wp.to_string(),
                message: "wp 不能小于 0".to_string(),
            });
        }

        if ws == 0.0 && wp == 0.0 {
            return Err(ValidationError {
                row_number: row_num,
                field: "ws/wp".to_string(),
                value: format!("{}/{}", ws, wp),
                message: "ws 与 wp 不能同时为 0".to_string(),
            });
        }

        Ok(StrategyWeights {
            strategy_name,
            ws,
            wp,
            description,
        })
    }

    // === 辅助方法 ===

    fn get_string(row: &RawRow, field: &str, row_num: usize) -> Result<String, ValidationError> {
        let value = row
            .get(field)
            .map(|v| v.trim().to_string())
            .unwrap_or_default();

        if value.is_empty() {
            return Err(ValidationError {
                row_number: row_num,
                field: field.to_string(),
                value: value.clone(),
                message: format!("{}不能为空", field),
            });
        }

        Ok(value)
    }

    fn get_string_or_default(row: &RawRow, field: &str, default: &str) -> String {
        row.get(field)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| default.to_string())
    }

    fn get_optional_string(row: &RawRow, field: &str) -> Option<String> {
        row.get(field)
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    }

    fn get_float(row: &RawRow, field: &str, row_num: usize) -> Result<f64, ValidationError> {
        let value = row
            .get(field)
            .map(|v| v.trim().to_string())
            .unwrap_or_default();

        if value.is_empty() {
            return Err(ValidationError {
                row_number: row_num,
                field: field.to_string(),
                value: value.clone(),
                message: format!("{}不能为空", field),
            });
        }

        value.parse::<f64>().map_err(|_| ValidationError {
            row_number: row_num,
            field: field.to_string(),
            value: value.clone(),
            message: format!("{}必须是数字，当前值: {}", field, value),
        })
    }

    fn get_float_or_default(row: &RawRow, field: &str, default: f64) -> f64 {
        row.get(field)
            .and_then(|v| v.trim().parse::<f64>().ok())
            .unwrap_or(default)
    }

    fn get_int_or_default(row: &RawRow, field: &str, default: i64) -> i64 {
        row.get(field)
            .and_then(|v| {
                let trimmed = v.trim();
                // 尝试直接解析整数
                trimmed
                    .parse::<i64>()
                    .ok()
                    // 如果失败，尝试解析为浮点数再转整数
                    .or_else(|| trimmed.parse::<f64>().ok().map(|f| f as i64))
            })
            .unwrap_or(default)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_contracts() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("contract_id".into(), "C001".into());
            m.insert("customer_id".into(), "K001".into());
            m.insert("steel_grade".into(), "Q235".into());
            m.insert("thickness".into(), "10.5".into());
            m.insert("width".into(), "1500".into());
            m.insert("spec_family".into(), "常规".into());
            m.insert("pdd".into(), "2026-03-15".into());
            m.insert("days_to_pdd".into(), "16".into());
            m.insert("margin".into(), "0.15".into());
            m
        }];

        let (contracts, errors) = RowConverter::to_contracts(&rows);
        assert_eq!(contracts.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(contracts[0].contract_id, "C001");
        assert_eq!(contracts[0].thickness, 10.5);
    }

    #[test]
    fn test_to_customers() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("customer_id".into(), "K001".into());
            m.insert("customer_name".into(), "测试客户".into());
            m.insert("customer_level".into(), "A".into());
            m
        }];

        let (customers, errors) = RowConverter::to_customers(&rows);
        assert_eq!(customers.len(), 1);
        assert_eq!(errors.len(), 0);
        assert_eq!(customers[0].customer_id, "K001");
        assert_eq!(customers[0].customer_name, Some("测试客户".to_string()));
    }

    #[test]
    fn test_missing_required_field() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("customer_id".into(), "".into()); // empty required field
            m.insert("customer_level".into(), "A".into());
            m
        }];

        let (customers, errors) = RowConverter::to_customers(&rows);
        assert_eq!(customers.len(), 0);
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_to_strategy_weights() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("strategy_name".into(), "平衡".into());
            m.insert("ws".into(), "0.6".into());
            m.insert("wp".into(), "0.4".into());
            m.insert("description".into(), "测试策略".into());
            m
        }];

        let (items, errors) = RowConverter::to_strategy_weights(&rows);
        assert_eq!(items.len(), 1);
        assert!(errors.is_empty());
        assert_eq!(items[0].strategy_name, "平衡");
        assert_eq!(items[0].ws, 0.6);
        assert_eq!(items[0].wp, 0.4);
    }

    #[test]
    fn test_strategy_weights_invalid_zero_sum() {
        let rows = vec![{
            let mut m = RawRow::new();
            m.insert("strategy_name".into(), "无效策略".into());
            m.insert("ws".into(), "0".into());
            m.insert("wp".into(), "0".into());
            m
        }];

        let (items, errors) = RowConverter::to_strategy_weights(&rows);
        assert!(items.is_empty());
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "ws/wp");
    }
}
