use crate::db;
use crate::db::schema::{Contract, Customer};
use super::types::{ConflictRecord, ConflictStrategy};

pub struct ConflictHandler;

impl ConflictHandler {
    /// 检测合同数据冲突
    pub fn detect_contract_conflicts(contracts: &[Contract]) -> Result<Vec<ConflictRecord>, String> {
        let mut conflicts = Vec::new();

        for (idx, contract) in contracts.iter().enumerate() {
            // 检查数据库中是否存在相同的合同编号
            if let Ok(Some(existing)) = db::get_contract_optional(&contract.contract_id) {
                conflicts.push(ConflictRecord {
                    row_number: idx + 2, // +2: 跳过表头 + 从1开始
                    primary_key: contract.contract_id.clone(),
                    existing_data: serde_json::to_value(&existing).unwrap_or_default(),
                    new_data: serde_json::to_value(contract).unwrap_or_default(),
                    action: None,
                });
            }
        }

        Ok(conflicts)
    }

    /// 检测客户数据冲突
    pub fn detect_customer_conflicts(customers: &[Customer]) -> Result<Vec<ConflictRecord>, String> {
        let mut conflicts = Vec::new();

        for (idx, customer) in customers.iter().enumerate() {
            if let Ok(Some(existing)) = db::get_customer_optional(&customer.customer_id) {
                conflicts.push(ConflictRecord {
                    row_number: idx + 2,
                    primary_key: customer.customer_id.clone(),
                    existing_data: serde_json::to_value(&existing).unwrap_or_default(),
                    new_data: serde_json::to_value(customer).unwrap_or_default(),
                    action: None,
                });
            }
        }

        Ok(conflicts)
    }

    /// 根据冲突决策过滤数据
    #[allow(dead_code)]
    pub fn filter_by_decisions<T: Clone>(
        data: &[T],
        conflicts: &[ConflictRecord],
        get_key: impl Fn(&T) -> String,
        default_strategy: ConflictStrategy,
    ) -> (Vec<T>, Vec<T>, usize) {
        // 返回 (to_insert, to_update, skipped_count)
        let mut to_insert = Vec::new();
        let mut to_update = Vec::new();
        let mut skipped = 0;

        // 建立冲突键的决策映射
        let conflict_map: std::collections::HashMap<String, ConflictStrategy> = conflicts
            .iter()
            .map(|c| {
                let strategy = c.action.unwrap_or(default_strategy);
                (c.primary_key.clone(), strategy)
            })
            .collect();

        for item in data {
            let key = get_key(item);
            if let Some(&strategy) = conflict_map.get(&key) {
                match strategy {
                    ConflictStrategy::Overwrite => to_update.push(item.clone()),
                    ConflictStrategy::Skip => skipped += 1,
                }
            } else {
                // 不在冲突列表中，直接插入
                to_insert.push(item.clone());
            }
        }

        (to_insert, to_update, skipped)
    }
}
