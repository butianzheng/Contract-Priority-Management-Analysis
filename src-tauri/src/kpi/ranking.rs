//! 排名变化计算与 Explain 模块
//!
//! # 功能
//! - 对比两个时间点的合同排名
//! - 计算 S-Score 和 P-Score 各子项的变化
//! - 识别主要变化因素
//! - 生成自然语言解释

use crate::db::{self, RankingChangeDetail, ContractPriority};
use crate::scoring::{SScoreInput, PScoreInput, generate_s_score_explain, generate_p_score_explain_with_aggregation};
use crate::config;
use std::collections::HashMap;

/// 排名变化计算结果
pub struct RankingChangeResult {
    /// 排名变化明细列表
    pub changes: Vec<RankingChangeDetail>,
    /// 上升合同数
    pub up_count: i64,
    /// 下降合同数
    pub down_count: i64,
    /// 不变合同数
    pub unchanged_count: i64,
    /// 平均变化幅度
    pub avg_change: f64,
    /// 最大上升
    pub max_up: i64,
    /// 最大下降
    pub max_down: i64,
}

/// 计算排名变化（当前 vs 历史快照）
///
/// # 参数
/// - current_priorities: 当前的优先级结果（已排序）
/// - previous_results: 历史的沙盘计算结果
/// - snapshot_id: 当前会议快照 ID
/// - compare_snapshot_id: 对比的会议快照 ID
pub fn calculate_ranking_changes(
    current_priorities: &[ContractPriority],
    previous_results: &[db::SandboxResult],
    snapshot_id: i64,
    compare_snapshot_id: Option<i64>,
    strategy: &str,
) -> Result<RankingChangeResult, String> {
    // 建立历史数据索引
    let prev_by_id: HashMap<&str, &db::SandboxResult> = previous_results.iter()
        .map(|r| (r.contract_id.as_str(), r))
        .collect();

    // 加载评分配置
    let scoring_config = config::load_scoring_config()?;
    let s_weights = config::load_strategy_scoring_weights(strategy)?;
    let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;
    let p2_curve_config = db::get_p2_curve_config()?;

    let mut changes = Vec::new();
    let mut up_count = 0i64;
    let mut down_count = 0i64;
    let mut unchanged_count = 0i64;
    let mut total_abs_change = 0i64;
    let mut max_up = 0i64;
    let mut max_down = 0i64;

    // 获取策略权重
    let weights = db::get_strategy_weights(strategy)?;

    for (new_rank, priority) in current_priorities.iter().enumerate() {
        let new_rank = new_rank as i64 + 1; // 排名从 1 开始

        // 查找历史数据
        let prev = prev_by_id.get(priority.contract.contract_id.as_str());

        // 计算当前的 S/P Score 详情
        let customer = db::get_customer(&priority.contract.customer_id).unwrap_or_else(|_| db::Customer {
            customer_id: priority.contract.customer_id.clone(),
            customer_name: None,
            customer_level: "C".to_string(),
            credit_level: None,
            customer_group: None,
        });

        let s_input = SScoreInput {
            customer_level: customer.customer_level.clone(),
            margin: priority.contract.margin,
            days_to_pdd: priority.contract.days_to_pdd,
        };
        let s_explain = generate_s_score_explain(
            &s_input,
            s_weights.w1,
            s_weights.w2,
            s_weights.w3,
            &scoring_config,
        );

        let p_input = PScoreInput {
            steel_grade: priority.contract.steel_grade.clone(),
            thickness: priority.contract.thickness,
            width: priority.contract.width,
            spec_family: priority.contract.spec_family.clone(),
            days_to_pdd: priority.contract.days_to_pdd,
        };
        let aggregation_stats = aggregation_stats_map.get(&priority.contract.contract_id);
        let p_explain = generate_p_score_explain_with_aggregation(
            &p_input,
            &scoring_config,
            aggregation_stats,
            Some(&p2_curve_config),
            0.5, 0.3, 0.2, // P-Score 子权重
        )?;

        // 计算变化
        let (old_rank, old_priority, old_s_score, old_p_score, s1_old, s2_old, s3_old, p1_old, p2_old, p3_old) =
            if let Some(prev_data) = prev {
                (
                    prev_data.priority_rank,
                    Some(prev_data.priority),
                    Some(prev_data.s_score),
                    Some(prev_data.p_score),
                    prev_data.s1_score,
                    prev_data.s2_score,
                    prev_data.s3_score,
                    prev_data.p1_score,
                    prev_data.p2_score,
                    prev_data.p3_score,
                )
            } else {
                (None, None, None, None, None, None, None, None, None, None)
            };

        let rank_change = old_rank.map(|old| old - new_rank); // 正数表示上升
        let priority_change = old_priority.map(|old| priority.priority - old);
        let s_score_change = old_s_score.map(|old| priority.s_score - old);
        let p_score_change = old_p_score.map(|old| priority.p_score - old);

        let s1_new = s_explain.s1_customer_level.score;
        let s2_new = s_explain.s2_margin.score;
        let s3_new = s_explain.s3_urgency.score;
        let p1_new = p_explain.p1_difficulty.score;
        let p2_new = p_explain.p2_aggregation.score;
        let p3_new = p_explain.p3_rhythm.score;

        let s1_change = s1_old.map(|old| s1_new - old);
        let s2_change = s2_old.map(|old| s2_new - old);
        let s3_change = s3_old.map(|old| s3_new - old);
        let p1_change = p1_old.map(|old| p1_new - old);
        let p2_change = p2_old.map(|old| p2_new - old);
        let p3_change = p3_old.map(|old| p3_new - old);

        // 识别主要变化因素
        let (primary_factor, primary_factor_name) = identify_primary_factor(
            s1_change, s2_change, s3_change,
            p1_change, p2_change, p3_change,
        );

        // 生成解释文本
        let explain_text = generate_explain_text(
            rank_change,
            &primary_factor,
            &primary_factor_name,
            s1_change, s2_change, s3_change,
            p1_change, p2_change, p3_change,
        );

        // 统计
        if let Some(change) = rank_change {
            if change > 0 {
                up_count += 1;
                max_up = max_up.max(change);
            } else if change < 0 {
                down_count += 1;
                max_down = max_down.min(change);
            } else {
                unchanged_count += 1;
            }
            total_abs_change += change.abs();
        }

        changes.push(RankingChangeDetail {
            change_id: None,
            snapshot_id,
            compare_snapshot_id,
            contract_id: priority.contract.contract_id.clone(),
            old_rank,
            new_rank,
            rank_change,
            old_priority,
            new_priority: priority.priority,
            priority_change,
            old_s_score,
            new_s_score: priority.s_score,
            s_score_change,
            s1_change,
            s1_old,
            s1_new: Some(s1_new),
            s2_change,
            s2_old,
            s2_new: Some(s2_new),
            s3_change,
            s3_old,
            s3_new: Some(s3_new),
            old_p_score,
            new_p_score: priority.p_score,
            p_score_change,
            p1_change,
            p1_old,
            p1_new: Some(p1_new),
            p2_change,
            p2_old,
            p2_new: Some(p2_new),
            p3_change,
            p3_old,
            p3_new: Some(p3_new),
            primary_factor: Some(primary_factor),
            primary_factor_name: Some(primary_factor_name),
            explain_text: Some(explain_text),
            ws_used: Some(weights.ws),
            wp_used: Some(weights.wp),
            created_at: None,
        });
    }

    let total = current_priorities.len() as i64;
    let avg_change = if total > 0 {
        total_abs_change as f64 / total as f64
    } else {
        0.0
    };

    Ok(RankingChangeResult {
        changes,
        up_count,
        down_count,
        unchanged_count,
        avg_change,
        max_up,
        max_down: max_down.abs(),
    })
}

