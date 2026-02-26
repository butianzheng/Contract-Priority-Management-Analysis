use crate::db::schema::{Contract, ContractPriority, Customer, ProcessDifficulty};

pub struct JsonHandler;

impl JsonHandler {
    /// 生成合同数据 JSON
    pub fn generate_contracts(contracts: &[Contract]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(contracts).map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成客户数据 JSON
    pub fn generate_customers(customers: &[Customer]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(customers).map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成优先级结果 JSON
    pub fn generate_priorities(priorities: &[ContractPriority]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(priorities).map_err(|e| format!("JSON 序列化错误: {}", e))
    }

    /// 生成工艺难度数据 JSON
    pub fn generate_process_difficulty(items: &[ProcessDifficulty]) -> Result<Vec<u8>, String> {
        serde_json::to_vec_pretty(items).map_err(|e| format!("JSON 序列化错误: {}", e))
    }
}
