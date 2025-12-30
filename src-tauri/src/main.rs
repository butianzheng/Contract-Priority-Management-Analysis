// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod config;  // 🆕 配置模块
mod db;
mod io;      // 🆕 导入/导出模块
mod kpi;     // 🆕 KPI 计算引擎模块
mod scoring;
mod validation;  // 🆕 数据校验模块

fn main() {
    // 构建上下文，确保数据库路径与 tauri.conf.json 配置一致
    let context = tauri::generate_context!();

    // 初始化数据库
    if let Err(e) = db::initialize_database(context.config()) {
        eprintln!("Failed to initialize database: {}", e);
        std::process::exit(1);
    }

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::compute_priority,
            commands::compute_all_priorities,
            commands::get_contracts,
            commands::get_strategies,
            commands::set_alpha,
            commands::get_intervention_history,
            commands::get_all_intervention_logs,
            // Phase 2: 配置管理命令
            commands::get_scoring_configs,
            commands::update_config,
            commands::get_config_history,
            commands::rollback_config,
            commands::get_all_strategy_weights,
            commands::get_strategy_weights_list,
            commands::update_strategy_weights,
            commands::upsert_strategy_weight,
            commands::delete_strategy_weight,
            // Phase 4: 筛选器预设命令
            commands::get_filter_presets,
            commands::save_filter_preset,
            commands::delete_filter_preset,
            commands::set_default_filter_preset,
            // Phase 5: 批量操作命令
            commands::batch_adjust_alpha,
            commands::batch_restore_alpha,
            commands::get_batch_operations,
            commands::get_batch_operation_contracts,
            // 统一历史记录查询
            commands::get_unified_history,
            // 导入/导出命令
            commands::preview_import,
            commands::execute_import,
            commands::export_data,
            commands::get_customers,
            commands::get_process_difficulty,
            // Phase 8: 清洗规则管理命令
            commands::get_transform_rules,
            commands::get_transform_rules_by_category,
            commands::get_transform_rule,
            commands::create_transform_rule,
            commands::update_transform_rule,
            commands::delete_transform_rule,
            commands::toggle_transform_rule,
            commands::get_transform_rule_history,
            commands::get_transform_execution_history,
            commands::test_transform_rule,
            commands::execute_transform_rule,
            // Phase 9: 规格族管理命令
            commands::get_spec_families,
            commands::get_enabled_spec_families,
            commands::get_spec_family,
            commands::create_spec_family,
            commands::update_spec_family,
            commands::delete_spec_family,
            commands::toggle_spec_family,
            commands::get_spec_family_history,
            // Phase 10: n日节拍配置管理命令
            commands::get_rhythm_configs,
            commands::get_active_rhythm_config,
            commands::get_rhythm_config,
            commands::create_rhythm_config,
            commands::update_rhythm_config,
            commands::delete_rhythm_config,
            commands::activate_rhythm_config,
            commands::get_rhythm_labels,
            commands::upsert_rhythm_label,
            commands::delete_rhythm_label,
            commands::get_rhythm_config_history,
            // Phase 11: 评分透明化
            commands::explain_priority,
            // Phase 13: 数据校验与质量报告
            commands::get_data_quality_report,
            commands::get_missing_value_strategies,
            commands::update_missing_value_strategy,
            commands::compute_all_priorities_with_validation,
            // Phase 14: 策略版本化（可回放、可复盘）
            commands::create_strategy_version,
            commands::get_strategy_versions,
            commands::get_strategy_version,
            commands::get_active_strategy_version,
            commands::activate_strategy_version,
            commands::lock_strategy_version,
            commands::unlock_strategy_version,
            commands::delete_strategy_version,
            commands::get_strategy_version_history,
            // Phase 14: 沙盘会话管理
            commands::create_sandbox_session,
            commands::get_sandbox_sessions,
            commands::get_sandbox_session,
            commands::run_sandbox_calculation,
            commands::get_sandbox_results,
            // Phase 14: 版本对比
            commands::compare_strategy_versions,
            commands::get_version_comparison,
            // Phase 15: 导入/清洗冲突解决机制产品化
            commands::get_import_audit_history,
            commands::get_import_audit,
            commands::get_pending_import_conflicts,
            commands::resolve_import_conflict,
            commands::batch_resolve_import_conflicts,
            commands::rollback_import,
            commands::get_import_statistics,
            commands::get_field_alignment_rules,
            commands::create_field_alignment_rule,
            commands::get_duplicate_detection_config,
            commands::get_pending_similar_pairs,
            commands::resolve_similar_pair,
            commands::check_duplicate_file_import,
            // Phase 16: 会议驾驶舱 KPI 固化
            commands::create_meeting_snapshot,
            commands::get_meeting_snapshots,
            commands::get_meeting_snapshot,
            commands::update_meeting_snapshot_status,
            commands::delete_meeting_snapshot,
            commands::get_meeting_kpi_configs,
            commands::get_meeting_kpi_configs_by_category,
            commands::create_risk_contract_flag,
            commands::get_risk_contracts,
            commands::update_risk_contract_status,
            commands::get_risk_summary_stats,
            commands::save_ranking_change_details,
            commands::get_ranking_changes,
            commands::get_ranking_change_stats,
            commands::get_consensus_templates,
            commands::get_default_consensus_template,
            commands::create_meeting_action_item,
            commands::get_meeting_action_items,
            commands::update_meeting_action_item_status,
            commands::delete_meeting_action_item,
            // Phase 16: KPI 计算引擎
            commands::calculate_meeting_kpis,
            commands::calculate_single_kpi,
            commands::identify_risk_contracts,
            commands::calculate_ranking_changes,
            // Phase 16 P2: 维度聚合分析
            commands::analyze_customer_protection,
            commands::analyze_rhythm_flow,
            // Phase 16 P3: 共识包生成
            commands::generate_consensus_package,
            // Phase 16 P3: CSV 导出
            commands::export_consensus_csv,
            commands::export_contracts_ranking_csv,
        ])
        .run(context)
        .expect("error while running tauri application");
}
