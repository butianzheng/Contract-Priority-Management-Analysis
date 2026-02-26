use super::init::get_connection;
use super::schema::*;
use rusqlite::{params, OptionalExtension, Result};

/// 获取单个合同
pub fn get_contract(contract_id: &str) -> Result<Contract, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT contract_id, customer_id, steel_grade, thickness, width, spec_family, pdd, days_to_pdd, margin FROM contract_master WHERE contract_id = ?1")
        .map_err(|e| e.to_string())?;

    let contract = stmt
        .query_row(params![contract_id], |row| {
            Ok(Contract {
                contract_id: row.get(0)?,
                customer_id: row.get(1)?,
                steel_grade: row.get(2)?,
                thickness: row.get(3)?,
                width: row.get(4)?,
                spec_family: row.get(5)?,
                pdd: row.get(6)?,
                days_to_pdd: row.get(7)?,
                margin: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(contract)
}

/// 获取所有合同列表
pub fn list_contracts() -> Result<Vec<Contract>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT contract_id, customer_id, steel_grade, thickness, width, spec_family, pdd, days_to_pdd, margin FROM contract_master ORDER BY pdd ASC")
        .map_err(|e| e.to_string())?;

    let contracts = stmt
        .query_map([], |row| {
            Ok(Contract {
                contract_id: row.get(0)?,
                customer_id: row.get(1)?,
                steel_grade: row.get(2)?,
                thickness: row.get(3)?,
                width: row.get(4)?,
                spec_family: row.get(5)?,
                pdd: row.get(6)?,
                days_to_pdd: row.get(7)?,
                margin: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(contracts)
}

/// 获取客户信息
pub fn get_customer(customer_id: &str) -> Result<Customer, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT customer_id, customer_name, customer_level, credit_level, customer_group FROM customer_master WHERE customer_id = ?1")
        .map_err(|e| e.to_string())?;

    let customer = stmt
        .query_row(params![customer_id], |row| {
            Ok(Customer {
                customer_id: row.get(0)?,
                customer_name: row.get(1)?,
                customer_level: row.get(2)?,
                credit_level: row.get(3)?,
                customer_group: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(customer)
}

/// 【性能优化】批量获取所有客户数据
///
/// 返回 HashMap<customer_id, Customer>，用于批量查询场景
/// 避免 N+1 查询问题
pub fn get_all_customers_map() -> Result<std::collections::HashMap<String, Customer>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT customer_id, customer_name, customer_level, credit_level, customer_group FROM customer_master")
        .map_err(|e| e.to_string())?;

    let customers = stmt
        .query_map([], |row| {
            Ok(Customer {
                customer_id: row.get(0)?,
                customer_name: row.get(1)?,
                customer_level: row.get(2)?,
                credit_level: row.get(3)?,
                customer_group: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 构建 HashMap 以便快速查找
    let mut customer_map = std::collections::HashMap::new();
    for customer in customers {
        customer_map.insert(customer.customer_id.clone(), customer);
    }

    Ok(customer_map)
}

/// 获取策略权重
pub fn get_strategy_weights(strategy_name: &str) -> Result<StrategyWeights, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT strategy_name, ws, wp, description FROM strategy_weights WHERE strategy_name = ?1")
        .map_err(|e| e.to_string())?;

    let weights = stmt
        .query_row(params![strategy_name], |row| {
            Ok(StrategyWeights {
                strategy_name: row.get(0)?,
                ws: row.get(1)?,
                wp: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?;

    Ok(weights)
}

/// 获取策略权重（可选）
pub fn get_strategy_weight_optional(
    strategy_name: &str,
) -> Result<Option<StrategyWeights>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT strategy_name, ws, wp, description FROM strategy_weights WHERE strategy_name = ?1")
        .map_err(|e| e.to_string())?;

    stmt.query_row(params![strategy_name], |row| {
        Ok(StrategyWeights {
            strategy_name: row.get(0)?,
            ws: row.get(1)?,
            wp: row.get(2)?,
            description: row.get(3)?,
        })
    })
    .optional()
    .map_err(|e| e.to_string())
}

/// 获取所有策略列表
pub fn list_strategies() -> Result<Vec<String>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT strategy_name FROM strategy_weights ORDER BY strategy_name")
        .map_err(|e| e.to_string())?;

    let strategies = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<String>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(strategies)
}

/// 获取所有策略权重 (ws, wp)
pub fn list_all_strategy_weights() -> Result<Vec<StrategyWeights>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT strategy_name, ws, wp, description FROM strategy_weights ORDER BY strategy_name")
        .map_err(|e| e.to_string())?;

    let weights = stmt
        .query_map([], |row| {
            Ok(StrategyWeights {
                strategy_name: row.get(0)?,
                ws: row.get(1)?,
                wp: row.get(2)?,
                description: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<StrategyWeights>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(weights)
}

/// 创建或更新策略权重 (ws, wp)
/// 同时确保 strategy_scoring_weights 表中也有对应记录（使用默认 w1, w2, w3）
pub fn upsert_strategy_weight(
    strategy_name: &str,
    ws: f64,
    wp: f64,
    description: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 插入或更新 strategy_weights 表
    conn.execute(
        "INSERT INTO strategy_weights (strategy_name, ws, wp, description, created_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now','localtime'))
         ON CONFLICT(strategy_name)
         DO UPDATE SET ws = ?2, wp = ?3, description = ?4",
        params![strategy_name, ws, wp, description],
    )
    .map_err(|e| format!("保存策略权重失败: {}", e))?;

    // 2. 确保 strategy_scoring_weights 表中也有对应记录（如果不存在则使用默认值）
    conn.execute(
        "INSERT INTO strategy_scoring_weights (strategy_name, w1, w2, w3, description, created_at, updated_at)
         VALUES (?1, 0.4, 0.3, 0.3, ?2, datetime('now','localtime'), datetime('now','localtime'))
         ON CONFLICT(strategy_name) DO NOTHING",
        params![strategy_name, description],
    ).map_err(|e| format!("创建评分权重失败: {}", e))?;

    Ok(())
}

/// 删除策略权重
pub fn delete_strategy_weight(strategy_name: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let affected = conn
        .execute(
            "DELETE FROM strategy_weights WHERE strategy_name = ?1",
            params![strategy_name],
        )
        .map_err(|e| format!("删除策略权重失败: {}", e))?;

    if affected == 0 {
        return Err(format!("策略 '{}' 不存在", strategy_name));
    }

    Ok(())
}

/// 查询工艺难度分数
pub fn get_process_difficulty_score(
    steel_grade: &str,
    thickness: f64,
    width: f64,
) -> Result<f64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT difficulty_score FROM process_difficulty
             WHERE steel_grade = ?1
             AND thickness_min <= ?2 AND thickness_max >= ?2
             AND width_min <= ?3 AND width_max >= ?3
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let score = stmt
        .query_row(params![steel_grade, thickness, width], |row| row.get(0))
        .unwrap_or(50.0); // 默认中等难度

    Ok(score)
}

/// 查询节拍标签加分（P3 评分）- 支持 n 日可配置周期
///
/// # 数据来源
/// - `rhythm_config` 表：获取当前激活的周期配置
/// - `rhythm_label` 表：查询匹配的节拍标签
///
/// # 参数
/// - `days_to_pdd`: 距交期天数（用于计算周期日）
/// - `spec_family`: 规格族代码
///
/// # 返回
/// - 匹配成功：返回 (bonus_score, rhythm_day, cycle_days, label_name)
/// - 未匹配：返回错误（由调用者处理默认值）
///
/// # 静态规则声明
/// 本函数仅查询配置表，不访问任何实时数据。
pub fn get_rhythm_bonus_with_config(
    days_to_pdd: i64,
    spec_family: &str,
) -> Result<(f64, i32, i32, Option<String>), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取当前激活的节拍配置
    let (config_id, cycle_days): (i64, i32) = conn
        .query_row(
            "SELECT config_id, cycle_days FROM rhythm_config WHERE is_active = 1 LIMIT 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|_| "未找到激活的节拍配置".to_string())?;

    // 2. 计算周期日（1 到 cycle_days）
    let rhythm_day = ((days_to_pdd.rem_euclid(cycle_days as i64)) + 1) as i32;

    // 3. 查询匹配的节拍标签
    let result = conn.query_row(
        "SELECT bonus_score, label_name FROM rhythm_label
         WHERE config_id = ?1
         AND rhythm_day = ?2
         AND (match_spec LIKE '%' || ?3 || '%' OR match_spec = '*' OR match_spec IS NULL)
         LIMIT 1",
        params![config_id, rhythm_day, spec_family],
        |row| Ok((row.get::<_, f64>(0)?, row.get::<_, String>(1)?)),
    );

    match result {
        Ok((score, label)) => Ok((score, rhythm_day, cycle_days, Some(label))),
        Err(_) => {
            // 未找到匹配的标签，返回默认分数
            Ok((50.0, rhythm_day, cycle_days, None))
        }
    }
}

/// 查询节拍标签加分（P3 评分）- 简化接口，向后兼容
///
/// # 数据来源
/// `rhythm_label` 表（静态配置表）
///
/// # 参数
/// - `rhythm_day`: 周期日（1 到 cycle_days）
/// - `spec_family`: 规格族代码
///
/// # 返回
/// - 匹配成功：返回 bonus_score
/// - 未匹配：返回错误（由调用者处理默认值）
///
/// # 静态规则声明
/// 本函数仅查询配置表，不访问任何实时数据。
#[allow(dead_code)]
pub fn get_rhythm_bonus(rhythm_day: i32, spec_family: &str) -> Result<f64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取当前激活的配置 ID
    let config_id: i64 = conn
        .query_row(
            "SELECT config_id FROM rhythm_config WHERE is_active = 1 LIMIT 1",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1); // 默认使用 config_id = 1

    let mut stmt = conn
        .prepare(
            "SELECT bonus_score FROM rhythm_label
             WHERE config_id = ?1
             AND rhythm_day = ?2
             AND (match_spec LIKE '%' || ?3 || '%' OR match_spec = '*' OR match_spec IS NULL)
             LIMIT 1",
        )
        .map_err(|e| e.to_string())?;

    let score: f64 = stmt
        .query_row(params![config_id, rhythm_day, spec_family], |row| {
            row.get(0)
        })
        .map_err(|e| format!("未找到匹配的节拍标签: {}", e))?;

    Ok(score)
}

/// 获取当前激活的节拍配置
pub fn get_active_rhythm_config() -> Result<RhythmConfig, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT config_id, config_name, cycle_days, description, is_active, created_by, created_at, updated_at
         FROM rhythm_config WHERE is_active = 1 LIMIT 1",
        [],
        |row| {
            Ok(RhythmConfig {
                config_id: row.get(0)?,
                config_name: row.get(1)?,
                cycle_days: row.get(2)?,
                description: row.get(3)?,
                is_active: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("获取激活节拍配置失败: {}", e))
}

/// 记录人工干预
pub fn log_intervention(log: InterventionLog) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO intervention_log (contract_id, alpha_value, reason, user) VALUES (?1, ?2, ?3, ?4)",
        params![log.contract_id, log.alpha_value, log.reason, log.user],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取合同的最新 alpha 值
pub fn get_latest_alpha(contract_id: &str) -> Result<Option<f64>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT alpha_value FROM intervention_log WHERE contract_id = ?1 ORDER BY timestamp DESC LIMIT 1")
        .map_err(|e| e.to_string())?;

    let alpha = stmt.query_row(params![contract_id], |row| row.get(0)).ok();

    Ok(alpha)
}

/// 【性能优化】批量获取所有合同的最新 alpha 值
///
/// 返回 HashMap<contract_id, alpha_value>，用于批量查询场景
/// 避免 N+1 查询问题
///
/// 使用子查询获取每个合同的最新 alpha 记录
pub fn get_all_latest_alphas() -> Result<std::collections::HashMap<String, f64>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 使用子查询获取每个合同的最新 alpha 值
    // 方案：找出每个 contract_id 的最大 timestamp，然后获取对应的 alpha_value
    let mut stmt = conn
        .prepare(
            "SELECT i1.contract_id, i1.alpha_value
             FROM intervention_log i1
             INNER JOIN (
                 SELECT contract_id, MAX(timestamp) as max_timestamp
                 FROM intervention_log
                 GROUP BY contract_id
             ) i2
             ON i1.contract_id = i2.contract_id AND i1.timestamp = i2.max_timestamp",
        )
        .map_err(|e| e.to_string())?;

    let alphas = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, f64>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 构建 HashMap
    let mut alpha_map = std::collections::HashMap::new();
    for (contract_id, alpha_value) in alphas {
        alpha_map.insert(contract_id, alpha_value);
    }

    Ok(alpha_map)
}

/// 获取合同的所有干预历史
pub fn get_intervention_history(contract_id: &str) -> Result<Vec<InterventionLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, contract_id, alpha_value, reason, user, timestamp FROM intervention_log WHERE contract_id = ?1 ORDER BY timestamp DESC")
        .map_err(|e| e.to_string())?;

    let logs = stmt
        .query_map(params![contract_id], |row| {
            Ok(InterventionLog {
                id: row.get(0)?,
                contract_id: row.get(1)?,
                alpha_value: row.get(2)?,
                reason: row.get(3)?,
                user: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}

/// 获取所有干预日志（支持分页）
pub fn get_all_intervention_logs(limit: Option<i64>) -> Result<Vec<InterventionLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = match limit {
        Some(l) => format!(
            "SELECT id, contract_id, alpha_value, reason, user, timestamp
             FROM intervention_log
             ORDER BY timestamp DESC
             LIMIT {}",
            l
        ),
        None => "SELECT id, contract_id, alpha_value, reason, user, timestamp
                 FROM intervention_log
                 ORDER BY timestamp DESC"
            .to_string(),
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let logs = stmt
        .query_map([], |row| {
            Ok(InterventionLog {
                id: row.get(0)?,
                contract_id: row.get(1)?,
                alpha_value: row.get(2)?,
                reason: row.get(3)?,
                user: row.get(4)?,
                timestamp: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}

// ============================================
// 配置相关查询方法 (Phase 1: 配置化)
// ============================================

use crate::config::StrategyScoringWeights;
use std::collections::HashMap;

/// 获取所有评分配置（返回 HashMap<config_key, config_value>）
pub fn get_all_scoring_configs() -> Result<HashMap<String, String>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT config_key, config_value FROM scoring_config WHERE is_active = 1")
        .map_err(|e| e.to_string())?;

    let configs = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<HashMap<_, _>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(configs)
}

/// 获取策略的评分子权重
pub fn get_strategy_scoring_weights(strategy_name: &str) -> Result<StrategyScoringWeights, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT strategy_name, w1, w2, w3 FROM strategy_scoring_weights WHERE strategy_name = ?1")
        .map_err(|e| e.to_string())?;

    let weights = stmt
        .query_row(params![strategy_name], |row| {
            Ok(StrategyScoringWeights {
                strategy_name: row.get(0)?,
                w1: row.get(1)?,
                w2: row.get(2)?,
                w3: row.get(3)?,
            })
        })
        .unwrap_or_else(|e| {
            // 如果查询失败，返回默认权重
            eprintln!("获取策略权重失败: {}, 使用默认权重", e);
            StrategyScoringWeights::default()
        });

    Ok(weights)
}

// ============================================
// Phase 2: 配置管理相关方法
// ============================================

/// 获取所有评分配置项（带完整信息）
pub fn list_scoring_configs() -> Result<Vec<ScoringConfigItem>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT config_key, config_value, value_type, category, description, default_value, min_value, max_value FROM scoring_config WHERE is_active = 1 ORDER BY category, config_key")
        .map_err(|e| e.to_string())?;

    let configs = stmt
        .query_map([], |row| {
            Ok(ScoringConfigItem {
                config_key: row.get(0)?,
                config_value: row.get(1)?,
                value_type: row.get(2)?,
                category: row.get(3)?,
                description: row.get(4)?,
                default_value: row.get(5)?,
                min_value: row.get(6)?,
                max_value: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(configs)
}

/// 更新单个配置项
pub fn update_scoring_config(
    config_key: &str,
    new_value: &str,
    changed_by: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取旧值
    let old_value: String = conn
        .query_row(
            "SELECT config_value FROM scoring_config WHERE config_key = ?1",
            params![config_key],
            |row| row.get(0),
        )
        .map_err(|e| format!("配置项不存在: {}", e))?;

    // 2. 更新配置值
    conn.execute(
        "UPDATE scoring_config SET config_value = ?1, updated_at = datetime('now','localtime') WHERE config_key = ?2",
        params![new_value, config_key],
    )
    .map_err(|e| format!("更新配置失败: {}", e))?;

    // 3. 记录变更日志
    conn.execute(
        "INSERT INTO config_change_log (config_key, old_value, new_value, change_reason, changed_by) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![config_key, old_value, new_value, reason, changed_by],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(())
}

/// 获取配置变更历史
pub fn get_config_change_history(
    config_key: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<ConfigChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let (query, params_vec): (String, Vec<Box<dyn rusqlite::ToSql>>) = match (config_key, limit) {
        (Some(key), Some(lim)) => (
            "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at FROM config_change_log WHERE config_key = ?1 ORDER BY changed_at DESC LIMIT ?2".to_string(),
            vec![Box::new(key.to_string()), Box::new(lim)],
        ),
        (Some(key), None) => (
            "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at FROM config_change_log WHERE config_key = ?1 ORDER BY changed_at DESC".to_string(),
            vec![Box::new(key.to_string())],
        ),
        (None, Some(lim)) => (
            "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at FROM config_change_log ORDER BY changed_at DESC LIMIT ?1".to_string(),
            vec![Box::new(lim)],
        ),
        (None, None) => (
            "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at FROM config_change_log ORDER BY changed_at DESC".to_string(),
            vec![],
        ),
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let params_refs: Vec<&dyn rusqlite::ToSql> = params_vec.iter().map(|b| b.as_ref()).collect();

    let logs = stmt
        .query_map(params_refs.as_slice(), |row| {
            Ok(ConfigChangeLog {
                id: row.get(0)?,
                config_key: row.get(1)?,
                old_value: row.get(2)?,
                new_value: row.get(3)?,
                change_reason: row.get(4)?,
                changed_by: row.get(5)?,
                changed_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}

/// 回滚配置到指定版本
pub fn rollback_config(log_id: i64, changed_by: &str, reason: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取日志记录
    let (config_key, old_value): (String, String) = conn
        .query_row(
            "SELECT config_key, old_value FROM config_change_log WHERE id = ?1",
            params![log_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("找不到变更日志: {}", e))?;

    // 2. 获取当前值
    let current_value: String = conn
        .query_row(
            "SELECT config_value FROM scoring_config WHERE config_key = ?1",
            params![config_key],
            |row| row.get(0),
        )
        .map_err(|e| format!("配置项不存在: {}", e))?;

    // 3. 回滚配置
    conn.execute(
        "UPDATE scoring_config SET config_value = ?1, updated_at = datetime('now','localtime') WHERE config_key = ?2",
        params![old_value, config_key],
    )
    .map_err(|e| format!("回滚配置失败: {}", e))?;

    // 4. 记录回滚日志
    let rollback_reason = format!("回滚到版本#{}: {}", log_id, reason);
    conn.execute(
        "INSERT INTO config_change_log (config_key, old_value, new_value, change_reason, changed_by) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![config_key, current_value, old_value, rollback_reason, changed_by],
    )
    .map_err(|e| format!("记录回滚日志失败: {}", e))?;

    Ok(())
}

/// 获取所有策略的评分权重
pub fn list_all_strategy_scoring_weights() -> Result<Vec<StrategyScoringWeights>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT strategy_name, w1, w2, w3 FROM strategy_scoring_weights ORDER BY strategy_name",
        )
        .map_err(|e| e.to_string())?;

    let weights = stmt
        .query_map([], |row| {
            Ok(StrategyScoringWeights {
                strategy_name: row.get(0)?,
                w1: row.get(1)?,
                w2: row.get(2)?,
                w3: row.get(3)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(weights)
}

/// 更新策略的评分权重
pub fn update_strategy_scoring_weights(
    strategy_name: &str,
    w1: f64,
    w2: f64,
    w3: f64,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 验证权重和为1
    let sum = w1 + w2 + w3;
    if (sum - 1.0).abs() > 0.001 {
        return Err(format!("权重之和必须为 1.0，当前为 {:.3}", sum));
    }

    // 更新或插入
    conn.execute(
        "INSERT INTO strategy_scoring_weights (strategy_name, w1, w2, w3, updated_at)
         VALUES (?1, ?2, ?3, ?4, datetime('now','localtime'))
         ON CONFLICT(strategy_name) DO UPDATE SET
         w1 = ?2, w2 = ?3, w3 = ?4, updated_at = datetime('now','localtime')",
        params![strategy_name, w1, w2, w3],
    )
    .map_err(|e| format!("更新策略权重失败: {}", e))?;

    Ok(())
}

// ============================================
// Phase 4: 筛选器预设相关操作
// ============================================

/// 获取所有筛选器预设
pub fn list_filter_presets() -> Result<Vec<FilterPreset>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT preset_id, preset_name, filter_json, description, created_by, created_at, is_default FROM filter_presets ORDER BY is_default DESC, preset_name")
        .map_err(|e| e.to_string())?;

    let presets = stmt
        .query_map([], |row| {
            Ok(FilterPreset {
                preset_id: row.get(0)?,
                preset_name: row.get(1)?,
                filter_json: row.get(2)?,
                description: row.get(3)?,
                created_by: row.get(4)?,
                created_at: row.get(5)?,
                is_default: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(presets)
}

/// 保存筛选器预设
pub fn save_filter_preset(
    preset_name: &str,
    filter_json: &str,
    description: &str,
    created_by: &str,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 检查预设名称是否已存在
    let exists: bool = conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM filter_presets WHERE preset_name = ?1)",
            params![preset_name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    if exists {
        return Err(format!("筛选器预设 '{}' 已存在", preset_name));
    }

    // 插入新预设
    conn.execute(
        "INSERT INTO filter_presets (preset_name, filter_json, description, created_by, is_default)
         VALUES (?1, ?2, ?3, ?4, 0)",
        params![preset_name, filter_json, description, created_by],
    )
    .map_err(|e| format!("保存筛选器预设失败: {}", e))?;

    Ok(())
}

/// 删除筛选器预设
pub fn delete_filter_preset(preset_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 检查是否为默认预设
    let is_default: i64 = conn
        .query_row(
            "SELECT is_default FROM filter_presets WHERE preset_id = ?1",
            params![preset_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("查询预设失败: {}", e))?;

    if is_default == 1 {
        return Err("无法删除默认预设".to_string());
    }

    // 删除预设
    let rows_affected = conn
        .execute(
            "DELETE FROM filter_presets WHERE preset_id = ?1",
            params![preset_id],
        )
        .map_err(|e| format!("删除筛选器预设失败: {}", e))?;

    if rows_affected == 0 {
        return Err("预设不存在".to_string());
    }

    Ok(())
}

/// 设置默认筛选器预设
pub fn set_default_filter_preset(preset_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 先将所有预设的 is_default 设为 0
    if let Err(e) = conn.execute("UPDATE filter_presets SET is_default = 0", []) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("更新预设失败: {}", e));
    }

    // 再将指定预设的 is_default 设为 1
    let rows = conn.execute(
        "UPDATE filter_presets SET is_default = 1 WHERE preset_id = ?1",
        params![preset_id],
    );

    match rows {
        Ok(0) => {
            let _ = conn.execute("ROLLBACK", []);
            Err("预设不存在".to_string())
        }
        Ok(_) => {
            conn.execute("COMMIT", [])
                .map_err(|e| format!("提交事务失败: {}", e))?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(format!("设置默认预设失败: {}", e))
        }
    }
}

// ============================================
// Phase 5: 批量操作相关函数
// ============================================

/// 批量调整合同的 alpha 值
pub fn batch_adjust_alpha(
    contract_ids: Vec<String>,
    alpha: f64,
    reason: &str,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 1. 创建批量操作记录
    let result = conn.execute(
        "INSERT INTO batch_operations (operation_type, contract_count, reason, user)
         VALUES ('adjust', ?1, ?2, ?3)",
        params![contract_ids.len() as i64, reason, user],
    );

    if let Err(e) = result {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("创建批量操作记录失败: {}", e));
    }

    let batch_id = conn.last_insert_rowid();

    // 2. 为每个合同记录干预日志
    for contract_id in &contract_ids {
        let result = conn.execute(
            "INSERT INTO intervention_log (contract_id, alpha_value, reason, user, batch_id)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![contract_id, alpha, reason, user, batch_id],
        );

        if let Err(e) = result {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("记录干预日志失败: {}", e));
        }
    }

    // 3. 提交事务
    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(batch_id)
}

/// 批量恢复合同的 alpha 值（设为 null）
pub fn batch_restore_alpha(
    contract_ids: Vec<String>,
    reason: &str,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 1. 创建批量操作记录
    let result = conn.execute(
        "INSERT INTO batch_operations (operation_type, contract_count, reason, user)
         VALUES ('restore', ?1, ?2, ?3)",
        params![contract_ids.len() as i64, reason, user],
    );

    if let Err(e) = result {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("创建批量操作记录失败: {}", e));
    }

    let batch_id = conn.last_insert_rowid();

    // 2. 为每个合同记录干预日志（alpha 设为 1.0 表示恢复原值）
    for contract_id in &contract_ids {
        let result = conn.execute(
            "INSERT INTO intervention_log (contract_id, alpha_value, reason, user, batch_id)
             VALUES (?1, 1.0, ?2, ?3, ?4)",
            params![contract_id, reason, user, batch_id],
        );

        if let Err(e) = result {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("记录恢复日志失败: {}", e));
        }
    }

    // 3. 提交事务
    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(batch_id)
}

/// 获取批量操作历史
pub fn list_batch_operations(limit: Option<i64>) -> Result<Vec<BatchOperation>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if let Some(limit) = limit {
        format!(
            "SELECT batch_id, operation_type, contract_count, reason, user, created_at
             FROM batch_operations
             ORDER BY batch_id DESC
             LIMIT {}",
            limit
        )
    } else {
        "SELECT batch_id, operation_type, contract_count, reason, user, created_at
         FROM batch_operations
         ORDER BY batch_id DESC"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let operations = stmt
        .query_map([], |row| {
            Ok(BatchOperation {
                batch_id: row.get(0)?,
                operation_type: row.get(1)?,
                contract_count: row.get(2)?,
                reason: row.get(3)?,
                user: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(operations)
}

/// 获取批量操作涉及的合同ID列表
pub fn get_batch_operation_contracts(batch_id: i64) -> Result<Vec<String>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT contract_id FROM intervention_log WHERE batch_id = ?1")
        .map_err(|e| e.to_string())?;

    let contract_ids = stmt
        .query_map(params![batch_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(contract_ids)
}

// ============================================
// 统一历史记录查询
// ============================================

/// 获取所有类型的变更历史（统一视图）
///
/// 参数：
/// - entry_type: 可选，筛选类型 ("config_change" | "alpha_adjust" | "batch_operation")
/// - user: 可选，筛选操作人
/// - limit: 可选，限制返回数量
pub fn get_unified_history(
    entry_type: Option<&str>,
    user: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<UnifiedHistoryEntry>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;
    let mut entries = Vec::new();

    // 1. 获取配置变更历史
    if entry_type.is_none() || entry_type == Some("config_change") {
        let config_query = if let Some(u) = user {
            format!(
                "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at
                 FROM config_change_log
                 WHERE changed_by = '{}'
                 ORDER BY changed_at DESC",
                u
            )
        } else {
            "SELECT id, config_key, old_value, new_value, change_reason, changed_by, changed_at
             FROM config_change_log
             ORDER BY changed_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&config_query).map_err(|e| e.to_string())?;
        let config_logs = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let config_key: String = row.get(1)?;
                let old_value: String = row.get(2)?;
                let new_value: String = row.get(3)?;
                let change_reason: Option<String> = row.get(4)?;
                let changed_by: String = row.get(5)?;
                let changed_at: Option<String> = row.get(6)?;

                let details = serde_json::json!({
                    "config_key": config_key,
                    "old_value": old_value,
                    "new_value": new_value,
                });

                Ok(UnifiedHistoryEntry {
                    id: format!("config_{}", id),
                    entry_type: "config_change".to_string(),
                    timestamp: changed_at.unwrap_or_else(|| "".to_string()),
                    user: changed_by,
                    description: format!("配置变更: {}", config_key),
                    reason: change_reason,
                    details,
                })
            })
            .map_err(|e| e.to_string())?;

        for log in config_logs {
            entries.push(log.map_err(|e| e.to_string())?);
        }
    }

    // 2. 获取Alpha调整历史（单个合同调整）
    if entry_type.is_none() || entry_type == Some("alpha_adjust") {
        let alpha_query = if let Some(u) = user {
            format!(
                "SELECT id, contract_id, alpha_value, reason, user, timestamp
                 FROM intervention_log
                 WHERE user = '{}' AND (batch_id IS NULL OR batch_id = 0)
                 ORDER BY timestamp DESC",
                u
            )
        } else {
            "SELECT id, contract_id, alpha_value, reason, user, timestamp
             FROM intervention_log
             WHERE (batch_id IS NULL OR batch_id = 0)
             ORDER BY timestamp DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&alpha_query).map_err(|e| e.to_string())?;
        let alpha_logs = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let contract_id: String = row.get(1)?;
                let alpha_value: f64 = row.get(2)?;
                let reason: String = row.get(3)?;
                let user: String = row.get(4)?;
                let timestamp: Option<String> = row.get(5)?;

                let details = serde_json::json!({
                    "contract_id": contract_id,
                    "alpha_value": alpha_value,
                });

                Ok(UnifiedHistoryEntry {
                    id: format!("alpha_{}", id),
                    entry_type: "alpha_adjust".to_string(),
                    timestamp: timestamp.unwrap_or_else(|| "".to_string()),
                    user,
                    description: format!("调整合同优先级: {}", contract_id),
                    reason: Some(reason),
                    details,
                })
            })
            .map_err(|e| e.to_string())?;

        for log in alpha_logs {
            entries.push(log.map_err(|e| e.to_string())?);
        }
    }

    // 3. 获取批量操作历史
    if entry_type.is_none() || entry_type == Some("batch_operation") {
        let batch_query = if let Some(u) = user {
            format!(
                "SELECT batch_id, operation_type, contract_count, reason, user, created_at
                 FROM batch_operations
                 WHERE user = '{}'
                 ORDER BY created_at DESC",
                u
            )
        } else {
            "SELECT batch_id, operation_type, contract_count, reason, user, created_at
             FROM batch_operations
             ORDER BY created_at DESC"
                .to_string()
        };

        let mut stmt = conn.prepare(&batch_query).map_err(|e| e.to_string())?;
        let batch_logs = stmt
            .query_map([], |row| {
                let batch_id: i64 = row.get(0)?;
                let operation_type: String = row.get(1)?;
                let contract_count: i64 = row.get(2)?;
                let reason: String = row.get(3)?;
                let user: String = row.get(4)?;
                let created_at: Option<String> = row.get(5)?;

                let op_desc = match operation_type.as_str() {
                    "adjust" => "批量调整优先级",
                    "restore" => "批量恢复优先级",
                    _ => "批量操作",
                };

                let details = serde_json::json!({
                    "batch_id": batch_id,
                    "operation_type": operation_type,
                    "contract_count": contract_count,
                });

                Ok(UnifiedHistoryEntry {
                    id: format!("batch_{}", batch_id),
                    entry_type: "batch_operation".to_string(),
                    timestamp: created_at.unwrap_or_else(|| "".to_string()),
                    user,
                    description: format!("{}: {} 个合同", op_desc, contract_count),
                    reason: Some(reason),
                    details,
                })
            })
            .map_err(|e| e.to_string())?;

        for log in batch_logs {
            entries.push(log.map_err(|e| e.to_string())?);
        }
    }

    // 4. 按时间戳降序排序
    entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // 5. 应用 limit
    if let Some(limit) = limit {
        entries.truncate(limit as usize);
    }

    Ok(entries)
}

// ============================================
// 导入/导出相关函数
// ============================================

/// 检查合同是否存在（可选返回）
pub fn get_contract_optional(contract_id: &str) -> Result<Option<Contract>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT contract_id, customer_id, steel_grade, thickness, width, spec_family, pdd, days_to_pdd, margin FROM contract_master WHERE contract_id = ?1")
        .map_err(|e| e.to_string())?;

    let contract = stmt
        .query_row(params![contract_id], |row| {
            Ok(Contract {
                contract_id: row.get(0)?,
                customer_id: row.get(1)?,
                steel_grade: row.get(2)?,
                thickness: row.get(3)?,
                width: row.get(4)?,
                spec_family: row.get(5)?,
                pdd: row.get(6)?,
                days_to_pdd: row.get(7)?,
                margin: row.get(8)?,
            })
        })
        .ok();

    Ok(contract)
}

/// 检查客户是否存在（可选返回）
pub fn get_customer_optional(customer_id: &str) -> Result<Option<Customer>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT customer_id, customer_name, customer_level, credit_level, customer_group FROM customer_master WHERE customer_id = ?1")
        .map_err(|e| e.to_string())?;

    let customer = stmt
        .query_row(params![customer_id], |row| {
            Ok(Customer {
                customer_id: row.get(0)?,
                customer_name: row.get(1)?,
                customer_level: row.get(2)?,
                credit_level: row.get(3)?,
                customer_group: row.get(4)?,
            })
        })
        .ok();

    Ok(customer)
}

/// 插入合同
pub fn insert_contract(contract: &Contract) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO contract_master (contract_id, customer_id, steel_grade, thickness, width, spec_family, pdd, days_to_pdd, margin)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            contract.contract_id,
            contract.customer_id,
            contract.steel_grade,
            contract.thickness,
            contract.width,
            contract.spec_family,
            contract.pdd,
            contract.days_to_pdd,
            contract.margin
        ],
    )
    .map_err(|e| format!("插入合同失败: {}", e))?;

    Ok(())
}

/// 更新合同
pub fn update_contract(contract: &Contract) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE contract_master SET customer_id = ?2, steel_grade = ?3, thickness = ?4, width = ?5, spec_family = ?6, pdd = ?7, days_to_pdd = ?8, margin = ?9
         WHERE contract_id = ?1",
        params![
            contract.contract_id,
            contract.customer_id,
            contract.steel_grade,
            contract.thickness,
            contract.width,
            contract.spec_family,
            contract.pdd,
            contract.days_to_pdd,
            contract.margin
        ],
    )
    .map_err(|e| format!("更新合同失败: {}", e))?;

    Ok(())
}

/// 插入客户
pub fn insert_customer(customer: &Customer) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO customer_master (customer_id, customer_name, customer_level, credit_level, customer_group)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            customer.customer_id,
            customer.customer_name,
            customer.customer_level,
            customer.credit_level,
            customer.customer_group
        ],
    )
    .map_err(|e| format!("插入客户失败: {}", e))?;

    Ok(())
}

/// 更新客户
pub fn update_customer(customer: &Customer) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE customer_master SET customer_name = ?2, customer_level = ?3, credit_level = ?4, customer_group = ?5
         WHERE customer_id = ?1",
        params![
            customer.customer_id,
            customer.customer_name,
            customer.customer_level,
            customer.credit_level,
            customer.customer_group
        ],
    )
    .map_err(|e| format!("更新客户失败: {}", e))?;

    Ok(())
}

/// 获取所有客户列表
pub fn list_customers() -> Result<Vec<Customer>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT customer_id, customer_name, customer_level, credit_level, customer_group FROM customer_master ORDER BY customer_id")
        .map_err(|e| e.to_string())?;

    let customers = stmt
        .query_map([], |row| {
            Ok(Customer {
                customer_id: row.get(0)?,
                customer_name: row.get(1)?,
                customer_level: row.get(2)?,
                credit_level: row.get(3)?,
                customer_group: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(customers)
}

/// 获取所有工艺难度配置
pub fn list_process_difficulty() -> Result<Vec<ProcessDifficulty>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare("SELECT id, steel_grade, thickness_min, thickness_max, width_min, width_max, difficulty_level, difficulty_score FROM process_difficulty ORDER BY steel_grade, thickness_min")
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map([], |row| {
            Ok(ProcessDifficulty {
                id: row.get(0)?,
                steel_grade: row.get(1)?,
                thickness_min: row.get(2)?,
                thickness_max: row.get(3)?,
                width_min: row.get(4)?,
                width_max: row.get(5)?,
                difficulty_level: row.get(6)?,
                difficulty_score: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

/// 根据 ID 获取工艺难度配置（可选）
pub fn get_process_difficulty_optional_by_id(id: i64) -> Result<Option<ProcessDifficulty>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let item = conn
        .query_row(
            "SELECT id, steel_grade, thickness_min, thickness_max, width_min, width_max, difficulty_level, difficulty_score
             FROM process_difficulty
             WHERE id = ?1",
            params![id],
            |row| {
                Ok(ProcessDifficulty {
                    id: row.get(0)?,
                    steel_grade: row.get(1)?,
                    thickness_min: row.get(2)?,
                    thickness_max: row.get(3)?,
                    width_min: row.get(4)?,
                    width_max: row.get(5)?,
                    difficulty_level: row.get(6)?,
                    difficulty_score: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("查询工艺难度配置失败: {}", e))?;

    Ok(item)
}

/// 按自然键获取工艺难度配置（可选）
pub fn get_process_difficulty_optional_by_key(
    steel_grade: &str,
    thickness_min: f64,
    thickness_max: f64,
    width_min: f64,
    width_max: f64,
) -> Result<Option<ProcessDifficulty>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let item = conn
        .query_row(
            "SELECT id, steel_grade, thickness_min, thickness_max, width_min, width_max, difficulty_level, difficulty_score
             FROM process_difficulty
             WHERE steel_grade = ?1
               AND thickness_min = ?2
               AND thickness_max = ?3
               AND width_min = ?4
               AND width_max = ?5",
            params![steel_grade, thickness_min, thickness_max, width_min, width_max],
            |row| {
                Ok(ProcessDifficulty {
                    id: row.get(0)?,
                    steel_grade: row.get(1)?,
                    thickness_min: row.get(2)?,
                    thickness_max: row.get(3)?,
                    width_min: row.get(4)?,
                    width_max: row.get(5)?,
                    difficulty_level: row.get(6)?,
                    difficulty_score: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(|e| format!("查询工艺难度配置失败: {}", e))?;

    Ok(item)
}

/// 插入工艺难度配置
pub fn insert_process_difficulty(item: &ProcessDifficulty) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO process_difficulty
            (steel_grade, thickness_min, thickness_max, width_min, width_max, difficulty_level, difficulty_score)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            item.steel_grade,
            item.thickness_min,
            item.thickness_max,
            item.width_min,
            item.width_max,
            item.difficulty_level,
            item.difficulty_score
        ],
    )
    .map_err(|e| format!("插入工艺难度配置失败: {}", e))?;

    Ok(())
}

/// 更新工艺难度配置（按 ID）
pub fn update_process_difficulty(item: &ProcessDifficulty) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE process_difficulty
         SET steel_grade = ?2,
             thickness_min = ?3,
             thickness_max = ?4,
             width_min = ?5,
             width_max = ?6,
             difficulty_level = ?7,
             difficulty_score = ?8
         WHERE id = ?1",
        params![
            item.id,
            item.steel_grade,
            item.thickness_min,
            item.thickness_max,
            item.width_min,
            item.width_max,
            item.difficulty_level,
            item.difficulty_score
        ],
    )
    .map_err(|e| format!("更新工艺难度配置失败: {}", e))?;

    Ok(())
}

// ============================================
// Phase 8: 清洗规则管理
// ============================================

/// 获取所有清洗规则
pub fn list_transform_rules() -> Result<Vec<TransformRule>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT rule_id, rule_name, category, description, enabled, priority, config_json,
                    created_by, created_at, updated_at
             FROM transform_rules
             ORDER BY category, priority",
        )
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map([], |row| {
            Ok(TransformRule {
                rule_id: row.get(0)?,
                rule_name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                enabled: row.get(4)?,
                priority: row.get(5)?,
                config_json: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}

/// 按分类获取清洗规则
pub fn list_transform_rules_by_category(category: &str) -> Result<Vec<TransformRule>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT rule_id, rule_name, category, description, enabled, priority, config_json,
                    created_by, created_at, updated_at
             FROM transform_rules
             WHERE category = ?1
             ORDER BY priority",
        )
        .map_err(|e| e.to_string())?;

    let rules = stmt
        .query_map(params![category], |row| {
            Ok(TransformRule {
                rule_id: row.get(0)?,
                rule_name: row.get(1)?,
                category: row.get(2)?,
                description: row.get(3)?,
                enabled: row.get(4)?,
                priority: row.get(5)?,
                config_json: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(rules)
}

/// 获取单个清洗规则
pub fn get_transform_rule(rule_id: i64) -> Result<TransformRule, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT rule_id, rule_name, category, description, enabled, priority, config_json,
                    created_by, created_at, updated_at
             FROM transform_rules
             WHERE rule_id = ?1",
        )
        .map_err(|e| e.to_string())?;

    stmt.query_row(params![rule_id], |row| {
        Ok(TransformRule {
            rule_id: row.get(0)?,
            rule_name: row.get(1)?,
            category: row.get(2)?,
            description: row.get(3)?,
            enabled: row.get(4)?,
            priority: row.get(5)?,
            config_json: row.get(6)?,
            created_by: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    })
    .map_err(|e| format!("获取规则失败: {}", e))
}

/// 创建清洗规则
pub fn create_transform_rule(
    rule_name: &str,
    category: &str,
    description: Option<&str>,
    priority: i64,
    config_json: &str,
    created_by: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO transform_rules (rule_name, category, description, enabled, priority, config_json, created_by)
         VALUES (?1, ?2, ?3, 1, ?4, ?5, ?6)",
        params![rule_name, category, description, priority, config_json, created_by],
    )
    .map_err(|e| format!("创建规则失败: {}", e))?;

    let rule_id = conn.last_insert_rowid();

    // 记录变更日志
    conn.execute(
        "INSERT INTO transform_rule_change_log (rule_id, change_type, new_value, changed_by)
         VALUES (?1, 'create', ?2, ?3)",
        params![
            rule_id,
            serde_json::json!({
                "rule_name": rule_name,
                "category": category,
                "description": description,
                "priority": priority,
                "config_json": config_json
            })
            .to_string(),
            created_by
        ],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(rule_id)
}

/// 更新清洗规则
pub fn update_transform_rule(
    rule_id: i64,
    rule_name: &str,
    description: Option<&str>,
    priority: i64,
    config_json: &str,
    updated_by: &str,
    change_reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取旧值用于日志记录
    let old_rule = get_transform_rule(rule_id)?;
    let old_value = serde_json::json!({
        "rule_name": old_rule.rule_name,
        "description": old_rule.description,
        "priority": old_rule.priority,
        "config_json": old_rule.config_json
    });

    conn.execute(
        "UPDATE transform_rules
         SET rule_name = ?2, description = ?3, priority = ?4, config_json = ?5, updated_at = CURRENT_TIMESTAMP
         WHERE rule_id = ?1",
        params![rule_id, rule_name, description, priority, config_json],
    )
    .map_err(|e| format!("更新规则失败: {}", e))?;

    // 记录变更日志
    let new_value = serde_json::json!({
        "rule_name": rule_name,
        "description": description,
        "priority": priority,
        "config_json": config_json
    });

    conn.execute(
        "INSERT INTO transform_rule_change_log (rule_id, change_type, old_value, new_value, change_reason, changed_by)
         VALUES (?1, 'update', ?2, ?3, ?4, ?5)",
        params![rule_id, old_value.to_string(), new_value.to_string(), change_reason, updated_by],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(())
}

/// 删除清洗规则
pub fn delete_transform_rule(
    rule_id: i64,
    deleted_by: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取旧值用于日志记录
    let old_rule = get_transform_rule(rule_id)?;
    let old_value = serde_json::json!({
        "rule_name": old_rule.rule_name,
        "category": old_rule.category,
        "description": old_rule.description,
        "priority": old_rule.priority,
        "config_json": old_rule.config_json
    });

    // 记录变更日志（在删除前记录）
    conn.execute(
        "INSERT INTO transform_rule_change_log (rule_id, change_type, old_value, change_reason, changed_by)
         VALUES (?1, 'delete', ?2, ?3, ?4)",
        params![rule_id, old_value.to_string(), reason, deleted_by],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    conn.execute(
        "DELETE FROM transform_rules WHERE rule_id = ?1",
        params![rule_id],
    )
    .map_err(|e| format!("删除规则失败: {}", e))?;

    Ok(())
}

/// 切换规则启用/禁用状态
pub fn toggle_transform_rule_enabled(
    rule_id: i64,
    enabled: bool,
    updated_by: &str,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let enabled_value: i64 = if enabled { 1 } else { 0 };
    let change_type = if enabled { "enable" } else { "disable" };

    conn.execute(
        "UPDATE transform_rules SET enabled = ?2, updated_at = CURRENT_TIMESTAMP WHERE rule_id = ?1",
        params![rule_id, enabled_value],
    )
    .map_err(|e| format!("更新规则状态失败: {}", e))?;

    // 记录变更日志
    conn.execute(
        "INSERT INTO transform_rule_change_log (rule_id, change_type, old_value, new_value, changed_by)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            rule_id,
            change_type,
            if enabled { "disabled" } else { "enabled" },
            if enabled { "enabled" } else { "disabled" },
            updated_by
        ],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(())
}

/// 获取规则变更历史
pub fn list_transform_rule_change_log(
    rule_id: Option<i64>,
    limit: i64,
) -> Result<Vec<TransformRuleChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if rule_id.is_some() {
        "SELECT cl.change_id, cl.rule_id, r.rule_name, cl.change_type, cl.old_value, cl.new_value,
                cl.change_reason, cl.changed_by, cl.changed_at
         FROM transform_rule_change_log cl
         LEFT JOIN transform_rules r ON cl.rule_id = r.rule_id
         WHERE cl.rule_id = ?1
         ORDER BY cl.changed_at DESC
         LIMIT ?2"
    } else {
        "SELECT cl.change_id, cl.rule_id, r.rule_name, cl.change_type, cl.old_value, cl.new_value,
                cl.change_reason, cl.changed_by, cl.changed_at
         FROM transform_rule_change_log cl
         LEFT JOIN transform_rules r ON cl.rule_id = r.rule_id
         ORDER BY cl.changed_at DESC
         LIMIT ?2"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(rid) = rule_id {
        stmt.query_map(params![rid, limit], |row| {
            Ok(TransformRuleChangeLog {
                change_id: row.get(0)?,
                rule_id: row.get(1)?,
                rule_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(TransformRuleChangeLog {
                change_id: row.get(0)?,
                rule_id: row.get(1)?,
                rule_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(logs)
}

/// 记录规则执行日志
pub fn log_transform_execution(
    rule_id: i64,
    records_processed: i64,
    records_modified: i64,
    status: &str,
    error_message: Option<&str>,
    executed_by: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO transform_execution_log (rule_id, records_processed, records_modified, status, error_message, executed_by)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![rule_id, records_processed, records_modified, status, error_message, executed_by],
    )
    .map_err(|e| format!("记录执行日志失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取规则执行历史
pub fn list_transform_execution_log(
    rule_id: Option<i64>,
    limit: i64,
) -> Result<Vec<TransformExecutionLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if rule_id.is_some() {
        "SELECT el.log_id, el.rule_id, r.rule_name, el.execution_time, el.records_processed,
                el.records_modified, el.status, el.error_message, el.executed_by
         FROM transform_execution_log el
         LEFT JOIN transform_rules r ON el.rule_id = r.rule_id
         WHERE el.rule_id = ?1
         ORDER BY el.execution_time DESC
         LIMIT ?2"
    } else {
        "SELECT el.log_id, el.rule_id, r.rule_name, el.execution_time, el.records_processed,
                el.records_modified, el.status, el.error_message, el.executed_by
         FROM transform_execution_log el
         LEFT JOIN transform_rules r ON el.rule_id = r.rule_id
         ORDER BY el.execution_time DESC
         LIMIT ?2"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(rid) = rule_id {
        stmt.query_map(params![rid, limit], |row| {
            Ok(TransformExecutionLog {
                log_id: row.get(0)?,
                rule_id: row.get(1)?,
                rule_name: row.get(2)?,
                execution_time: row.get(3)?,
                records_processed: row.get(4)?,
                records_modified: row.get(5)?,
                status: row.get(6)?,
                error_message: row.get(7)?,
                executed_by: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(TransformExecutionLog {
                log_id: row.get(0)?,
                rule_id: row.get(1)?,
                rule_name: row.get(2)?,
                execution_time: row.get(3)?,
                records_processed: row.get(4)?,
                records_modified: row.get(5)?,
                status: row.get(6)?,
                error_message: row.get(7)?,
                executed_by: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(logs)
}

// ============================================
// Phase 9: 规格族管理操作
// ============================================

/// 获取所有规格族
pub fn list_spec_families() -> Result<Vec<SpecFamily>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT family_id, family_name, family_code, description, factor,
                    steel_grades, thickness_min, thickness_max, width_min, width_max,
                    enabled, sort_order, created_by, created_at, updated_at
             FROM spec_family_master
             ORDER BY sort_order, family_name",
        )
        .map_err(|e| e.to_string())?;

    let families = stmt
        .query_map([], |row| {
            Ok(SpecFamily {
                family_id: row.get(0)?,
                family_name: row.get(1)?,
                family_code: row.get(2)?,
                description: row.get(3)?,
                factor: row.get(4)?,
                steel_grades: row.get(5)?,
                thickness_min: row.get(6)?,
                thickness_max: row.get(7)?,
                width_min: row.get(8)?,
                width_max: row.get(9)?,
                enabled: row.get(10)?,
                sort_order: row.get(11)?,
                created_by: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(families)
}

/// 获取启用的规格族
pub fn list_enabled_spec_families() -> Result<Vec<SpecFamily>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT family_id, family_name, family_code, description, factor,
                    steel_grades, thickness_min, thickness_max, width_min, width_max,
                    enabled, sort_order, created_by, created_at, updated_at
             FROM spec_family_master
             WHERE enabled = 1
             ORDER BY sort_order, family_name",
        )
        .map_err(|e| e.to_string())?;

    let families = stmt
        .query_map([], |row| {
            Ok(SpecFamily {
                family_id: row.get(0)?,
                family_name: row.get(1)?,
                family_code: row.get(2)?,
                description: row.get(3)?,
                factor: row.get(4)?,
                steel_grades: row.get(5)?,
                thickness_min: row.get(6)?,
                thickness_max: row.get(7)?,
                width_min: row.get(8)?,
                width_max: row.get(9)?,
                enabled: row.get(10)?,
                sort_order: row.get(11)?,
                created_by: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(families)
}

/// 获取单个规格族
pub fn get_spec_family(family_id: i64) -> Result<SpecFamily, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let family = conn
        .query_row(
            "SELECT family_id, family_name, family_code, description, factor,
                    steel_grades, thickness_min, thickness_max, width_min, width_max,
                    enabled, sort_order, created_by, created_at, updated_at
             FROM spec_family_master
             WHERE family_id = ?",
            params![family_id],
            |row| {
                Ok(SpecFamily {
                    family_id: row.get(0)?,
                    family_name: row.get(1)?,
                    family_code: row.get(2)?,
                    description: row.get(3)?,
                    factor: row.get(4)?,
                    steel_grades: row.get(5)?,
                    thickness_min: row.get(6)?,
                    thickness_max: row.get(7)?,
                    width_min: row.get(8)?,
                    width_max: row.get(9)?,
                    enabled: row.get(10)?,
                    sort_order: row.get(11)?,
                    created_by: row.get(12)?,
                    created_at: row.get(13)?,
                    updated_at: row.get(14)?,
                })
            },
        )
        .map_err(|e| format!("规格族不存在: {}", e))?;

    Ok(family)
}

/// 根据名称或代码获取规格族
#[allow(dead_code)]
pub fn get_spec_family_by_name_or_code(name_or_code: &str) -> Result<Option<SpecFamily>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT family_id, family_name, family_code, description, factor,
                    steel_grades, thickness_min, thickness_max, width_min, width_max,
                    enabled, sort_order, created_by, created_at, updated_at
             FROM spec_family_master
             WHERE family_name = ? OR family_code = ?",
        params![name_or_code, name_or_code],
        |row| {
            Ok(SpecFamily {
                family_id: row.get(0)?,
                family_name: row.get(1)?,
                family_code: row.get(2)?,
                description: row.get(3)?,
                factor: row.get(4)?,
                steel_grades: row.get(5)?,
                thickness_min: row.get(6)?,
                thickness_max: row.get(7)?,
                width_min: row.get(8)?,
                width_max: row.get(9)?,
                enabled: row.get(10)?,
                sort_order: row.get(11)?,
                created_by: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        },
    );

    match result {
        Ok(family) => Ok(Some(family)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 创建规格族
pub fn create_spec_family(
    family_name: &str,
    family_code: &str,
    description: Option<&str>,
    factor: f64,
    steel_grades: Option<&str>,
    thickness_min: Option<f64>,
    thickness_max: Option<f64>,
    width_min: Option<f64>,
    width_max: Option<f64>,
    sort_order: i64,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 插入规格族
    conn.execute(
        "INSERT INTO spec_family_master
            (family_name, family_code, description, factor, steel_grades,
             thickness_min, thickness_max, width_min, width_max,
             sort_order, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            family_name,
            family_code,
            description,
            factor,
            steel_grades,
            thickness_min,
            thickness_max,
            width_min,
            width_max,
            sort_order,
            user
        ],
    )
    .map_err(|e| format!("创建规格族失败: {}", e))?;

    let family_id = conn.last_insert_rowid();

    // 记录变更日志
    let new_value = serde_json::json!({
        "family_name": family_name,
        "family_code": family_code,
        "description": description,
        "factor": factor,
        "steel_grades": steel_grades,
        "thickness_min": thickness_min,
        "thickness_max": thickness_max,
        "width_min": width_min,
        "width_max": width_max,
        "sort_order": sort_order
    });

    conn.execute(
        "INSERT INTO spec_family_change_log
            (family_id, family_name, change_type, new_value, changed_by, change_reason)
         VALUES (?, ?, 'create', ?, ?, '新建规格族')",
        params![family_id, family_name, new_value.to_string(), user],
    )
    .map_err(|e| e.to_string())?;

    Ok(family_id)
}

/// 更新规格族
pub fn update_spec_family(
    family_id: i64,
    family_name: &str,
    family_code: &str,
    description: Option<&str>,
    factor: f64,
    steel_grades: Option<&str>,
    thickness_min: Option<f64>,
    thickness_max: Option<f64>,
    width_min: Option<f64>,
    width_max: Option<f64>,
    sort_order: i64,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取旧值用于日志
    let old_family = get_spec_family(family_id)?;
    let old_value = serde_json::json!({
        "family_name": old_family.family_name,
        "family_code": old_family.family_code,
        "description": old_family.description,
        "factor": old_family.factor,
        "steel_grades": old_family.steel_grades,
        "thickness_min": old_family.thickness_min,
        "thickness_max": old_family.thickness_max,
        "width_min": old_family.width_min,
        "width_max": old_family.width_max,
        "sort_order": old_family.sort_order
    });

    // 更新规格族
    conn.execute(
        "UPDATE spec_family_master SET
            family_name = ?, family_code = ?, description = ?, factor = ?,
            steel_grades = ?, thickness_min = ?, thickness_max = ?,
            width_min = ?, width_max = ?, sort_order = ?,
            updated_at = datetime('now','localtime')
         WHERE family_id = ?",
        params![
            family_name,
            family_code,
            description,
            factor,
            steel_grades,
            thickness_min,
            thickness_max,
            width_min,
            width_max,
            sort_order,
            family_id
        ],
    )
    .map_err(|e| format!("更新规格族失败: {}", e))?;

    // 记录变更日志
    let new_value = serde_json::json!({
        "family_name": family_name,
        "family_code": family_code,
        "description": description,
        "factor": factor,
        "steel_grades": steel_grades,
        "thickness_min": thickness_min,
        "thickness_max": thickness_max,
        "width_min": width_min,
        "width_max": width_max,
        "sort_order": sort_order
    });

    conn.execute(
        "INSERT INTO spec_family_change_log
            (family_id, family_name, change_type, old_value, new_value, changed_by, change_reason)
         VALUES (?, ?, 'update', ?, ?, ?, ?)",
        params![
            family_id,
            family_name,
            old_value.to_string(),
            new_value.to_string(),
            user,
            reason
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 删除规格族
pub fn delete_spec_family(family_id: i64, user: &str, reason: Option<&str>) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取旧值用于日志
    let old_family = get_spec_family(family_id)?;
    let old_value = serde_json::json!({
        "family_name": old_family.family_name,
        "family_code": old_family.family_code,
        "description": old_family.description,
        "factor": old_family.factor
    });

    // 记录变更日志（在删除之前）
    conn.execute(
        "INSERT INTO spec_family_change_log
            (family_id, family_name, change_type, old_value, changed_by, change_reason)
         VALUES (?, ?, 'delete', ?, ?, ?)",
        params![
            family_id,
            old_family.family_name,
            old_value.to_string(),
            user,
            reason
        ],
    )
    .map_err(|e| e.to_string())?;

    // 删除规格族
    conn.execute(
        "DELETE FROM spec_family_master WHERE family_id = ?",
        params![family_id],
    )
    .map_err(|e| format!("删除规格族失败: {}", e))?;

    Ok(())
}

/// 切换规格族启用/禁用状态
pub fn toggle_spec_family_enabled(family_id: i64, enabled: bool, user: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let enabled_int = if enabled { 1 } else { 0 };
    let change_type = if enabled { "enable" } else { "disable" };

    // 获取当前状态
    let family = get_spec_family(family_id)?;

    // 更新状态
    conn.execute(
        "UPDATE spec_family_master SET enabled = ?, updated_at = datetime('now','localtime')
         WHERE family_id = ?",
        params![enabled_int, family_id],
    )
    .map_err(|e| format!("切换状态失败: {}", e))?;

    // 记录变更日志
    conn.execute(
        "INSERT INTO spec_family_change_log
            (family_id, family_name, change_type, old_value, new_value, changed_by, change_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![
            family_id,
            family.family_name,
            change_type,
            format!("{{\"enabled\": {}}}", family.enabled),
            format!("{{\"enabled\": {}}}", enabled_int),
            user,
            if enabled {
                "启用规格族"
            } else {
                "禁用规格族"
            }
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 获取规格族变更历史
pub fn list_spec_family_change_log(
    family_id: Option<i64>,
    limit: i64,
) -> Result<Vec<SpecFamilyChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if family_id.is_some() {
        "SELECT change_id, family_id, family_name, change_type,
                old_value, new_value, change_reason, changed_by, changed_at
         FROM spec_family_change_log
         WHERE family_id = ?1
         ORDER BY changed_at DESC
         LIMIT ?2"
    } else {
        "SELECT change_id, family_id, family_name, change_type,
                old_value, new_value, change_reason, changed_by, changed_at
         FROM spec_family_change_log
         ORDER BY changed_at DESC
         LIMIT ?2"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(fid) = family_id {
        stmt.query_map(params![fid, limit], |row| {
            Ok(SpecFamilyChangeLog {
                change_id: row.get(0)?,
                family_id: row.get(1)?,
                family_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(SpecFamilyChangeLog {
                change_id: row.get(0)?,
                family_id: row.get(1)?,
                family_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(logs)
}

/// 获取规格族系数（用于评分计算）
#[allow(dead_code)]
pub fn get_spec_family_factor(family_name_or_code: &str) -> Result<f64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let factor = conn.query_row(
        "SELECT factor FROM spec_family_master
             WHERE (family_name = ? OR family_code = ?) AND enabled = 1",
        params![family_name_or_code, family_name_or_code],
        |row| row.get::<_, f64>(0),
    );

    match factor {
        Ok(f) => Ok(f),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(1.0), // 默认系数
        Err(e) => Err(e.to_string()),
    }
}

// ============================================
// Phase 10: n日节拍配置管理
// ============================================

/// 获取所有节拍配置
pub fn list_rhythm_configs() -> Result<Vec<RhythmConfig>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT config_id, config_name, cycle_days, description, is_active, created_by, created_at, updated_at
             FROM rhythm_config
             ORDER BY is_active DESC, config_name"
        )
        .map_err(|e| e.to_string())?;

    let configs = stmt
        .query_map([], |row| {
            Ok(RhythmConfig {
                config_id: row.get(0)?,
                config_name: row.get(1)?,
                cycle_days: row.get(2)?,
                description: row.get(3)?,
                is_active: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(configs)
}

/// 获取单个节拍配置
pub fn get_rhythm_config(config_id: i64) -> Result<RhythmConfig, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT config_id, config_name, cycle_days, description, is_active, created_by, created_at, updated_at
         FROM rhythm_config WHERE config_id = ?",
        params![config_id],
        |row| {
            Ok(RhythmConfig {
                config_id: row.get(0)?,
                config_name: row.get(1)?,
                cycle_days: row.get(2)?,
                description: row.get(3)?,
                is_active: row.get(4)?,
                created_by: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    )
    .map_err(|e| format!("节拍配置不存在: {}", e))
}

/// 创建节拍配置
pub fn create_rhythm_config(
    config_name: &str,
    cycle_days: i32,
    description: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 验证周期天数
    if cycle_days < 1 || cycle_days > 30 {
        return Err(format!("周期天数必须在 1-30 之间，当前值: {}", cycle_days));
    }

    conn.execute(
        "INSERT INTO rhythm_config (config_name, cycle_days, description, is_active, created_by)
         VALUES (?, ?, ?, 0, ?)",
        params![config_name, cycle_days, description, user],
    )
    .map_err(|e| format!("创建节拍配置失败: {}", e))?;

    let config_id = conn.last_insert_rowid();

    // 记录变更日志
    let new_value = serde_json::json!({
        "config_name": config_name,
        "cycle_days": cycle_days,
        "description": description
    });

    conn.execute(
        "INSERT INTO rhythm_config_change_log
            (config_id, config_name, change_type, new_value, changed_by, change_reason)
         VALUES (?, ?, 'create', ?, ?, '新建节拍配置')",
        params![config_id, config_name, new_value.to_string(), user],
    )
    .map_err(|e| e.to_string())?;

    Ok(config_id)
}

/// 更新节拍配置
pub fn update_rhythm_config(
    config_id: i64,
    config_name: &str,
    cycle_days: i32,
    description: Option<&str>,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 验证周期天数
    if cycle_days < 1 || cycle_days > 30 {
        return Err(format!("周期天数必须在 1-30 之间，当前值: {}", cycle_days));
    }

    // 获取旧值
    let old_config = get_rhythm_config(config_id)?;
    let old_value = serde_json::json!({
        "config_name": old_config.config_name,
        "cycle_days": old_config.cycle_days,
        "description": old_config.description
    });

    conn.execute(
        "UPDATE rhythm_config SET config_name = ?, cycle_days = ?, description = ?, updated_at = datetime('now','localtime')
         WHERE config_id = ?",
        params![config_name, cycle_days, description, config_id],
    )
    .map_err(|e| format!("更新节拍配置失败: {}", e))?;

    // 记录变更日志
    let new_value = serde_json::json!({
        "config_name": config_name,
        "cycle_days": cycle_days,
        "description": description
    });

    conn.execute(
        "INSERT INTO rhythm_config_change_log
            (config_id, config_name, change_type, old_value, new_value, changed_by, change_reason)
         VALUES (?, ?, 'update', ?, ?, ?, ?)",
        params![
            config_id,
            config_name,
            old_value.to_string(),
            new_value.to_string(),
            user,
            reason
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 删除节拍配置
pub fn delete_rhythm_config(
    config_id: i64,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取配置信息
    let config = get_rhythm_config(config_id)?;

    // 不允许删除激活的配置
    if config.is_active == 1 {
        return Err("不能删除激活状态的节拍配置".to_string());
    }

    let old_value = serde_json::json!({
        "config_name": config.config_name,
        "cycle_days": config.cycle_days,
        "description": config.description
    });

    // 记录变更日志（在删除之前）
    conn.execute(
        "INSERT INTO rhythm_config_change_log
            (config_id, config_name, change_type, old_value, changed_by, change_reason)
         VALUES (?, ?, 'delete', ?, ?, ?)",
        params![
            config_id,
            config.config_name,
            old_value.to_string(),
            user,
            reason
        ],
    )
    .map_err(|e| e.to_string())?;

    // 删除关联的节拍标签
    conn.execute(
        "DELETE FROM rhythm_label WHERE config_id = ?",
        params![config_id],
    )
    .map_err(|e| format!("删除关联节拍标签失败: {}", e))?;

    // 删除配置
    conn.execute(
        "DELETE FROM rhythm_config WHERE config_id = ?",
        params![config_id],
    )
    .map_err(|e| format!("删除节拍配置失败: {}", e))?;

    Ok(())
}

/// 激活节拍配置（同时禁用其他配置）
pub fn activate_rhythm_config(config_id: i64, user: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取配置信息
    let config = get_rhythm_config(config_id)?;

    // 已经是激活状态
    if config.is_active == 1 {
        return Ok(());
    }

    // 获取当前激活的配置（如果有）
    let old_active: Option<(i64, String)> = conn
        .query_row(
            "SELECT config_id, config_name FROM rhythm_config WHERE is_active = 1",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .ok();

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 禁用所有配置
    if let Err(e) = conn.execute(
        "UPDATE rhythm_config SET is_active = 0, updated_at = datetime('now','localtime')",
        [],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("禁用配置失败: {}", e));
    }

    // 激活指定配置
    if let Err(e) = conn.execute(
        "UPDATE rhythm_config SET is_active = 1, updated_at = datetime('now','localtime') WHERE config_id = ?",
        params![config_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("激活配置失败: {}", e));
    }

    // 记录禁用旧配置的日志
    if let Some((old_id, old_name)) = old_active {
        if let Err(e) = conn.execute(
            "INSERT INTO rhythm_config_change_log
                (config_id, config_name, change_type, old_value, new_value, changed_by, change_reason)
             VALUES (?, ?, 'deactivate', '{\"is_active\": 1}', '{\"is_active\": 0}', ?, '切换到其他配置')",
            params![old_id, old_name, user],
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("记录变更日志失败: {}", e));
        }
    }

    // 记录激活新配置的日志
    if let Err(e) = conn.execute(
        "INSERT INTO rhythm_config_change_log
            (config_id, config_name, change_type, old_value, new_value, changed_by, change_reason)
         VALUES (?, ?, 'activate', '{\"is_active\": 0}', '{\"is_active\": 1}', ?, '激活节拍配置')",
        params![config_id, config.config_name, user],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("记录变更日志失败: {}", e));
    }

    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(())
}

/// 获取节拍配置的标签列表
pub fn list_rhythm_labels(config_id: i64) -> Result<Vec<RhythmLabel>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT id, config_id, rhythm_day, label_name, match_spec, bonus_score, description
             FROM rhythm_label
             WHERE config_id = ?
             ORDER BY rhythm_day",
        )
        .map_err(|e| e.to_string())?;

    let labels = stmt
        .query_map(params![config_id], |row| {
            Ok(RhythmLabel {
                id: row.get(0)?,
                config_id: row.get(1)?,
                rhythm_day: row.get(2)?,
                label_name: row.get(3)?,
                match_spec: row.get(4)?,
                bonus_score: row.get(5)?,
                description: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(labels)
}

/// 创建或更新节拍标签
pub fn upsert_rhythm_label(
    config_id: i64,
    rhythm_day: i32,
    label_name: &str,
    match_spec: Option<&str>,
    bonus_score: f64,
    description: Option<&str>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取配置的周期天数
    let config = get_rhythm_config(config_id)?;
    if rhythm_day < 1 || rhythm_day > config.cycle_days {
        return Err(format!(
            "周期日必须在 1-{} 之间，当前值: {}",
            config.cycle_days, rhythm_day
        ));
    }

    conn.execute(
        "INSERT INTO rhythm_label (config_id, rhythm_day, label_name, match_spec, bonus_score, description)
         VALUES (?, ?, ?, ?, ?, ?)
         ON CONFLICT(config_id, rhythm_day) DO UPDATE SET
         label_name = excluded.label_name,
         match_spec = excluded.match_spec,
         bonus_score = excluded.bonus_score,
         description = excluded.description",
        params![config_id, rhythm_day, label_name, match_spec, bonus_score, description],
    )
    .map_err(|e| format!("保存节拍标签失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 删除节拍标签
pub fn delete_rhythm_label(label_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute("DELETE FROM rhythm_label WHERE id = ?", params![label_id])
        .map_err(|e| format!("删除节拍标签失败: {}", e))?;

    Ok(())
}

/// 获取节拍配置变更历史
pub fn list_rhythm_config_change_log(
    config_id: Option<i64>,
    limit: i64,
) -> Result<Vec<RhythmConfigChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if config_id.is_some() {
        "SELECT change_id, config_id, config_name, change_type,
                old_value, new_value, change_reason, changed_by, changed_at
         FROM rhythm_config_change_log
         WHERE config_id = ?1
         ORDER BY changed_at DESC
         LIMIT ?2"
    } else {
        "SELECT change_id, config_id, config_name, change_type,
                old_value, new_value, change_reason, changed_by, changed_at
         FROM rhythm_config_change_log
         ORDER BY changed_at DESC
         LIMIT ?2"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(cid) = config_id {
        stmt.query_map(params![cid, limit], |row| {
            Ok(RhythmConfigChangeLog {
                change_id: row.get(0)?,
                config_id: row.get(1)?,
                config_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(RhythmConfigChangeLog {
                change_id: row.get(0)?,
                config_id: row.get(1)?,
                config_name: row.get(2)?,
                change_type: row.get(3)?,
                old_value: row.get(4)?,
                new_value: row.get(5)?,
                change_reason: row.get(6)?,
                changed_by: row.get(7)?,
                changed_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(logs)
}

// ============================================
// Phase 12: 聚合度配置管理
// ============================================

/// 获取所有聚合区间配置
pub fn list_aggregation_bins() -> Result<Vec<AggregationBin>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT bin_id, dimension, bin_name, bin_code, min_value, max_value,
                    sort_order, enabled, description, created_at, updated_at
             FROM aggregation_bins
             WHERE enabled = 1
             ORDER BY dimension, sort_order",
        )
        .map_err(|e| e.to_string())?;

    let bins = stmt
        .query_map([], |row| {
            Ok(AggregationBin {
                bin_id: row.get(0)?,
                dimension: row.get(1)?,
                bin_name: row.get(2)?,
                bin_code: row.get(3)?,
                min_value: row.get(4)?,
                max_value: row.get(5)?,
                sort_order: row.get(6)?,
                enabled: row.get(7)?,
                description: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(bins)
}

/// 获取某一维度的聚合区间配置
pub fn list_aggregation_bins_by_dimension(dimension: &str) -> Result<Vec<AggregationBin>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT bin_id, dimension, bin_name, bin_code, min_value, max_value,
                    sort_order, enabled, description, created_at, updated_at
             FROM aggregation_bins
             WHERE dimension = ? AND enabled = 1
             ORDER BY sort_order",
        )
        .map_err(|e| e.to_string())?;

    let bins = stmt
        .query_map(params![dimension], |row| {
            Ok(AggregationBin {
                bin_id: row.get(0)?,
                dimension: row.get(1)?,
                bin_name: row.get(2)?,
                bin_code: row.get(3)?,
                min_value: row.get(4)?,
                max_value: row.get(5)?,
                sort_order: row.get(6)?,
                enabled: row.get(7)?,
                description: row.get(8)?,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(bins)
}

/// 根据值查找对应的区间代码
///
/// # 参数
/// - dimension: 维度类型，'width' 或 'thickness'
/// - value: 要匹配的数值
///
/// # 返回
/// (bin_code, bin_name)
pub fn get_bin_for_value(dimension: &str, value: f64) -> Result<(String, String), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT bin_code, bin_name FROM aggregation_bins
         WHERE dimension = ? AND enabled = 1
           AND min_value <= ? AND max_value > ?
         ORDER BY sort_order
         LIMIT 1",
        params![dimension, value, value],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    match result {
        Ok((code, name)) => Ok((code, name)),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // 未找到匹配的区间，使用默认值
            Ok(("UNKNOWN".to_string(), "未知".to_string()))
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 获取 P2 曲线参数配置
pub fn get_p2_curve_config() -> Result<P2CurveConfig, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT config_id, curve_type, log_base, log_scale, min_score, max_score,
                min_count_for_max, alpha, beta, description, updated_by, updated_at
         FROM p2_curve_config
         WHERE config_id = 1",
        [],
        |row| {
            Ok(P2CurveConfig {
                config_id: row.get(0)?,
                curve_type: row.get(1)?,
                log_base: row.get(2)?,
                log_scale: row.get(3)?,
                min_score: row.get(4)?,
                max_score: row.get(5)?,
                min_count_for_max: row.get(6)?,
                alpha: row.get(7)?,
                beta: row.get(8)?,
                description: row.get(9)?,
                updated_by: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    );

    match result {
        Ok(config) => Ok(config),
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // 返回默认配置
            Ok(P2CurveConfig::default())
        }
        Err(e) => Err(e.to_string()),
    }
}

/// 更新 P2 曲线参数配置
#[allow(dead_code)]
pub fn update_p2_curve_config(
    curve_type: &str,
    log_base: f64,
    log_scale: f64,
    min_score: f64,
    max_score: f64,
    min_count_for_max: i64,
    alpha: f64,
    beta: f64,
    description: Option<&str>,
    user: &str,
) -> Result<(), String> {
    // 验证 alpha + beta = 1.0
    if (alpha + beta - 1.0).abs() > 0.001 {
        return Err(format!(
            "alpha + beta 必须等于 1.0，当前值: {} + {} = {}",
            alpha,
            beta,
            alpha + beta
        ));
    }

    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取旧值用于日志
    let old_config = get_p2_curve_config()?;
    let old_value = serde_json::json!({
        "curve_type": old_config.curve_type,
        "log_base": old_config.log_base,
        "log_scale": old_config.log_scale,
        "alpha": old_config.alpha,
        "beta": old_config.beta
    });

    conn.execute(
        "INSERT INTO p2_curve_config (config_id, curve_type, log_base, log_scale, min_score, max_score,
                                      min_count_for_max, alpha, beta, description, updated_by, updated_at)
         VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, datetime('now','localtime'))
         ON CONFLICT(config_id) DO UPDATE SET
         curve_type = excluded.curve_type,
         log_base = excluded.log_base,
         log_scale = excluded.log_scale,
         min_score = excluded.min_score,
         max_score = excluded.max_score,
         min_count_for_max = excluded.min_count_for_max,
         alpha = excluded.alpha,
         beta = excluded.beta,
         description = excluded.description,
         updated_by = excluded.updated_by,
         updated_at = excluded.updated_at",
        params![curve_type, log_base, log_scale, min_score, max_score, min_count_for_max, alpha, beta, description, user],
    )
    .map_err(|e| format!("更新 P2 曲线配置失败: {}", e))?;

    // 记录变更日志
    let new_value = serde_json::json!({
        "curve_type": curve_type,
        "log_base": log_base,
        "log_scale": log_scale,
        "alpha": alpha,
        "beta": beta
    });

    conn.execute(
        "INSERT INTO aggregation_config_change_log
            (table_name, record_id, change_type, old_value, new_value, changed_by, change_reason)
         VALUES ('p2_curve_config', 1, 'update', ?, ?, ?, '更新P2曲线参数')",
        params![old_value.to_string(), new_value.to_string(), user],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

/// 计算合同的聚合统计信息
///
/// 返回合同所属聚合键的同类合同数量
/// 聚合键 = 规格族 + 钢种 + 厚度段 + 宽度段
pub fn get_contract_aggregation_stats(
    spec_family: &str,
    steel_grade: &str,
    thickness: f64,
    width: f64,
) -> Result<AggregationStats, String> {
    // 获取厚度段和宽度段
    let (thickness_bin_code, thickness_bin_name) = get_bin_for_value("thickness", thickness)?;
    let (width_bin_code, width_bin_name) = get_bin_for_value("width", width)?;

    // 构建聚合键
    let aggregation_key = format!(
        "{}|{}|{}|{}",
        spec_family, steel_grade, thickness_bin_code, width_bin_code
    );

    let conn = get_connection().map_err(|e| e.to_string())?;

    // 统计同聚合键的合同数量
    // 这里需要动态计算，因为合同池会变化
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM contract_master c
             WHERE c.spec_family = ?
               AND c.steel_grade = ?
               AND EXISTS (
                   SELECT 1 FROM aggregation_bins b
                   WHERE b.dimension = 'thickness' AND b.enabled = 1
                     AND b.min_value <= c.thickness AND b.max_value > c.thickness
                     AND b.bin_code = ?
               )
               AND EXISTS (
                   SELECT 1 FROM aggregation_bins b
                   WHERE b.dimension = 'width' AND b.enabled = 1
                     AND b.min_value <= c.width AND b.max_value > c.width
                     AND b.bin_code = ?
               )",
            params![spec_family, steel_grade, thickness_bin_code, width_bin_code],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;

    Ok(AggregationStats {
        aggregation_key,
        spec_family: spec_family.to_string(),
        steel_grade: steel_grade.to_string(),
        thickness_bin_code,
        thickness_bin_name,
        width_bin_code,
        width_bin_name,
        contract_count: count,
    })
}

/// 批量获取所有合同的聚合统计信息
///
/// 返回一个 HashMap，key 为合同 ID，value 为聚合统计信息
pub fn get_all_contracts_aggregation_stats(
) -> Result<std::collections::HashMap<String, AggregationStats>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取所有启用的区间配置
    let thickness_bins = list_aggregation_bins_by_dimension("thickness")?;
    let width_bins = list_aggregation_bins_by_dimension("width")?;

    // 为每个合同计算聚合键
    let mut stmt = conn
        .prepare(
            "SELECT contract_id, spec_family, steel_grade, thickness, width FROM contract_master",
        )
        .map_err(|e| e.to_string())?;

    let contracts: Vec<(String, String, String, f64, f64)> = stmt
        .query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 辅助函数：查找值对应的区间
    fn find_bin(bins: &[AggregationBin], value: f64) -> (String, String) {
        for bin in bins {
            if bin.min_value <= value && bin.max_value > value {
                return (bin.bin_code.clone(), bin.bin_name.clone());
            }
        }
        ("UNKNOWN".to_string(), "未知".to_string())
    }

    // 构建聚合键到合同列表的映射
    let mut aggregation_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    let mut contract_to_key: std::collections::HashMap<
        String,
        (String, String, String, String, String, String, String),
    > = std::collections::HashMap::new();

    for (contract_id, spec_family, steel_grade, thickness, width) in &contracts {
        let (thickness_bin_code, thickness_bin_name) = find_bin(&thickness_bins, *thickness);
        let (width_bin_code, width_bin_name) = find_bin(&width_bins, *width);
        let aggregation_key = format!(
            "{}|{}|{}|{}",
            spec_family, steel_grade, thickness_bin_code, width_bin_code
        );

        aggregation_map
            .entry(aggregation_key.clone())
            .or_default()
            .push(contract_id.clone());

        contract_to_key.insert(
            contract_id.clone(),
            (
                aggregation_key,
                spec_family.clone(),
                steel_grade.clone(),
                thickness_bin_code,
                thickness_bin_name,
                width_bin_code,
                width_bin_name,
            ),
        );
    }

    // 构建最终结果
    let mut result: std::collections::HashMap<String, AggregationStats> =
        std::collections::HashMap::new();

    for (contract_id, (key, spec_family, steel_grade, t_code, t_name, w_code, w_name)) in
        contract_to_key
    {
        let count = aggregation_map
            .get(&key)
            .map(|v| v.len() as i64)
            .unwrap_or(0);

        result.insert(
            contract_id,
            AggregationStats {
                aggregation_key: key,
                spec_family,
                steel_grade,
                thickness_bin_code: t_code,
                thickness_bin_name: t_name,
                width_bin_code: w_code,
                width_bin_name: w_name,
                contract_count: count,
            },
        );
    }

    Ok(result)
}

/// 刷新聚合统计缓存
///
/// 将所有合同的聚合统计信息写入缓存表，供快速查询
#[allow(dead_code)]
pub fn refresh_aggregation_stats_cache() -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 清空缓存
    conn.execute("DELETE FROM aggregation_stats_cache", [])
        .map_err(|e| format!("清空缓存失败: {}", e))?;

    // 获取所有合同的聚合统计
    let all_stats = get_all_contracts_aggregation_stats()?;

    // 按聚合键分组
    let mut key_to_contracts: std::collections::HashMap<String, (AggregationStats, Vec<String>)> =
        std::collections::HashMap::new();

    for (contract_id, stats) in all_stats {
        key_to_contracts
            .entry(stats.aggregation_key.clone())
            .or_insert_with(|| (stats.clone(), Vec::new()))
            .1
            .push(contract_id);
    }

    // 写入缓存
    let mut count = 0i64;
    for (key, (stats, contract_ids)) in key_to_contracts {
        let contract_ids_json = serde_json::to_string(&contract_ids).unwrap_or_default();

        conn.execute(
            "INSERT INTO aggregation_stats_cache
                (aggregation_key, spec_family, steel_grade, thickness_bin, width_bin, contract_count, contract_ids)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
            params![
                key,
                stats.spec_family,
                stats.steel_grade,
                stats.thickness_bin_code,
                stats.width_bin_code,
                stats.contract_count,
                contract_ids_json
            ],
        )
        .map_err(|e| format!("写入缓存失败: {}", e))?;

        count += 1;
    }

    Ok(count)
}

/// 从缓存中获取聚合统计
#[allow(dead_code)]
pub fn get_aggregation_stats_from_cache(
    aggregation_key: &str,
) -> Result<Option<AggregationStatsCache>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT cache_id, aggregation_key, spec_family, steel_grade, thickness_bin, width_bin,
                contract_count, contract_ids, last_updated
         FROM aggregation_stats_cache
         WHERE aggregation_key = ?",
        params![aggregation_key],
        |row| {
            Ok(AggregationStatsCache {
                cache_id: row.get(0)?,
                aggregation_key: row.get(1)?,
                spec_family: row.get(2)?,
                steel_grade: row.get(3)?,
                thickness_bin: row.get(4)?,
                width_bin: row.get(5)?,
                contract_count: row.get(6)?,
                contract_ids: row.get(7)?,
                last_updated: row.get(8)?,
            })
        },
    );

    match result {
        Ok(cache) => Ok(Some(cache)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 获取聚合统计汇总（按规格族分组）
#[allow(dead_code)]
pub fn get_aggregation_summary() -> Result<Vec<serde_json::Value>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT spec_family, thickness_bin, width_bin, SUM(contract_count) as total_count
             FROM aggregation_stats_cache
             GROUP BY spec_family, thickness_bin, width_bin
             ORDER BY spec_family, thickness_bin, width_bin",
        )
        .map_err(|e| e.to_string())?;

    let summaries: Vec<serde_json::Value> = stmt
        .query_map([], |row| {
            let spec_family: String = row.get(0)?;
            let thickness_bin: String = row.get(1)?;
            let width_bin: String = row.get(2)?;
            let total_count: i64 = row.get(3)?;

            Ok(serde_json::json!({
                "spec_family": spec_family,
                "thickness_bin": thickness_bin,
                "width_bin": width_bin,
                "total_count": total_count
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(summaries)
}

// ============================================
// Phase 13: 数据校验与缺失值策略
// ============================================

/// 获取所有缺失值策略配置
pub fn list_missing_value_strategies() -> Result<Vec<MissingValueStrategy>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT strategy_id, field_name, field_label, module, is_required,
                    strategy, default_value, default_description, affects_score, description, sort_order
             FROM missing_value_strategy
             ORDER BY sort_order"
        )
        .map_err(|e| e.to_string())?;

    let strategies = stmt
        .query_map([], |row| {
            Ok(MissingValueStrategy {
                strategy_id: row.get(0)?,
                field_name: row.get(1)?,
                field_label: row.get(2)?,
                module: row.get(3)?,
                is_required: row.get::<_, i64>(4)? == 1,
                strategy: row.get(5)?,
                default_value: row.get(6)?,
                default_description: row.get(7)?,
                affects_score: row.get(8)?,
                description: row.get(9)?,
                sort_order: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(strategies)
}

/// 获取单个字段的缺失值策略
#[allow(dead_code)]
pub fn get_missing_value_strategy(
    field_name: &str,
) -> Result<Option<MissingValueStrategy>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT strategy_id, field_name, field_label, module, is_required,
                strategy, default_value, default_description, affects_score, description, sort_order
         FROM missing_value_strategy
         WHERE field_name = ?",
        params![field_name],
        |row| {
            Ok(MissingValueStrategy {
                strategy_id: row.get(0)?,
                field_name: row.get(1)?,
                field_label: row.get(2)?,
                module: row.get(3)?,
                is_required: row.get::<_, i64>(4)? == 1,
                strategy: row.get(5)?,
                default_value: row.get(6)?,
                default_description: row.get(7)?,
                affects_score: row.get(8)?,
                description: row.get(9)?,
                sort_order: row.get(10)?,
            })
        },
    );

    match result {
        Ok(strategy) => Ok(Some(strategy)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 更新缺失值策略
pub fn update_missing_value_strategy(
    field_name: &str,
    strategy: &str,
    default_value: Option<&str>,
    default_description: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE missing_value_strategy
         SET strategy = ?, default_value = ?, default_description = ?, updated_at = datetime('now','localtime')
         WHERE field_name = ?",
        params![strategy, default_value, default_description, field_name],
    )
    .map_err(|e| format!("更新缺失值策略失败: {}", e))?;

    Ok(())
}

/// 记录校验日志
#[allow(dead_code)]
pub fn log_validation(
    batch_id: &str,
    total_contracts: i64,
    valid_contracts: i64,
    warning_contracts: i64,
    error_contracts: i64,
    details_json: Option<&str>,
    validated_by: Option<&str>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO validation_log (batch_id, total_contracts, valid_contracts, warning_contracts, error_contracts, details_json, validated_by)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
        params![batch_id, total_contracts, valid_contracts, warning_contracts, error_contracts, details_json, validated_by],
    )
    .map_err(|e| format!("记录校验日志失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 记录合同校验问题
#[allow(dead_code)]
pub fn log_contract_validation_issues(
    batch_id: &str,
    issues: &[(
        String,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        String,
        Option<String>,
    )],
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    for (
        contract_id,
        field_name,
        issue_type,
        severity,
        original_value,
        default_value_used,
        message,
        suggested_fix,
    ) in issues
    {
        conn.execute(
            "INSERT INTO contract_validation_issues
                (batch_id, contract_id, field_name, issue_type, severity, original_value, default_value_used, message, suggested_fix)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![batch_id, contract_id, field_name, issue_type, severity, original_value, default_value_used, message, suggested_fix],
        )
        .map_err(|e| format!("记录校验问题失败: {}", e))?;
    }

    Ok(())
}

/// 获取最近的校验日志
#[allow(dead_code)]
pub fn get_recent_validation_logs(limit: i64) -> Result<Vec<serde_json::Value>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT log_id, batch_id, validation_time, total_contracts, valid_contracts,
                    warning_contracts, error_contracts, validated_by
             FROM validation_log
             ORDER BY validation_time DESC
             LIMIT ?",
        )
        .map_err(|e| e.to_string())?;

    let logs = stmt
        .query_map(params![limit], |row| {
            Ok(serde_json::json!({
                "log_id": row.get::<_, i64>(0)?,
                "batch_id": row.get::<_, String>(1)?,
                "validation_time": row.get::<_, Option<String>>(2)?,
                "total_contracts": row.get::<_, i64>(3)?,
                "valid_contracts": row.get::<_, i64>(4)?,
                "warning_contracts": row.get::<_, i64>(5)?,
                "error_contracts": row.get::<_, i64>(6)?,
                "validated_by": row.get::<_, Option<String>>(7)?
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(logs)
}

/// 获取字段问题汇总
#[allow(dead_code)]
pub fn get_field_issue_summary() -> Result<Vec<serde_json::Value>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT mvs.field_name, mvs.field_label, mvs.affects_score, mvs.default_value, mvs.default_description,
                    COUNT(DISTINCT cvi.contract_id) as affected_count
             FROM missing_value_strategy mvs
             LEFT JOIN contract_validation_issues cvi ON mvs.field_name = cvi.field_name AND cvi.is_resolved = 0
             GROUP BY mvs.field_name
             ORDER BY affected_count DESC, mvs.sort_order"
        )
        .map_err(|e| e.to_string())?;

    let summary = stmt
        .query_map([], |row| {
            Ok(serde_json::json!({
                "field_name": row.get::<_, String>(0)?,
                "field_label": row.get::<_, String>(1)?,
                "affects_score": row.get::<_, Option<String>>(2)?,
                "default_value": row.get::<_, Option<String>>(3)?,
                "default_description": row.get::<_, Option<String>>(4)?,
                "affected_count": row.get::<_, i64>(5)?
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(summary)
}

/// 获取指定合同的校验问题
#[allow(dead_code)]
pub fn get_contract_validation_issues(contract_id: &str) -> Result<Vec<serde_json::Value>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT issue_id, field_name, issue_type, severity, original_value,
                    default_value_used, message, suggested_fix, is_resolved, created_at
             FROM contract_validation_issues
             WHERE contract_id = ?
             ORDER BY created_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let issues = stmt
        .query_map(params![contract_id], |row| {
            Ok(serde_json::json!({
                "issue_id": row.get::<_, i64>(0)?,
                "field_name": row.get::<_, String>(1)?,
                "issue_type": row.get::<_, String>(2)?,
                "severity": row.get::<_, String>(3)?,
                "original_value": row.get::<_, Option<String>>(4)?,
                "default_value_used": row.get::<_, Option<String>>(5)?,
                "message": row.get::<_, String>(6)?,
                "suggested_fix": row.get::<_, Option<String>>(7)?,
                "is_resolved": row.get::<_, i64>(8)? == 1,
                "created_at": row.get::<_, Option<String>>(9)?
            }))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(issues)
}

/// 标记问题为已解决
#[allow(dead_code)]
pub fn resolve_validation_issue(issue_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE contract_validation_issues SET is_resolved = 1 WHERE issue_id = ?",
        params![issue_id],
    )
    .map_err(|e| format!("标记问题已解决失败: {}", e))?;

    Ok(())
}

/// 批量标记问题为已解决
#[allow(dead_code)]
pub fn resolve_validation_issues_by_contract(contract_id: &str) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let affected = conn.execute(
        "UPDATE contract_validation_issues SET is_resolved = 1 WHERE contract_id = ? AND is_resolved = 0",
        params![contract_id],
    )
    .map_err(|e| format!("批量标记问题已解决失败: {}", e))?;

    Ok(affected as i64)
}

// ============================================
// Phase 14: 策略版本化（可回放、可复盘）
// ============================================

/// 创建策略版本快照
///
/// # 功能
/// 1. 获取策略当前的所有配置参数
/// 2. 将配置表序列化为 JSON 快照
/// 3. 创建新版本记录
/// 4. 可选：设为激活版本
///
/// # 返回
/// 新版本的 version_id
pub fn create_strategy_version(
    strategy_name: &str,
    version_tag: Option<&str>,
    description: Option<&str>,
    change_reason: Option<&str>,
    user: &str,
    set_active: bool,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 1. 获取策略权重（ws, wp）
    let weights = get_strategy_weights(strategy_name)?;

    // 2. 获取评分子权重（w1, w2, w3）
    let s_weights = get_strategy_scoring_weights(strategy_name)?;

    // 3. 获取下一个版本号
    let version_number: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM strategy_version WHERE strategy_name = ?",
            params![strategy_name],
            |row| row.get(0),
        )
        .map_err(|e| format!("获取版本号失败: {}", e))?;

    // 4. 创建配置快照
    let scoring_config_snapshot = create_scoring_config_snapshot()?;
    let p2_curve_config_snapshot = create_p2_curve_config_snapshot()?;
    let aggregation_bins_snapshot = create_aggregation_bins_snapshot()?;
    let rhythm_config_snapshot = create_rhythm_config_snapshot()?;

    // 5. 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 6. 如果设为激活，先禁用该策略的其他激活版本
    if set_active {
        if let Err(e) = conn.execute(
            "UPDATE strategy_version SET is_active = 0 WHERE strategy_name = ? AND is_active = 1",
            params![strategy_name],
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("禁用旧版本失败: {}", e));
        }
    }

    // 7. 插入新版本
    let result = conn.execute(
        "INSERT INTO strategy_version
            (version_number, version_tag, strategy_name, ws, wp, w1, w2, w3,
             w_p1, w_p2, w_p3, scoring_config_snapshot, p2_curve_config_snapshot,
             aggregation_bins_snapshot, rhythm_config_snapshot, description,
             change_reason, created_by, is_active)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0.5, 0.3, 0.2, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            version_number,
            version_tag,
            strategy_name,
            weights.ws,
            weights.wp,
            s_weights.w1,
            s_weights.w2,
            s_weights.w3,
            scoring_config_snapshot,
            p2_curve_config_snapshot,
            aggregation_bins_snapshot,
            rhythm_config_snapshot,
            description,
            change_reason,
            user,
            if set_active { 1 } else { 0 }
        ],
    );

    if let Err(e) = result {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("创建版本失败: {}", e));
    }

    let version_id = conn.last_insert_rowid();

    // 8. 记录变更日志
    if let Err(e) = conn.execute(
        "INSERT INTO strategy_version_change_log (version_id, change_type, new_value, change_reason, changed_by)
         VALUES (?, 'create', ?, ?, ?)",
        params![
            version_id,
            serde_json::json!({
                "version_number": version_number,
                "strategy_name": strategy_name,
                "ws": weights.ws,
                "wp": weights.wp,
                "is_active": set_active
            }).to_string(),
            change_reason,
            user
        ],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("记录变更日志失败: {}", e));
    }

    // 9. 提交事务
    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(version_id)
}

/// 创建 scoring_config 表的 JSON 快照
fn create_scoring_config_snapshot() -> Result<String, String> {
    let configs = get_all_scoring_configs()?;
    serde_json::to_string(&configs).map_err(|e| format!("序列化配置失败: {}", e))
}

/// 创建 P2 曲线配置的 JSON 快照
fn create_p2_curve_config_snapshot() -> Result<Option<String>, String> {
    let config = get_p2_curve_config()?;
    let json =
        serde_json::to_string(&config).map_err(|e| format!("序列化 P2 曲线配置失败: {}", e))?;
    Ok(Some(json))
}

/// 创建聚合区间配置的 JSON 快照
fn create_aggregation_bins_snapshot() -> Result<Option<String>, String> {
    let bins = list_aggregation_bins()?;
    let json =
        serde_json::to_string(&bins).map_err(|e| format!("序列化聚合区间配置失败: {}", e))?;
    Ok(Some(json))
}

/// 创建节拍配置的 JSON 快照
fn create_rhythm_config_snapshot() -> Result<Option<String>, String> {
    // 获取当前激活的节拍配置
    match get_active_rhythm_config() {
        Ok(config) => {
            // 同时获取该配置的标签列表
            let labels = list_rhythm_labels(config.config_id.unwrap_or(1))?;
            let snapshot = serde_json::json!({
                "config": config,
                "labels": labels
            });
            Ok(Some(snapshot.to_string()))
        }
        Err(_) => Ok(None),
    }
}

/// 获取策略的所有版本列表
pub fn list_strategy_versions(strategy_name: &str) -> Result<Vec<StrategyVersionSummary>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT version_id, version_number, version_tag, strategy_name, ws, wp,
                    is_active, is_locked, created_by, created_at, description
             FROM strategy_version
             WHERE strategy_name = ?
             ORDER BY version_number DESC",
        )
        .map_err(|e| e.to_string())?;

    let versions = stmt
        .query_map(params![strategy_name], |row| {
            Ok(StrategyVersionSummary {
                version_id: row.get(0)?,
                version_number: row.get(1)?,
                version_tag: row.get(2)?,
                strategy_name: row.get(3)?,
                ws: row.get(4)?,
                wp: row.get(5)?,
                is_active: row.get::<_, i64>(6)? == 1,
                is_locked: row.get::<_, i64>(7)? == 1,
                created_by: row.get(8)?,
                created_at: row.get::<_, String>(9)?,
                description: row.get(10)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(versions)
}

/// 获取单个策略版本的完整信息
pub fn get_strategy_version(version_id: i64) -> Result<StrategyVersion, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT version_id, version_number, version_tag, strategy_name, ws, wp,
                w1, w2, w3, w_p1, w_p2, w_p3, scoring_config_snapshot,
                p2_curve_config_snapshot, aggregation_bins_snapshot, rhythm_config_snapshot,
                description, change_reason, created_by, created_at, is_active, is_locked
         FROM strategy_version
         WHERE version_id = ?",
        params![version_id],
        |row| {
            Ok(StrategyVersion {
                version_id: row.get(0)?,
                version_number: row.get(1)?,
                version_tag: row.get(2)?,
                strategy_name: row.get(3)?,
                ws: row.get(4)?,
                wp: row.get(5)?,
                w1: row.get(6)?,
                w2: row.get(7)?,
                w3: row.get(8)?,
                w_p1: row.get(9)?,
                w_p2: row.get(10)?,
                w_p3: row.get(11)?,
                scoring_config_snapshot: row.get(12)?,
                p2_curve_config_snapshot: row.get(13)?,
                aggregation_bins_snapshot: row.get(14)?,
                rhythm_config_snapshot: row.get(15)?,
                description: row.get(16)?,
                change_reason: row.get(17)?,
                created_by: row.get(18)?,
                created_at: row.get(19)?,
                is_active: row.get(20)?,
                is_locked: row.get(21)?,
            })
        },
    )
    .map_err(|e| format!("获取策略版本失败: {}", e))
}

/// 获取策略的当前激活版本
pub fn get_active_strategy_version(strategy_name: &str) -> Result<Option<StrategyVersion>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT version_id, version_number, version_tag, strategy_name, ws, wp,
                w1, w2, w3, w_p1, w_p2, w_p3, scoring_config_snapshot,
                p2_curve_config_snapshot, aggregation_bins_snapshot, rhythm_config_snapshot,
                description, change_reason, created_by, created_at, is_active, is_locked
         FROM strategy_version
         WHERE strategy_name = ? AND is_active = 1",
        params![strategy_name],
        |row| {
            Ok(StrategyVersion {
                version_id: row.get(0)?,
                version_number: row.get(1)?,
                version_tag: row.get(2)?,
                strategy_name: row.get(3)?,
                ws: row.get(4)?,
                wp: row.get(5)?,
                w1: row.get(6)?,
                w2: row.get(7)?,
                w3: row.get(8)?,
                w_p1: row.get(9)?,
                w_p2: row.get(10)?,
                w_p3: row.get(11)?,
                scoring_config_snapshot: row.get(12)?,
                p2_curve_config_snapshot: row.get(13)?,
                aggregation_bins_snapshot: row.get(14)?,
                rhythm_config_snapshot: row.get(15)?,
                description: row.get(16)?,
                change_reason: row.get(17)?,
                created_by: row.get(18)?,
                created_at: row.get(19)?,
                is_active: row.get(20)?,
                is_locked: row.get(21)?,
            })
        },
    );

    match result {
        Ok(version) => Ok(Some(version)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 激活指定版本
pub fn activate_strategy_version(version_id: i64, user: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取版本信息
    let version = get_strategy_version(version_id)?;

    if version.is_active == 1 {
        return Ok(()); // 已经是激活状态
    }

    // 开启事务
    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    // 禁用该策略的其他激活版本
    if let Err(e) = conn.execute(
        "UPDATE strategy_version SET is_active = 0 WHERE strategy_name = ? AND is_active = 1",
        params![version.strategy_name],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("禁用旧版本失败: {}", e));
    }

    // 激活指定版本
    if let Err(e) = conn.execute(
        "UPDATE strategy_version SET is_active = 1 WHERE version_id = ?",
        params![version_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("激活版本失败: {}", e));
    }

    // 记录变更日志
    if let Err(e) = conn.execute(
        "INSERT INTO strategy_version_change_log (version_id, change_type, old_value, new_value, changed_by)
         VALUES (?, 'activate', '{\"is_active\": 0}', '{\"is_active\": 1}', ?)",
        params![version_id, user],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("记录变更日志失败: {}", e));
    }

    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(())
}

/// 锁定版本（锁定后不可删除）
pub fn lock_strategy_version(
    version_id: i64,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE strategy_version SET is_locked = 1 WHERE version_id = ?",
        params![version_id],
    )
    .map_err(|e| format!("锁定版本失败: {}", e))?;

    conn.execute(
        "INSERT INTO strategy_version_change_log (version_id, change_type, old_value, new_value, change_reason, changed_by)
         VALUES (?, 'lock', '{\"is_locked\": 0}', '{\"is_locked\": 1}', ?, ?)",
        params![version_id, reason, user],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(())
}

/// 解锁版本
pub fn unlock_strategy_version(
    version_id: i64,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE strategy_version SET is_locked = 0 WHERE version_id = ?",
        params![version_id],
    )
    .map_err(|e| format!("解锁版本失败: {}", e))?;

    conn.execute(
        "INSERT INTO strategy_version_change_log (version_id, change_type, old_value, new_value, change_reason, changed_by)
         VALUES (?, 'unlock', '{\"is_locked\": 1}', '{\"is_locked\": 0}', ?, ?)",
        params![version_id, reason, user],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(())
}

/// 删除版本（仅可删除未锁定的非激活版本）
pub fn delete_strategy_version(
    version_id: i64,
    user: &str,
    reason: Option<&str>,
) -> Result<(), String> {
    let version = get_strategy_version(version_id)?;

    if version.is_active == 1 {
        return Err("不能删除激活状态的版本".to_string());
    }

    if version.is_locked == 1 {
        return Err("不能删除已锁定的版本".to_string());
    }

    let conn = get_connection().map_err(|e| e.to_string())?;

    // 记录变更日志（在删除前）
    conn.execute(
        "INSERT INTO strategy_version_change_log (version_id, change_type, old_value, change_reason, changed_by)
         VALUES (?, 'delete', ?, ?, ?)",
        params![
            version_id,
            serde_json::json!({
                "version_number": version.version_number,
                "strategy_name": version.strategy_name
            }).to_string(),
            reason,
            user
        ],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    // 删除版本
    conn.execute(
        "DELETE FROM strategy_version WHERE version_id = ?",
        params![version_id],
    )
    .map_err(|e| format!("删除版本失败: {}", e))?;

    Ok(())
}

/// 获取版本变更历史
pub fn list_strategy_version_change_log(
    version_id: Option<i64>,
    limit: i64,
) -> Result<Vec<StrategyVersionChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if version_id.is_some() {
        "SELECT log_id, version_id, change_type, old_value, new_value, change_reason, changed_by, changed_at
         FROM strategy_version_change_log
         WHERE version_id = ?1
         ORDER BY changed_at DESC
         LIMIT ?2"
    } else {
        "SELECT log_id, version_id, change_type, old_value, new_value, change_reason, changed_by, changed_at
         FROM strategy_version_change_log
         ORDER BY changed_at DESC
         LIMIT ?2"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(vid) = version_id {
        stmt.query_map(params![vid, limit], |row| {
            Ok(StrategyVersionChangeLog {
                log_id: row.get(0)?,
                version_id: row.get(1)?,
                change_type: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                change_reason: row.get(5)?,
                changed_by: row.get(6)?,
                changed_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(StrategyVersionChangeLog {
                log_id: row.get(0)?,
                version_id: row.get(1)?,
                change_type: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                change_reason: row.get(5)?,
                changed_by: row.get(6)?,
                changed_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(logs)
}

// ============================================
// 沙盘会话管理
// ============================================

/// 创建沙盘会话
pub fn create_sandbox_session(
    session_name: &str,
    strategy_version_id: i64,
    description: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 验证版本存在
    let _ = get_strategy_version(strategy_version_id)?;

    conn.execute(
        "INSERT INTO sandbox_session (session_name, session_type, strategy_version_id, status, description, created_by)
         VALUES (?, 'sandbox', ?, 'draft', ?, ?)",
        params![session_name, strategy_version_id, description, user],
    )
    .map_err(|e| format!("创建沙盘会话失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取沙盘会话列表
pub fn list_sandbox_sessions(limit: Option<i64>) -> Result<Vec<SandboxSession>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if let Some(lim) = limit {
        format!(
            "SELECT session_id, session_name, session_type, strategy_version_id, contract_snapshot_time,
                    status, total_contracts, result_summary, description, created_by, created_at, completed_at
             FROM sandbox_session
             ORDER BY created_at DESC
             LIMIT {}",
            lim
        )
    } else {
        "SELECT session_id, session_name, session_type, strategy_version_id, contract_snapshot_time,
                status, total_contracts, result_summary, description, created_by, created_at, completed_at
         FROM sandbox_session
         ORDER BY created_at DESC".to_string()
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let sessions = stmt
        .query_map([], |row| {
            Ok(SandboxSession {
                session_id: row.get(0)?,
                session_name: row.get(1)?,
                session_type: row.get(2)?,
                strategy_version_id: row.get(3)?,
                contract_snapshot_time: row.get(4)?,
                status: row.get(5)?,
                total_contracts: row.get(6)?,
                result_summary: row.get(7)?,
                description: row.get(8)?,
                created_by: row.get(9)?,
                created_at: row.get(10)?,
                completed_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(sessions)
}

/// 获取单个沙盘会话
pub fn get_sandbox_session(session_id: i64) -> Result<SandboxSession, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT session_id, session_name, session_type, strategy_version_id, contract_snapshot_time,
                status, total_contracts, result_summary, description, created_by, created_at, completed_at
         FROM sandbox_session
         WHERE session_id = ?",
        params![session_id],
        |row| {
            Ok(SandboxSession {
                session_id: row.get(0)?,
                session_name: row.get(1)?,
                session_type: row.get(2)?,
                strategy_version_id: row.get(3)?,
                contract_snapshot_time: row.get(4)?,
                status: row.get(5)?,
                total_contracts: row.get(6)?,
                result_summary: row.get(7)?,
                description: row.get(8)?,
                created_by: row.get(9)?,
                created_at: row.get(10)?,
                completed_at: row.get(11)?,
            })
        },
    )
    .map_err(|e| format!("获取沙盘会话失败: {}", e))
}

/// 保存沙盘计算结果
pub fn save_sandbox_results(session_id: i64, results: &[SandboxResult]) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    for result in results {
        if let Err(e) = conn.execute(
            "INSERT INTO sandbox_result
                (session_id, contract_id, contract_snapshot, customer_snapshot,
                 s_score, p_score, priority, alpha,
                 s1_score, s2_score, s3_score, p1_score, p2_score, p3_score,
                 aggregation_key, aggregation_count, priority_rank)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                session_id,
                result.contract_id,
                result.contract_snapshot,
                result.customer_snapshot,
                result.s_score,
                result.p_score,
                result.priority,
                result.alpha,
                result.s1_score,
                result.s2_score,
                result.s3_score,
                result.p1_score,
                result.p2_score,
                result.p3_score,
                result.aggregation_key,
                result.aggregation_count,
                result.priority_rank
            ],
        ) {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("保存结果失败: {}", e));
        }
    }

    // 更新会话状态
    if let Err(e) = conn.execute(
        "UPDATE sandbox_session
         SET status = 'completed',
             total_contracts = ?,
             contract_snapshot_time = datetime('now','localtime'),
             completed_at = datetime('now','localtime')
         WHERE session_id = ?",
        params![results.len() as i64, session_id],
    ) {
        let _ = conn.execute("ROLLBACK", []);
        return Err(format!("更新会话状态失败: {}", e));
    }

    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(())
}

/// 获取沙盘会话的计算结果
pub fn get_sandbox_results(session_id: i64) -> Result<Vec<SandboxResult>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT result_id, session_id, contract_id, contract_snapshot, customer_snapshot,
                    s_score, p_score, priority, alpha,
                    s1_score, s2_score, s3_score, p1_score, p2_score, p3_score,
                    aggregation_key, aggregation_count, priority_rank
             FROM sandbox_result
             WHERE session_id = ?
             ORDER BY priority_rank",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map(params![session_id], |row| {
            Ok(SandboxResult {
                result_id: row.get(0)?,
                session_id: row.get(1)?,
                contract_id: row.get(2)?,
                contract_snapshot: row.get(3)?,
                customer_snapshot: row.get(4)?,
                s_score: row.get(5)?,
                p_score: row.get(6)?,
                priority: row.get(7)?,
                alpha: row.get(8)?,
                s1_score: row.get(9)?,
                s2_score: row.get(10)?,
                s3_score: row.get(11)?,
                p1_score: row.get(12)?,
                p2_score: row.get(13)?,
                p3_score: row.get(14)?,
                aggregation_key: row.get(15)?,
                aggregation_count: row.get(16)?,
                priority_rank: row.get(17)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(results)
}

// ============================================
// 版本对比
// ============================================

/// 创建版本对比记录
pub fn create_version_comparison(
    version_a_id: i64,
    version_b_id: i64,
    comparison_details: &str,
    contracts_compared: i64,
    rank_changes: i64,
    avg_priority_diff: f64,
    max_rank_change: i64,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO version_comparison
            (version_a_id, version_b_id, contracts_compared, rank_changes, avg_priority_diff, max_rank_change, comparison_details, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        params![version_a_id, version_b_id, contracts_compared, rank_changes, avg_priority_diff, max_rank_change, comparison_details, user],
    )
    .map_err(|e| format!("创建对比记录失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取版本对比记录
pub fn get_version_comparison(comparison_id: i64) -> Result<VersionComparison, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT comparison_id, version_a_id, version_b_id, contracts_compared, rank_changes,
                avg_priority_diff, max_rank_change, comparison_details, created_by, created_at
         FROM version_comparison
         WHERE comparison_id = ?",
        params![comparison_id],
        |row| {
            Ok(VersionComparison {
                comparison_id: row.get(0)?,
                version_a_id: row.get(1)?,
                version_b_id: row.get(2)?,
                contracts_compared: row.get(3)?,
                rank_changes: row.get(4)?,
                avg_priority_diff: row.get(5)?,
                max_rank_change: row.get(6)?,
                comparison_details: row.get(7)?,
                created_by: row.get(8)?,
                created_at: row.get(9)?,
            })
        },
    )
    .map_err(|e| format!("获取对比记录失败: {}", e))
}

// ============================================
// Phase 15: 导入/清洗冲突解决机制产品化
// ============================================

/// 创建导入审计记录
#[allow(dead_code)]
pub fn create_import_audit(
    import_type: &str,
    file_name: &str,
    file_format: &str,
    file_hash: Option<&str>,
    file_size: Option<i64>,
    conflict_strategy: &str,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO import_audit_log
            (import_type, file_name, file_format, file_hash, file_size, conflict_strategy, status, imported_by)
         VALUES (?, ?, ?, ?, ?, ?, 'pending', ?)",
        params![import_type, file_name, file_format, file_hash, file_size, conflict_strategy, user],
    )
    .map_err(|e| format!("创建导入审计记录失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 更新导入审计记录状态
#[allow(dead_code)]
pub fn update_import_audit_status(
    audit_id: i64,
    total_rows: i64,
    valid_rows: i64,
    error_rows: i64,
    conflict_rows: i64,
    imported_count: i64,
    updated_count: i64,
    skipped_count: i64,
    status: &str,
    error_message: Option<&str>,
    validation_errors: Option<&str>,
    validation_warnings: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE import_audit_log
         SET total_rows = ?, valid_rows = ?, error_rows = ?, conflict_rows = ?,
             imported_count = ?, updated_count = ?, skipped_count = ?,
             status = ?, error_message = ?,
             validation_errors = ?, validation_warnings = ?,
             completed_at = datetime('now','localtime')
         WHERE audit_id = ?",
        params![
            total_rows,
            valid_rows,
            error_rows,
            conflict_rows,
            imported_count,
            updated_count,
            skipped_count,
            status,
            error_message,
            validation_errors,
            validation_warnings,
            audit_id
        ],
    )
    .map_err(|e| format!("更新导入审计状态失败: {}", e))?;

    Ok(())
}

/// 获取导入审计记录
pub fn get_import_audit(audit_id: i64) -> Result<ImportAuditLog, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT audit_id, import_type, file_name, file_format, file_hash, file_size,
                total_rows, valid_rows, error_rows, conflict_rows,
                imported_count, updated_count, skipped_count,
                conflict_strategy, status, error_message,
                applied_transform_rules, imported_by, started_at, completed_at,
                validation_errors, validation_warnings
         FROM import_audit_log
         WHERE audit_id = ?",
        params![audit_id],
        |row| {
            Ok(ImportAuditLog {
                audit_id: row.get(0)?,
                import_type: row.get(1)?,
                file_name: row.get(2)?,
                file_format: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                total_rows: row.get(6)?,
                valid_rows: row.get(7)?,
                error_rows: row.get(8)?,
                conflict_rows: row.get(9)?,
                imported_count: row.get(10)?,
                updated_count: row.get(11)?,
                skipped_count: row.get(12)?,
                conflict_strategy: row.get(13)?,
                status: row.get(14)?,
                error_message: row.get(15)?,
                applied_transform_rules: row.get(16)?,
                imported_by: row.get(17)?,
                started_at: row.get(18)?,
                completed_at: row.get(19)?,
                validation_errors: row.get(20)?,
                validation_warnings: row.get(21)?,
            })
        },
    )
    .map_err(|e| format!("获取导入审计记录失败: {}", e))
}

/// 获取导入审计历史
pub fn list_import_audits(
    import_type: Option<&str>,
    status: Option<&str>,
    limit: i64,
) -> Result<Vec<ImportAuditLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut query = String::from(
        "SELECT audit_id, import_type, file_name, file_format, file_hash, file_size,
                total_rows, valid_rows, error_rows, conflict_rows,
                imported_count, updated_count, skipped_count,
                conflict_strategy, status, error_message,
                applied_transform_rules, imported_by, started_at, completed_at,
                validation_errors, validation_warnings
         FROM import_audit_log
         WHERE 1=1",
    );

    if import_type.is_some() {
        query.push_str(" AND import_type = ?1");
    }
    if status.is_some() {
        query.push_str(" AND status = ?2");
    }
    query.push_str(" ORDER BY started_at DESC LIMIT ?3");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let audits = stmt
        .query_map(params![import_type, status, limit], |row| {
            Ok(ImportAuditLog {
                audit_id: row.get(0)?,
                import_type: row.get(1)?,
                file_name: row.get(2)?,
                file_format: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                total_rows: row.get(6)?,
                valid_rows: row.get(7)?,
                error_rows: row.get(8)?,
                conflict_rows: row.get(9)?,
                imported_count: row.get(10)?,
                updated_count: row.get(11)?,
                skipped_count: row.get(12)?,
                conflict_strategy: row.get(13)?,
                status: row.get(14)?,
                error_message: row.get(15)?,
                applied_transform_rules: row.get(16)?,
                imported_by: row.get(17)?,
                started_at: row.get(18)?,
                completed_at: row.get(19)?,
                validation_errors: row.get(20)?,
                validation_warnings: row.get(21)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(audits)
}

/// 记录导入冲突
#[allow(dead_code)]
pub fn log_import_conflict(
    audit_id: i64,
    row_number: i64,
    primary_key: &str,
    existing_data: &str,
    new_data: &str,
    changed_fields: Option<&str>,
    field_diffs: Option<&str>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO import_conflict_log
            (audit_id, row_number, primary_key, existing_data, new_data, changed_fields, field_diffs, action)
         VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')",
        params![audit_id, row_number, primary_key, existing_data, new_data, changed_fields, field_diffs],
    )
    .map_err(|e| format!("记录导入冲突失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取导入的待处理冲突
pub fn get_pending_conflicts(audit_id: i64) -> Result<Vec<ImportConflictLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT conflict_id, audit_id, row_number, primary_key, existing_data, new_data,
                    changed_fields, field_diffs, action, action_reason, decided_by, decided_at, created_at
             FROM import_conflict_log
             WHERE audit_id = ? AND action = 'pending'
             ORDER BY row_number"
        )
        .map_err(|e| e.to_string())?;

    let conflicts = stmt
        .query_map(params![audit_id], |row| {
            Ok(ImportConflictLog {
                conflict_id: row.get(0)?,
                audit_id: row.get(1)?,
                row_number: row.get(2)?,
                primary_key: row.get(3)?,
                existing_data: row.get(4)?,
                new_data: row.get(5)?,
                changed_fields: row.get(6)?,
                field_diffs: row.get(7)?,
                action: row.get(8)?,
                action_reason: row.get(9)?,
                decided_by: row.get(10)?,
                decided_at: row.get(11)?,
                created_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(conflicts)
}

/// 解决单个冲突
pub fn resolve_import_conflict(
    conflict_id: i64,
    action: &str,
    action_reason: Option<&str>,
    user: &str,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE import_conflict_log
         SET action = ?, action_reason = ?, decided_by = ?, decided_at = datetime('now','localtime')
         WHERE conflict_id = ?",
        params![action, action_reason, user, conflict_id],
    )
    .map_err(|e| format!("解决冲突失败: {}", e))?;

    Ok(())
}

/// 批量解决冲突
pub fn batch_resolve_conflicts(
    audit_id: i64,
    action: &str,
    action_reason: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let affected = conn
        .execute(
            "UPDATE import_conflict_log
         SET action = ?, action_reason = ?, decided_by = ?, decided_at = datetime('now','localtime')
         WHERE audit_id = ? AND action = 'pending'",
            params![action, action_reason, user, audit_id],
        )
        .map_err(|e| format!("批量解决冲突失败: {}", e))?;

    Ok(affected as i64)
}

/// 获取字段对齐规则
pub fn list_field_alignment_rules(
    data_type: Option<&str>,
    include_disabled: bool,
) -> Result<Vec<FieldAlignmentRule>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if data_type.is_some() && include_disabled {
        "SELECT rule_id, rule_name, data_type, source_type, description, enabled, priority,
                field_mapping, value_transform, default_values, created_by, created_at, updated_at
         FROM field_alignment_rule
         WHERE data_type = ?
         ORDER BY enabled DESC, priority"
    } else if data_type.is_some() {
        "SELECT rule_id, rule_name, data_type, source_type, description, enabled, priority,
                field_mapping, value_transform, default_values, created_by, created_at, updated_at
         FROM field_alignment_rule
         WHERE data_type = ? AND enabled = 1
         ORDER BY priority"
    } else if include_disabled {
        "SELECT rule_id, rule_name, data_type, source_type, description, enabled, priority,
                field_mapping, value_transform, default_values, created_by, created_at, updated_at
         FROM field_alignment_rule
         ORDER BY data_type, enabled DESC, priority"
    } else {
        "SELECT rule_id, rule_name, data_type, source_type, description, enabled, priority,
                field_mapping, value_transform, default_values, created_by, created_at, updated_at
         FROM field_alignment_rule
         WHERE enabled = 1
         ORDER BY data_type, priority"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let rules = if let Some(dt) = data_type {
        stmt.query_map(params![dt], |row| {
            Ok(FieldAlignmentRule {
                rule_id: row.get(0)?,
                rule_name: row.get(1)?,
                data_type: row.get(2)?,
                source_type: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get(5)?,
                priority: row.get(6)?,
                field_mapping: row.get(7)?,
                value_transform: row.get(8)?,
                default_values: row.get(9)?,
                created_by: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], |row| {
            Ok(FieldAlignmentRule {
                rule_id: row.get(0)?,
                rule_name: row.get(1)?,
                data_type: row.get(2)?,
                source_type: row.get(3)?,
                description: row.get(4)?,
                enabled: row.get(5)?,
                priority: row.get(6)?,
                field_mapping: row.get(7)?,
                value_transform: row.get(8)?,
                default_values: row.get(9)?,
                created_by: row.get(10)?,
                created_at: row.get(11)?,
                updated_at: row.get(12)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(rules)
}

/// 创建字段对齐规则
pub fn create_field_alignment_rule(
    rule_name: &str,
    data_type: &str,
    source_type: Option<&str>,
    description: Option<&str>,
    priority: i64,
    field_mapping: &str,
    value_transform: Option<&str>,
    default_values: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO field_alignment_rule
            (rule_name, data_type, source_type, description, enabled, priority,
             field_mapping, value_transform, default_values, created_by)
         VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?)",
        params![
            rule_name,
            data_type,
            source_type,
            description,
            priority,
            field_mapping,
            value_transform,
            default_values,
            user
        ],
    )
    .map_err(|e| format!("创建字段对齐规则失败: {}", e))?;

    let rule_id = conn.last_insert_rowid();

    // 记录变更日志
    conn.execute(
        "INSERT INTO field_alignment_change_log (rule_id, change_type, new_value, changed_by)
         VALUES (?, 'create', ?, ?)",
        params![rule_id, field_mapping, user],
    )
    .map_err(|e| format!("记录变更日志失败: {}", e))?;

    Ok(rule_id)
}

/// 保存字段对齐规则（rule_id 存在则更新，不存在则创建）
pub fn save_field_alignment_rule(rule: &FieldAlignmentRule, user: &str) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    if let Some(rule_id) = rule.rule_id {
        let old_mapping: Option<String> = conn
            .query_row(
                "SELECT field_mapping FROM field_alignment_rule WHERE rule_id = ?1",
                params![rule_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| format!("查询字段对齐规则失败: {}", e))?;

        let old_mapping = old_mapping.ok_or_else(|| format!("字段对齐规则不存在: {}", rule_id))?;

        conn.execute(
            "UPDATE field_alignment_rule
             SET rule_name = ?1,
                 data_type = ?2,
                 source_type = ?3,
                 description = ?4,
                 enabled = ?5,
                 priority = ?6,
                 field_mapping = ?7,
                 value_transform = ?8,
                 default_values = ?9,
                 updated_at = CURRENT_TIMESTAMP
             WHERE rule_id = ?10",
            params![
                rule.rule_name,
                rule.data_type,
                rule.source_type,
                rule.description,
                rule.enabled,
                rule.priority,
                rule.field_mapping,
                rule.value_transform,
                rule.default_values,
                rule_id
            ],
        )
        .map_err(|e| format!("更新字段对齐规则失败: {}", e))?;

        conn.execute(
            "INSERT INTO field_alignment_change_log (rule_id, change_type, old_value, new_value, changed_by)
             VALUES (?1, 'update', ?2, ?3, ?4)",
            params![rule_id, old_mapping, rule.field_mapping, user],
        )
        .map_err(|e| format!("记录字段对齐规则变更失败: {}", e))?;

        Ok(rule_id)
    } else {
        create_field_alignment_rule(
            &rule.rule_name,
            &rule.data_type,
            rule.source_type.as_deref(),
            rule.description.as_deref(),
            rule.priority,
            &rule.field_mapping,
            rule.value_transform.as_deref(),
            rule.default_values.as_deref(),
            user,
        )
    }
}

/// 删除字段对齐规则
pub fn delete_field_alignment_rule(rule_id: i64, user: &str) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let old_mapping: Option<String> = conn
        .query_row(
            "SELECT field_mapping FROM field_alignment_rule WHERE rule_id = ?1",
            params![rule_id],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("查询字段对齐规则失败: {}", e))?;

    let old_mapping = old_mapping.ok_or_else(|| format!("字段对齐规则不存在: {}", rule_id))?;

    conn.execute(
        "INSERT INTO field_alignment_change_log (rule_id, change_type, old_value, changed_by)
         VALUES (?1, 'delete', ?2, ?3)",
        params![rule_id, old_mapping, user],
    )
    .map_err(|e| format!("记录字段对齐规则删除日志失败: {}", e))?;

    conn.execute(
        "DELETE FROM field_alignment_rule WHERE rule_id = ?1",
        params![rule_id],
    )
    .map_err(|e| format!("删除字段对齐规则失败: {}", e))?;

    Ok(())
}

/// 获取字段对齐规则变更日志
pub fn list_field_alignment_change_logs(
    rule_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<FieldAlignmentChangeLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;
    let safe_limit = limit.unwrap_or(50).max(1);

    let query = if rule_id.is_some() {
        "SELECT log_id, rule_id, change_type, old_value, new_value, change_reason, changed_by, changed_at
         FROM field_alignment_change_log
         WHERE rule_id = ?1
         ORDER BY changed_at DESC, log_id DESC
         LIMIT ?2"
    } else {
        "SELECT log_id, rule_id, change_type, old_value, new_value, change_reason, changed_by, changed_at
         FROM field_alignment_change_log
         ORDER BY changed_at DESC, log_id DESC
         LIMIT ?1"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let logs = if let Some(id) = rule_id {
        stmt.query_map(params![id, safe_limit], |row| {
            Ok(FieldAlignmentChangeLog {
                log_id: row.get(0)?,
                rule_id: row.get(1)?,
                change_type: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                change_reason: row.get(5)?,
                changed_by: row.get(6)?,
                changed_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询字段映射变更日志失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析字段映射变更日志失败: {}", e))?
    } else {
        stmt.query_map(params![safe_limit], |row| {
            Ok(FieldAlignmentChangeLog {
                log_id: row.get(0)?,
                rule_id: row.get(1)?,
                change_type: row.get(2)?,
                old_value: row.get(3)?,
                new_value: row.get(4)?,
                change_reason: row.get(5)?,
                changed_by: row.get(6)?,
                changed_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("查询字段映射变更日志失败: {}", e))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("解析字段映射变更日志失败: {}", e))?
    };

    Ok(logs)
}

/// 获取重复检测配置
pub fn get_duplicate_detection_config(
    data_type: &str,
) -> Result<Option<DuplicateDetectionConfig>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT config_id, config_name, data_type, description, is_active,
                primary_key_fields, fuzzy_match_fields, fuzzy_threshold,
                time_field, time_window_days, business_rules, default_action,
                created_by, created_at, updated_at
         FROM duplicate_detection_config
         WHERE data_type = ? AND is_active = 1",
        params![data_type],
        |row| {
            Ok(DuplicateDetectionConfig {
                config_id: row.get(0)?,
                config_name: row.get(1)?,
                data_type: row.get(2)?,
                description: row.get(3)?,
                is_active: row.get(4)?,
                primary_key_fields: row.get(5)?,
                fuzzy_match_fields: row.get(6)?,
                fuzzy_threshold: row.get(7)?,
                time_field: row.get(8)?,
                time_window_days: row.get(9)?,
                business_rules: row.get(10)?,
                default_action: row.get(11)?,
                created_by: row.get(12)?,
                created_at: row.get(13)?,
                updated_at: row.get(14)?,
            })
        },
    );

    match result {
        Ok(config) => Ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

/// 保存导入快照（用于回滚）
#[allow(dead_code)]
pub fn save_import_snapshot(
    audit_id: i64,
    data_type: &str,
    primary_key: &str,
    action_type: &str,
    before_data: Option<&str>,
    after_data: Option<&str>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO import_snapshot (audit_id, data_type, primary_key, action_type, before_data, after_data)
         VALUES (?, ?, ?, ?, ?, ?)",
        params![audit_id, data_type, primary_key, action_type, before_data, after_data],
    )
    .map_err(|e| format!("保存导入快照失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 回滚导入操作
pub fn rollback_import(audit_id: i64) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 获取快照
    let mut stmt = conn
        .prepare(
            "SELECT snapshot_id, data_type, primary_key, action_type, before_data
             FROM import_snapshot
             WHERE audit_id = ?
             ORDER BY snapshot_id DESC",
        )
        .map_err(|e| e.to_string())?;

    let snapshots: Vec<(i64, String, String, String, Option<String>)> = stmt
        .query_map(params![audit_id], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get(3)?,
                row.get(4)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    let mut rollback_count = 0i64;

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    for (_snapshot_id, data_type, primary_key, action_type, before_data) in snapshots {
        let result: Result<usize, String> = match action_type.as_str() {
            "insert" => {
                // 回滚插入：删除记录
                match data_type.as_str() {
                    "contracts" => conn
                        .execute(
                            "DELETE FROM contract_master WHERE contract_id = ?",
                            params![primary_key],
                        )
                        .map_err(|e| e.to_string()),
                    "customers" => conn
                        .execute(
                            "DELETE FROM customer_master WHERE customer_id = ?",
                            params![primary_key],
                        )
                        .map_err(|e| e.to_string()),
                    _ => Ok(0),
                }
            }
            "update" => {
                // 回滚更新：恢复旧数据
                if let Some(before) = before_data {
                    match data_type.as_str() {
                        "contracts" => {
                            let contract: Contract = serde_json::from_str(&before)
                                .map_err(|e| format!("解析合同数据失败: {}", e))?;
                            update_contract(&contract).map(|_| 1)
                        }
                        "customers" => {
                            let customer: Customer = serde_json::from_str(&before)
                                .map_err(|e| format!("解析客户数据失败: {}", e))?;
                            update_customer(&customer).map(|_| 1)
                        }
                        _ => Ok(0),
                    }
                } else {
                    Ok(0)
                }
            }
            _ => Ok(0),
        };

        match result {
            Ok(n) => rollback_count += n as i64,
            Err(e) => {
                let _ = conn.execute("ROLLBACK", []);
                return Err(format!("回滚失败: {}", e));
            }
        }
    }

    // 更新审计记录状态
    conn.execute(
        "UPDATE import_audit_log SET status = 'rolled_back' WHERE audit_id = ?",
        params![audit_id],
    )
    .map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("更新审计状态失败: {}", e)
    })?;

    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(rollback_count)
}

/// 获取导入统计信息
pub fn get_import_statistics() -> Result<Vec<ImportStatistics>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT import_type,
                    COUNT(*) as total_imports,
                    SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as successful_imports,
                    SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_imports,
                    COALESCE(SUM(total_rows), 0) as total_rows_processed,
                    COALESCE(SUM(imported_count), 0) as total_imported,
                    COALESCE(SUM(updated_count), 0) as total_updated,
                    COALESCE(SUM(skipped_count), 0) as total_skipped,
                    COALESCE(SUM(conflict_rows), 0) as total_conflicts,
                    MAX(started_at) as last_import_time
             FROM import_audit_log
             GROUP BY import_type",
        )
        .map_err(|e| e.to_string())?;

    let stats = stmt
        .query_map([], |row| {
            Ok(ImportStatistics {
                import_type: row.get(0)?,
                total_imports: row.get(1)?,
                successful_imports: row.get(2)?,
                failed_imports: row.get(3)?,
                total_rows_processed: row.get(4)?,
                total_imported: row.get(5)?,
                total_updated: row.get(6)?,
                total_skipped: row.get(7)?,
                total_conflicts: row.get(8)?,
                last_import_time: row.get(9)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(stats)
}

/// 记录相似记录对
#[allow(dead_code)]
pub fn log_similar_record_pair(
    audit_id: Option<i64>,
    data_type: &str,
    record_a_key: &str,
    record_b_key: &str,
    similarity_score: f64,
    matching_fields: Option<&str>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO similar_record_pair
            (audit_id, data_type, record_a_key, record_b_key, similarity_score, matching_fields, status)
         VALUES (?, ?, ?, ?, ?, ?, 'pending')",
        params![audit_id, data_type, record_a_key, record_b_key, similarity_score, matching_fields],
    )
    .map_err(|e| format!("记录相似记录对失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取待处理的相似记录对
pub fn get_pending_similar_pairs(
    data_type: Option<&str>,
    limit: i64,
) -> Result<Vec<SimilarRecordPair>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if data_type.is_some() {
        "SELECT pair_id, audit_id, data_type, record_a_key, record_b_key, similarity_score,
                matching_fields, status, resolved_by, resolved_at, resolution_note, created_at
         FROM similar_record_pair
         WHERE data_type = ? AND status = 'pending'
         ORDER BY similarity_score DESC
         LIMIT ?"
    } else {
        "SELECT pair_id, audit_id, data_type, record_a_key, record_b_key, similarity_score,
                matching_fields, status, resolved_by, resolved_at, resolution_note, created_at
         FROM similar_record_pair
         WHERE status = 'pending'
         ORDER BY similarity_score DESC
         LIMIT ?"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let pairs = if let Some(dt) = data_type {
        stmt.query_map(params![dt, limit], |row| {
            Ok(SimilarRecordPair {
                pair_id: row.get(0)?,
                audit_id: row.get(1)?,
                data_type: row.get(2)?,
                record_a_key: row.get(3)?,
                record_b_key: row.get(4)?,
                similarity_score: row.get(5)?,
                matching_fields: row.get(6)?,
                status: row.get(7)?,
                resolved_by: row.get(8)?,
                resolved_at: row.get(9)?,
                resolution_note: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map(params![limit], |row| {
            Ok(SimilarRecordPair {
                pair_id: row.get(0)?,
                audit_id: row.get(1)?,
                data_type: row.get(2)?,
                record_a_key: row.get(3)?,
                record_b_key: row.get(4)?,
                similarity_score: row.get(5)?,
                matching_fields: row.get(6)?,
                status: row.get(7)?,
                resolved_by: row.get(8)?,
                resolved_at: row.get(9)?,
                resolution_note: row.get(10)?,
                created_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(pairs)
}

/// 解决相似记录对
pub fn resolve_similar_pair(
    pair_id: i64,
    status: &str,
    resolution_note: Option<&str>,
    user: &str,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE similar_record_pair
         SET status = ?, resolution_note = ?, resolved_by = ?, resolved_at = datetime('now','localtime')
         WHERE pair_id = ?",
        params![status, resolution_note, user, pair_id],
    )
    .map_err(|e| format!("解决相似记录对失败: {}", e))?;

    Ok(())
}

/// 检测重复文件（通过文件哈希）
pub fn check_duplicate_import(file_hash: &str) -> Result<Option<ImportAuditLog>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT audit_id, import_type, file_name, file_format, file_hash, file_size,
                total_rows, valid_rows, error_rows, conflict_rows,
                imported_count, updated_count, skipped_count,
                conflict_strategy, status, error_message,
                applied_transform_rules, imported_by, started_at, completed_at,
                validation_errors, validation_warnings
         FROM import_audit_log
         WHERE file_hash = ? AND status = 'success'
         ORDER BY started_at DESC
         LIMIT 1",
        params![file_hash],
        |row| {
            Ok(ImportAuditLog {
                audit_id: row.get(0)?,
                import_type: row.get(1)?,
                file_name: row.get(2)?,
                file_format: row.get(3)?,
                file_hash: row.get(4)?,
                file_size: row.get(5)?,
                total_rows: row.get(6)?,
                valid_rows: row.get(7)?,
                error_rows: row.get(8)?,
                conflict_rows: row.get(9)?,
                imported_count: row.get(10)?,
                updated_count: row.get(11)?,
                skipped_count: row.get(12)?,
                conflict_strategy: row.get(13)?,
                status: row.get(14)?,
                error_message: row.get(15)?,
                applied_transform_rules: row.get(16)?,
                imported_by: row.get(17)?,
                started_at: row.get(18)?,
                completed_at: row.get(19)?,
                validation_errors: row.get(20)?,
                validation_warnings: row.get(21)?,
            })
        },
    );

    match result {
        Ok(audit) => Ok(Some(audit)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// ============================================
// Phase 16: 会议驾驶舱 KPI 固化
// Meeting Cockpit KPI Solidification
// ============================================

use super::schema::{
    ConsensusTemplate, MeetingActionItem, MeetingKpiConfig, MeetingSnapshot,
    MeetingSnapshotSummary, RankingChangeDetail, RiskContractFlag,
};

// ============================================
// 会议快照 CRUD
// ============================================

/// 创建会议快照
pub fn create_meeting_snapshot(
    meeting_type: &str,
    meeting_date: &str,
    snapshot_name: &str,
    strategy_version_id: Option<i64>,
    strategy_name: Option<&str>,
    kpi_summary: &str,
    risk_summary: &str,
    recommendation: Option<&str>,
    contract_rankings: &str,
    ranking_changes: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO meeting_snapshot
            (meeting_type, meeting_date, snapshot_name, strategy_version_id, strategy_name,
             kpi_summary, risk_summary, recommendation, contract_rankings, ranking_changes,
             consensus_status, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', ?)",
        params![
            meeting_type,
            meeting_date,
            snapshot_name,
            strategy_version_id,
            strategy_name,
            kpi_summary,
            risk_summary,
            recommendation,
            contract_rankings,
            ranking_changes,
            user
        ],
    )
    .map_err(|e| format!("创建会议快照失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取会议快照列表
pub fn list_meeting_snapshots(
    meeting_type: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<MeetingSnapshotSummary>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if meeting_type.is_some() {
        format!(
            "SELECT snapshot_id, meeting_type, meeting_date, snapshot_name, strategy_name,
                    strategy_version_number, consensus_status, created_by, created_at,
                    risk_count, change_count, action_count
             FROM v_meeting_snapshot_summary
             WHERE meeting_type = ?1
             ORDER BY meeting_date DESC, created_at DESC
             LIMIT {}",
            limit.unwrap_or(50)
        )
    } else {
        format!(
            "SELECT snapshot_id, meeting_type, meeting_date, snapshot_name, strategy_name,
                    strategy_version_number, consensus_status, created_by, created_at,
                    risk_count, change_count, action_count
             FROM v_meeting_snapshot_summary
             ORDER BY meeting_date DESC, created_at DESC
             LIMIT {}",
            limit.unwrap_or(50)
        )
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let snapshots = if let Some(mt) = meeting_type {
        stmt.query_map(params![mt], |row| {
            Ok(MeetingSnapshotSummary {
                snapshot_id: row.get(0)?,
                meeting_type: row.get(1)?,
                meeting_date: row.get(2)?,
                snapshot_name: row.get(3)?,
                strategy_name: row.get(4)?,
                strategy_version_number: row.get(5)?,
                consensus_status: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                risk_count: row.get(9)?,
                change_count: row.get(10)?,
                action_count: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], |row| {
            Ok(MeetingSnapshotSummary {
                snapshot_id: row.get(0)?,
                meeting_type: row.get(1)?,
                meeting_date: row.get(2)?,
                snapshot_name: row.get(3)?,
                strategy_name: row.get(4)?,
                strategy_version_number: row.get(5)?,
                consensus_status: row.get(6)?,
                created_by: row.get(7)?,
                created_at: row.get(8)?,
                risk_count: row.get(9)?,
                change_count: row.get(10)?,
                action_count: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(snapshots)
}

/// 获取单个会议快照
pub fn get_meeting_snapshot(snapshot_id: i64) -> Result<MeetingSnapshot, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.query_row(
        "SELECT snapshot_id, meeting_type, meeting_date, snapshot_name,
                strategy_version_id, strategy_name, kpi_summary, risk_summary,
                recommendation, contract_rankings, ranking_changes,
                consensus_status, approved_by, approved_at, created_by, created_at, updated_at
         FROM meeting_snapshot
         WHERE snapshot_id = ?",
        params![snapshot_id],
        |row| {
            Ok(MeetingSnapshot {
                snapshot_id: row.get(0)?,
                meeting_type: row.get(1)?,
                meeting_date: row.get(2)?,
                snapshot_name: row.get(3)?,
                strategy_version_id: row.get(4)?,
                strategy_name: row.get(5)?,
                kpi_summary: row.get(6)?,
                risk_summary: row.get(7)?,
                recommendation: row.get(8)?,
                contract_rankings: row.get(9)?,
                ranking_changes: row.get(10)?,
                consensus_status: row.get(11)?,
                approved_by: row.get(12)?,
                approved_at: row.get(13)?,
                created_by: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        },
    )
    .map_err(|e| format!("获取会议快照失败: {}", e))
}

/// 更新会议快照状态
pub fn update_meeting_snapshot_status(
    snapshot_id: i64,
    status: &str,
    approved_by: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    if status == "approved" && approved_by.is_some() {
        conn.execute(
            "UPDATE meeting_snapshot
             SET consensus_status = ?, approved_by = ?, approved_at = datetime('now','localtime'),
                 updated_at = datetime('now','localtime')
             WHERE snapshot_id = ?",
            params![status, approved_by, snapshot_id],
        )
        .map_err(|e| format!("更新快照状态失败: {}", e))?;
    } else {
        conn.execute(
            "UPDATE meeting_snapshot
             SET consensus_status = ?, updated_at = datetime('now','localtime')
             WHERE snapshot_id = ?",
            params![status, snapshot_id],
        )
        .map_err(|e| format!("更新快照状态失败: {}", e))?;
    }

    Ok(())
}

/// 删除会议快照
pub fn delete_meeting_snapshot(snapshot_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM meeting_snapshot WHERE snapshot_id = ?",
        params![snapshot_id],
    )
    .map_err(|e| format!("删除会议快照失败: {}", e))?;

    Ok(())
}

// ============================================
// KPI 配置查询
// ============================================

/// 获取所有启用的 KPI 配置
pub fn list_meeting_kpi_configs() -> Result<Vec<MeetingKpiConfig>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT kpi_id, kpi_code, kpi_name, kpi_category, calculation_type,
                    calculation_formula, data_source, filter_condition,
                    display_format, display_unit, decimal_places,
                    threshold_good, threshold_warning, threshold_danger, threshold_direction,
                    sort_order, enabled, description, business_meaning, created_at, updated_at
             FROM meeting_kpi_config
             WHERE enabled = 1
             ORDER BY kpi_category, sort_order",
        )
        .map_err(|e| e.to_string())?;

    let configs = stmt
        .query_map([], |row| {
            Ok(MeetingKpiConfig {
                kpi_id: row.get(0)?,
                kpi_code: row.get(1)?,
                kpi_name: row.get(2)?,
                kpi_category: row.get(3)?,
                calculation_type: row.get(4)?,
                calculation_formula: row.get(5)?,
                data_source: row.get(6)?,
                filter_condition: row.get(7)?,
                display_format: row.get(8)?,
                display_unit: row.get(9)?,
                decimal_places: row.get(10)?,
                threshold_good: row.get(11)?,
                threshold_warning: row.get(12)?,
                threshold_danger: row.get(13)?,
                threshold_direction: row.get(14)?,
                sort_order: row.get(15)?,
                enabled: row.get(16)?,
                description: row.get(17)?,
                business_meaning: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(configs)
}

/// 获取指定类别的 KPI 配置
pub fn list_meeting_kpi_configs_by_category(
    category: &str,
) -> Result<Vec<MeetingKpiConfig>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT kpi_id, kpi_code, kpi_name, kpi_category, calculation_type,
                    calculation_formula, data_source, filter_condition,
                    display_format, display_unit, decimal_places,
                    threshold_good, threshold_warning, threshold_danger, threshold_direction,
                    sort_order, enabled, description, business_meaning, created_at, updated_at
             FROM meeting_kpi_config
             WHERE kpi_category = ? AND enabled = 1
             ORDER BY sort_order",
        )
        .map_err(|e| e.to_string())?;

    let configs = stmt
        .query_map(params![category], |row| {
            Ok(MeetingKpiConfig {
                kpi_id: row.get(0)?,
                kpi_code: row.get(1)?,
                kpi_name: row.get(2)?,
                kpi_category: row.get(3)?,
                calculation_type: row.get(4)?,
                calculation_formula: row.get(5)?,
                data_source: row.get(6)?,
                filter_condition: row.get(7)?,
                display_format: row.get(8)?,
                display_unit: row.get(9)?,
                decimal_places: row.get(10)?,
                threshold_good: row.get(11)?,
                threshold_warning: row.get(12)?,
                threshold_danger: row.get(13)?,
                threshold_direction: row.get(14)?,
                sort_order: row.get(15)?,
                enabled: row.get(16)?,
                description: row.get(17)?,
                business_meaning: row.get(18)?,
                created_at: row.get(19)?,
                updated_at: row.get(20)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(configs)
}

// ============================================
// 风险合同标记 CRUD
// ============================================

/// 创建风险合同标记
pub fn create_risk_contract_flag(
    snapshot_id: i64,
    contract_id: &str,
    risk_type: &str,
    risk_level: &str,
    risk_score: Option<f64>,
    risk_description: &str,
    risk_factors: Option<&str>,
    affected_kpis: Option<&str>,
    potential_loss: Option<f64>,
    suggested_action: Option<&str>,
    action_priority: Option<i64>,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO risk_contract_flag
            (snapshot_id, contract_id, risk_type, risk_level, risk_score,
             risk_description, risk_factors, affected_kpis, potential_loss,
             suggested_action, action_priority, status)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'open')",
        params![
            snapshot_id,
            contract_id,
            risk_type,
            risk_level,
            risk_score,
            risk_description,
            risk_factors,
            affected_kpis,
            potential_loss,
            suggested_action,
            action_priority
        ],
    )
    .map_err(|e| format!("创建风险标记失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取会议快照的风险合同列表
pub fn list_risk_contracts_by_snapshot(snapshot_id: i64) -> Result<Vec<RiskContractFlag>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT flag_id, snapshot_id, contract_id, risk_type, risk_level, risk_score,
                    risk_description, risk_factors, affected_kpis, potential_loss,
                    potential_loss_unit, suggested_action, action_priority,
                    status, handled_by, handled_at, handling_note, created_at, updated_at
             FROM risk_contract_flag
             WHERE snapshot_id = ?
             ORDER BY risk_level DESC, risk_score DESC",
        )
        .map_err(|e| e.to_string())?;

    let flags = stmt
        .query_map(params![snapshot_id], |row| {
            Ok(RiskContractFlag {
                flag_id: row.get(0)?,
                snapshot_id: row.get(1)?,
                contract_id: row.get(2)?,
                risk_type: row.get(3)?,
                risk_level: row.get(4)?,
                risk_score: row.get(5)?,
                risk_description: row.get(6)?,
                risk_factors: row.get(7)?,
                affected_kpis: row.get(8)?,
                potential_loss: row.get(9)?,
                potential_loss_unit: row.get(10)?,
                suggested_action: row.get(11)?,
                action_priority: row.get(12)?,
                status: row.get(13)?,
                handled_by: row.get(14)?,
                handled_at: row.get(15)?,
                handling_note: row.get(16)?,
                created_at: row.get(17)?,
                updated_at: row.get(18)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(flags)
}

/// 更新风险合同处理状态
pub fn update_risk_contract_status(
    flag_id: i64,
    status: &str,
    handled_by: &str,
    handling_note: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE risk_contract_flag
         SET status = ?, handled_by = ?, handled_at = datetime('now','localtime'),
             handling_note = ?, updated_at = datetime('now','localtime')
         WHERE flag_id = ?",
        params![status, handled_by, handling_note, flag_id],
    )
    .map_err(|e| format!("更新风险状态失败: {}", e))?;

    Ok(())
}

// ============================================
// 排名变化明细
// ============================================

/// 批量保存排名变化明细
pub fn save_ranking_change_details(
    snapshot_id: i64,
    compare_snapshot_id: Option<i64>,
    details: &[RankingChangeDetail],
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute("BEGIN TRANSACTION", [])
        .map_err(|e| format!("开启事务失败: {}", e))?;

    let mut count = 0i64;

    for detail in details {
        let result = conn.execute(
            "INSERT INTO ranking_change_detail
                (snapshot_id, compare_snapshot_id, contract_id,
                 old_rank, new_rank, rank_change,
                 old_priority, new_priority, priority_change,
                 old_s_score, new_s_score, s_score_change,
                 s1_change, s1_old, s1_new,
                 s2_change, s2_old, s2_new,
                 s3_change, s3_old, s3_new,
                 old_p_score, new_p_score, p_score_change,
                 p1_change, p1_old, p1_new,
                 p2_change, p2_old, p2_new,
                 p3_change, p3_old, p3_new,
                 primary_factor, primary_factor_name, explain_text,
                 ws_used, wp_used)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?,
                     ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                snapshot_id,
                compare_snapshot_id,
                detail.contract_id,
                detail.old_rank,
                detail.new_rank,
                detail.rank_change,
                detail.old_priority,
                detail.new_priority,
                detail.priority_change,
                detail.old_s_score,
                detail.new_s_score,
                detail.s_score_change,
                detail.s1_change,
                detail.s1_old,
                detail.s1_new,
                detail.s2_change,
                detail.s2_old,
                detail.s2_new,
                detail.s3_change,
                detail.s3_old,
                detail.s3_new,
                detail.old_p_score,
                detail.new_p_score,
                detail.p_score_change,
                detail.p1_change,
                detail.p1_old,
                detail.p1_new,
                detail.p2_change,
                detail.p2_old,
                detail.p2_new,
                detail.p3_change,
                detail.p3_old,
                detail.p3_new,
                detail.primary_factor,
                detail.primary_factor_name,
                detail.explain_text,
                detail.ws_used,
                detail.wp_used
            ],
        );

        if let Err(e) = result {
            let _ = conn.execute("ROLLBACK", []);
            return Err(format!("保存排名变化失败: {}", e));
        }

        count += 1;
    }

    conn.execute("COMMIT", []).map_err(|e| {
        let _ = conn.execute("ROLLBACK", []);
        format!("提交事务失败: {}", e)
    })?;

    Ok(count)
}

/// 获取会议快照的排名变化明细
pub fn list_ranking_changes_by_snapshot(
    snapshot_id: i64,
    limit: Option<i64>,
) -> Result<Vec<RankingChangeDetail>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = format!(
        "SELECT change_id, snapshot_id, compare_snapshot_id, contract_id,
                old_rank, new_rank, rank_change,
                old_priority, new_priority, priority_change,
                old_s_score, new_s_score, s_score_change,
                s1_change, s1_old, s1_new,
                s2_change, s2_old, s2_new,
                s3_change, s3_old, s3_new,
                old_p_score, new_p_score, p_score_change,
                p1_change, p1_old, p1_new,
                p2_change, p2_old, p2_new,
                p3_change, p3_old, p3_new,
                primary_factor, primary_factor_name, explain_text,
                ws_used, wp_used, created_at
         FROM ranking_change_detail
         WHERE snapshot_id = ?
         ORDER BY ABS(rank_change) DESC
         LIMIT {}",
        limit.unwrap_or(100)
    );

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let details = stmt
        .query_map(params![snapshot_id], |row| {
            Ok(RankingChangeDetail {
                change_id: row.get(0)?,
                snapshot_id: row.get(1)?,
                compare_snapshot_id: row.get(2)?,
                contract_id: row.get(3)?,
                old_rank: row.get(4)?,
                new_rank: row.get(5)?,
                rank_change: row.get(6)?,
                old_priority: row.get(7)?,
                new_priority: row.get(8)?,
                priority_change: row.get(9)?,
                old_s_score: row.get(10)?,
                new_s_score: row.get(11)?,
                s_score_change: row.get(12)?,
                s1_change: row.get(13)?,
                s1_old: row.get(14)?,
                s1_new: row.get(15)?,
                s2_change: row.get(16)?,
                s2_old: row.get(17)?,
                s2_new: row.get(18)?,
                s3_change: row.get(19)?,
                s3_old: row.get(20)?,
                s3_new: row.get(21)?,
                old_p_score: row.get(22)?,
                new_p_score: row.get(23)?,
                p_score_change: row.get(24)?,
                p1_change: row.get(25)?,
                p1_old: row.get(26)?,
                p1_new: row.get(27)?,
                p2_change: row.get(28)?,
                p2_old: row.get(29)?,
                p2_new: row.get(30)?,
                p3_change: row.get(31)?,
                p3_old: row.get(32)?,
                p3_new: row.get(33)?,
                primary_factor: row.get(34)?,
                primary_factor_name: row.get(35)?,
                explain_text: row.get(36)?,
                ws_used: row.get(37)?,
                wp_used: row.get(38)?,
                created_at: row.get(39)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(details)
}

// ============================================
// 共识模板
// ============================================

/// 获取共识模板列表
pub fn list_consensus_templates(
    meeting_type: Option<&str>,
) -> Result<Vec<ConsensusTemplate>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let query = if meeting_type.is_some() {
        "SELECT template_id, template_code, template_name, meeting_type,
                template_config, output_formats, is_default, enabled,
                description, created_by, created_at, updated_at
         FROM consensus_template
         WHERE (meeting_type = ?1 OR meeting_type = 'all') AND enabled = 1
         ORDER BY is_default DESC, template_name"
    } else {
        "SELECT template_id, template_code, template_name, meeting_type,
                template_config, output_formats, is_default, enabled,
                description, created_by, created_at, updated_at
         FROM consensus_template
         WHERE enabled = 1
         ORDER BY meeting_type, is_default DESC, template_name"
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let templates = if let Some(mt) = meeting_type {
        stmt.query_map(params![mt], |row| {
            Ok(ConsensusTemplate {
                template_id: row.get(0)?,
                template_code: row.get(1)?,
                template_name: row.get(2)?,
                meeting_type: row.get(3)?,
                template_config: row.get(4)?,
                output_formats: row.get(5)?,
                is_default: row.get(6)?,
                enabled: row.get(7)?,
                description: row.get(8)?,
                created_by: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    } else {
        stmt.query_map([], |row| {
            Ok(ConsensusTemplate {
                template_id: row.get(0)?,
                template_code: row.get(1)?,
                template_name: row.get(2)?,
                meeting_type: row.get(3)?,
                template_config: row.get(4)?,
                output_formats: row.get(5)?,
                is_default: row.get(6)?,
                enabled: row.get(7)?,
                description: row.get(8)?,
                created_by: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?
    };

    Ok(templates)
}

/// 获取默认共识模板
pub fn get_default_consensus_template(
    meeting_type: &str,
) -> Result<Option<ConsensusTemplate>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let result = conn.query_row(
        "SELECT template_id, template_code, template_name, meeting_type,
                template_config, output_formats, is_default, enabled,
                description, created_by, created_at, updated_at
         FROM consensus_template
         WHERE (meeting_type = ? OR meeting_type = 'all') AND is_default = 1 AND enabled = 1
         LIMIT 1",
        params![meeting_type],
        |row| {
            Ok(ConsensusTemplate {
                template_id: row.get(0)?,
                template_code: row.get(1)?,
                template_name: row.get(2)?,
                meeting_type: row.get(3)?,
                template_config: row.get(4)?,
                output_formats: row.get(5)?,
                is_default: row.get(6)?,
                enabled: row.get(7)?,
                description: row.get(8)?,
                created_by: row.get(9)?,
                created_at: row.get(10)?,
                updated_at: row.get(11)?,
            })
        },
    );

    match result {
        Ok(template) => Ok(Some(template)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

// ============================================
// 会议行动项 CRUD
// ============================================

/// 创建会议行动项
pub fn create_meeting_action_item(
    snapshot_id: i64,
    action_title: &str,
    action_description: Option<&str>,
    action_category: Option<&str>,
    priority: i64,
    due_date: Option<&str>,
    assignee: Option<&str>,
    department: Option<&str>,
    related_contracts: Option<&str>,
    user: &str,
) -> Result<i64, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO meeting_action_item
            (snapshot_id, action_title, action_description, action_category,
             priority, due_date, assignee, department, related_contracts,
             status, created_by)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'open', ?)",
        params![
            snapshot_id,
            action_title,
            action_description,
            action_category,
            priority,
            due_date,
            assignee,
            department,
            related_contracts,
            user
        ],
    )
    .map_err(|e| format!("创建行动项失败: {}", e))?;

    Ok(conn.last_insert_rowid())
}

/// 获取会议快照的行动项列表
pub fn list_action_items_by_snapshot(snapshot_id: i64) -> Result<Vec<MeetingActionItem>, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let mut stmt = conn
        .prepare(
            "SELECT action_id, snapshot_id, action_title, action_description, action_category,
                    priority, due_date, assignee, department, related_contracts,
                    status, completion_rate, completed_at, notes, created_by, created_at, updated_at
             FROM meeting_action_item
             WHERE snapshot_id = ?
             ORDER BY priority, due_date",
        )
        .map_err(|e| e.to_string())?;

    let items = stmt
        .query_map(params![snapshot_id], |row| {
            Ok(MeetingActionItem {
                action_id: row.get(0)?,
                snapshot_id: row.get(1)?,
                action_title: row.get(2)?,
                action_description: row.get(3)?,
                action_category: row.get(4)?,
                priority: row.get(5)?,
                due_date: row.get(6)?,
                assignee: row.get(7)?,
                department: row.get(8)?,
                related_contracts: row.get(9)?,
                status: row.get(10)?,
                completion_rate: row.get(11)?,
                completed_at: row.get(12)?,
                notes: row.get(13)?,
                created_by: row.get(14)?,
                created_at: row.get(15)?,
                updated_at: row.get(16)?,
            })
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    Ok(items)
}

/// 更新行动项状态
pub fn update_action_item_status(
    action_id: i64,
    status: &str,
    completion_rate: Option<i64>,
    notes: Option<&str>,
) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    if status == "completed" {
        conn.execute(
            "UPDATE meeting_action_item
             SET status = ?, completion_rate = 100, completed_at = datetime('now','localtime'),
                 notes = ?, updated_at = datetime('now','localtime')
             WHERE action_id = ?",
            params![status, notes, action_id],
        )
        .map_err(|e| format!("更新行动项状态失败: {}", e))?;
    } else {
        conn.execute(
            "UPDATE meeting_action_item
             SET status = ?, completion_rate = ?, notes = ?,
                 updated_at = datetime('now','localtime')
             WHERE action_id = ?",
            params![status, completion_rate, notes, action_id],
        )
        .map_err(|e| format!("更新行动项状态失败: {}", e))?;
    }

    Ok(())
}

/// 删除行动项
pub fn delete_action_item(action_id: i64) -> Result<(), String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM meeting_action_item WHERE action_id = ?",
        params![action_id],
    )
    .map_err(|e| format!("删除行动项失败: {}", e))?;

    Ok(())
}

// ============================================
// 风险汇总查询
// ============================================

/// 获取风险汇总统计
pub fn get_risk_summary_stats(snapshot_id: i64) -> Result<serde_json::Value, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    // 按类型和等级统计
    let mut stmt = conn
        .prepare(
            "SELECT risk_type, risk_level, COUNT(*) as count, SUM(COALESCE(potential_loss, 0)) as total_loss
             FROM risk_contract_flag
             WHERE snapshot_id = ?
             GROUP BY risk_type, risk_level"
        )
        .map_err(|e| e.to_string())?;

    let stats: Vec<(String, String, i64, f64)> = stmt
        .query_map(params![snapshot_id], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })
        .map_err(|e| e.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| e.to_string())?;

    // 汇总统计
    let mut by_type: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut by_level: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
    let mut total_count = 0i64;
    let mut total_loss = 0.0f64;

    for (risk_type, risk_level, count, loss) in stats {
        *by_type.entry(risk_type).or_insert(0) += count;
        *by_level.entry(risk_level).or_insert(0) += count;
        total_count += count;
        total_loss += loss;
    }

    Ok(serde_json::json!({
        "total_risk_count": total_count,
        "by_type": by_type,
        "by_level": by_level,
        "total_potential_loss": total_loss,
        "high_risk_count": by_level.get("high").unwrap_or(&0),
        "medium_risk_count": by_level.get("medium").unwrap_or(&0),
        "low_risk_count": by_level.get("low").unwrap_or(&0)
    }))
}

// ============================================
// 排名变化统计
// ============================================

/// 获取排名变化统计
pub fn get_ranking_change_stats(snapshot_id: i64) -> Result<serde_json::Value, String> {
    let conn = get_connection().map_err(|e| e.to_string())?;

    let stats = conn.query_row(
        "SELECT
            COUNT(*) as total,
            SUM(CASE WHEN rank_change > 0 THEN 1 ELSE 0 END) as up_count,
            SUM(CASE WHEN rank_change < 0 THEN 1 ELSE 0 END) as down_count,
            SUM(CASE WHEN rank_change = 0 OR rank_change IS NULL THEN 1 ELSE 0 END) as unchanged_count,
            AVG(ABS(COALESCE(rank_change, 0))) as avg_change,
            MAX(COALESCE(rank_change, 0)) as max_up,
            MIN(COALESCE(rank_change, 0)) as max_down
         FROM ranking_change_detail
         WHERE snapshot_id = ?",
        params![snapshot_id],
        |row| {
            Ok(serde_json::json!({
                "total_contracts": row.get::<_, i64>(0)?,
                "up_count": row.get::<_, i64>(1)?,
                "down_count": row.get::<_, i64>(2)?,
                "unchanged_count": row.get::<_, i64>(3)?,
                "avg_change": row.get::<_, f64>(4)?,
                "max_up": row.get::<_, i64>(5)?,
                "max_down": row.get::<_, i64>(6)?
            }))
        },
    );

    match stats {
        Ok(s) => Ok(s),
        Err(_) => Ok(serde_json::json!({
            "total_contracts": 0,
            "up_count": 0,
            "down_count": 0,
            "unchanged_count": 0,
            "avg_change": 0.0,
            "max_up": 0,
            "max_down": 0
        })),
    }
}
