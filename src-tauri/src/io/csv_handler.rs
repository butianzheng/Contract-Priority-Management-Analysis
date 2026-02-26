use crate::db::schema::{Contract, ContractPriority, Customer, ProcessDifficulty};
use csv::WriterBuilder;

pub struct CsvHandler;

impl CsvHandler {
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
        writer
            .write_record(&[
                "contract_id",
                "customer_id",
                "steel_grade",
                "thickness",
                "width",
                "spec_family",
                "pdd",
                "days_to_pdd",
                "margin",
                "s_score",
                "p_score",
                "priority",
                "alpha",
            ])
            .map_err(|e| e.to_string())?;

        for p in priorities {
            writer
                .write_record(&[
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
                ])
                .map_err(|e| e.to_string())?;
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
