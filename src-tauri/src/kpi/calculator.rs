//! KPI 计算器
//!
//! 实现 16 个 KPI 的具体计算逻辑

use crate::db::{self, ContractPriority, Customer, KpiValue, KpiSummary, MeetingKpiConfig};
use chrono::Datelike;
use std::collections::HashMap;

/// KPI 计算输入数据
pub struct KpiCalculationInput {
    /// 优先级计算结果（已排序）
    pub priorities: Vec<ContractPriority>,
    /// 客户数据映射
    pub customers: HashMap<String, Customer>,
    /// 总合同数
    pub total_contracts: usize,
}

impl KpiCalculationInput {
    /// 从数据库加载计算所需数据
    pub fn load(strategy: &str) -> Result<Self, String> {
        // 获取所有合同的优先级（已排序）
        let contracts = db::list_contracts()?;
        let weights = db::get_strategy_weights(strategy)?;
        let scoring_config = crate::config::load_scoring_config()?;
        let s_weights = crate::config::load_strategy_scoring_weights(strategy)?;
        let aggregation_stats_map = db::get_all_contracts_aggregation_stats()?;
        let p2_curve_config = db::get_p2_curve_config()?;

        let mut priorities = Vec::new();
        let mut customers = HashMap::new();

        for contract in &contracts {
            // 获取客户数据
            let customer = db::get_customer(&contract.customer_id).unwrap_or_else(|_| Customer {
                customer_id: contract.customer_id.clone(),
                customer_name: None,
                customer_level: "C".to_string(),
                credit_level: None,
                customer_group: None,
            });
            customers.insert(contract.customer_id.clone(), customer.clone());

            // 计算 S-Score
            let s_input = crate::scoring::SScoreInput {
                customer_level: customer.customer_level.clone(),
                margin: contract.margin,
                days_to_pdd: contract.days_to_pdd,
            };
            let s_score = crate::scoring::calc_s_score(
                s_input,
                s_weights.w1,
                s_weights.w2,
                s_weights.w3,
                &scoring_config,
            );

            // 计算 P-Score
            let p_input = crate::scoring::PScoreInput {
                steel_grade: contract.steel_grade.clone(),
                thickness: contract.thickness,
                width: contract.width,
                spec_family: contract.spec_family.clone(),
                days_to_pdd: contract.days_to_pdd,
            };
            let aggregation_stats = aggregation_stats_map.get(&contract.contract_id);
            let p_score = crate::scoring::calc_p_score_with_aggregation(
                p_input,
                &scoring_config,
                aggregation_stats,
                Some(&p2_curve_config),
            )?;

            // 计算优先级
            let mut priority = crate::scoring::calc_priority(s_score, p_score, weights.ws, weights.wp);

            // 应用 alpha
            let alpha = db::get_latest_alpha(&contract.contract_id).ok().flatten();
            if let Some(a) = alpha {
                priority = crate::scoring::apply_alpha(priority, a);
            }

            priorities.push(ContractPriority {
                contract: contract.clone(),
                s_score,
                p_score,
                priority,
                alpha,
            });
        }

        // 按优先级降序排序
        priorities.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap());

        let total_contracts = priorities.len();

        Ok(Self {
            priorities,
            customers,
            total_contracts,
        })
    }
}

/// 计算所有 KPI
pub fn calculate_all_kpis(input: &KpiCalculationInput) -> Result<KpiSummary, String> {
    let configs = db::list_meeting_kpi_configs()?;

    let mut leadership = Vec::new();
    let mut sales = Vec::new();
    let mut production = Vec::new();
    let mut finance = Vec::new();

    for config in configs {
        let kpi_value = calculate_single_kpi(&config, input)?;

        match config.kpi_category.as_str() {
            "leadership" => leadership.push(kpi_value),
            "sales" => sales.push(kpi_value),
            "production" => production.push(kpi_value),
            "finance" => finance.push(kpi_value),
            _ => {}
        }
    }

    Ok(KpiSummary {
        leadership,
        sales,
        production,
        finance,
    })
}

