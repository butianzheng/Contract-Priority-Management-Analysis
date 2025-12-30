//! 客户保障分析模块
//!
//! # 功能
//! 按客户维度聚合合同数据，分析客户保障情况：
//! - 每个客户的合同数量与排名分布
//! - 客户等级与合同优先级匹配度
//! - 识别"高等级低覆盖"的风险客户
//!
//! # 使用场景
//! 在会议驾驶舱中展示客户视角的保障状态

use crate::db;
use crate::kpi::KpiCalculationInput;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单个客户的保障分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProtectionSummary {
    /// 客户 ID
    pub customer_id: String,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 客户等级 (A/B/C)
    pub customer_level: String,
    /// 该客户的合同数量
    pub contract_count: i64,
    /// 该客户合同的平均排名
    pub avg_rank: f64,
    /// 最高排名（数字越小越好）
    pub best_rank: i64,
    /// 最低排名
    pub worst_rank: i64,
    /// 排名标准差（衡量排名分散程度）
    pub rank_std_dev: f64,
    /// 在 Top N 内的合同数量
    pub top_n_count: i64,
    /// 该客户合同的总毛利
    pub total_margin: f64,
    /// 保障评分 (0-100)
    /// 计算逻辑：综合考虑客户等级与平均排名的匹配度
    pub protection_score: f64,
    /// 保障状态：good / warning / risk
    pub protection_status: String,
    /// 风险描述（如果有）
    pub risk_description: Option<String>,
}

/// 客户保障分析总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerProtectionAnalysis {
    /// 分析的客户总数
    pub total_customers: i64,
    /// 良好保障的客户数
    pub good_count: i64,
    /// 警告状态的客户数
    pub warning_count: i64,
    /// 风险状态的客户数
    pub risk_count: i64,
    /// A 级客户覆盖率（在 Top 50% 内的比例）
    pub level_a_coverage: f64,
    /// B 级客户覆盖率
    pub level_b_coverage: f64,
    /// 每个客户的详细分析
    pub customers: Vec<CustomerProtectionSummary>,
    /// 风险客户列表（高等级但保障差）
    pub risk_customers: Vec<CustomerProtectionSummary>,
}

/// 客户保障分析配置
pub struct CustomerProtectionConfig {
    /// Top N 定义（默认 50）
    pub top_n: usize,
    /// A 级客户期望排名百分位（默认前 30%）
    pub level_a_expected_percentile: f64,
    /// B 级客户期望排名百分位（默认前 50%）
    pub level_b_expected_percentile: f64,
    /// 保障良好的阈值分数
    pub good_threshold: f64,
    /// 风险警告的阈值分数
    pub warning_threshold: f64,
}

impl Default for CustomerProtectionConfig {
    fn default() -> Self {
        Self {
            top_n: 50,
            level_a_expected_percentile: 0.3,
            level_b_expected_percentile: 0.5,
            good_threshold: 70.0,
            warning_threshold: 50.0,
        }
    }
}

