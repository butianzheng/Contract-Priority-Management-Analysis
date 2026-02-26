use super::types::{ConflictRecord, ConflictStrategy};
use crate::db;
use crate::db::schema::{Contract, Customer, ProcessDifficulty, StrategyWeights};

pub struct ConflictHandler;

impl ConflictHandler {
    pub fn process_difficulty_id_key(id: i64) -> String {
        format!("id:{}", id)
    }

    pub fn process_difficulty_natural_key(item: &ProcessDifficulty) -> String {
        format!(
            "key:{}|{}|{}|{}|{}",
            item.steel_grade.trim(),
            item.thickness_min,
            item.thickness_max,
            item.width_min,
            item.width_max
        )
    }

    /// 检测合同数据冲突
    pub fn detect_contract_conflicts(
        contracts: &[Contract],
    ) -> Result<Vec<ConflictRecord>, String> {
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
    pub fn detect_customer_conflicts(
        customers: &[Customer],
    ) -> Result<Vec<ConflictRecord>, String> {
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

    /// 检测工艺难度冲突
    pub fn detect_process_difficulty_conflicts(
        items: &[ProcessDifficulty],
    ) -> Result<Vec<ConflictRecord>, String> {
        let mut conflicts = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            let row_number = idx + 2;

            if item.id > 0 {
                if let Some(existing) = db::get_process_difficulty_optional_by_id(item.id)? {
                    conflicts.push(ConflictRecord {
                        row_number,
                        primary_key: Self::process_difficulty_id_key(item.id),
                        existing_data: serde_json::to_value(&existing).unwrap_or_default(),
                        new_data: serde_json::to_value(item).unwrap_or_default(),
                        action: None,
                    });
                    continue;
                }
            }

            if let Some(existing) = db::get_process_difficulty_optional_by_key(
                &item.steel_grade,
                item.thickness_min,
                item.thickness_max,
                item.width_min,
                item.width_max,
            )? {
                conflicts.push(ConflictRecord {
                    row_number,
                    primary_key: Self::process_difficulty_natural_key(item),
                    existing_data: serde_json::to_value(&existing).unwrap_or_default(),
                    new_data: serde_json::to_value(item).unwrap_or_default(),
                    action: None,
                });
            }
        }

        Ok(conflicts)
    }

    /// 检测策略权重冲突（strategy_name 主键）
    pub fn detect_strategy_weight_conflicts(
        items: &[StrategyWeights],
    ) -> Result<Vec<ConflictRecord>, String> {
        let mut conflicts = Vec::new();

        for (idx, item) in items.iter().enumerate() {
            if let Some(existing) = db::get_strategy_weight_optional(&item.strategy_name)? {
                conflicts.push(ConflictRecord {
                    row_number: idx + 2,
                    primary_key: item.strategy_name.clone(),
                    existing_data: serde_json::to_value(&existing).unwrap_or_default(),
                    new_data: serde_json::to_value(item).unwrap_or_default(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Once;

    static TEST_DB_INIT: Once = Once::new();
    static UNIQUE_SEQ: AtomicU64 = AtomicU64::new(1);

    fn ensure_test_db() {
        TEST_DB_INIT.call_once(|| {
            let context = tauri::generate_context!();
            if let Err(err) = crate::db::initialize_database(context.config()) {
                match err {
                    rusqlite::Error::InvalidQuery => {}
                    _ => panic!("初始化测试数据库失败: {}", err),
                }
            }
        });
    }

    fn unique_steel_grade(prefix: &str) -> String {
        let seq = UNIQUE_SEQ.fetch_add(1, Ordering::Relaxed);
        format!("UT-CF-{}-{}", prefix, seq)
    }

    fn seed_process_difficulty(
        steel_grade: &str,
        thickness_min: f64,
        thickness_max: f64,
        width_min: f64,
        width_max: f64,
        difficulty_level: &str,
        difficulty_score: f64,
    ) -> ProcessDifficulty {
        let seed = ProcessDifficulty {
            id: 0,
            steel_grade: steel_grade.to_string(),
            thickness_min,
            thickness_max,
            width_min,
            width_max,
            difficulty_level: difficulty_level.to_string(),
            difficulty_score,
        };

        db::insert_process_difficulty(&seed).expect("插入测试工艺难度失败");
        db::get_process_difficulty_optional_by_key(
            steel_grade,
            thickness_min,
            thickness_max,
            width_min,
            width_max,
        )
        .expect("查询测试工艺难度失败")
        .expect("测试工艺难度记录不存在")
    }

    #[test]
    fn test_process_difficulty_keys() {
        let item = ProcessDifficulty {
            id: 12,
            steel_grade: "Q235".to_string(),
            thickness_min: 1.2,
            thickness_max: 2.5,
            width_min: 900.0,
            width_max: 1200.0,
            difficulty_level: "中".to_string(),
            difficulty_score: 55.0,
        };

        assert_eq!(ConflictHandler::process_difficulty_id_key(12), "id:12");
        assert_eq!(
            ConflictHandler::process_difficulty_natural_key(&item),
            "key:Q235|1.2|2.5|900|1200"
        );
    }

    #[test]
    fn test_detect_process_difficulty_conflicts_by_id_output() {
        ensure_test_db();

        let steel_grade = unique_steel_grade("id");
        let existing = seed_process_difficulty(&steel_grade, 1.0, 2.0, 1000.0, 1200.0, "中", 55.0);
        let incoming = ProcessDifficulty {
            id: existing.id,
            steel_grade: steel_grade.clone(),
            thickness_min: 1.0,
            thickness_max: 2.0,
            width_min: 1000.0,
            width_max: 1200.0,
            difficulty_level: "高".to_string(),
            difficulty_score: 88.0,
        };

        let conflicts = ConflictHandler::detect_process_difficulty_conflicts(&[incoming.clone()])
            .expect("检测工艺难度 id 冲突失败");
        assert_eq!(conflicts.len(), 1);

        let conflict = &conflicts[0];
        assert_eq!(conflict.row_number, 2);
        assert_eq!(
            conflict.primary_key,
            ConflictHandler::process_difficulty_id_key(existing.id)
        );
        assert_eq!(conflict.action, None);
        assert_eq!(
            conflict
                .existing_data
                .get("id")
                .and_then(|v| v.as_i64())
                .unwrap_or_default(),
            existing.id
        );
        assert_eq!(
            conflict
                .new_data
                .get("difficulty_score")
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
            88.0
        );
    }

    #[test]
    fn test_detect_process_difficulty_conflicts_by_natural_key_output() {
        ensure_test_db();

        let steel_grade = unique_steel_grade("natural");
        let _existing = seed_process_difficulty(&steel_grade, 1.1, 2.1, 1010.0, 1210.0, "中", 56.0);
        let incoming = ProcessDifficulty {
            id: 0,
            steel_grade: steel_grade.clone(),
            thickness_min: 1.1,
            thickness_max: 2.1,
            width_min: 1010.0,
            width_max: 1210.0,
            difficulty_level: "高".to_string(),
            difficulty_score: 90.0,
        };

        let conflicts = ConflictHandler::detect_process_difficulty_conflicts(&[incoming.clone()])
            .expect("检测工艺难度自然键冲突失败");
        assert_eq!(conflicts.len(), 1);

        let conflict = &conflicts[0];
        assert_eq!(conflict.row_number, 2);
        assert_eq!(
            conflict.primary_key,
            ConflictHandler::process_difficulty_natural_key(&incoming)
        );
        assert_eq!(conflict.action, None);
        assert_eq!(
            conflict
                .existing_data
                .get("steel_grade")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            steel_grade
        );
        assert_eq!(
            conflict
                .new_data
                .get("difficulty_level")
                .and_then(|v| v.as_str())
                .unwrap_or_default(),
            "高"
        );
    }

    #[test]
    fn test_detect_strategy_weight_conflicts_output() {
        ensure_test_db();

        let strategy_name = format!("UT-STRAT-{}", UNIQUE_SEQ.fetch_add(1, Ordering::Relaxed));
        db::upsert_strategy_weight(&strategy_name, 0.5, 0.5, Some("测试策略"))
            .expect("插入测试策略失败");

        let incoming = StrategyWeights {
            strategy_name: strategy_name.clone(),
            ws: 0.7,
            wp: 0.3,
            description: Some("导入覆盖".to_string()),
        };

        let conflicts = ConflictHandler::detect_strategy_weight_conflicts(&[incoming.clone()])
            .expect("检测策略权重冲突失败");

        assert_eq!(conflicts.len(), 1);
        let conflict = &conflicts[0];
        assert_eq!(conflict.row_number, 2);
        assert_eq!(conflict.primary_key, strategy_name);
        assert_eq!(conflict.action, None);
        assert_eq!(
            conflict
                .new_data
                .get("ws")
                .and_then(|v| v.as_f64())
                .unwrap_or_default(),
            0.7
        );
    }
}