/// 计算单个 KPI
pub fn calculate_single_kpi(
    config: &MeetingKpiConfig,
    input: &KpiCalculationInput,
) -> Result<KpiValue, String> {
    let value = match config.kpi_code.as_str() {
        // ============================================
        // 领导视角 KPI (L01-L04)
        // ============================================
        "L01_HIGH_PRIORITY_RATIO" => calc_high_priority_ratio(input),
        "L02_CUSTOMER_COVERAGE" => calc_customer_coverage(input),
        "L03_DELIVERY_FORECAST" => calc_delivery_forecast(input),
        "L04_MARGIN_INDEX" => calc_margin_index(input),

        // ============================================
        // 销售视角 KPI (S01-S04)
        // ============================================
        "S01_CUSTOMER_RISK" => calc_customer_risk_count(input),
        "S02_VIP_COVERAGE" => calc_vip_coverage(input),
        "S03_NEW_CUSTOMER_PRIORITY" => calc_new_customer_avg_rank(input),
        "S04_URGENT_HANDLING" => calc_urgent_handling_rate(input),

        // ============================================
        // 生产视角 KPI (P01-P04)
        // ============================================
        "P01_RHYTHM_MATCH" => calc_rhythm_match_rate(input),
        "P02_SPEC_AGGREGATION" => calc_spec_aggregation(input),
        "P03_DIFFICULTY_BALANCE" => calc_difficulty_balance(input),
        "P04_CAPACITY_FORECAST" => calc_capacity_forecast(input),

        // ============================================
        // 财务视角 KPI (F01-F04)
        // ============================================
        "F01_MARGIN_CONTRIBUTION" => calc_margin_contribution(input),
        "F02_HIGH_MARGIN_RATIO" => calc_high_margin_ratio(input),
        "F03_RISK_EXPOSURE" => calc_risk_exposure(input),
        "F04_CREDIT_RISK" => calc_credit_risk_count(input),

        _ => 0.0,
    };

    // 格式化显示值
    let display_value = format_kpi_value(value, config);

    // 计算状态（红黄绿灯）
    let status = calculate_kpi_status(value, config);

    Ok(KpiValue {
        kpi_code: config.kpi_code.clone(),
        kpi_name: config.kpi_name.clone(),
        value,
        display_value,
        status,
        change: None,
        change_direction: None,
        business_meaning: config.business_meaning.clone(),
    })
}

// ============================================
// 领导视角 KPI 计算实现
// ============================================

/// L01: 高优合同占比
/// 计算优先级 Top 20% 的合同占总合同的比例
fn calc_high_priority_ratio(input: &KpiCalculationInput) -> f64 {
    if input.total_contracts == 0 {
        return 0.0;
    }

    let top_20_percent_count = (input.total_contracts as f64 * 0.2).ceil() as usize;
    let top_20_percent_count = top_20_percent_count.min(input.priorities.len());

    // 已经按优先级排序，取前 20% 的平均优先级
    let avg_priority: f64 = if top_20_percent_count > 0 {
        input.priorities.iter()
            .take(top_20_percent_count)
            .map(|p| p.priority)
            .sum::<f64>() / top_20_percent_count as f64
    } else {
        0.0
    };

    // 转换为百分比形式
    avg_priority * 100.0
}

/// L02: 重点客户覆盖率
/// A/B级客户有在产合同的比例
fn calc_customer_coverage(input: &KpiCalculationInput) -> f64 {
    // 统计 A/B 级客户
    let ab_customers: Vec<&Customer> = input.customers.values()
        .filter(|c| c.customer_level == "A" || c.customer_level == "B")
        .collect();

    if ab_customers.is_empty() {
        return 100.0; // 没有 A/B 客户，视为 100% 覆盖
    }

    // 检查有合同的 A/B 客户
    let customers_with_contracts: std::collections::HashSet<&str> = input.priorities.iter()
        .map(|p| p.contract.customer_id.as_str())
        .collect();

    let covered_count = ab_customers.iter()
        .filter(|c| customers_with_contracts.contains(c.customer_id.as_str()))
        .count();

    (covered_count as f64 / ab_customers.len() as f64) * 100.0
}

/// L03: 交期达成率预测
/// 预计能按期交付的合同比例
fn calc_delivery_forecast(input: &KpiCalculationInput) -> f64 {
    if input.total_contracts == 0 {
        return 100.0;
    }

    // 简化规则：days_to_pdd > 0 且排名靠前的视为能按期交付
    // 实际场景应考虑产能、工艺难度等因素
    let on_time_count = input.priorities.iter()
        .enumerate()
        .filter(|(rank, p)| {
            // 交期充足（> 3天）或排名靠前
            p.contract.days_to_pdd > 3 || *rank < input.total_contracts / 2
        })
        .count();

    (on_time_count as f64 / input.total_contracts as f64) * 100.0
}