/// 识别主要变化因素
fn identify_primary_factor(
    s1_change: Option<f64>,
    s2_change: Option<f64>,
    s3_change: Option<f64>,
    p1_change: Option<f64>,
    p2_change: Option<f64>,
    p3_change: Option<f64>,
) -> (String, String) {
    let factors = [
        (s1_change.unwrap_or(0.0).abs(), "s1_customer", "客户等级"),
        (s2_change.unwrap_or(0.0).abs(), "s2_margin", "毛利水平"),
        (s3_change.unwrap_or(0.0).abs(), "s3_urgency", "紧急程度"),
        (p1_change.unwrap_or(0.0).abs(), "p1_difficulty", "工艺难度"),
        (p2_change.unwrap_or(0.0).abs(), "p2_aggregation", "聚合程度"),
        (p3_change.unwrap_or(0.0).abs(), "p3_rhythm", "节拍匹配"),
    ];

    let max_factor = factors.iter()
        .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
        .unwrap();

    if max_factor.0 < 0.01 {
        ("unchanged".to_string(), "无明显变化".to_string())
    } else {
        (max_factor.1.to_string(), max_factor.2.to_string())
    }
}

/// 生成解释文本
fn generate_explain_text(
    rank_change: Option<i64>,
    primary_factor: &str,
    primary_factor_name: &str,
    s1_change: Option<f64>,
    s2_change: Option<f64>,
    s3_change: Option<f64>,
    p1_change: Option<f64>,
    p2_change: Option<f64>,
    p3_change: Option<f64>,
) -> String {
    let rank_desc = match rank_change {
        Some(c) if c > 0 => format!("排名上升 {} 位", c),
        Some(c) if c < 0 => format!("排名下降 {} 位", c.abs()),
        Some(_) => "排名不变".to_string(),
        None => "新增合同".to_string(),
    };

    if primary_factor == "unchanged" {
        return format!("{}，各项评分无明显变化", rank_desc);
    }

    let change_value = match primary_factor {
        "s1_customer" => s1_change,
        "s2_margin" => s2_change,
        "s3_urgency" => s3_change,
        "p1_difficulty" => p1_change,
        "p2_aggregation" => p2_change,
        "p3_rhythm" => p3_change,
        _ => None,
    };

    let change_desc = match change_value {
        Some(v) if v > 0.0 => format!("{} +{:.1}", primary_factor_name, v),
        Some(v) if v < 0.0 => format!("{} {:.1}", primary_factor_name, v),
        _ => format!("{} 变化", primary_factor_name),
    };

    format!("{}，主因：{}", rank_desc, change_desc)
}

/// 计算与上次会议快照的排名变化
pub fn calculate_ranking_changes_from_snapshots(
    current_snapshot_id: i64,
    previous_snapshot_id: i64,
    strategy: &str,
) -> Result<RankingChangeResult, String> {
    // 获取历史沙盘结果
    let previous_results = db::get_sandbox_results(previous_snapshot_id)?;

    // 获取当前优先级
    let input = crate::kpi::KpiCalculationInput::load(strategy)?;

    calculate_ranking_changes(
        &input.priorities,
        &previous_results,
        current_snapshot_id,
        Some(previous_snapshot_id),
        strategy,
    )
}

/// 保存排名变化到数据库
pub fn save_ranking_changes(
    snapshot_id: i64,
    compare_snapshot_id: Option<i64>,
    changes: &[RankingChangeDetail],
) -> Result<i64, String> {
    db::save_ranking_change_details(snapshot_id, compare_snapshot_id, changes)
}
