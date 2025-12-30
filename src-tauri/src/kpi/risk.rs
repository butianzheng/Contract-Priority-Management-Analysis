//! 风险合同识别模块
//!
//! # 功能
//! 自动识别以下类型的风险合同：
//! - delivery_delay: 交期延迟风险
//! - customer_downgrade: 客户降级风险（高优客户排名靠后）
//! - margin_loss: 毛利损失风险
//! - rhythm_mismatch: 节拍不匹配风险
//!
//! # 使用场景
//! 在创建会议快照时，自动扫描合同池并标记风险合同

use crate::db::{self, RiskContractFlag};
use crate::kpi::KpiCalculationInput;
use chrono::Datelike;
use std::collections::HashMap;

/// 风险识别结果
pub struct RiskIdentificationResult {
    /// 识别到的风险合同列表
    pub risk_contracts: Vec<RiskContractFlag>,
    /// 按风险类型统计
    pub stats_by_type: HashMap<String, i64>,
    /// 高风险数量
    pub high_risk_count: i64,
    /// 中风险数量
    pub medium_risk_count: i64,
    /// 低风险数量
    pub low_risk_count: i64,
}

/// 风险识别配置
pub struct RiskIdentificationConfig {
    /// 交期延迟阈值（天）
    pub delivery_delay_threshold: i64,
    /// 客户降级排名阈值（百分比）
    pub customer_downgrade_rank_threshold: f64,
    /// 高毛利阈值
    pub high_margin_threshold: f64,
    /// 毛利损失排名阈值（百分比）
    pub margin_loss_rank_threshold: f64,
}

impl Default for RiskIdentificationConfig {
    fn default() -> Self {
        Self {
            delivery_delay_threshold: 7,
            customer_downgrade_rank_threshold: 0.5, // 后 50%
            high_margin_threshold: 15.0,
            margin_loss_rank_threshold: 0.5, // 后 50%
        }
    }
}

/// 识别所有风险合同
pub fn identify_all_risks(
    input: &KpiCalculationInput,
    snapshot_id: Option<i64>,
    config: &RiskIdentificationConfig,
) -> Result<RiskIdentificationResult, String> {
    let mut risk_contracts = Vec::new();
    let mut stats_by_type: HashMap<String, i64> = HashMap::new();

    // 1. 识别交期延迟风险
    let delivery_risks = identify_delivery_delay_risks(input, snapshot_id, config);
    for risk in &delivery_risks {
        *stats_by_type.entry(risk.risk_type.clone()).or_insert(0) += 1;
    }
    risk_contracts.extend(delivery_risks);

    // 2. 识别客户降级风险
    let customer_risks = identify_customer_downgrade_risks(input, snapshot_id, config);
    for risk in &customer_risks {
        *stats_by_type.entry(risk.risk_type.clone()).or_insert(0) += 1;
    }
    risk_contracts.extend(customer_risks);

    // 3. 识别毛利损失风险
    let margin_risks = identify_margin_loss_risks(input, snapshot_id, config);
    for risk in &margin_risks {
        *stats_by_type.entry(risk.risk_type.clone()).or_insert(0) += 1;
    }
    risk_contracts.extend(margin_risks);

    // 4. 识别节拍不匹配风险
    let rhythm_risks = identify_rhythm_mismatch_risks(input, snapshot_id);
    for risk in &rhythm_risks {
        *stats_by_type.entry(risk.risk_type.clone()).or_insert(0) += 1;
    }
    risk_contracts.extend(rhythm_risks);

    // 统计风险等级
    let high_risk_count = risk_contracts.iter()
        .filter(|r| r.risk_level == "high")
        .count() as i64;
    let medium_risk_count = risk_contracts.iter()
        .filter(|r| r.risk_level == "medium")
        .count() as i64;
    let low_risk_count = risk_contracts.iter()
        .filter(|r| r.risk_level == "low")
        .count() as i64;

    Ok(RiskIdentificationResult {
        risk_contracts,
        stats_by_type,
        high_risk_count,
        medium_risk_count,
        low_risk_count,
    })
}