/// L04: 毛利保障指数
/// 加权平均毛利水平（考虑优先级权重）
fn calc_margin_index(input: &KpiCalculationInput) -> f64 {
    if input.priorities.is_empty() {
        return 0.0;
    }

    // 使用优先级作为权重计算加权平均毛利
    let total_weight: f64 = input.priorities.iter().map(|p| p.priority).sum();
    if total_weight == 0.0 {
        return 0.0;
    }

    let weighted_margin: f64 = input.priorities.iter()
        .map(|p| p.contract.margin * p.priority)
        .sum();

    // 归一化到 0-100 范围
    (weighted_margin / total_weight).min(100.0).max(0.0)
}

// ============================================
// 销售视角 KPI 计算实现
// ============================================

/// S01: 客户满意风险数
/// 高优客户被降低优先级的合同数
fn calc_customer_risk_count(input: &KpiCalculationInput) -> f64 {
    // 找出 A 级客户的合同，但排名在后 50% 的
    let half_rank = input.total_contracts / 2;

    input.priorities.iter()
        .enumerate()
        .filter(|(rank, p)| {
            let customer = input.customers.get(&p.contract.customer_id);
            let is_high_priority_customer = customer
                .map(|c| c.customer_level == "A")
                .unwrap_or(false);

            is_high_priority_customer && *rank >= half_rank
        })
        .count() as f64
}

/// S02: VIP客户保障率
/// VIP客户合同进入 Top 30% 的比例
fn calc_vip_coverage(input: &KpiCalculationInput) -> f64 {
    // VIP = A 级客户
    let vip_contracts: Vec<usize> = input.priorities.iter()
        .enumerate()
        .filter(|(_, p)| {
            input.customers.get(&p.contract.customer_id)
                .map(|c| c.customer_level == "A")
                .unwrap_or(false)
        })
        .map(|(rank, _)| rank)
        .collect();

    if vip_contracts.is_empty() {
        return 100.0;
    }

    let top_30_percent_rank = (input.total_contracts as f64 * 0.3).ceil() as usize;
    let in_top_30 = vip_contracts.iter()
        .filter(|&&rank| rank < top_30_percent_rank)
        .count();

    (in_top_30 as f64 / vip_contracts.len() as f64) * 100.0
}

/// S03: 新客户平均排名
/// 新客户订单的平均排名
fn calc_new_customer_avg_rank(input: &KpiCalculationInput) -> f64 {
    // 假设 C 级客户为新客户（实际应有更明确的标识）
    let new_customer_ranks: Vec<usize> = input.priorities.iter()
        .enumerate()
        .filter(|(_, p)| {
            input.customers.get(&p.contract.customer_id)
                .map(|c| c.customer_level == "C")
                .unwrap_or(true)
        })
        .map(|(rank, _)| rank + 1) // 排名从 1 开始
        .collect();

    if new_customer_ranks.is_empty() {
        return 0.0;
    }

    new_customer_ranks.iter().sum::<usize>() as f64 / new_customer_ranks.len() as f64
}

/// S04: 紧急订单响应率
/// 7天内交期订单进入 Top 50 的比例
fn calc_urgent_handling_rate(input: &KpiCalculationInput) -> f64 {
    // 找出 7 天内交期的合同
    let urgent_contracts: Vec<usize> = input.priorities.iter()
        .enumerate()
        .filter(|(_, p)| p.contract.days_to_pdd <= 7)
        .map(|(rank, _)| rank)
        .collect();

    if urgent_contracts.is_empty() {
        return 100.0; // 没有紧急订单，视为 100%
    }

    let top_50_rank = 50.min(input.total_contracts);
    let in_top_50 = urgent_contracts.iter()
        .filter(|&&rank| rank < top_50_rank)
        .count();

    (in_top_50 as f64 / urgent_contracts.len() as f64) * 100.0
}

// ============================================
// 生产视角 KPI 计算实现
// ============================================

