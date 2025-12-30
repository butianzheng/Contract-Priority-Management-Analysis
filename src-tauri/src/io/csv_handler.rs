use csv::{ReaderBuilder, WriterBuilder};
use std::io::Cursor;
use crate::db::schema::{Contract, Customer, ContractPriority, ProcessDifficulty};
use super::types::ValidationError;
use super::validator::Validator;

pub struct CsvHandler;

impl CsvHandler {
    /// 解析 CSV 为合同数据
    pub fn parse_contracts(content: &[u8]) -> Result<(Vec<Contract>, Vec<ValidationError>), String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(content));

        let mut contracts = Vec::new();
        let mut errors = Vec::new();

        for (idx, result) in reader.deserialize().enumerate() {
            let row_num = idx + 2; // +1 for 0-index, +1 for header
            match result {
                Ok(contract) => {
                    let contract: Contract = contract;
                    match Validator::validate_contract(&contract, row_num) {
                        Ok(_) => contracts.push(contract),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => {
                    errors.push(ValidationError {
                        row_number: row_num,
                        field: "row".to_string(),
                        value: "".to_string(),
                        message: format!("解析错误: {}", e),
                    });
                }
            }
        }

        Ok((contracts, errors))
    }

    /// 解析 CSV 为客户数据
    pub fn parse_customers(content: &[u8]) -> Result<(Vec<Customer>, Vec<ValidationError>), String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(content));

        let mut customers = Vec::new();
        let mut errors = Vec::new();

        for (idx, result) in reader.deserialize().enumerate() {
            let row_num = idx + 2;
            match result {
                Ok(customer) => {
                    let customer: Customer = customer;
                    match Validator::validate_customer(&customer, row_num) {
                        Ok(_) => customers.push(customer),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => {
                    errors.push(ValidationError {
                        row_number: row_num,
                        field: "row".to_string(),
                        value: "".to_string(),
                        message: format!("解析错误: {}", e),
                    });
                }
            }
        }

        Ok((customers, errors))
    }

    /// 解析 CSV 为工艺难度数据
    #[allow(dead_code)]
    pub fn parse_process_difficulty(content: &[u8]) -> Result<(Vec<ProcessDifficulty>, Vec<ValidationError>), String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(content));

        let mut items = Vec::new();
        let mut errors = Vec::new();

        for (idx, result) in reader.deserialize().enumerate() {
            let row_num = idx + 2;
            match result {
                Ok(item) => {
                    let item: ProcessDifficulty = item;
                    match Validator::validate_process_difficulty(&item, row_num) {
                        Ok(_) => items.push(item),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => {
                    errors.push(ValidationError {
                        row_number: row_num,
                        field: "row".to_string(),
                        value: "".to_string(),
                        message: format!("解析错误: {}", e),
                    });
                }
            }
        }

        Ok((items, errors))
    }

    /// 生成合同数据 CSV
    pub fn generate_contracts(contracts: &[Contract]) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);

        for contract in contracts {
            writer.serialize(contract).map_err(|e| e.to_string())?;
        }

        writer.into_inner().map_err(|e| e.to_string())
    }

    /// 生成客户数据 CSV
    pub fn generate_customers(customers: &[Customer]) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);

        for customer in customers {
            writer.serialize(customer).map_err(|e| e.to_string())?;
        }

        writer.into_inner().map_err(|e| e.to_string())
    }

    /// 生成优先级结果 CSV
    pub fn generate_priorities(priorities: &[ContractPriority]) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);

        // 写入表头
        writer.write_record(&[
            "contract_id", "customer_id", "steel_grade", "thickness", "width",
            "spec_family", "pdd", "days_to_pdd", "margin",
            "s_score", "p_score", "priority", "alpha"
        ]).map_err(|e| e.to_string())?;

        for p in priorities {
            writer.write_record(&[
                &p.contract.contract_id,
                &p.contract.customer_id,
                &p.contract.steel_grade,
                &p.contract.thickness.to_string(),
                &p.contract.width.to_string(),
                &p.contract.spec_family,
                &p.contract.pdd,
                &p.contract.days_to_pdd.to_string(),
                &p.contract.margin.to_string(),
                &format!("{:.2}", p.s_score),
                &format!("{:.2}", p.p_score),
                &format!("{:.2}", p.priority),
                &p.alpha.map(|a| format!("{:.2}", a)).unwrap_or_default(),
            ]).map_err(|e| e.to_string())?;
        }

        writer.into_inner().map_err(|e| e.to_string())
    }

    /// 生成工艺难度数据 CSV
    pub fn generate_process_difficulty(items: &[ProcessDifficulty]) -> Result<Vec<u8>, String> {
        let mut writer = WriterBuilder::new().from_writer(vec![]);

        for item in items {
            writer.serialize(item).map_err(|e| e.to_string())?;
        }

        writer.into_inner().map_err(|e| e.to_string())
    }
}
