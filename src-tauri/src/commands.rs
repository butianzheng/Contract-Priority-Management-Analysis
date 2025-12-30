use crate::db::{self, ContractPriority, InterventionLog, ScoringConfigItem, ConfigChangeLog, FilterPreset, BatchOperation, UnifiedHistoryEntry, Contract, Customer, ProcessDifficulty, StrategyWeights, PriorityExplain};
// Phase 16: 会议驾驶舱类型导入
use crate::db::{MeetingSnapshot, MeetingSnapshotSummary, MeetingKpiConfig, RiskContractFlag, RankingChangeDetail, ConsensusTemplate, MeetingActionItem};
use crate::scoring::{self, SScoreInput, PScoreInput, generate_s_score_explain, generate_p_score_explain_with_aggregation};
use crate::config::{self, StrategyScoringWeights};  // 🆕 导入配置模块
use crate::io::{self, FileFormat, ImportDataType, ConflictStrategy, ImportPreview, ImportResult, ExportOptions, ExportResult, ConflictRecord};

/// 计算单个合同的优先级
#[tauri::command]
pub async fn compute_priority(contract_id: String, strategy: String) -> Result<f64, String> {
    // 1. 获取合同数据
    let contract = db::get_contract(&contract_id)?;

    // 2. 获取客户数据（若缺失则使用默认兜底，避免整单计算失败）
    let customer = match db::get_customer(&contract.customer_id) {
        Ok(c) => c,
        Err(err) => {
            eprintln!("Warning: customer '{}' missing ({}), using default level C", contract.customer_id, err);
            db::Customer {
                customer_id: contract.customer_id.clone(),
                customer_name: None,
                customer_level: "C".to_string(),
                credit_level: None,
                customer_group: None,
            }
        }
    };

    // 3. 获取策略权重
    let weights = db::get_strategy_weights(&strategy)?;

    // 4. 🆕 加载评分配置
    let scoring_config = config::load_scoring_config()?;

    // 5. 🆕 获取策略的 S-Score 子权重
    let s_weights = config::load_strategy_scoring_weights(&strategy)?;

    // 6. 计算 S-Score（使用配置化参数）
    let s_input = SScoreInput {
        customer_level: customer.customer_level,
        margin: contract.margin,
        days_to_pdd: contract.days_to_pdd,
    };
    let s_score = scoring::calc_s_score(s_input, s_weights.w1, s_weights.w2, s_weights.w3, &scoring_config);

    // 7. 🆕 获取合同的聚合统计
    let aggregation_stats = db::get_contract_aggregation_stats(
        &contract.spec_family,
        &contract.steel_grade,
        contract.thickness,
        contract.width,
    )?;

    // 8. 🆕 获取 P2 曲线配置
    let p2_curve_config = db::get_p2_curve_config()?;

    // 9. 计算 P-Score（使用配置化参数 + 聚合统计）
    // 静态规则：仅使用合同属性、配置表数据和合同池快照
    let p_input = PScoreInput {
        steel_grade: contract.steel_grade,
        thickness: contract.thickness,
        width: contract.width,
        spec_family: contract.spec_family,
        days_to_pdd: contract.days_to_pdd,  // P3 输入：节拍匹配
    };
    let p_score = scoring::calc_p_score_with_aggregation(
        p_input,
        &scoring_config,
        Some(&aggregation_stats),
        Some(&p2_curve_config),
    )?;

    // 10. 计算综合优先级
    let mut priority = scoring::calc_priority(s_score, p_score, weights.ws, weights.wp);

    // 11. 应用人工调整系数（如果有）
    if let Ok(Some(alpha)) = db::get_latest_alpha(&contract_id) {
        priority = scoring::apply_alpha(priority, alpha);
    }

    Ok(priority)
}

/// 批量计算所有合同的优先级
#[tauri::command]
pub async fn compute_all_priorities(strategy: String) -> Result<Vec<ContractPriority>, String> {
    // 1. 获取所有合同
    let contracts = db::list_contracts()?;

    // 2. 获取策略权重
    let weights = db::get_strategy_weights(&strategy)?;

    // 3. 🆕 加载评分配置（批量计算只需加载一次）
    let scoring_config = config::load_scoring_config()?;

    // 4. 🆕 获取策略的 S-Score 子权重
    let s_weights = config::load_strategy_scoring_weights(&strategy)?;

    // 5. 🆕 批量获取所有合同的聚合统计（一次性计算，避免重复查询）
    let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;

    // 6. 🆕 获取 P2 曲线配置
    let p2_curve_config = db::get_p2_curve_config()?;

    // 7. 🔥 性能优化：批量获取客户数据和 alpha 值（避免 N+1 查询）
    let customers_map = db::get_all_customers_map()?;
    let alphas_map = db::get_all_latest_alphas()?;

    // 8. 批量计算
    let mut results = Vec::new();

    for contract in contracts {
        // 🔥 优化：从 HashMap 获取客户数据（O(1) 查找）
        let customer = customers_map.get(&contract.customer_id)
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("Warning: customer '{}' missing, using default level C", contract.customer_id);
                db::Customer {
                    customer_id: contract.customer_id.clone(),
                    customer_name: None,
                    customer_level: "C".to_string(),
                    credit_level: None,
                    customer_group: None,
                }
            });

        // 计算 S-Score（使用配置化参数）
        let s_input = SScoreInput {
            customer_level: customer.customer_level,
            margin: contract.margin,
            days_to_pdd: contract.days_to_pdd,
        };
        let s_score = scoring::calc_s_score(s_input, s_weights.w1, s_weights.w2, s_weights.w3, &scoring_config);

        // 计算 P-Score（使用配置化参数 + 聚合统计）
        // 静态规则：仅使用合同属性、配置表数据和合同池快照
        let p_input = PScoreInput {
            steel_grade: contract.steel_grade.clone(),
            thickness: contract.thickness,
            width: contract.width,
            spec_family: contract.spec_family.clone(),
            days_to_pdd: contract.days_to_pdd,  // P3 输入：节拍匹配
        };

        // 🆕 使用新的 P-Score 计算接口（带聚合统计）
        let aggregation_stats = aggregation_stats_map.get(&contract.contract_id);
        let p_score = scoring::calc_p_score_with_aggregation(
            p_input,
            &scoring_config,
            aggregation_stats,
            Some(&p2_curve_config),
        )?;

        // 计算综合优先级
        let mut priority = scoring::calc_priority(s_score, p_score, weights.ws, weights.wp);

        // 🔥 优化：从 HashMap 获取 alpha 值（O(1) 查找）
        let alpha = alphas_map.get(&contract.contract_id).copied();
        if let Some(a) = alpha {
            priority = scoring::apply_alpha(priority, a);
        }

        results.push(ContractPriority {
            contract,
            s_score,
            p_score,
            priority,
            alpha,
        });
    }

    // 按优先级降序排序
    results.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

    Ok(results)
}

/// 获取合同列表
#[tauri::command]
pub async fn get_contracts() -> Result<Vec<db::Contract>, String> {
    db::list_contracts()
}

/// 获取策略列表
#[tauri::command]
pub async fn get_strategies() -> Result<Vec<String>, String> {
    db::list_strategies()
}

/// 设置人工调整系数 alpha
/// Alpha 范围：[0.5, 2.0]，默认 1.0
#[tauri::command]
pub async fn set_alpha(
    contract_id: String,
    alpha: f64,
    reason: String,
    user: String,
) -> Result<(), String> {
    // 验证 Alpha 范围
    if alpha < 0.5 || alpha > 2.0 {
        return Err(format!(
            "Alpha 值必须在 0.5 ~ 2.0 之间，当前值: {}",
            alpha
        ));
    }

    // 验证调整原因不能为空
    if reason.trim().is_empty() {
        return Err("调整原因不能为空".to_string());
    }

    let log = InterventionLog {
        id: None,
        contract_id,
        alpha_value: alpha,
        reason,
        user,
        timestamp: None,
    };

    db::log_intervention(log)
}

/// 获取合同的干预历史
#[tauri::command]
pub async fn get_intervention_history(contract_id: String) -> Result<Vec<InterventionLog>, String> {
    db::get_intervention_history(&contract_id)
}

/// 获取所有干预日志（支持分页）
#[tauri::command]
pub async fn get_all_intervention_logs(limit: Option<i64>) -> Result<Vec<InterventionLog>, String> {
    db::get_all_intervention_logs(limit)
}

// ============================================
// Phase 2: 配置管理相关命令
// ============================================

/// 获取所有评分配置项
#[tauri::command]
pub async fn get_scoring_configs() -> Result<Vec<ScoringConfigItem>, String> {
    db::list_scoring_configs()
}

/// 更新配置项
#[tauri::command]
pub async fn update_config(
    config_key: String,
    new_value: String,
    changed_by: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::update_scoring_config(&config_key, &new_value, &changed_by, reason.as_deref())
}