/// P01: 节拍匹配度
/// 符合当日节拍标签的合同比例
fn calc_rhythm_match_rate(input: &KpiCalculationInput) -> f64 {
    // 获取当前激活的节拍配置
    let rhythm_config = match db::get_active_rhythm_config() {
        Ok(config) => config,
        Err(_) => return 50.0, // 默认值
    };

    let rhythm_labels = match db::list_rhythm_labels(rhythm_config.config_id.unwrap_or(1)) {
        Ok(labels) => labels,
        Err(_) => return 50.0,
    };

    if rhythm_labels.is_empty() || input.priorities.is_empty() {
        return 50.0;
    }

    // 计算当前是周期的第几天
    let today = chrono::Local::now().ordinal() as i32;
    let rhythm_day = ((today - 1) % rhythm_config.cycle_days) + 1;

    // 找出当天的节拍标签
    let today_labels: Vec<_> = rhythm_labels.iter()
        .filter(|l| l.rhythm_day == rhythm_day)
        .collect();

    if today_labels.is_empty() {
        return 50.0;
    }

    // 检查 Top 100 合同的匹配度
    let top_100 = input.priorities.iter().take(100);
    let matched_count = top_100
        .filter(|p| {
            today_labels.iter().any(|label| {
                match &label.match_spec {
                    Some(spec) if spec == "*" => true,
                    Some(spec) => spec.split(',')
                        .any(|s| s.trim() == p.contract.spec_family),
                    None => true,
                }
            })
        })
        .count();

    let total = input.priorities.len().min(100);
    if total == 0 {
        return 50.0;
    }

    (matched_count as f64 / total as f64) * 100.0
}

/// P02: 规格聚合度
/// Top 100 合同的规格聚合程度
fn calc_spec_aggregation(input: &KpiCalculationInput) -> f64 {
    let top_100: Vec<_> = input.priorities.iter().take(100).collect();

    if top_100.is_empty() {
        return 0.0;
    }

    // 统计规格族分布
    let mut spec_counts: HashMap<&str, usize> = HashMap::new();
    for p in &top_100 {
        *spec_counts.entry(&p.contract.spec_family).or_insert(0) += 1;
    }

    // 计算聚合度：最大规格族占比 * 100
    let max_count = spec_counts.values().max().copied().unwrap_or(0);

    (max_count as f64 / top_100.len() as f64) * 100.0
}

/// P03: 工艺难度均衡度
/// 高难度合同在各时段的分布均衡程度
fn calc_difficulty_balance(input: &KpiCalculationInput) -> f64 {
    // 获取工艺难度配置
    let difficulties = match db::list_process_difficulty() {
        Ok(d) => d,
        Err(_) => return 50.0,
    };

    if difficulties.is_empty() || input.priorities.is_empty() {
        return 50.0;
    }

    // 找出高难度合同（难度等级为 "高" 或 "H"）
    let high_difficulty_contracts: Vec<_> = input.priorities.iter()
        .filter(|p| {
            difficulties.iter().any(|d| {
                d.steel_grade == p.contract.steel_grade
                    && p.contract.thickness >= d.thickness_min
                    && p.contract.thickness < d.thickness_max
                    && p.contract.width >= d.width_min
                    && p.contract.width < d.width_max
                    && (d.difficulty_level == "高" || d.difficulty_level == "H")
            })
        })
        .collect();

    if high_difficulty_contracts.is_empty() {
        return 100.0; // 没有高难度合同，视为完全均衡
    }

    // 按排名分成 4 个时段，检查分布
    let total = input.total_contracts;
    let quarter = total / 4;

    let mut quarters = [0usize; 4];
    for p in &high_difficulty_contracts {
        let rank = input.priorities.iter()
            .position(|x| x.contract.contract_id == p.contract.contract_id)
            .unwrap_or(0);

        let q = (rank / quarter.max(1)).min(3);
        quarters[q] += 1;
    }

    // 计算标准差，标准差越小越均衡
    let avg = high_difficulty_contracts.len() as f64 / 4.0;
    let variance: f64 = quarters.iter()
        .map(|&c| (c as f64 - avg).powi(2))
        .sum::<f64>() / 4.0;
    let std_dev = variance.sqrt();

    // 转换为 0-100 分数，标准差越小分数越高
    (100.0 - std_dev * 10.0).max(0.0).min(100.0)
}

/// P04: 产能利用率
/// 预计产能利用率
fn calc_capacity_forecast(_input: &KpiCalculationInput) -> f64 {
    // 简化实现：返回固定值
    // 实际应根据产能配置和合同数量计算
    85.0
}