/// 执行客户保障分析
///
/// # 参数
/// - input: KPI 计算输入（包含合同优先级和客户数据）
/// - config: 分析配置
///
/// # 返回
/// 客户保障分析结果
pub fn analyze_customer_protection(
    input: &KpiCalculationInput,
    config: &CustomerProtectionConfig,
) -> Result<CustomerProtectionAnalysis, String> {
    let total = input.total_contracts;

    // 1. 按客户分组合同
    let mut customer_contracts: HashMap<String, Vec<(usize, &db::ContractPriority)>> = HashMap::new();

    for (rank, priority) in input.priorities.iter().enumerate() {
        customer_contracts
            .entry(priority.contract.customer_id.clone())
            .or_default()
            .push((rank + 1, priority)); // rank 从 1 开始
    }

    // 2. 计算每个客户的保障指标
    let mut summaries: Vec<CustomerProtectionSummary> = Vec::new();
    let mut level_a_in_top = 0;
    let mut level_a_total = 0;
    let mut level_b_in_top = 0;
    let mut level_b_total = 0;

    let _top_threshold = (total as f64 * 0.5) as usize;

    for (customer_id, contracts) in &customer_contracts {
        let customer = input.customers.get(customer_id);
        let customer_level = customer
            .map(|c| c.customer_level.clone())
            .unwrap_or_else(|| "C".to_string());
        let customer_name = customer.and_then(|c| c.customer_name.clone());

        // 计算排名统计
        let ranks: Vec<i64> = contracts.iter().map(|(r, _)| *r as i64).collect();
        let contract_count = ranks.len() as i64;
        let avg_rank = ranks.iter().sum::<i64>() as f64 / contract_count as f64;
        let best_rank = *ranks.iter().min().unwrap_or(&0);
        let worst_rank = *ranks.iter().max().unwrap_or(&0);

        // 计算标准差
        let variance = ranks.iter()
            .map(|r| (*r as f64 - avg_rank).powi(2))
            .sum::<f64>() / contract_count as f64;
        let rank_std_dev = variance.sqrt();

        // 计算 Top N 内的合同数
        let top_n_count = contracts.iter()
            .filter(|(r, _)| *r <= config.top_n)
            .count() as i64;

        // 计算总毛利
        let total_margin: f64 = contracts.iter()
            .map(|(_, p)| p.contract.margin)
            .sum();

        // 计算保障评分
        let protection_score = calculate_protection_score(
            &customer_level,
            avg_rank,
            total,
            &config,
        );

        // 确定保障状态
        let (protection_status, risk_description) = if protection_score >= config.good_threshold {
            ("good".to_string(), None)
        } else if protection_score >= config.warning_threshold {
            ("warning".to_string(), Some(format!(
                "{} 级客户平均排名 {:.0}，低于预期",
                customer_level, avg_rank
            )))
        } else {
            ("risk".to_string(), Some(format!(
                "{} 级客户平均排名 {:.0}，严重低于预期，建议关注",
                customer_level, avg_rank
            )))
        };

        // 统计客户等级覆盖
        match customer_level.as_str() {
            "A" => {
                level_a_total += 1;
                if avg_rank <= (total as f64 * config.level_a_expected_percentile) as usize as f64 {
                    level_a_in_top += 1;
                }
            }
            "B" => {
                level_b_total += 1;
                if avg_rank <= (total as f64 * config.level_b_expected_percentile) as usize as f64 {
                    level_b_in_top += 1;
                }
            }
            _ => {}
        }

        summaries.push(CustomerProtectionSummary {
            customer_id: customer_id.clone(),
            customer_name,
            customer_level,
            contract_count,
            avg_rank,
            best_rank,
            worst_rank,
            rank_std_dev,
            top_n_count,
            total_margin,
            protection_score,
            protection_status,
            risk_description,
        });
    }

    // 3. 按保障评分排序（评分低的在前，便于关注风险客户）
    summaries.sort_by(|a, b| a.protection_score.partial_cmp(&b.protection_score).unwrap());

    // 4. 统计
    let good_count = summaries.iter().filter(|s| s.protection_status == "good").count() as i64;
    let warning_count = summaries.iter().filter(|s| s.protection_status == "warning").count() as i64;
    let risk_count = summaries.iter().filter(|s| s.protection_status == "risk").count() as i64;

    let level_a_coverage = if level_a_total > 0 {
        level_a_in_top as f64 / level_a_total as f64 * 100.0
    } else {
        100.0
    };

    let level_b_coverage = if level_b_total > 0 {
        level_b_in_top as f64 / level_b_total as f64 * 100.0
    } else {
        100.0
    };

    // 5. 提取风险客户
    let risk_customers: Vec<_> = summaries.iter()
        .filter(|s| s.protection_status == "risk" ||
                   (s.customer_level == "A" && s.protection_status == "warning"))
        .cloned()
        .collect();

    Ok(CustomerProtectionAnalysis {
        total_customers: summaries.len() as i64,
        good_count,
        warning_count,
        risk_count,
        level_a_coverage,
        level_b_coverage,
        customers: summaries,
        risk_customers,
    })
}

/// 计算客户保障评分
///
/// 评分逻辑：
/// - 基于客户等级设定期望排名百分位
/// - 实际排名越接近或优于期望，评分越高
fn calculate_protection_score(
    customer_level: &str,
    avg_rank: f64,
    total: usize,
    config: &CustomerProtectionConfig,
) -> f64 {
    // 根据客户等级确定期望百分位
    let expected_percentile = match customer_level {
        "A" => config.level_a_expected_percentile,
        "B" => config.level_b_expected_percentile,
        _ => 0.7, // C 级客户期望在前 70%
    };

    let expected_rank = (total as f64 * expected_percentile).max(1.0);
    let _actual_percentile = avg_rank / total as f64;

    // 计算评分
    // 如果实际排名优于期望，满分 100
    // 否则按偏离程度扣分
    if avg_rank <= expected_rank {
        100.0
    } else {
        let deviation = (avg_rank - expected_rank) / (total as f64 - expected_rank);
        (100.0 * (1.0 - deviation)).max(0.0)
    }
}

/// 获取客户保障分析（Tauri 命令辅助函数）
pub fn get_customer_protection_analysis(
    strategy: &str,
) -> Result<CustomerProtectionAnalysis, String> {
    let input = KpiCalculationInput::load(strategy)?;
    let config = CustomerProtectionConfig::default();
    analyze_customer_protection(&input, &config)
}