/// 获取配置变更历史
#[tauri::command]
pub async fn get_config_history(
    config_key: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ConfigChangeLog>, String> {
    db::get_config_change_history(config_key.as_deref(), limit)
}

/// 回滚配置
#[tauri::command]
pub async fn rollback_config(
    log_id: i64,
    changed_by: String,
    reason: String,
) -> Result<(), String> {
    db::rollback_config(log_id, &changed_by, &reason)
}

/// 获取所有策略的评分权重 (w1, w2, w3)
#[tauri::command]
pub async fn get_all_strategy_weights() -> Result<Vec<StrategyScoringWeights>, String> {
    db::list_all_strategy_scoring_weights()
}

/// 获取所有策略权重 (ws, wp)
#[tauri::command]
pub async fn get_strategy_weights_list() -> Result<Vec<StrategyWeights>, String> {
    db::list_all_strategy_weights()
}

/// 更新策略的评分权重
#[tauri::command]
pub async fn update_strategy_weights(
    strategy_name: String,
    w1: f64,
    w2: f64,
    w3: f64,
) -> Result<(), String> {
    db::update_strategy_scoring_weights(&strategy_name, w1, w2, w3)
}

/// 创建或更新策略权重 (ws, wp)
#[tauri::command]
pub async fn upsert_strategy_weight(
    strategy_name: String,
    ws: f64,
    wp: f64,
    description: Option<String>,
) -> Result<(), String> {
    db::upsert_strategy_weight(&strategy_name, ws, wp, description.as_deref())
}

/// 删除策略权重
#[tauri::command]
pub async fn delete_strategy_weight(strategy_name: String) -> Result<(), String> {
    db::delete_strategy_weight(&strategy_name)
}

// ============================================
// Phase 4: 筛选器预设相关命令
// ============================================

/// 获取所有筛选器预设
#[tauri::command]
pub async fn get_filter_presets() -> Result<Vec<FilterPreset>, String> {
    db::list_filter_presets()
}

/// 保存筛选器预设
#[tauri::command]
pub async fn save_filter_preset(
    preset_name: String,
    filter_json: String,
    description: String,
    created_by: String,
) -> Result<(), String> {
    db::save_filter_preset(&preset_name, &filter_json, &description, &created_by)
}

/// 删除筛选器预设
#[tauri::command]
pub async fn delete_filter_preset(preset_id: i64) -> Result<(), String> {
    db::delete_filter_preset(preset_id)
}

/// 设置默认筛选器预设
#[tauri::command]
pub async fn set_default_filter_preset(preset_id: i64) -> Result<(), String> {
    db::set_default_filter_preset(preset_id)
}

// ============================================
// Phase 5: 批量操作相关命令
// ============================================

/// 批量调整合同的 alpha 值
/// Alpha 范围：[0.5, 2.0]，默认 1.0
#[tauri::command]
pub async fn batch_adjust_alpha(
    contract_ids: Vec<String>,
    alpha: f64,
    reason: String,
    user: String,
) -> Result<i64, String> {
    // 验证 Alpha 范围
    if alpha < 0.5 || alpha > 2.0 {
        return Err(format!(
            "Alpha 值必须在 0.5 ~ 2.0 之间，当前值: {}",
            alpha
        ));
    }

    // 验证调整原因不能为空
    if reason.trim().is_empty() {
        return Err("调整原因不能为空".to_string());
    }

    db::batch_adjust_alpha(contract_ids, alpha, &reason, &user)
}

/// 批量恢复合同的 alpha 值
#[tauri::command]
pub async fn batch_restore_alpha(
    contract_ids: Vec<String>,
    reason: String,
    user: String,
) -> Result<i64, String> {
    db::batch_restore_alpha(contract_ids, &reason, &user)
}

/// 获取批量操作历史
#[tauri::command]
pub async fn get_batch_operations(limit: Option<i64>) -> Result<Vec<BatchOperation>, String> {
    db::list_batch_operations(limit)
}

/// 获取批量操作涉及的合同列表
#[tauri::command]
pub async fn get_batch_operation_contracts(batch_id: i64) -> Result<Vec<String>, String> {
    db::get_batch_operation_contracts(batch_id)
}

// ============================================
// 统一历史记录查询
// ============================================

/// 获取统一的变更历史（整合配置变更、Alpha调整、批量操作）
#[tauri::command]
pub async fn get_unified_history(
    entry_type: Option<String>,
    user: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<UnifiedHistoryEntry>, String> {
    db::get_unified_history(
        entry_type.as_deref(),
        user.as_deref(),
        limit,
    )
}

// ============================================
// 导入/导出相关命令
// ============================================

/// 预览导入文件
#[tauri::command]
pub async fn preview_import(
    file_path: String,
    data_type: ImportDataType,
    format: FileFormat,
) -> Result<ImportPreview, String> {
    // 1. 读取文件内容
    let content = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 2. 根据格式和类型解析数据
    let (total_rows, valid_rows, errors, conflicts, sample_data) = match (data_type, format) {
        (ImportDataType::Contracts, FileFormat::Csv) => {
            let (contracts, errors) = io::CsvHandler::parse_contracts(&content)?;
            let conflicts = io::ConflictHandler::detect_contract_conflicts(&contracts)?;
            let sample: Vec<serde_json::Value> = contracts.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (contracts.len() + errors.len(), contracts.len(), errors, conflicts, sample)
        }
        (ImportDataType::Contracts, FileFormat::Json) => {
            let (contracts, errors) = io::JsonHandler::parse_contracts(&content)?;
            let conflicts = io::ConflictHandler::detect_contract_conflicts(&contracts)?;
            let sample: Vec<serde_json::Value> = contracts.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (contracts.len() + errors.len(), contracts.len(), errors, conflicts, sample)
        }
        (ImportDataType::Contracts, FileFormat::Excel) => {
            let (contracts, errors) = io::ExcelHandler::parse_contracts(&content)?;
            let conflicts = io::ConflictHandler::detect_contract_conflicts(&contracts)?;
            let sample: Vec<serde_json::Value> = contracts.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (contracts.len() + errors.len(), contracts.len(), errors, conflicts, sample)
        }
        (ImportDataType::Customers, FileFormat::Csv) => {
            let (customers, errors) = io::CsvHandler::parse_customers(&content)?;
            let conflicts = io::ConflictHandler::detect_customer_conflicts(&customers)?;
            let sample: Vec<serde_json::Value> = customers.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (customers.len() + errors.len(), customers.len(), errors, conflicts, sample)
        }
        (ImportDataType::Customers, FileFormat::Json) => {
            let (customers, errors) = io::JsonHandler::parse_customers(&content)?;
            let conflicts = io::ConflictHandler::detect_customer_conflicts(&customers)?;
            let sample: Vec<serde_json::Value> = customers.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (customers.len() + errors.len(), customers.len(), errors, conflicts, sample)
        }
        (ImportDataType::Customers, FileFormat::Excel) => {
            let (customers, errors) = io::ExcelHandler::parse_customers(&content)?;
            let conflicts = io::ConflictHandler::detect_customer_conflicts(&customers)?;
            let sample: Vec<serde_json::Value> = customers.iter().take(5)
                .map(|c| serde_json::to_value(c).unwrap_or_default())
                .collect();
            (customers.len() + errors.len(), customers.len(), errors, conflicts, sample)
        }
        _ => return Err("不支持的数据类型和格式组合".to_string()),
    };

    Ok(ImportPreview {
        total_rows,
        valid_rows,
        error_rows: errors.len(),
        conflicts,
        validation_errors: errors,
        sample_data,
    })
}

/// 执行导入
#[tauri::command]
pub async fn execute_import(
    file_path: String,
    data_type: ImportDataType,
    format: FileFormat,
    conflict_strategy: ConflictStrategy,
    conflict_decisions: Option<Vec<ConflictRecord>>,
) -> Result<ImportResult, String> {
    // 1. 读取文件
    let content = std::fs::read(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 2. 解析数据
    match data_type {
        ImportDataType::Contracts => {
            let (contracts, errors) = match format {
                FileFormat::Csv => io::CsvHandler::parse_contracts(&content)?,
                FileFormat::Json => io::JsonHandler::parse_contracts(&content)?,
                FileFormat::Excel => io::ExcelHandler::parse_contracts(&content)?,
            };

            if !errors.is_empty() && contracts.is_empty() {
                return Ok(ImportResult {
                    success: false,
                    imported_count: 0,
                    skipped_count: 0,
                    error_count: errors.len(),
                    errors,
                    message: "数据验证失败，无有效数据可导入".to_string(),
                });
            }

            // 执行导入
            import_contracts_with_conflict(&contracts, conflict_strategy, conflict_decisions, errors)
        }
        ImportDataType::Customers => {
            let (customers, errors) = match format {
                FileFormat::Csv => io::CsvHandler::parse_customers(&content)?,
                FileFormat::Json => io::JsonHandler::parse_customers(&content)?,
                FileFormat::Excel => io::ExcelHandler::parse_customers(&content)?,
            };

            if !errors.is_empty() && customers.is_empty() {
                return Ok(ImportResult {
                    success: false,
                    imported_count: 0,
                    skipped_count: 0,
                    error_count: errors.len(),
                    errors,
                    message: "数据验证失败，无有效数据可导入".to_string(),
                });
            }

            import_customers_with_conflict(&customers, conflict_strategy, conflict_decisions, errors)
        }
        _ => Err("暂不支持的数据类型".to_string()),
    }
}

fn import_contracts_with_conflict(
    contracts: &[Contract],
    strategy: ConflictStrategy,
    decisions: Option<Vec<ConflictRecord>>,
    parse_errors: Vec<io::ValidationError>,
) -> Result<ImportResult, String> {
    let mut imported = 0;
    let mut skipped = 0;

    for contract in contracts {
        // 检查是否存在
        let exists = db::get_contract_optional(&contract.contract_id)?.is_some();

        let should_import = if exists {
            // 查找用户决策
            let user_decision = decisions.as_ref()
                .and_then(|d| d.iter().find(|r| r.primary_key == contract.contract_id))
                .and_then(|r| r.action);

            match user_decision.unwrap_or(strategy) {
                ConflictStrategy::Skip => false,
                ConflictStrategy::Overwrite => true,
            }
        } else {
            true
        };

        if should_import {
            if exists {
                db::update_contract(contract)?;
            } else {
                db::insert_contract(contract)?;
            }
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(ImportResult {
        success: true,
        imported_count: imported,
        skipped_count: skipped,
        error_count: parse_errors.len(),
        errors: parse_errors,
        message: format!("导入完成：成功 {} 条，跳过 {} 条", imported, skipped),
    })
}

fn import_customers_with_conflict(
    customers: &[Customer],
    strategy: ConflictStrategy,
    decisions: Option<Vec<ConflictRecord>>,
    parse_errors: Vec<io::ValidationError>,
) -> Result<ImportResult, String> {
    let mut imported = 0;
    let mut skipped = 0;

    for customer in customers {
        let exists = db::get_customer_optional(&customer.customer_id)?.is_some();

        let should_import = if exists {
            let user_decision = decisions.as_ref()
                .and_then(|d| d.iter().find(|r| r.primary_key == customer.customer_id))
                .and_then(|r| r.action);

            match user_decision.unwrap_or(strategy) {
                ConflictStrategy::Skip => false,
                ConflictStrategy::Overwrite => true,
            }
        } else {
            true
        };

        if should_import {
            if exists {
                db::update_customer(customer)?;
            } else {
                db::insert_customer(customer)?;
            }
            imported += 1;
        } else {
            skipped += 1;
        }
    }

    Ok(ImportResult {
        success: true,
        imported_count: imported,
        skipped_count: skipped,
        error_count: parse_errors.len(),
        errors: parse_errors,
        message: format!("导入完成：成功 {} 条，跳过 {} 条", imported, skipped),
    })
}

/// 导出数据
#[tauri::command]
pub async fn export_data(
    file_path: String,
    options: ExportOptions,
) -> Result<ExportResult, String> {
    let (content, row_count) = match (options.data_type, options.include_computed) {
        // 导出优先级计算结果
        (ImportDataType::Contracts, true) => {
            let strategy = options.strategy.unwrap_or_else(|| "均衡".to_string());
            let priorities = compute_all_priorities(strategy).await?;
            let count = priorities.len();
            let data = match options.format {
                FileFormat::Csv => io::CsvHandler::generate_priorities(&priorities)?,
                FileFormat::Excel => io::ExcelHandler::generate_priorities(&priorities)?,
                FileFormat::Json => io::JsonHandler::generate_priorities(&priorities)?,
            };
            (data, count)
        }
        // 导出原始合同数据
        (ImportDataType::Contracts, false) => {
            let contracts = db::list_contracts()?;
            let count = contracts.len();
            let data = match options.format {
                FileFormat::Csv => io::CsvHandler::generate_contracts(&contracts)?,
                FileFormat::Excel => io::ExcelHandler::generate_contracts(&contracts)?,
                FileFormat::Json => io::JsonHandler::generate_contracts(&contracts)?,
            };
            (data, count)
        }
        // 导出客户数据
        (ImportDataType::Customers, _) => {
            let customers = db::list_customers()?;
            let count = customers.len();
            let data = match options.format {
                FileFormat::Csv => io::CsvHandler::generate_customers(&customers)?,
                FileFormat::Excel => io::ExcelHandler::generate_customers(&customers)?,
                FileFormat::Json => io::JsonHandler::generate_customers(&customers)?,
            };
            (data, count)
        }
        // 导出工艺难度配置
        (ImportDataType::ProcessDifficulty, _) => {
            let items = db::list_process_difficulty()?;
            let count = items.len();
            let data = match options.format {
                FileFormat::Csv => io::CsvHandler::generate_process_difficulty(&items)?,
                FileFormat::Excel => io::ExcelHandler::generate_process_difficulty(&items)?,
                FileFormat::Json => io::JsonHandler::generate_process_difficulty(&items)?,
            };
            (data, count)
        }
        _ => return Err("不支持的导出类型".to_string()),
    };

    // 写入文件
    std::fs::write(&file_path, &content)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(ExportResult {
        success: true,
        file_path,
        row_count,
        message: format!("导出成功，共 {} 条数据", row_count),
    })
}

/// 获取客户列表
#[tauri::command]
pub async fn get_customers() -> Result<Vec<Customer>, String> {
    db::list_customers()
}

/// 获取工艺难度配置
#[tauri::command]
pub async fn get_process_difficulty() -> Result<Vec<ProcessDifficulty>, String> {
    db::list_process_difficulty()
}

// ============================================
// Phase 8: 清洗规则管理命令
// ============================================

use crate::db::schema::{TransformRule, TransformRuleChangeLog, TransformExecutionLog, RuleTestResult};

/// 获取所有清洗规则
#[tauri::command]
pub async fn get_transform_rules() -> Result<Vec<TransformRule>, String> {
    db::list_transform_rules()
}

/// 按分类获取清洗规则
#[tauri::command]
pub async fn get_transform_rules_by_category(category: String) -> Result<Vec<TransformRule>, String> {
    db::list_transform_rules_by_category(&category)
}

/// 获取单个清洗规则
#[tauri::command]
pub async fn get_transform_rule(rule_id: i64) -> Result<TransformRule, String> {
    db::get_transform_rule(rule_id)
}

/// 创建清洗规则
#[tauri::command]
pub async fn create_transform_rule(
    rule_name: String,
    category: String,
    description: Option<String>,
    priority: i64,
    config_json: String,
    user: String,
) -> Result<i64, String> {
    // 验证 JSON 格式
    serde_json::from_str::<serde_json::Value>(&config_json)
        .map_err(|e| format!("配置 JSON 格式无效: {}", e))?;

    db::create_transform_rule(
        &rule_name,
        &category,
        description.as_deref(),
        priority,
        &config_json,
        &user,
    )
}

/// 更新清洗规则
#[tauri::command]
pub async fn update_transform_rule(
    rule_id: i64,
    rule_name: String,
    description: Option<String>,
    priority: i64,
    config_json: String,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    // 验证 JSON 格式
    serde_json::from_str::<serde_json::Value>(&config_json)
        .map_err(|e| format!("配置 JSON 格式无效: {}", e))?;

    db::update_transform_rule(
        rule_id,
        &rule_name,
        description.as_deref(),
        priority,
        &config_json,
        &user,
        reason.as_deref(),
    )
}

/// 删除清洗规则
#[tauri::command]
pub async fn delete_transform_rule(
    rule_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::delete_transform_rule(rule_id, &user, reason.as_deref())
}

/// 切换规则启用/禁用状态
#[tauri::command]
pub async fn toggle_transform_rule(
    rule_id: i64,
    enabled: bool,
    user: String,
) -> Result<(), String> {
    db::toggle_transform_rule_enabled(rule_id, enabled, &user)
}

/// 获取规则变更历史
#[tauri::command]
pub async fn get_transform_rule_history(
    rule_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TransformRuleChangeLog>, String> {
    db::list_transform_rule_change_log(rule_id, limit.unwrap_or(50))
}

/// 获取规则执行历史
#[tauri::command]
pub async fn get_transform_execution_history(
    rule_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<TransformExecutionLog>, String> {
    db::list_transform_execution_log(rule_id, limit.unwrap_or(50))
}

/// 测试规则（预览模式，不实际修改数据）
#[tauri::command]
pub async fn test_transform_rule(
    rule_id: i64,
    sample_size: Option<i64>,
) -> Result<RuleTestResult, String> {
    let rule = db::get_transform_rule(rule_id)?;
    let contracts = db::list_contracts()?;

    let sample_size = sample_size.unwrap_or(5) as usize;
    let sample_contracts: Vec<_> = contracts.into_iter().take(sample_size).collect();

    // 解析规则配置
    let config: serde_json::Value = serde_json::from_str(&rule.config_json)
        .map_err(|e| format!("规则配置解析失败: {}", e))?;

    // 模拟规则执行（根据规则类型进行不同处理）
    let rule_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("");

    let (output_sample, records_matched) = match rule_type {
        "regex_replace" => {
            // 正则替换示例
            let field = config.get("field").and_then(|v| v.as_str()).unwrap_or("");
            let pattern = config.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
            let replacement = config.get("replacement").and_then(|v| v.as_str()).unwrap_or("");

            let regex = regex::Regex::new(pattern)
                .map_err(|e| format!("正则表达式无效: {}", e))?;

            let mut matched = 0i64;
            let mut results: Vec<serde_json::Value> = vec![];

            for contract in &sample_contracts {
                let field_value = match field {
                    "steel_grade" => &contract.steel_grade,
                    "customer_id" => &contract.customer_id,
                    "contract_id" => &contract.contract_id,
                    _ => continue,
                };

                if regex.is_match(field_value) {
                    matched += 1;
                    let new_value = regex.replace_all(field_value, replacement).to_string();
                    results.push(serde_json::json!({
                        "contract_id": contract.contract_id,
                        "field": field,
                        "old_value": field_value,
                        "new_value": new_value
                    }));
                }
            }

            (serde_json::json!(results), matched)
        }
        "range_classify" => {
            // 范围分类示例
            let source_field = config.get("source_field").and_then(|v| v.as_str()).unwrap_or("");
            let target_field = config.get("target_field").and_then(|v| v.as_str()).unwrap_or("");
            let ranges = config.get("ranges").and_then(|v| v.as_array()).cloned().unwrap_or_default();

            let mut results: Vec<serde_json::Value> = vec![];

            for contract in &sample_contracts {
                let value = match source_field {
                    "thickness" => contract.thickness,
                    "width" => contract.width,
                    "margin" => contract.margin,
                    _ => continue,
                };

                for range in &ranges {
                    let min = range.get("min").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let max = range.get("max").and_then(|v| v.as_f64()).unwrap_or(f64::MAX);
                    let label = range.get("label").and_then(|v| v.as_str()).unwrap_or("");

                    if value >= min && value < max {
                        results.push(serde_json::json!({
                            "contract_id": contract.contract_id,
                            source_field: value,
                            target_field: label
                        }));
                        break;
                    }
                }
            }

            let matched = results.len() as i64;
            (serde_json::json!(results), matched)
        }
        "value_mapping" => {
            // 值映射示例
            let source_field = config.get("source_field").and_then(|v| v.as_str()).unwrap_or("");
            let target_field = config.get("target_field").and_then(|v| v.as_str()).unwrap_or("");
            let mapping = config.get("mapping").cloned().unwrap_or(serde_json::json!({}));
            let default = config.get("default").and_then(|v| v.as_f64()).unwrap_or(0.0);

            let mut results: Vec<serde_json::Value> = vec![];

            // 这里需要关联客户数据才能进行等级映射
            // 简化示例：仅显示映射规则
            results.push(serde_json::json!({
                "type": "value_mapping",
                "source_field": source_field,
                "target_field": target_field,
                "mapping": mapping,
                "default": default,
                "note": "需要关联客户数据进行实际映射"
            }));

            (serde_json::json!(results), sample_contracts.len() as i64)
        }
        "condition_mapping" => {
            // 条件映射示例
            let source_field = config.get("source_field").and_then(|v| v.as_str()).unwrap_or("");
            let target_field = config.get("target_field").and_then(|v| v.as_str()).unwrap_or("");
            let conditions = config.get("conditions").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            let default_label = config.get("default").and_then(|v| v.as_str()).unwrap_or("");

            let mut results: Vec<serde_json::Value> = vec![];

            for contract in &sample_contracts {
                let value = match source_field {
                    "days_to_pdd" => contract.days_to_pdd as f64,
                    "thickness" => contract.thickness,
                    "width" => contract.width,
                    "margin" => contract.margin,
                    _ => continue,
                };

                let mut label = default_label.to_string();
                for cond in &conditions {
                    let operator = cond.get("operator").and_then(|v| v.as_str()).unwrap_or("");
                    let cond_value = cond.get("value").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let cond_label = cond.get("label").and_then(|v| v.as_str()).unwrap_or("");

                    let matched = match operator {
                        "<=" => value <= cond_value,
                        "<" => value < cond_value,
                        ">=" => value >= cond_value,
                        ">" => value > cond_value,
                        "==" => (value - cond_value).abs() < 0.001,
                        _ => false,
                    };

                    if matched {
                        label = cond_label.to_string();
                        break;
                    }
                }

                results.push(serde_json::json!({
                    "contract_id": contract.contract_id,
                    source_field: value,
                    target_field: label
                }));
            }

            let matched = results.len() as i64;
            (serde_json::json!(results), matched)
        }
        _ => {
            // 未知类型，返回规则配置信息
            (serde_json::json!({
                "rule_type": rule_type,
                "config": config,
                "note": "未实现的规则类型预览"
            }), 0)
        }
    };

    Ok(RuleTestResult {
        success: true,
        input_sample: serde_json::json!(sample_contracts.iter().map(|c| {
            serde_json::json!({
                "contract_id": c.contract_id,
                "steel_grade": c.steel_grade,
                "thickness": c.thickness,
                "width": c.width,
                "days_to_pdd": c.days_to_pdd,
                "margin": c.margin
            })
        }).collect::<Vec<_>>()),
        output_sample,
        records_matched,
        error_message: None,
    })
}

/// 执行清洗规则（实际修改数据）
#[tauri::command]
pub async fn execute_transform_rule(
    rule_id: i64,
    user: String,
) -> Result<TransformExecutionLog, String> {
    let rule = db::get_transform_rule(rule_id)?;

    if rule.enabled == 0 {
        return Err("规则已禁用，无法执行".to_string());
    }

    // 获取合同数据
    let contracts = db::list_contracts()?;
    let total_count = contracts.len() as i64;

    // 解析规则配置
    let config: serde_json::Value = serde_json::from_str(&rule.config_json)
        .map_err(|e| format!("规则配置解析失败: {}", e))?;

    let rule_type = config.get("type").and_then(|v| v.as_str()).unwrap_or("");

    // 注意：实际的数据修改需要根据规则类型实现
    // 这里提供框架，具体的数据修改逻辑需要根据业务需求实现

    let (modified_count, status, error_msg): (i64, &str, Option<String>) = match rule_type {
        "regex_replace" | "format_id" => {
            // 字段标准化类规则
            // 这里应该实现实际的数据修改逻辑
            // 由于涉及数据库写操作，需要谨慎实现
            (0, "success", Some("标准化规则已记录，实际数据修改需要在事务中执行".to_string()))
        }
        "range_classify" | "spec_family_classify" => {
            // 分类提取类规则
            // 这类规则通常是计算派生字段，可能需要添加新列
            (0, "success", Some("分类规则预览成功，派生字段计算在优先级计算时动态执行".to_string()))
        }
        "value_mapping" | "range_normalize" => {
            // 归一化类规则
            // 这类规则通常在评分计算时动态应用
            (0, "success", Some("归一化规则已配置，将在评分计算时应用".to_string()))
        }
        "condition_mapping" => {
            // 标签映射类规则
            // 这类规则生成派生标签
            (0, "success", Some("标签映射规则已配置，标签在查询时动态计算".to_string()))
        }
        _ => {
            (0, "failed", Some(format!("未知的规则类型: {}", rule_type)))
        }
    };

    // 记录执行日志
    let log_id = db::log_transform_execution(
        rule_id,
        total_count,
        modified_count,
        status,
        error_msg.as_deref(),
        &user,
    )?;

    // 返回执行结果
    Ok(TransformExecutionLog {
        log_id: Some(log_id),
        rule_id,
        rule_name: Some(rule.rule_name),
        execution_time: Some(chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        records_processed: total_count,
        records_modified: modified_count,
        status: status.to_string(),
        error_message: error_msg,
        executed_by: user,
    })
}

// ============================================
// Phase 9: 规格族管理命令
// ============================================

use crate::db::schema::{SpecFamily, SpecFamilyChangeLog};

/// 获取所有规格族
#[tauri::command]
pub async fn get_spec_families() -> Result<Vec<SpecFamily>, String> {
    db::list_spec_families()
}

/// 获取启用的规格族
#[tauri::command]
pub async fn get_enabled_spec_families() -> Result<Vec<SpecFamily>, String> {
    db::list_enabled_spec_families()
}

/// 获取单个规格族
#[tauri::command]
pub async fn get_spec_family(family_id: i64) -> Result<SpecFamily, String> {
    db::get_spec_family(family_id)
}

/// 创建规格族
#[tauri::command]
pub async fn create_spec_family(
    family_name: String,
    family_code: String,
    description: Option<String>,
    factor: f64,
    steel_grades: Option<String>,
    thickness_min: Option<f64>,
    thickness_max: Option<f64>,
    width_min: Option<f64>,
    width_max: Option<f64>,
    sort_order: i64,
    user: String,
) -> Result<i64, String> {
    // 验证 steel_grades JSON 格式（如果提供）
    if let Some(ref grades) = steel_grades {
        serde_json::from_str::<serde_json::Value>(grades)
            .map_err(|e| format!("钢种列表 JSON 格式无效: {}", e))?;
    }

    db::create_spec_family(
        &family_name,
        &family_code,
        description.as_deref(),
        factor,
        steel_grades.as_deref(),
        thickness_min,
        thickness_max,
        width_min,
        width_max,
        sort_order,
        &user,
    )
}

/// 更新规格族
#[tauri::command]
pub async fn update_spec_family(
    family_id: i64,
    family_name: String,
    family_code: String,
    description: Option<String>,
    factor: f64,
    steel_grades: Option<String>,
    thickness_min: Option<f64>,
    thickness_max: Option<f64>,
    width_min: Option<f64>,
    width_max: Option<f64>,
    sort_order: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    // 验证 steel_grades JSON 格式（如果提供）
    if let Some(ref grades) = steel_grades {
        serde_json::from_str::<serde_json::Value>(grades)
            .map_err(|e| format!("钢种列表 JSON 格式无效: {}", e))?;
    }

    db::update_spec_family(
        family_id,
        &family_name,
        &family_code,
        description.as_deref(),
        factor,
        steel_grades.as_deref(),
        thickness_min,
        thickness_max,
        width_min,
        width_max,
        sort_order,
        &user,
        reason.as_deref(),
    )
}

/// 删除规格族
#[tauri::command]
pub async fn delete_spec_family(
    family_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::delete_spec_family(family_id, &user, reason.as_deref())
}

/// 切换规格族启用/禁用状态
#[tauri::command]
pub async fn toggle_spec_family(
    family_id: i64,
    enabled: bool,
    user: String,
) -> Result<(), String> {
    db::toggle_spec_family_enabled(family_id, enabled, &user)
}

/// 获取规格族变更历史
#[tauri::command]
pub async fn get_spec_family_history(
    family_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<SpecFamilyChangeLog>, String> {
    db::list_spec_family_change_log(family_id, limit.unwrap_or(50))
}

// ============================================
// Phase 10: n日节拍配置管理命令
// ============================================

use crate::db::schema::{RhythmConfig, RhythmLabel, RhythmConfigChangeLog};

/// 获取所有节拍配置
#[tauri::command]
pub async fn get_rhythm_configs() -> Result<Vec<RhythmConfig>, String> {
    db::list_rhythm_configs()
}

/// 获取当前激活的节拍配置
#[tauri::command]
pub async fn get_active_rhythm_config() -> Result<RhythmConfig, String> {
    db::get_active_rhythm_config()
}

/// 获取单个节拍配置
#[tauri::command]
pub async fn get_rhythm_config(config_id: i64) -> Result<RhythmConfig, String> {
    db::get_rhythm_config(config_id)
}

/// 创建节拍配置
#[tauri::command]
pub async fn create_rhythm_config(
    config_name: String,
    cycle_days: i32,
    description: Option<String>,
    user: String,
) -> Result<i64, String> {
    // 验证周期天数
    if cycle_days < 1 || cycle_days > 30 {
        return Err(format!("周期天数必须在 1-30 之间，当前值: {}", cycle_days));
    }

    db::create_rhythm_config(&config_name, cycle_days, description.as_deref(), &user)
}

/// 更新节拍配置
#[tauri::command]
pub async fn update_rhythm_config(
    config_id: i64,
    config_name: String,
    cycle_days: i32,
    description: Option<String>,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    // 验证周期天数
    if cycle_days < 1 || cycle_days > 30 {
        return Err(format!("周期天数必须在 1-30 之间，当前值: {}", cycle_days));
    }

    db::update_rhythm_config(
        config_id,
        &config_name,
        cycle_days,
        description.as_deref(),
        &user,
        reason.as_deref(),
    )
}

/// 删除节拍配置
#[tauri::command]
pub async fn delete_rhythm_config(
    config_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::delete_rhythm_config(config_id, &user, reason.as_deref())
}

/// 激活节拍配置
/// 同时禁用其他配置，确保只有一个激活配置
#[tauri::command]
pub async fn activate_rhythm_config(config_id: i64, user: String) -> Result<(), String> {
    db::activate_rhythm_config(config_id, &user)
}

/// 获取节拍配置的标签列表
#[tauri::command]
pub async fn get_rhythm_labels(config_id: i64) -> Result<Vec<RhythmLabel>, String> {
    db::list_rhythm_labels(config_id)
}

/// 创建或更新节拍标签
#[tauri::command]
pub async fn upsert_rhythm_label(
    config_id: i64,
    rhythm_day: i32,
    label_name: String,
    match_spec: Option<String>,
    bonus_score: f64,
    description: Option<String>,
) -> Result<i64, String> {
    // 验证分数范围
    if bonus_score < 0.0 || bonus_score > 100.0 {
        return Err(format!("加分必须在 0-100 之间，当前值: {}", bonus_score));
    }

    db::upsert_rhythm_label(
        config_id,
        rhythm_day,
        &label_name,
        match_spec.as_deref(),
        bonus_score,
        description.as_deref(),
    )
}

/// 删除节拍标签
#[tauri::command]
pub async fn delete_rhythm_label(label_id: i64) -> Result<(), String> {
    db::delete_rhythm_label(label_id)
}

/// 获取节拍配置变更历史
#[tauri::command]
pub async fn get_rhythm_config_history(
    config_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<RhythmConfigChangeLog>, String> {
    db::list_rhythm_config_change_log(config_id, limit.unwrap_or(50))
}

// ============================================
// Phase 11: 评分透明化 - Explain API
// ============================================

/// P-Score 默认子权重
const DEFAULT_W_P1: f64 = 0.5;
const DEFAULT_W_P2: f64 = 0.3;
const DEFAULT_W_P3: f64 = 0.2;

/// 获取合同优先级的详细评分拆分（Explain）
///
/// # 用途
/// 让业务人员能够复算、复核任何一笔合同的优先级，避免"黑箱评分"。
///
/// # 返回
/// - 完整的 S-Score 拆分（S1/S2/S3）
/// - 完整的 P-Score 拆分（P1/P2/P3）
/// - 所有权重和贡献值
/// - 计算验证结果
/// - 最终优先级公式汇总
#[tauri::command]
pub async fn explain_priority(contract_id: String, strategy: String) -> Result<PriorityExplain, String> {
    // 1. 获取合同数据
    let contract = db::get_contract(&contract_id)?;

    // 2. 获取客户数据
    let customer = db::get_customer(&contract.customer_id)?;

    // 3. 获取策略权重（ws, wp）
    let weights = db::get_strategy_weights(&strategy)?;

    // 4. 加载评分配置
    let scoring_config = config::load_scoring_config()?;

    // 5. 获取策略的 S-Score 子权重（w1, w2, w3）
    let s_weights = config::load_strategy_scoring_weights(&strategy)?;

    // 6. 生成 S-Score Explain
    let s_input = SScoreInput {
        customer_level: customer.customer_level.clone(),
        margin: contract.margin,
        days_to_pdd: contract.days_to_pdd,
    };
    let s_score_explain = generate_s_score_explain(
        &s_input,
        s_weights.w1,
        s_weights.w2,
        s_weights.w3,
        &scoring_config,
    );
    let s_score = s_score_explain.total_score;

    // 7. 🆕 获取合同的聚合统计
    let aggregation_stats = db::get_contract_aggregation_stats(
        &contract.spec_family,
        &contract.steel_grade,
        contract.thickness,
        contract.width,
    )?;

    // 8. 🆕 获取 P2 曲线配置
    let p2_curve_config = db::get_p2_curve_config()?;

    // 9. 生成 P-Score Explain（使用新的聚合度计算）
    let p_input = PScoreInput {
        steel_grade: contract.steel_grade.clone(),
        thickness: contract.thickness,
        width: contract.width,
        spec_family: contract.spec_family.clone(),
        days_to_pdd: contract.days_to_pdd,
    };
    let p_score_explain = generate_p_score_explain_with_aggregation(
        &p_input,
        &scoring_config,
        Some(&aggregation_stats),
        Some(&p2_curve_config),
        DEFAULT_W_P1,
        DEFAULT_W_P2,
        DEFAULT_W_P3,
    )?;
    let p_score = p_score_explain.total_score;

    // 10. 计算基础优先级
    let base_priority = scoring::calc_priority(s_score, p_score, weights.ws, weights.wp);

    // 11. 获取 Alpha 并计算最终优先级
    let alpha = db::get_latest_alpha(&contract_id).ok().flatten();
    let final_priority = if let Some(a) = alpha {
        scoring::apply_alpha(base_priority, a)
    } else {
        base_priority
    };

    // 12. 生成公式汇总
    let formula_summary = if let Some(a) = alpha {
        format!(
            "S-Score = {:.2} × {:.2} + {:.2} × {:.2} + {:.2} × {:.2} = {:.2}\n\
             P-Score = {:.2} × {:.2} + {:.2} × {:.2} + {:.2} × {:.2} = {:.2}\n\
             Base Priority = {:.2} × {:.2} + {:.2} × {:.2} = {:.2}\n\
             Final Priority = {:.2} × {:.3} = {:.2}",
            s_score_explain.s1_customer_level.score, s_weights.w1,
            s_score_explain.s2_margin.score, s_weights.w2,
            s_score_explain.s3_urgency.score, s_weights.w3,
            s_score,
            p_score_explain.p1_difficulty.score, DEFAULT_W_P1,
            p_score_explain.p2_aggregation.score, DEFAULT_W_P2,
            p_score_explain.p3_rhythm.score, DEFAULT_W_P3,
            p_score,
            s_score, weights.ws,
            p_score, weights.wp,
            base_priority,
            base_priority, a, final_priority
        )
    } else {
        format!(
            "S-Score = {:.2} × {:.2} + {:.2} × {:.2} + {:.2} × {:.2} = {:.2}\n\
             P-Score = {:.2} × {:.2} + {:.2} × {:.2} + {:.2} × {:.2} = {:.2}\n\
             Final Priority = {:.2} × {:.2} + {:.2} × {:.2} = {:.2}",
            s_score_explain.s1_customer_level.score, s_weights.w1,
            s_score_explain.s2_margin.score, s_weights.w2,
            s_score_explain.s3_urgency.score, s_weights.w3,
            s_score,
            p_score_explain.p1_difficulty.score, DEFAULT_W_P1,
            p_score_explain.p2_aggregation.score, DEFAULT_W_P2,
            p_score_explain.p3_rhythm.score, DEFAULT_W_P3,
            p_score,
            s_score, weights.ws,
            p_score, weights.wp,
            final_priority
        )
    };

    // 13. 验证所有计算是否一致
    let all_verifications_passed = s_score_explain.verification_passed
        && p_score_explain.verification_passed
        && (base_priority - (weights.ws * s_score + weights.wp * p_score)).abs() < 0.001;

    Ok(PriorityExplain {
        contract_id,
        strategy_name: strategy,
        s_score_explain,
        p_score_explain,
        s_score,
        p_score,
        ws: weights.ws,
        wp: weights.wp,
        base_priority,
        alpha,
        final_priority,
        formula_summary,
        all_verifications_passed,
    })
}

// ============================================
// Phase 13: 数据校验与质量报告
// ============================================

use crate::db::{DataQualityReport, MissingValueStrategy, ContractPriorityWithValidation, DefaultValueUsed};
use crate::validation;

/// 获取数据质量报告
///
/// # 用途
/// 生成完整的数据质量报告，包含：
/// - 校验汇总（有效/警告/错误合同数）
/// - 按字段统计的问题
/// - 问题合同列表
/// - 修复建议
#[tauri::command]
pub async fn get_data_quality_report() -> Result<DataQualityReport, String> {
    let (_, report) = validation::validate_all_contracts()?;
    Ok(report)
}

/// 获取缺失值策略配置
#[tauri::command]
pub async fn get_missing_value_strategies() -> Result<Vec<MissingValueStrategy>, String> {
    db::list_missing_value_strategies()
}

/// 更新缺失值策略
#[tauri::command]
pub async fn update_missing_value_strategy(
    field_name: String,
    strategy: String,
    default_value: Option<String>,
    default_description: Option<String>,
) -> Result<(), String> {
    // 验证策略值
    if !["default", "skip", "error"].contains(&strategy.as_str()) {
        return Err(format!("无效的策略类型: {}，必须是 default/skip/error", strategy));
    }

    db::update_missing_value_strategy(
        &field_name,
        &strategy,
        default_value.as_deref(),
        default_description.as_deref(),
    )
}

/// 批量计算所有合同的优先级（带校验状态）
///
/// # 返回
/// 每条合同的优先级结果都附带校验状态和警告信息
#[tauri::command]
pub async fn compute_all_priorities_with_validation(
    strategy: String,
) -> Result<Vec<ContractPriorityWithValidation>, String> {
    // 1. 获取策略权重
    let weights = db::get_strategy_weights(&strategy)?;

    // 2. 加载评分配置
    let scoring_config = config::load_scoring_config()?;

    // 3. 获取策略的 S-Score 子权重
    let s_weights = config::load_strategy_scoring_weights(&strategy)?;

    // 4. 批量获取所有合同的聚合统计
    let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;

    // 5. 获取 P2 曲线配置
    let p2_curve_config = db::get_p2_curve_config()?;

    // 6. 校验并获取所有合同
    let (validated_contracts, _) = validation::validate_all_contracts()?;

    // 7. 批量计算（仅计算可计算的合同）
    let mut results: Vec<ContractPriorityWithValidation> = Vec::new();

    for validated in validated_contracts {
        // 跳过无法计算的合同
        if !validated.validation.can_calculate {
            continue;
        }

        let contract = &validated.contract;

        // 获取客户数据（使用校验后的 customer_id）
        let customer = match db::get_customer(&contract.customer_id) {
            Ok(c) => c,
            Err(_) => {
                // 如果客户不存在，创建一个默认客户用于计算
                db::Customer {
                    customer_id: contract.customer_id.clone(),
                    customer_name: None,
                    customer_level: "C".to_string(),  // 默认最低等级
                    credit_level: None,
                    customer_group: None,
                }
            }
        };

        // 校验客户数据
        let validated_customer = validation::validate_customer(&customer, None)?;

        // 计算 S-Score
        let s_input = SScoreInput {
            customer_level: validated_customer.customer.customer_level.clone(),
            margin: contract.margin,
            days_to_pdd: contract.days_to_pdd,
        };
        let s_score = scoring::calc_s_score(s_input, s_weights.w1, s_weights.w2, s_weights.w3, &scoring_config);

        // 计算 P-Score
        let p_input = PScoreInput {
            steel_grade: contract.steel_grade.clone(),
            thickness: contract.thickness,
            width: contract.width,
            spec_family: contract.spec_family.clone(),
            days_to_pdd: contract.days_to_pdd,
        };
        let aggregation_stats = aggregation_stats_map.get(&contract.contract_id);
        let p_score = scoring::calc_p_score_with_aggregation(
            p_input,
            &scoring_config,
            aggregation_stats,
            Some(&p2_curve_config),
        )?;

        // 计算综合优先级
        let mut priority = scoring::calc_priority(s_score, p_score, weights.ws, weights.wp);

        // 应用 alpha（如果有）
        let alpha = db::get_latest_alpha(&contract.contract_id).ok().flatten();
        if let Some(a) = alpha {
            priority = scoring::apply_alpha(priority, a);
        }

        // 生成警告信息
        let warnings = validation::get_warning_messages(&validated.validation);

        // 合并合同和客户的默认值使用记录
        let mut all_defaults: Vec<DefaultValueUsed> = validated.defaults_used.clone();
        all_defaults.extend(validated_customer.defaults_used.clone());

        let priority_result = db::ContractPriority {
            contract: contract.clone(),
            s_score,
            p_score,
            priority,
            alpha,
        };

        results.push(ContractPriorityWithValidation {
            priority_result,
            validation_status: validated.validation.status.clone(),
            warnings,
            used_defaults: !all_defaults.is_empty(),
            default_values_used: all_defaults,
        });
    }

    // 按优先级降序排序
    results.sort_by(|a, b| {
        b.priority_result.priority
            .partial_cmp(&a.priority_result.priority)
            .unwrap()
    });

    Ok(results)
}

// ============================================
// Phase 14: 策略版本化（可回放、可复盘）
// ============================================

use crate::db::{
    StrategyVersion, StrategyVersionSummary, StrategyVersionChangeLog,
    SandboxSession, SandboxResult, VersionComparison, VersionComparisonItem,
};

/// 创建策略版本快照
///
/// # 用途
/// 每次策略参数变更后调用，生成版本快照。
/// 也可手动调用，用于沙盘预演前锁定参数。
#[tauri::command]
pub async fn create_strategy_version(
    strategy_name: String,
    version_tag: Option<String>,
    description: Option<String>,
    change_reason: Option<String>,
    user: String,
    set_active: bool,
) -> Result<i64, String> {
    db::create_strategy_version(
        &strategy_name,
        version_tag.as_deref(),
        description.as_deref(),
        change_reason.as_deref(),
        &user,
        set_active,
    )
}

/// 获取策略的版本列表
#[tauri::command]
pub async fn get_strategy_versions(strategy_name: String) -> Result<Vec<StrategyVersionSummary>, String> {
    db::list_strategy_versions(&strategy_name)
}

/// 获取策略版本详情（包含完整配置快照）
#[tauri::command]
pub async fn get_strategy_version(version_id: i64) -> Result<StrategyVersion, String> {
    db::get_strategy_version(version_id)
}

/// 获取策略的当前激活版本
#[tauri::command]
pub async fn get_active_strategy_version(strategy_name: String) -> Result<Option<StrategyVersion>, String> {
    db::get_active_strategy_version(&strategy_name)
}

/// 激活指定版本（用于回滚到历史版本）
#[tauri::command]
pub async fn activate_strategy_version(version_id: i64, user: String) -> Result<(), String> {
    db::activate_strategy_version(version_id, &user)
}

/// 锁定版本（防止误删除，用于重要版本）
#[tauri::command]
pub async fn lock_strategy_version(
    version_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::lock_strategy_version(version_id, &user, reason.as_deref())
}

/// 解锁版本
#[tauri::command]
pub async fn unlock_strategy_version(
    version_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::unlock_strategy_version(version_id, &user, reason.as_deref())
}

/// 删除版本（仅可删除未锁定的非激活版本）
#[tauri::command]
pub async fn delete_strategy_version(
    version_id: i64,
    user: String,
    reason: Option<String>,
) -> Result<(), String> {
    db::delete_strategy_version(version_id, &user, reason.as_deref())
}

/// 获取版本变更历史
#[tauri::command]
pub async fn get_strategy_version_history(
    version_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<StrategyVersionChangeLog>, String> {
    db::list_strategy_version_change_log(version_id, limit.unwrap_or(50))
}

// ============================================
// 沙盘会话管理命令
// ============================================

/// 创建沙盘会话
///
/// # 参数
/// - session_name: 会话名称（如"2024年12月排产预演"）
/// - strategy_version_id: 使用的策略版本
/// - description: 会话描述
#[tauri::command]
pub async fn create_sandbox_session(
    session_name: String,
    strategy_version_id: i64,
    description: Option<String>,
    user: String,
) -> Result<i64, String> {
    db::create_sandbox_session(&session_name, strategy_version_id, description.as_deref(), &user)
}

/// 获取沙盘会话列表
#[tauri::command]
pub async fn get_sandbox_sessions(limit: Option<i64>) -> Result<Vec<SandboxSession>, String> {
    db::list_sandbox_sessions(limit)
}

/// 获取沙盘会话详情
#[tauri::command]
pub async fn get_sandbox_session(session_id: i64) -> Result<SandboxSession, String> {
    db::get_sandbox_session(session_id)
}

/// 执行沙盘计算
///
/// # 功能
/// 使用指定版本的策略参数计算所有合同的优先级，
/// 并保存计算结果，确保可复现。
#[tauri::command]
pub async fn run_sandbox_calculation(session_id: i64) -> Result<i64, String> {
    // 1. 获取会话信息
    let session = db::get_sandbox_session(session_id)?;

    // 2. 获取策略版本
    let version = db::get_strategy_version(session.strategy_version_id)?;

    // 3. 获取所有合同
    let contracts = db::list_contracts()?;

    // 4. 加载评分配置（从版本快照反序列化）
    let scoring_config: std::collections::HashMap<String, String> =
        serde_json::from_str(&version.scoring_config_snapshot)
            .map_err(|e| format!("解析评分配置快照失败: {}", e))?;

    // 5. 加载 P2 曲线配置（如果有快照）
    let p2_curve_config = if let Some(ref snapshot) = version.p2_curve_config_snapshot {
        serde_json::from_str(snapshot)
            .map_err(|e| format!("解析 P2 曲线配置快照失败: {}", e))?
    } else {
        db::P2CurveConfig::default()
    };

    // 6. 批量获取聚合统计
    let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;

    // 7. 构建 ScoringConfig 结构
    let scoring_config_struct = config::build_scoring_config_from_map(&scoring_config)?;

    // 8. 计算每个合同的优先级
    let mut results: Vec<SandboxResult> = Vec::new();

    for contract in &contracts {
        // 获取客户数据
        let customer = match db::get_customer(&contract.customer_id) {
            Ok(c) => c,
            Err(_) => db::Customer {
                customer_id: contract.customer_id.clone(),
                customer_name: None,
                customer_level: "C".to_string(),
                credit_level: None,
                customer_group: None,
            }
        };

        // 计算 S-Score（使用版本中的权重）
        let s_input = SScoreInput {
            customer_level: customer.customer_level.clone(),
            margin: contract.margin,
            days_to_pdd: contract.days_to_pdd,
        };

        // 使用版本中保存的 S-Score 子权重
        let s_score_explain = scoring::generate_s_score_explain(
            &s_input,
            version.w1,
            version.w2,
            version.w3,
            &scoring_config_struct,
        );
        let s_score = s_score_explain.total_score;

        // 计算 P-Score
        let p_input = PScoreInput {
            steel_grade: contract.steel_grade.clone(),
            thickness: contract.thickness,
            width: contract.width,
            spec_family: contract.spec_family.clone(),
            days_to_pdd: contract.days_to_pdd,
        };
        let aggregation_stats = aggregation_stats_map.get(&contract.contract_id);
        let p_score_explain = scoring::generate_p_score_explain_with_aggregation(
            &p_input,
            &scoring_config_struct,
            aggregation_stats,
            Some(&p2_curve_config),
            version.w_p1,
            version.w_p2,
            version.w_p3,
        )?;
        let p_score = p_score_explain.total_score;

        // 计算综合优先级（使用版本中的 ws, wp）
        let mut priority = scoring::calc_priority(s_score, p_score, version.ws, version.wp);

        // 应用 alpha
        let alpha = db::get_latest_alpha(&contract.contract_id).ok().flatten();
        if let Some(a) = alpha {
            priority = scoring::apply_alpha(priority, a);
        }

        // 构建结果
        results.push(SandboxResult {
            result_id: None,
            session_id,
            contract_id: contract.contract_id.clone(),
            contract_snapshot: serde_json::to_string(contract).unwrap_or_default(),
            customer_snapshot: Some(serde_json::to_string(&customer).unwrap_or_default()),
            s_score,
            p_score,
            priority,
            alpha,
            s1_score: Some(s_score_explain.s1_customer_level.score),
            s2_score: Some(s_score_explain.s2_margin.score),
            s3_score: Some(s_score_explain.s3_urgency.score),
            p1_score: Some(p_score_explain.p1_difficulty.score),
            p2_score: Some(p_score_explain.p2_aggregation.score),
            p3_score: Some(p_score_explain.p3_rhythm.score),
            aggregation_key: aggregation_stats.map(|s| s.aggregation_key.clone()),
            aggregation_count: aggregation_stats.map(|s| s.contract_count),
            priority_rank: None,
        });
    }

    // 9. 按优先级排序并设置排名
    results.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());
    for (i, result) in results.iter_mut().enumerate() {
        result.priority_rank = Some((i + 1) as i64);
    }

    // 10. 保存结果
    db::save_sandbox_results(session_id, &results)?;

    Ok(results.len() as i64)
}

/// 获取沙盘计算结果
#[tauri::command]
pub async fn get_sandbox_results(session_id: i64) -> Result<Vec<SandboxResult>, String> {
    db::get_sandbox_results(session_id)
}

// ============================================
// 版本对比命令
// ============================================

/// 对比两个版本的计算结果
///
/// # 功能
/// 使用两个版本分别计算当前合同池的优先级，
/// 对比结果差异，生成对比报告。
#[tauri::command]
pub async fn compare_strategy_versions(
    version_a_id: i64,
    version_b_id: i64,
    user: String,
) -> Result<VersionComparison, String> {
    // 1. 获取两个版本
    let version_a = db::get_strategy_version(version_a_id)?;
    let version_b = db::get_strategy_version(version_b_id)?;

    // 2. 获取所有合同
    let contracts = db::list_contracts()?;

    // 3. 加载评分配置
    let scoring_config_a: std::collections::HashMap<String, String> =
        serde_json::from_str(&version_a.scoring_config_snapshot)
            .map_err(|e| format!("解析版本A配置快照失败: {}", e))?;
    let scoring_config_b: std::collections::HashMap<String, String> =
        serde_json::from_str(&version_b.scoring_config_snapshot)
            .map_err(|e| format!("解析版本B配置快照失败: {}", e))?;

    let scoring_config_struct_a = config::build_scoring_config_from_map(&scoring_config_a)?;
    let scoring_config_struct_b = config::build_scoring_config_from_map(&scoring_config_b)?;

    // 4. 加载 P2 曲线配置
    let p2_curve_config_a: db::P2CurveConfig = if let Some(ref snapshot) = version_a.p2_curve_config_snapshot {
        serde_json::from_str(snapshot).unwrap_or_default()
    } else {
        db::P2CurveConfig::default()
    };
    let p2_curve_config_b: db::P2CurveConfig = if let Some(ref snapshot) = version_b.p2_curve_config_snapshot {
        serde_json::from_str(snapshot).unwrap_or_default()
    } else {
        db::P2CurveConfig::default()
    };

    // 5. 批量获取聚合统计
    let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;

    // 6. 分别计算两个版本的优先级
    let mut results_a: Vec<(String, f64)> = Vec::new();
    let mut results_b: Vec<(String, f64)> = Vec::new();

    for contract in &contracts {
        let customer = db::get_customer(&contract.customer_id).unwrap_or_else(|_| db::Customer {
            customer_id: contract.customer_id.clone(),
            customer_name: None,
            customer_level: "C".to_string(),
            credit_level: None,
            customer_group: None,
        });

        let s_input = SScoreInput {
            customer_level: customer.customer_level.clone(),
            margin: contract.margin,
            days_to_pdd: contract.days_to_pdd,
        };

        let p_input = PScoreInput {
            steel_grade: contract.steel_grade.clone(),
            thickness: contract.thickness,
            width: contract.width,
            spec_family: contract.spec_family.clone(),
            days_to_pdd: contract.days_to_pdd,
        };

        let aggregation_stats = aggregation_stats_map.get(&contract.contract_id);

        // 版本 A 计算
        let s_score_a = scoring::calc_s_score(
            s_input.clone(),
            version_a.w1,
            version_a.w2,
            version_a.w3,
            &scoring_config_struct_a,
        );
        let p_score_a = scoring::calc_p_score_with_aggregation(
            p_input.clone(),
            &scoring_config_struct_a,
            aggregation_stats,
            Some(&p2_curve_config_a),
        )?;
        let priority_a = scoring::calc_priority(s_score_a, p_score_a, version_a.ws, version_a.wp);

        // 版本 B 计算
        let s_score_b = scoring::calc_s_score(
            s_input,
            version_b.w1,
            version_b.w2,
            version_b.w3,
            &scoring_config_struct_b,
        );
        let p_score_b = scoring::calc_p_score_with_aggregation(
            p_input,
            &scoring_config_struct_b,
            aggregation_stats,
            Some(&p2_curve_config_b),
        )?;
        let priority_b = scoring::calc_priority(s_score_b, p_score_b, version_b.ws, version_b.wp);

        results_a.push((contract.contract_id.clone(), priority_a));
        results_b.push((contract.contract_id.clone(), priority_b));
    }

    // 7. 排序并计算排名
    results_a.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    results_b.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let rank_a: std::collections::HashMap<String, i64> = results_a
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (id.clone(), (i + 1) as i64))
        .collect();
    let rank_b: std::collections::HashMap<String, i64> = results_b
        .iter()
        .enumerate()
        .map(|(i, (id, _))| (id.clone(), (i + 1) as i64))
        .collect();

    let priority_a: std::collections::HashMap<String, f64> = results_a.into_iter().collect();
    let priority_b: std::collections::HashMap<String, f64> = results_b.into_iter().collect();

    // 8. 计算对比结果
    let mut comparison_items: Vec<VersionComparisonItem> = Vec::new();
    let mut total_diff = 0.0f64;
    let mut rank_change_count = 0i64;
    let mut max_rank_change = 0i64;

    for contract in &contracts {
        let id = &contract.contract_id;
        let p_a = *priority_a.get(id).unwrap_or(&0.0);
        let p_b = *priority_b.get(id).unwrap_or(&0.0);
        let r_a = *rank_a.get(id).unwrap_or(&0);
        let r_b = *rank_b.get(id).unwrap_or(&0);
        let rank_change = r_a - r_b; // 正数表示在版本B中排名更靠前

        total_diff += (p_a - p_b).abs();
        if rank_change != 0 {
            rank_change_count += 1;
        }
        max_rank_change = max_rank_change.max(rank_change.abs());

        comparison_items.push(VersionComparisonItem {
            contract_id: id.clone(),
            priority_a: p_a,
            priority_b: p_b,
            priority_diff: p_a - p_b,
            rank_a: r_a,
            rank_b: r_b,
            rank_change,
        });
    }

    let avg_priority_diff = if contracts.is_empty() {
        0.0
    } else {
        total_diff / contracts.len() as f64
    };

    // 9. 保存对比结果
    let comparison_details = serde_json::to_string(&comparison_items)
        .map_err(|e| format!("序列化对比结果失败: {}", e))?;

    let comparison_id = db::create_version_comparison(
        version_a_id,
        version_b_id,
        &comparison_details,
        contracts.len() as i64,
        rank_change_count,
        avg_priority_diff,
        max_rank_change,
        &user,
    )?;

    // 10. 返回对比结果
    db::get_version_comparison(comparison_id)
}

/// 获取版本对比详情
#[tauri::command]
pub async fn get_version_comparison(comparison_id: i64) -> Result<VersionComparison, String> {
    db::get_version_comparison(comparison_id)
}

// ============================================
// Phase 15: 导入/清洗冲突解决机制产品化
// ============================================

use crate::db::schema::{
    ImportAuditLog, ImportConflictLog, FieldAlignmentRule,
    DuplicateDetectionConfig, ImportStatistics, SimilarRecordPair,
};

/// 获取导入审计历史
#[tauri::command]
pub async fn get_import_audit_history(
    import_type: Option<String>,
    status: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<ImportAuditLog>, String> {
    db::list_import_audits(
        import_type.as_deref(),
        status.as_deref(),
        limit.unwrap_or(50),
    )
}

/// 获取单个导入审计记录
#[tauri::command]
pub async fn get_import_audit(audit_id: i64) -> Result<ImportAuditLog, String> {
    db::get_import_audit(audit_id)
}

/// 获取待处理的导入冲突
#[tauri::command]
pub async fn get_pending_import_conflicts(audit_id: i64) -> Result<Vec<ImportConflictLog>, String> {
    db::get_pending_conflicts(audit_id)
}

/// 解决单个导入冲突
#[tauri::command]
pub async fn resolve_import_conflict(
    conflict_id: i64,
    action: String,
    action_reason: Option<String>,
    user: String,
) -> Result<(), String> {
    if !["skip", "overwrite"].contains(&action.as_str()) {
        return Err(format!("无效的冲突处理动作: {}，必须是 skip 或 overwrite", action));
    }
    db::resolve_import_conflict(conflict_id, &action, action_reason.as_deref(), &user)
}

/// 批量解决导入冲突
#[tauri::command]
pub async fn batch_resolve_import_conflicts(
    audit_id: i64,
    action: String,
    action_reason: Option<String>,
    user: String,
) -> Result<i64, String> {
    if !["skip", "overwrite"].contains(&action.as_str()) {
        return Err(format!("无效的冲突处理动作: {}，必须是 skip 或 overwrite", action));
    }
    db::batch_resolve_conflicts(audit_id, &action, action_reason.as_deref(), &user)
}

/// 回滚导入操作
#[tauri::command]
pub async fn rollback_import(audit_id: i64) -> Result<i64, String> {
    db::rollback_import(audit_id)
}

/// 获取导入统计信息
#[tauri::command]
pub async fn get_import_statistics() -> Result<Vec<ImportStatistics>, String> {
    db::get_import_statistics()
}

/// 获取字段对齐规则
#[tauri::command]
pub async fn get_field_alignment_rules(
    data_type: Option<String>,
) -> Result<Vec<FieldAlignmentRule>, String> {
    db::list_field_alignment_rules(data_type.as_deref())
}

/// 创建字段对齐规则
#[tauri::command]
pub async fn create_field_alignment_rule(
    rule_name: String,
    data_type: String,
    source_type: Option<String>,
    description: Option<String>,
    priority: i64,
    field_mapping: String,
    value_transform: Option<String>,
    default_values: Option<String>,
    user: String,
) -> Result<i64, String> {
    // 验证 field_mapping 是有效的 JSON
    serde_json::from_str::<serde_json::Value>(&field_mapping)
        .map_err(|e| format!("字段映射 JSON 格式无效: {}", e))?;

    if let Some(ref vt) = value_transform {
        serde_json::from_str::<serde_json::Value>(vt)
            .map_err(|e| format!("值转换规则 JSON 格式无效: {}", e))?;
    }

    if let Some(ref dv) = default_values {
        serde_json::from_str::<serde_json::Value>(dv)
            .map_err(|e| format!("默认值 JSON 格式无效: {}", e))?;
    }

    db::create_field_alignment_rule(
        &rule_name,
        &data_type,
        source_type.as_deref(),
        description.as_deref(),
        priority,
        &field_mapping,
        value_transform.as_deref(),
        default_values.as_deref(),
        &user,
    )
}

/// 获取重复检测配置
#[tauri::command]
pub async fn get_duplicate_detection_config(
    data_type: String,
) -> Result<Option<DuplicateDetectionConfig>, String> {
    db::get_duplicate_detection_config(&data_type)
}

/// 获取待处理的相似记录对
#[tauri::command]
pub async fn get_pending_similar_pairs(
    data_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<SimilarRecordPair>, String> {
    db::get_pending_similar_pairs(data_type.as_deref(), limit.unwrap_or(50))
}

/// 解决相似记录对
#[tauri::command]
pub async fn resolve_similar_pair(
    pair_id: i64,
    status: String,
    resolution_note: Option<String>,
    user: String,
) -> Result<(), String> {
    if !["confirmed_same", "confirmed_diff", "merged", "ignored"].contains(&status.as_str()) {
        return Err(format!("无效的状态: {}，必须是 confirmed_same/confirmed_diff/merged/ignored", status));
    }
    db::resolve_similar_pair(pair_id, &status, resolution_note.as_deref(), &user)
}

/// 检测重复文件导入
#[tauri::command]
pub async fn check_duplicate_file_import(
    file_hash: String,
) -> Result<Option<ImportAuditLog>, String> {
    db::check_duplicate_import(&file_hash)
}

// ============================================
// Phase 16: 会议驾驶舱 KPI 固化
// Meeting Cockpit KPI Solidification
// ============================================

// --------------------------------------------
// 会议快照 CRUD
// --------------------------------------------

/// 创建会议快照
#[tauri::command]
pub async fn create_meeting_snapshot(
    meeting_type: String,
    meeting_date: String,
    snapshot_name: String,
    strategy_version_id: Option<i64>,
    strategy_name: Option<String>,
    kpi_summary: String,
    risk_summary: String,
    recommendation: Option<String>,
    contract_rankings: String,
    ranking_changes: Option<String>,
    user: String,
) -> Result<i64, String> {
    // 验证会议类型
    if !["production_sales", "business"].contains(&meeting_type.as_str()) {
        return Err(format!("无效的会议类型: {}，必须是 production_sales 或 business", meeting_type));
    }

    // 验证 JSON 格式
    serde_json::from_str::<serde_json::Value>(&kpi_summary)
        .map_err(|e| format!("KPI 摘要 JSON 格式无效: {}", e))?;
    serde_json::from_str::<serde_json::Value>(&risk_summary)
        .map_err(|e| format!("风险摘要 JSON 格式无效: {}", e))?;
    serde_json::from_str::<serde_json::Value>(&contract_rankings)
        .map_err(|e| format!("合同排名 JSON 格式无效: {}", e))?;

    if let Some(ref rc) = ranking_changes {
        serde_json::from_str::<serde_json::Value>(rc)
            .map_err(|e| format!("排名变化 JSON 格式无效: {}", e))?;
    }

    db::create_meeting_snapshot(
        &meeting_type,
        &meeting_date,
        &snapshot_name,
        strategy_version_id,
        strategy_name.as_deref(),
        &kpi_summary,
        &risk_summary,
        recommendation.as_deref(),
        &contract_rankings,
        ranking_changes.as_deref(),
        &user,
    )
}

/// 获取会议快照列表
#[tauri::command]
pub async fn get_meeting_snapshots(
    meeting_type: Option<String>,
    limit: Option<i64>,
) -> Result<Vec<MeetingSnapshotSummary>, String> {
    db::list_meeting_snapshots(meeting_type.as_deref(), limit)
}

/// 获取单个会议快照详情
#[tauri::command]
pub async fn get_meeting_snapshot(snapshot_id: i64) -> Result<MeetingSnapshot, String> {
    db::get_meeting_snapshot(snapshot_id)
}

/// 更新会议快照状态
#[tauri::command]
pub async fn update_meeting_snapshot_status(
    snapshot_id: i64,
    status: String,
    approved_by: Option<String>,
) -> Result<(), String> {
    // 验证状态值
    if !["draft", "pending", "approved", "archived"].contains(&status.as_str()) {
        return Err(format!("无效的状态: {}，必须是 draft/pending/approved/archived", status));
    }

    db::update_meeting_snapshot_status(snapshot_id, &status, approved_by.as_deref())
}

/// 删除会议快照
#[tauri::command]
pub async fn delete_meeting_snapshot(snapshot_id: i64) -> Result<(), String> {
    db::delete_meeting_snapshot(snapshot_id)
}

// --------------------------------------------
// KPI 配置查询
// --------------------------------------------

/// 获取所有启用的 KPI 配置
#[tauri::command]
pub async fn get_meeting_kpi_configs() -> Result<Vec<MeetingKpiConfig>, String> {
    db::list_meeting_kpi_configs()
}

/// 获取指定类别的 KPI 配置
#[tauri::command]
pub async fn get_meeting_kpi_configs_by_category(
    category: String,
) -> Result<Vec<MeetingKpiConfig>, String> {
    // 验证类别
    if !["leadership", "sales", "production", "finance"].contains(&category.as_str()) {
        return Err(format!("无效的 KPI 类别: {}，必须是 leadership/sales/production/finance", category));
    }

    db::list_meeting_kpi_configs_by_category(&category)
}

// --------------------------------------------
// 风险合同标记
// --------------------------------------------

/// 创建风险合同标记
#[tauri::command]
pub async fn create_risk_contract_flag(
    snapshot_id: i64,
    contract_id: String,
    risk_type: String,
    risk_level: String,
    risk_score: Option<f64>,
    risk_description: String,
    risk_factors: Option<String>,
    affected_kpis: Option<String>,
    potential_loss: Option<f64>,
    suggested_action: Option<String>,
    action_priority: Option<i64>,
) -> Result<i64, String> {
    // 验证风险类型
    if !["delivery_delay", "customer_downgrade", "margin_loss", "rhythm_mismatch", "other"].contains(&risk_type.as_str()) {
        return Err(format!("无效的风险类型: {}", risk_type));
    }

    // 验证风险等级
    if !["high", "medium", "low"].contains(&risk_level.as_str()) {
        return Err(format!("无效的风险等级: {}，必须是 high/medium/low", risk_level));
    }

    db::create_risk_contract_flag(
        snapshot_id,
        &contract_id,
        &risk_type,
        &risk_level,
        risk_score,
        &risk_description,
        risk_factors.as_deref(),
        affected_kpis.as_deref(),
        potential_loss,
        suggested_action.as_deref(),
        action_priority,
    )
}

/// 获取会议快照的风险合同列表
#[tauri::command]
pub async fn get_risk_contracts(snapshot_id: i64) -> Result<Vec<RiskContractFlag>, String> {
    db::list_risk_contracts_by_snapshot(snapshot_id)
}

/// 更新风险合同处理状态
#[tauri::command]
pub async fn update_risk_contract_status(
    flag_id: i64,
    status: String,
    handled_by: String,
    handling_note: Option<String>,
) -> Result<(), String> {
    // 验证状态
    if !["open", "in_progress", "resolved", "accepted"].contains(&status.as_str()) {
        return Err(format!("无效的状态: {}，必须是 open/in_progress/resolved/accepted", status));
    }

    db::update_risk_contract_status(flag_id, &status, &handled_by, handling_note.as_deref())
}

/// 获取风险汇总统计
#[tauri::command]
pub async fn get_risk_summary_stats(snapshot_id: i64) -> Result<serde_json::Value, String> {
    db::get_risk_summary_stats(snapshot_id)
}

// --------------------------------------------
// 排名变化明细
// --------------------------------------------

/// 批量保存排名变化明细
#[tauri::command]
pub async fn save_ranking_change_details(
    snapshot_id: i64,
    compare_snapshot_id: Option<i64>,
    details_json: String,
) -> Result<i64, String> {
    // 解析 JSON 为 RankingChangeDetail 数组
    let details: Vec<RankingChangeDetail> = serde_json::from_str(&details_json)
        .map_err(|e| format!("排名变化明细 JSON 格式无效: {}", e))?;

    db::save_ranking_change_details(snapshot_id, compare_snapshot_id, &details)
}

/// 获取会议快照的排名变化明细
#[tauri::command]
pub async fn get_ranking_changes(
    snapshot_id: i64,
    limit: Option<i64>,
) -> Result<Vec<RankingChangeDetail>, String> {
    db::list_ranking_changes_by_snapshot(snapshot_id, limit)
}

/// 获取排名变化统计
#[tauri::command]
pub async fn get_ranking_change_stats(snapshot_id: i64) -> Result<serde_json::Value, String> {
    db::get_ranking_change_stats(snapshot_id)
}

// --------------------------------------------
// 共识模板
// --------------------------------------------

/// 获取共识模板列表
#[tauri::command]
pub async fn get_consensus_templates(
    meeting_type: Option<String>,
) -> Result<Vec<ConsensusTemplate>, String> {
    db::list_consensus_templates(meeting_type.as_deref())
}

/// 获取默认共识模板
#[tauri::command]
pub async fn get_default_consensus_template(
    meeting_type: String,
) -> Result<Option<ConsensusTemplate>, String> {
    db::get_default_consensus_template(&meeting_type)
}

// --------------------------------------------
// 会议行动项 CRUD
// --------------------------------------------

/// 创建会议行动项
#[tauri::command]
pub async fn create_meeting_action_item(
    snapshot_id: i64,
    action_title: String,
    action_description: Option<String>,
    action_category: Option<String>,
    priority: i64,
    due_date: Option<String>,
    assignee: Option<String>,
    department: Option<String>,
    related_contracts: Option<String>,
    user: String,
) -> Result<i64, String> {
    // 验证优先级范围
    if priority < 1 || priority > 5 {
        return Err(format!("优先级必须在 1-5 之间，当前值: {}", priority));
    }

    db::create_meeting_action_item(
        snapshot_id,
        &action_title,
        action_description.as_deref(),
        action_category.as_deref(),
        priority,
        due_date.as_deref(),
        assignee.as_deref(),
        department.as_deref(),
        related_contracts.as_deref(),
        &user,
    )
}

/// 获取会议快照的行动项列表
#[tauri::command]
pub async fn get_meeting_action_items(snapshot_id: i64) -> Result<Vec<MeetingActionItem>, String> {
    db::list_action_items_by_snapshot(snapshot_id)
}

/// 更新行动项状态
#[tauri::command]
pub async fn update_meeting_action_item_status(
    action_id: i64,
    status: String,
    completion_rate: Option<i64>,
    notes: Option<String>,
) -> Result<(), String> {
    // 验证状态
    if !["open", "in_progress", "completed", "cancelled"].contains(&status.as_str()) {
        return Err(format!("无效的状态: {}，必须是 open/in_progress/completed/cancelled", status));
    }

    // 验证完成率
    if let Some(rate) = completion_rate {
        if rate < 0 || rate > 100 {
            return Err(format!("完成率必须在 0-100 之间，当前值: {}", rate));
        }
    }

    db::update_action_item_status(action_id, &status, completion_rate, notes.as_deref())
}

/// 删除行动项
#[tauri::command]
pub async fn delete_meeting_action_item(action_id: i64) -> Result<(), String> {
    db::delete_action_item(action_id)
}

// ============================================
// Phase 16: KPI 计算引擎命令
// ============================================

use crate::db::KpiSummary;
use crate::kpi;

/// 计算所有 KPI（四视角）
///
/// # 参数
/// - strategy: 策略名称，用于计算合同优先级
///
/// # 返回
/// 四视角 KPI 汇总，包含领导、销售、生产、财务四个维度的指标
#[tauri::command]
pub async fn calculate_meeting_kpis(strategy: String) -> Result<KpiSummary, String> {
    // 1. 加载计算所需数据
    let input = kpi::KpiCalculationInput::load(&strategy)?;

    // 2. 计算所有 KPI
    kpi::calculate_all_kpis(&input)
}

/// 计算单个 KPI
///
/// # 参数
/// - kpi_code: KPI 代码（如 L01_HIGH_PRIORITY_RATIO）
/// - strategy: 策略名称
///
/// # 返回
/// 单个 KPI 的计算结果
#[tauri::command]
pub async fn calculate_single_kpi(
    kpi_code: String,
    strategy: String,
) -> Result<db::KpiValue, String> {
    // 1. 获取 KPI 配置
    let configs = db::list_meeting_kpi_configs()?;
    let config = configs.iter()
        .find(|c| c.kpi_code == kpi_code)
        .ok_or_else(|| format!("未找到 KPI 配置: {}", kpi_code))?;

    // 2. 加载计算所需数据
    let input = kpi::KpiCalculationInput::load(&strategy)?;

    // 3. 计算单个 KPI
    kpi::calculate_single_kpi(config, &input)
}

/// 识别风险合同
///
/// # 参数
/// - strategy: 策略名称
/// - snapshot_id: 可选的会议快照 ID
/// - auto_save: 是否自动保存到数据库
///
/// # 返回
/// 风险识别结果，包含风险合同列表和统计信息
#[tauri::command]
pub async fn identify_risk_contracts(
    strategy: String,
    snapshot_id: Option<i64>,
    auto_save: bool,
) -> Result<serde_json::Value, String> {
    // 1. 加载计算所需数据
    let input = kpi::KpiCalculationInput::load(&strategy)?;

    // 2. 使用默认配置识别风险
    let config = kpi::RiskIdentificationConfig::default();
    let result = kpi::identify_all_risks(&input, snapshot_id, &config)?;

    // 3. 如果需要自动保存且有 snapshot_id
    if auto_save {
        if let Some(sid) = snapshot_id {
            kpi::save_risk_contracts(sid, &result.risk_contracts)?;
        }
    }

    // 4. 返回结果
    Ok(serde_json::json!({
        "total_count": result.risk_contracts.len(),
        "high_risk_count": result.high_risk_count,
        "medium_risk_count": result.medium_risk_count,
        "low_risk_count": result.low_risk_count,
        "stats_by_type": result.stats_by_type,
        "risk_contracts": result.risk_contracts,
    }))
}

/// 计算排名变化（与历史快照对比）
///
/// # 参数
/// - current_snapshot_id: 当前会议快照 ID
/// - previous_snapshot_id: 对比的历史快照 ID
/// - strategy: 策略名称
/// - auto_save: 是否自动保存到数据库
///
/// # 返回
/// 排名变化结果，包含变化明细和统计信息
#[tauri::command]
pub async fn calculate_ranking_changes(
    current_snapshot_id: i64,
    previous_snapshot_id: i64,
    strategy: String,
    auto_save: bool,
) -> Result<serde_json::Value, String> {
    // 计算排名变化
    let result = kpi::calculate_ranking_changes_from_snapshots(
        current_snapshot_id,
        previous_snapshot_id,
        &strategy,
    )?;

    // 如果需要自动保存
    if auto_save {
        kpi::save_ranking_changes(
            current_snapshot_id,
            Some(previous_snapshot_id),
            &result.changes,
        )?;
    }

    // 返回结果
    Ok(serde_json::json!({
        "total_count": result.changes.len(),
        "up_count": result.up_count,
        "down_count": result.down_count,
        "unchanged_count": result.unchanged_count,
        "avg_change": result.avg_change,
        "max_up": result.max_up,
        "max_down": result.max_down,
        "changes": result.changes,
    }))
}

// ============================================
// Phase 16 P2: 维度聚合分析命令
// ============================================

/// 客户保障分析
///
/// # 参数
/// - strategy: 策略名称
///
/// # 返回
/// 客户保障分析结果，包含每个客户的保障评分和风险客户列表
#[tauri::command]
pub async fn analyze_customer_protection(
    strategy: String,
) -> Result<kpi::CustomerProtectionAnalysis, String> {
    kpi::get_customer_protection_analysis(&strategy)
}

/// 节拍顺行分析
///
/// # 参数
/// - strategy: 策略名称
///
/// # 返回
/// 节拍顺行分析结果，包含每日节拍匹配情况和规格族分布
#[tauri::command]
pub async fn analyze_rhythm_flow(
    strategy: String,
) -> Result<kpi::RhythmFlowAnalysis, String> {
    kpi::get_rhythm_flow_analysis(&strategy)
}

// ============================================
// Phase 16 P3: 共识包生成命令
// ============================================

/// 生成会议共识包
///
/// # 参数
/// - strategy: 策略名称
/// - meeting_type: 会议类型（production_sales 产销会 / business 经营会）
/// - meeting_date: 会议日期（YYYY-MM-DD）
/// - user: 生成者
///
/// # 返回
/// 完整的共识包结构，包含 KPI、风险、客户分析、节拍分析等
#[tauri::command]
pub async fn generate_consensus_package(
    strategy: String,
    meeting_type: String,
    meeting_date: String,
    user: String,
) -> Result<kpi::ConsensusPackage, String> {
    // 验证会议类型
    if !["production_sales", "business"].contains(&meeting_type.as_str()) {
        return Err(format!(
            "无效的会议类型: {}，必须是 production_sales 或 business",
            meeting_type
        ));
    }

    kpi::get_consensus_package(&strategy, &meeting_type, &meeting_date, &user)
}

/// 导出共识包为 CSV 文件
///
/// # 参数
/// - strategy: 策略名称
/// - meeting_type: 会议类型
/// - meeting_date: 会议日期
/// - output_dir: 输出目录
/// - file_prefix: 文件名前缀
/// - user: 生成者
///
/// # 返回
/// CSV 导出结果，包含生成的文件列表
#[tauri::command]
pub async fn export_consensus_csv(
    strategy: String,
    meeting_type: String,
    meeting_date: String,
    output_dir: String,
    file_prefix: Option<String>,
    user: String,
) -> Result<kpi::CsvExportResult, String> {
    // 1. 生成共识包
    let package = kpi::get_consensus_package(&strategy, &meeting_type, &meeting_date, &user)?;

    // 2. 配置导出选项
    let config = kpi::CsvExportConfig {
        output_dir,
        file_prefix: file_prefix.unwrap_or_else(|| {
            format!("consensus_{}", meeting_date.replace("-", ""))
        }),
        add_bom: true,
        export_contracts: true,
        export_kpis: true,
        export_risks: true,
        export_customers: true,
        export_rhythm: true,
        export_recommendations: true,
    };

    // 3. 导出 CSV
    kpi::export_consensus_to_csv(&package, &config)
}

/// 导出合同排名表为单个 CSV 文件
///
/// # 参数
/// - strategy: 策略名称
/// - meeting_type: 会议类型
/// - meeting_date: 会议日期
/// - file_path: 输出文件完整路径
/// - user: 生成者
///
/// # 返回
/// CSV 导出结果
#[tauri::command]
pub async fn export_contracts_ranking_csv(
    strategy: String,
    meeting_type: String,
    meeting_date: String,
    file_path: String,
    user: String,
) -> Result<kpi::CsvExportResult, String> {
    // 1. 生成共识包
    let package = kpi::get_consensus_package(&strategy, &meeting_type, &meeting_date, &user)?;

    // 2. 导出合同排名
    kpi::export_contracts_ranking_csv(&package, &file_path)
}