/// 识别交期延迟风险
///
/// 规则：
/// - 交期 ≤ 7 天且排名在后 50% → 高风险
/// - 交期 ≤ 7 天且排名在后 30% → 中风险
/// - 交期 ≤ 3 天不论排名 → 高风险
fn identify_delivery_delay_risks(
    input: &KpiCalculationInput,
    snapshot_id: Option<i64>,
    config: &RiskIdentificationConfig,
) -> Vec<RiskContractFlag> {
    let total = input.total_contracts;
    let half_rank = (total as f64 * 0.5) as usize;
    let threshold_30 = (total as f64 * 0.3) as usize;

    input.priorities.iter()
        .enumerate()
        .filter_map(|(rank, p)| {
            let days = p.contract.days_to_pdd;

            // 判断是否为风险合同
            let (is_risk, level) = if days <= 3 {
                (true, "high")
            } else if days <= config.delivery_delay_threshold {
                if rank >= total - threshold_30 {
                    (true, "high")
                } else if rank >= half_rank {
                    (true, "medium")
                } else {
                    (false, "")
                }
            } else {
                (false, "")
            };

            if !is_risk {
                return None;
            }

            // 计算风险分数
            let risk_score = if level == "high" {
                90.0 - (days as f64 * 5.0).min(40.0)
            } else {
                60.0 - (days as f64 * 3.0).min(30.0)
            };

            Some(RiskContractFlag {
                flag_id: None,
                snapshot_id,
                contract_id: p.contract.contract_id.clone(),
                risk_type: "delivery_delay".to_string(),
                risk_level: level.to_string(),
                risk_score: Some(risk_score),
                risk_description: format!(
                    "合同 {} 距交期仅 {} 天，但排名第 {}",
                    p.contract.contract_id, days, rank + 1
                ),
                risk_factors: Some(serde_json::to_string(&vec![
                    format!("交期紧迫：{} 天", days),
                    format!("当前排名：{}", rank + 1),
                ]).unwrap_or_default()),
                affected_kpis: Some("[\"L03_DELIVERY_FORECAST\", \"S04_URGENT_HANDLING\"]".to_string()),
                potential_loss: Some(p.contract.margin * 0.1), // 假设延期损失毛利的 10%
                potential_loss_unit: Some("元".to_string()),
                suggested_action: Some("建议提高优先级或协调加急生产".to_string()),
                action_priority: Some(if level == "high" { 1 } else { 2 }),
                status: "open".to_string(),
                handled_by: None,
                handled_at: None,
                handling_note: None,
                created_at: None,
                updated_at: None,
            })
        })
        .collect()
}

/// 识别客户降级风险
///
/// 规则：
/// - A 级客户合同排在后 50% → 高风险
/// - B 级客户合同排在后 30% → 中风险
fn identify_customer_downgrade_risks(
    input: &KpiCalculationInput,
    snapshot_id: Option<i64>,
    config: &RiskIdentificationConfig,
) -> Vec<RiskContractFlag> {
    let total = input.total_contracts;
    let threshold_50 = (total as f64 * config.customer_downgrade_rank_threshold) as usize;
    let threshold_30 = (total as f64 * 0.3) as usize;

    input.priorities.iter()
        .enumerate()
        .filter_map(|(rank, p)| {
            let customer = input.customers.get(&p.contract.customer_id)?;

            let (is_risk, level) = match customer.customer_level.as_str() {
                "A" if rank >= total - threshold_50 => (true, "high"),
                "B" if rank >= total - threshold_30 => (true, "medium"),
                _ => (false, ""),
            };

            if !is_risk {
                return None;
            }

            let risk_score = if level == "high" {
                80.0 + (rank as f64 / total as f64) * 20.0
            } else {
                50.0 + (rank as f64 / total as f64) * 20.0
            };

            Some(RiskContractFlag {
                flag_id: None,
                snapshot_id,
                contract_id: p.contract.contract_id.clone(),
                risk_type: "customer_downgrade".to_string(),
                risk_level: level.to_string(),
                risk_score: Some(risk_score),
                risk_description: format!(
                    "{} 级客户 {} 的合同排名第 {}，可能影响客户满意度",
                    customer.customer_level,
                    customer.customer_name.as_deref().unwrap_or(&customer.customer_id),
                    rank + 1
                ),
                risk_factors: Some(serde_json::to_string(&vec![
                    format!("客户等级：{}", customer.customer_level),
                    format!("当前排名：{}/{}", rank + 1, total),
                    format!("优先级：{:.2}", p.priority),
                ]).unwrap_or_default()),
                affected_kpis: Some("[\"S01_CUSTOMER_RISK\", \"S02_VIP_COVERAGE\"]".to_string()),
                potential_loss: Some(p.contract.margin * 0.2), // 客户流失可能损失 20%
                potential_loss_unit: Some("元".to_string()),
                suggested_action: Some("建议与销售确认客户重要性，必要时提高优先级".to_string()),
                action_priority: Some(if level == "high" { 1 } else { 2 }),
                status: "open".to_string(),
                handled_by: None,
                handled_at: None,
                handling_note: None,
                created_at: None,
                updated_at: None,
            })
        })
        .collect()
}