// ============================================
// 财务视角 KPI 计算实现
// ============================================

/// F01: 预计毛利贡献
/// Top 100 合同的预计毛利总额（万元）
fn calc_margin_contribution(input: &KpiCalculationInput) -> f64 {
    input.priorities.iter()
        .take(100)
        .map(|p| p.contract.margin)
        .sum::<f64>() / 10000.0 // 转换为万元
}

/// F02: 高毛利合同占比
/// 毛利 > 15% 的合同占 Top 100 的比例
fn calc_high_margin_ratio(input: &KpiCalculationInput) -> f64 {
    let top_100: Vec<_> = input.priorities.iter().take(100).collect();

    if top_100.is_empty() {
        return 0.0;
    }

    let high_margin_count = top_100.iter()
        .filter(|p| p.contract.margin > 15.0)
        .count();

    (high_margin_count as f64 / top_100.len() as f64) * 100.0
}

/// F03: 风险敞口金额
/// 风险合同的潜在损失总额（万元）
fn calc_risk_exposure(input: &KpiCalculationInput) -> f64 {
    // 风险合同定义：交期 <= 3天 且排名在后 50%
    let half_rank = input.total_contracts / 2;

    input.priorities.iter()
        .enumerate()
        .filter(|(rank, p)| p.contract.days_to_pdd <= 3 && *rank >= half_rank)
        .map(|(_, p)| p.contract.margin)
        .sum::<f64>() / 10000.0
}

/// F04: 账期风险合同数
/// 信用等级低且金额大的合同数
fn calc_credit_risk_count(input: &KpiCalculationInput) -> f64 {
    input.priorities.iter()
        .filter(|p| {
            let customer = input.customers.get(&p.contract.customer_id);
            let is_low_credit = customer
                .and_then(|c| c.credit_level.as_ref())
                .map(|level| level == "低" || level == "L" || level == "C")
                .unwrap_or(false);

            // 假设毛利 > 50 为大额合同
            is_low_credit && p.contract.margin > 50.0
        })
        .count() as f64
}

// ============================================
// 辅助函数
// ============================================

/// 格式化 KPI 值
fn format_kpi_value(value: f64, config: &MeetingKpiConfig) -> String {
    let decimal_places = config.decimal_places.unwrap_or(2) as usize;

    match config.display_format.as_str() {
        "percent" => format!("{:.prec$}%", value, prec = decimal_places),
        "currency" => format!("{:.prec$}{}", value,
            config.display_unit.as_deref().unwrap_or("万元"), prec = decimal_places),
        "number" => {
            let unit = config.display_unit.as_deref().unwrap_or("");
            format!("{:.prec$}{}", value, unit, prec = decimal_places)
        }
        _ => format!("{:.2}", value),
    }
}

/// 计算 KPI 状态（红黄绿灯）
fn calculate_kpi_status(value: f64, config: &MeetingKpiConfig) -> String {
    let good = config.threshold_good.unwrap_or(80.0);
    let warning = config.threshold_warning.unwrap_or(60.0);
    let danger = config.threshold_danger.unwrap_or(40.0);
    let direction = config.threshold_direction.as_str();

    match direction {
        "higher_better" => {
            if value >= good {
                "good".to_string()
            } else if value >= warning {
                "warning".to_string()
            } else if value >= danger {
                "danger".to_string()
            } else {
                "danger".to_string()
            }
        }
        "lower_better" => {
            if value <= good {
                "good".to_string()
            } else if value <= warning {
                "warning".to_string()
            } else if value <= danger {
                "danger".to_string()
            } else {
                "danger".to_string()
            }
        }
        _ => "neutral".to_string(),
    }
}

/// 比较两期 KPI，计算变化
#[allow(dead_code)]
pub fn compare_kpi_values(current: &KpiValue, previous: &KpiValue) -> KpiValue {
    let change = current.value - previous.value;
    let change_direction = if change > 0.01 {
        Some("up".to_string())
    } else if change < -0.01 {
        Some("down".to_string())
    } else {
        Some("unchanged".to_string())
    };

    KpiValue {
        kpi_code: current.kpi_code.clone(),
        kpi_name: current.kpi_name.clone(),
        value: current.value,
        display_value: current.display_value.clone(),
        status: current.status.clone(),
        change: Some(change),
        change_direction,
        business_meaning: current.business_meaning.clone(),
    }
}
