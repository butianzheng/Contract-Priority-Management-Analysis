use crate::db::schema::{Contract, Customer, ContractPriority, ProcessDifficulty};
use super::types::ValidationError;
use super::validator::Validator;

pub struct JsonHandler;

impl JsonHandler {
    /// 解析 JSON 为合同数据
    pub fn parse_contracts(content: &[u8]) -> Result<(Vec<Contract>, Vec<ValidationError>), String> {
        let content_str = std::str::from_utf8(content)
            .map_err(|e| format!("UTF-8 解码错误: {}", e))?;

        let contracts: Vec<Contract> = serde_json::from_str(content_str)
            .map_err(|e| format!("JSON 解析错误: {}", e))?;

        let mut valid_contracts = Vec::new();
        let mut errors = Vec::new();

        for (idx, contract) in contracts.into_iter().enumerate() {
            let row_num = idx + 1;
            match Validator::validate_contract(&contract, row_num) {
                Ok(_) => valid_contracts.push(contract),
                Err(e) => errors.push(e),
            }
        }

        Ok((valid_contracts, errors))
    }

    /// 解析 JSON 为客户数据
    pub fn parse_customers(content: &[u8]) -> Result<(Vec<Customer>, Vec<ValidationError>), String> {
        let content_str = std::str::from_utf8(content)
            .map_err(|e| format!("UTF-8 解码错误: {}", e))?;

        let customers: Vec<Customer> = serde_json::from_str(content_str)
            .map_err(|e| format!("JSON 解析错误: {}", e))?;

        let mut valid_customers = Vec::new();
        let mut errors = Vec::new();

        for (idx, customer) in customers.into_iter().enumerate() {
            let row_num = idx + 1;
            match Validator::validate_customer(&customer, row_num) {
                Ok(_) => valid_customers.push(customer),
                Err(e) => errors.push(e),
            }
        }

        Ok((valid_customers, errors))
    }

    /// 解析 JSON 为工艺难度数据
    #[allow(dead_code)]
    pub fn parse_process_difficulty(content: &[u8]) -> Result<(Vec<ProcessDifficulty>, Vec<ValidationError>), String> {
        let content_str = std::str::from_utf8(content)
            .map_err(|e| format!("UTF-8 解码错误: {}", e))?;

        let items: Vec<ProcessDifficulty> = serde_json::from_str(content_str)
            .map_err(|e| format!("JSON 解析错误: {}", e))?;

        let mut valid_items = Vec::new();
        let mut errors = Vec::new();

        for (idx, item) in items.into_iter().enumerate() {
            let row_num = idx + 1;
            match Validator::validate_process_difficulty(&item, row_num) {
                Ok(_) => valid_items.push(item),
                Err(e) => errors.push(e),
            }
        }

        Ok((valid_items, errors))
    }

    /// 生成合同数据 JSON
    pub fn generate_contracts(contracts: &[Contract]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(contracts)
            .map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成客户数据 JSON
    pub fn generate_customers(customers: &[Customer]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(customers)
            .map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成优先级结果 JSON
    pub fn generate_priorities(priorities: &[ContractPriority]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(priorities)
            .map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成工艺难度数据 JSON
    pub fn generate_process_difficulty(items: &[ProcessDifficulty]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(items)
            .map_err(|e| format!("JSON 序列化错误: {}", e))
    }
}