/// 识别毛利损失风险
///
/// 规则：
/// - 毛利 > 15% 但排名在后 50% → 中风险
/// - 毛利 > 20% 但排名在后 30% → 高风险
fn identify_margin_loss_risks(
    input: &KpiCalculationInput,
    snapshot_id: Option<i64>,
    config: &RiskIdentificationConfig,
) -> Vec<RiskContractFlag> {
    let total = input.total_contracts;
    let threshold_50 = (total as f64 * config.margin_loss_rank_threshold) as usize;
    let threshold_30 = (total as f64 * 0.3) as usize;

    input.priorities.iter()
        .enumerate()
        .filter_map(|(rank, p)| {
            let margin = p.contract.margin;

            let (is_risk, level) = if margin > 20.0 && rank >= total - threshold_30 {
                (true, "high")
            } else if margin > config.high_margin_threshold && rank >= total - threshold_50 {
                (true, "medium")
            } else {
                (false, "")
            };

            if !is_risk {
                return None;
            }

            let risk_score = if level == "high" {
                75.0 + margin.min(25.0)
            } else {
                50.0 + margin.min(20.0)
            };

            Some(RiskContractFlag {
                flag_id: None,
                snapshot_id,
                contract_id: p.contract.contract_id.clone(),
                risk_type: "margin_loss".to_string(),
                risk_level: level.to_string(),
                risk_score: Some(risk_score),
                risk_description: format!(
                    "高毛利合同（{:.1}%）排名第 {}，可能延迟交付损失利润",
                    margin, rank + 1
                ),
                risk_factors: Some(serde_json::to_string(&vec![
                    format!("毛利率：{:.1}%", margin),
                    format!("当前排名：{}/{}", rank + 1, total),
                ]).unwrap_or_default()),
                affected_kpis: Some("[\"F01_MARGIN_CONTRIBUTION\", \"F02_HIGH_MARGIN_RATIO\"]".to_string()),
                potential_loss: Some(margin * 100.0), // 假设延期损失全部毛利
                potential_loss_unit: Some("元".to_string()),
                suggested_action: Some("建议评估是否提高优先级以保障毛利".to_string()),
                action_priority: Some(if level == "high" { 2 } else { 3 }),
                status: "open".to_string(),
                handled_by: None,
                handled_at: None,
                handling_note: None,
                created_at: None,
                updated_at: None,
            })
        })
        .collect()
}

/// 识别节拍不匹配风险
///
/// 规则：
/// - Top 100 合同中不符合当日节拍标签的合同
fn identify_rhythm_mismatch_risks(
    input: &KpiCalculationInput,
    snapshot_id: Option<i64>,
) -> Vec<RiskContractFlag> {
    // 获取节拍配置
    let rhythm_config = match db::get_active_rhythm_config() {
        Ok(config) => config,
        Err(_) => return Vec::new(),
    };

    let rhythm_labels = match db::list_rhythm_labels(rhythm_config.config_id.unwrap_or(1)) {
        Ok(labels) => labels,
        Err(_) => return Vec::new(),
    };

    if rhythm_labels.is_empty() {
        return Vec::new();
    }

    // 计算当前周期日
    let today = chrono::Local::now().ordinal() as i32;
    let rhythm_day = ((today - 1) % rhythm_config.cycle_days) + 1;

    // 找出当天的节拍标签
    let today_labels: Vec<_> = rhythm_labels.iter()
        .filter(|l| l.rhythm_day == rhythm_day)
        .collect();

    if today_labels.is_empty() {
        return Vec::new();
    }

    // 检查 Top 50 合同
    input.priorities.iter()
        .take(50)
        .enumerate()
        .filter_map(|(rank, p)| {
            // 检查是否匹配当日节拍
            let is_matched = today_labels.iter().any(|label| {
                match &label.match_spec {
                    Some(spec) if spec == "*" => true,
                    Some(spec) => spec.split(',')
                        .any(|s| s.trim() == p.contract.spec_family),
                    None => true,
                }
            });

            if is_matched {
                return None;
            }

            Some(RiskContractFlag {
                flag_id: None,
                snapshot_id,
                contract_id: p.contract.contract_id.clone(),
                risk_type: "rhythm_mismatch".to_string(),
                risk_level: "low".to_string(),
                risk_score: Some(40.0 - rank as f64 * 0.5),
                risk_description: format!(
                    "合同规格族 {} 不在今日节拍标签中（第 {} 天）",
                    p.contract.spec_family, rhythm_day
                ),
                risk_factors: Some(serde_json::to_string(&vec![
                    format!("规格族：{}", p.contract.spec_family),
                    format!("节拍日：{}/{}", rhythm_day, rhythm_config.cycle_days),
                    format!("排名：{}", rank + 1),
                ]).unwrap_or_default()),
                affected_kpis: Some("[\"P01_RHYTHM_MATCH\"]".to_string()),
                potential_loss: None,
                potential_loss_unit: None,
                suggested_action: Some("考虑调整生产顺序或更新节拍配置".to_string()),
                action_priority: Some(3),
                status: "open".to_string(),
                handled_by: None,
                handled_at: None,
                handling_note: None,
                created_at: None,
                updated_at: None,
            })
        })
        .collect()
}

/// 批量保存风险合同到数据库
pub fn save_risk_contracts(
    snapshot_id: i64,
    risks: &[RiskContractFlag],
) -> Result<i64, String> {
    let mut saved_count = 0i64;

    for risk in risks {
        db::create_risk_contract_flag(
            snapshot_id,
            &risk.contract_id,
            &risk.risk_type,
            &risk.risk_level,
            risk.risk_score,
            &risk.risk_description,
            risk.risk_factors.as_deref(),
            risk.affected_kpis.as_deref(),
            risk.potential_loss,
            risk.suggested_action.as_deref(),
            risk.action_priority,
        )?;
        saved_count += 1;
    }

    Ok(saved_count)
}
