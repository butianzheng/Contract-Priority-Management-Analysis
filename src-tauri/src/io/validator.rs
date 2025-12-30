use crate::db::schema::{Contract, Customer, ProcessDifficulty};
use super::types::ValidationError;

pub struct Validator;

impl Validator {
    /// 验证合同数据
    pub fn validate_contract(contract: &Contract, row_number: usize) -> Result<(), ValidationError> {
        // 1. 合同编号不能为空
        if contract.contract_id.trim().is_empty() {
            return Err(ValidationError {
                row_number,
                field: "contract_id".to_string(),
                value: contract.contract_id.clone(),
                message: "合同编号不能为空".to_string(),
            });
        }

        // 2. 客户编号不能为空
        if contract.customer_id.trim().is_empty() {
            return Err(ValidationError {
                row_number,
                field: "customer_id".to_string(),
                value: contract.customer_id.clone(),
                message: "客户编号不能为空".to_string(),
            });
        }

        // 3. 厚度必须为正数
        if contract.thickness <= 0.0 {
            return Err(ValidationError {
                row_number,
                field: "thickness".to_string(),
                value: contract.thickness.to_string(),
                message: "厚度必须大于0".to_string(),
            });
        }

        // 4. 宽度必须为正数
        if contract.width <= 0.0 {
            return Err(ValidationError {
                row_number,
                field: "width".to_string(),
                value: contract.width.to_string(),
                message: "宽度必须大于0".to_string(),
            });
        }

        // 5. 验证日期格式 (YYYY-MM-DD)
        if !Self::validate_date_format(&contract.pdd) {
            return Err(ValidationError {
                row_number,
                field: "pdd".to_string(),
                value: contract.pdd.clone(),
                message: "交期格式必须为 YYYY-MM-DD".to_string(),
            });
        }

        Ok(())
    }

    /// 验证客户数据
    pub fn validate_customer(customer: &Customer, row_number: usize) -> Result<(), ValidationError> {
        // 1. 客户编号不能为空
        if customer.customer_id.trim().is_empty() {
            return Err(ValidationError {
                row_number,
                field: "customer_id".to_string(),
                value: customer.customer_id.clone(),
                message: "客户编号不能为空".to_string(),
            });
        }

        // 2. 客户等级必须是 A/B/C
        if !["A", "B", "C"].contains(&customer.customer_level.as_str()) {
            return Err(ValidationError {
                row_number,
                field: "customer_level".to_string(),
                value: customer.customer_level.clone(),
                message: "客户等级必须是 A、B 或 C".to_string(),
            });
        }

        Ok(())
    }

    /// 验证工艺难度配置
    #[allow(dead_code)]
    pub fn validate_process_difficulty(pd: &ProcessDifficulty, row_number: usize) -> Result<(), ValidationError> {
        // 1. 钢种不能为空
        if pd.steel_grade.trim().is_empty() {
            return Err(ValidationError {
                row_number,
                field: "steel_grade".to_string(),
                value: pd.steel_grade.clone(),
                message: "钢种不能为空".to_string(),
            });
        }

        // 2. 厚度范围验证
        if pd.thickness_min < 0.0 || pd.thickness_max < pd.thickness_min {
            return Err(ValidationError {
                row_number,
                field: "thickness_range".to_string(),
                value: format!("{}-{}", pd.thickness_min, pd.thickness_max),
                message: "厚度范围无效".to_string(),
            });
        }

        // 3. 宽度范围验证
        if pd.width_min < 0.0 || pd.width_max < pd.width_min {
            return Err(ValidationError {
                row_number,
                field: "width_range".to_string(),
                value: format!("{}-{}", pd.width_min, pd.width_max),
                message: "宽度范围无效".to_string(),
            });
        }

        // 4. 难度分数验证 (0-100)
        if pd.difficulty_score < 0.0 || pd.difficulty_score > 100.0 {
            return Err(ValidationError {
                row_number,
                field: "difficulty_score".to_string(),
                value: pd.difficulty_score.to_string(),
                message: "难度分数必须在 0-100 之间".to_string(),
            });
        }

        Ok(())
    }

    fn validate_date_format(date_str: &str) -> bool {
        let parts: Vec<&str> = date_str.split('-').collect();
        if parts.len() != 3 {
            return false;
        }

        let year: Result<u32, _> = parts[0].parse();
        let month: Result<u32, _> = parts[1].parse();
        let day: Result<u32, _> = parts[2].parse();

        match (year, month, day) {
            (Ok(y), Ok(m), Ok(d)) => {
                y >= 2000 && y <= 2100 && m >= 1 && m <= 12 && d >= 1 && d <= 31
            }
            _ => false,
        }
    }
}
