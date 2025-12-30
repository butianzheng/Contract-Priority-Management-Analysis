//! 共识包生成模块
//!
//! # 功能
//! 将会议驾驶舱的所有数据打包成统一的共识包结构，便于：
//! - 会议材料导出（CSV/PDF）
//! - 会议记录归档
//! - 跨系统数据分享
//!
//! # 数据组成
//! 共识包包含以下核心数据：
//! - 会议元数据（类型、日期、策略信息）
//! - KPI 汇总（四视角 16 个 KPI）
//! - 风险合同列表（4 类风险）
//! - 排名变化明细
//! - 客户保障分析
//! - 节拍顺行分析
//! - Top N 合同排名

use crate::db::{KpiSummary, RiskContractFlag};
use crate::kpi::{
    CustomerProtectionAnalysis,
    RhythmFlowAnalysis,
    KpiCalculationInput, RiskIdentificationResult,
    calculate_all_kpis, identify_all_risks, RiskIdentificationConfig,
};
use crate::kpi::customer_analysis::{analyze_customer_protection, CustomerProtectionConfig};
use crate::kpi::rhythm_analysis::{analyze_rhythm_flow, RhythmAnalysisConfig};
use serde::{Deserialize, Serialize};

/// 共识包元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMetadata {
    /// 包 ID（UUID 格式）
    pub package_id: String,
    /// 生成时间
    pub generated_at: String,
    /// 会议类型：production_sales（产销会）/ business（经营会）
    pub meeting_type: String,
    /// 会议日期
    pub meeting_date: String,
    /// 使用的策略名称
    pub strategy_name: String,
    /// 策略版本 ID（如果有）
    pub strategy_version_id: Option<i64>,
    /// 生成者
    pub generated_by: String,
    /// 包版本（便于兼容性管理）
    pub package_version: String,
}

/// 合同排名摘要（用于共识包中的 Top N 展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractRankingSummary {
    /// 排名（从 1 开始）
    pub rank: i64,
    /// 合同 ID
    pub contract_id: String,
    /// 客户 ID
    pub customer_id: String,
    /// 客户名称
    pub customer_name: Option<String>,
    /// 客户等级
    pub customer_level: String,
    /// 规格族
    pub spec_family: String,
    /// 钢种
    pub steel_grade: String,
    /// 厚度
    pub thickness: f64,
    /// 宽度
    pub width: f64,
    /// 交期剩余天数
    pub days_to_pdd: i64,
    /// 毛利
    pub margin: f64,
    /// S-Score
    pub s_score: f64,
    /// P-Score
    pub p_score: f64,
    /// 最终优先级
    pub priority: f64,
    /// Alpha 调整系数
    pub alpha: Option<f64>,
    /// 排名变化（与上期对比）
    pub rank_change: Option<i64>,
    /// 变化方向：up / down / unchanged
    pub change_direction: Option<String>,
    /// 是否为风险合同
    pub has_risk: bool,
    /// 风险类型列表
    pub risk_types: Vec<String>,
}

/// 风险汇总统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskSummaryStats {
    /// 风险合同总数
    pub total_risk_count: i64,
    /// 高风险数量
    pub high_risk_count: i64,
    /// 中风险数量
    pub medium_risk_count: i64,
    /// 低风险数量
    pub low_risk_count: i64,
    /// 交期延迟风险数
    pub delivery_delay_count: i64,
    /// 客户降级风险数
    pub customer_downgrade_count: i64,
    /// 毛利损失风险数
    pub margin_loss_count: i64,
    /// 节拍不匹配风险数
    pub rhythm_mismatch_count: i64,
    /// 潜在损失总额
    pub total_potential_loss: f64,
}

/// 行动建议项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRecommendation {
    /// 优先级（1-5，1 最高）
    pub priority: i64,
    /// 类别：risk / kpi / rhythm / customer
    pub category: String,
    /// 建议标题
    pub title: String,
    /// 建议详情
    pub description: String,
    /// 相关合同 ID 列表
    pub related_contracts: Vec<String>,
    /// 相关 KPI 代码
    pub related_kpis: Vec<String>,
    /// 建议负责部门
    pub suggested_department: Option<String>,
}

/// 完整共识包结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusPackage {
    /// 元数据
    pub metadata: ConsensusMetadata,

    /// KPI 汇总（四视角）
    pub kpi_summary: KpiSummary,

    /// 风险汇总统计
    pub risk_summary: RiskSummaryStats,

    /// 风险合同详情列表
    pub risk_contracts: Vec<RiskContractFlag>,

    /// Top N 合同排名（默认 Top 100）
    pub top_contracts: Vec<ContractRankingSummary>,

    /// 客户保障分析
    pub customer_analysis: CustomerProtectionAnalysis,

    /// 节拍顺行分析
    pub rhythm_analysis: RhythmFlowAnalysis,

    /// 行动建议列表
    pub recommendations: Vec<ActionRecommendation>,

    /// 合同总数
    pub total_contracts: i64,

    /// 客户总数
    pub total_customers: i64,
}

/// 共识包生成配置
pub struct ConsensusPackageConfig {
    /// Top N 合同数量
    pub top_n: usize,
    /// 是否包含风险分析
    pub include_risk_analysis: bool,
    /// 是否包含客户分析
    pub include_customer_analysis: bool,
    /// 是否包含节拍分析
    pub include_rhythm_analysis: bool,
    /// 是否生成行动建议
    pub generate_recommendations: bool,
}

impl Default for ConsensusPackageConfig {
    fn default() -> Self {
        Self {
            top_n: 100,
            include_risk_analysis: true,
            include_customer_analysis: true,
            include_rhythm_analysis: true,
            generate_recommendations: true,
        }
    }
}

/// 生成共识包
///
/// # 参数
/// - strategy: 策略名称
/// - meeting_type: 会议类型
/// - meeting_date: 会议日期
/// - user: 生成者
/// - config: 生成配置
///
/// # 返回
/// 完整的共识包结构
pub fn generate_consensus_package(
    strategy: &str,
    meeting_type: &str,
    meeting_date: &str,
    user: &str,
    config: &ConsensusPackageConfig,
) -> Result<ConsensusPackage, String> {
    // 1. 加载计算输入数据
    let input = KpiCalculationInput::load(strategy)?;

    // 2. 计算 KPI
    let kpi_summary = calculate_all_kpis(&input)?;

    // 3. 识别风险合同
    let risk_result = if config.include_risk_analysis {
        let risk_config = RiskIdentificationConfig::default();
        identify_all_risks(&input, None, &risk_config)?
    } else {
        RiskIdentificationResult {
            risk_contracts: Vec::new(),
            stats_by_type: std::collections::HashMap::new(),
            high_risk_count: 0,
            medium_risk_count: 0,
            low_risk_count: 0,
        }
    };

    // 4. 客户保障分析
    let customer_analysis = if config.include_customer_analysis {
        let customer_config = CustomerProtectionConfig::default();
        analyze_customer_protection(&input, &customer_config)?
    } else {
        CustomerProtectionAnalysis {
            total_customers: 0,
            good_count: 0,
            warning_count: 0,
            risk_count: 0,
            level_a_coverage: 100.0,
            level_b_coverage: 100.0,
            customers: Vec::new(),
            risk_customers: Vec::new(),
        }
    };

    // 5. 节拍顺行分析
    let rhythm_analysis = if config.include_rhythm_analysis {
        let rhythm_config = RhythmAnalysisConfig::default();
        analyze_rhythm_flow(&input, &rhythm_config)?
    } else {
        RhythmFlowAnalysis {
            rhythm_config_name: "未配置".to_string(),
            cycle_days: 0,
            current_rhythm_day: 0,
            overall_match_rate: 0.0,
            smooth_days: 0,
            congested_days: 0,
            idle_days: 0,
            daily_summaries: Vec::new(),
            spec_family_summaries: Vec::new(),
            recommendations: Vec::new(),
        }
    };

    // 6. 构建风险合同 ID 集合（用于标记 Top N 中的风险合同）
    let risk_contract_ids: std::collections::HashMap<String, Vec<String>> = {
        let mut map = std::collections::HashMap::new();
        for risk in &risk_result.risk_contracts {
            map.entry(risk.contract_id.clone())
                .or_insert_with(Vec::new)
                .push(risk.risk_type.clone());
        }
        map
    };

    // 7. 构建 Top N 合同排名摘要
    let top_contracts: Vec<ContractRankingSummary> = input.priorities.iter()
        .take(config.top_n)
        .enumerate()
        .map(|(idx, p)| {
            let customer = input.customers.get(&p.contract.customer_id);
            let risk_types = risk_contract_ids
                .get(&p.contract.contract_id)
                .cloned()
                .unwrap_or_default();

            ContractRankingSummary {
                rank: (idx + 1) as i64,
                contract_id: p.contract.contract_id.clone(),
                customer_id: p.contract.customer_id.clone(),
                customer_name: customer.and_then(|c| c.customer_name.clone()),
                customer_level: customer.map(|c| c.customer_level.clone()).unwrap_or_else(|| "C".to_string()),
                spec_family: p.contract.spec_family.clone(),
                steel_grade: p.contract.steel_grade.clone(),
                thickness: p.contract.thickness,
                width: p.contract.width,
                days_to_pdd: p.contract.days_to_pdd,
                margin: p.contract.margin,
                s_score: p.s_score,
                p_score: p.p_score,
                priority: p.priority,
                alpha: p.alpha,
                rank_change: None, // TODO: 需要与上期对比计算
                change_direction: None,
                has_risk: !risk_types.is_empty(),
                risk_types,
            }
        })
        .collect();

    // 8. 构建风险汇总统计
    let risk_summary = RiskSummaryStats {
        total_risk_count: risk_result.risk_contracts.len() as i64,
        high_risk_count: risk_result.high_risk_count,
        medium_risk_count: risk_result.medium_risk_count,
        low_risk_count: risk_result.low_risk_count,
        delivery_delay_count: *risk_result.stats_by_type.get("delivery_delay").unwrap_or(&0),
        customer_downgrade_count: *risk_result.stats_by_type.get("customer_downgrade").unwrap_or(&0),
        margin_loss_count: *risk_result.stats_by_type.get("margin_loss").unwrap_or(&0),
        rhythm_mismatch_count: *risk_result.stats_by_type.get("rhythm_mismatch").unwrap_or(&0),
        total_potential_loss: risk_result.risk_contracts.iter()
            .filter_map(|r| r.potential_loss)
            .sum(),
    };

    // 9. 生成行动建议
    let recommendations = if config.generate_recommendations {
        generate_action_recommendations(
            &kpi_summary,
            &risk_result,
            &customer_analysis,
            &rhythm_analysis,
        )
    } else {
        Vec::new()
    };

    // 10. 构建元数据
    let metadata = ConsensusMetadata {
        package_id: generate_package_id(),
        generated_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        meeting_type: meeting_type.to_string(),
        meeting_date: meeting_date.to_string(),
        strategy_name: strategy.to_string(),
        strategy_version_id: None, // TODO: 从策略版本管理获取
        generated_by: user.to_string(),
        package_version: "1.0.0".to_string(),
    };

    Ok(ConsensusPackage {
        metadata,
        kpi_summary,
        risk_summary,
        risk_contracts: risk_result.risk_contracts,
        top_contracts,
        customer_analysis,
        rhythm_analysis,
        recommendations,
        total_contracts: input.total_contracts as i64,
        total_customers: input.customers.len() as i64,
    })
}

/// 生成行动建议
fn generate_action_recommendations(
    kpi_summary: &KpiSummary,
    risk_result: &RiskIdentificationResult,
    customer_analysis: &CustomerProtectionAnalysis,
    rhythm_analysis: &RhythmFlowAnalysis,
) -> Vec<ActionRecommendation> {
    let mut recommendations = Vec::new();

    // 1. 基于风险合同生成建议
    if risk_result.high_risk_count > 0 {
        let high_risk_contracts: Vec<String> = risk_result.risk_contracts.iter()
            .filter(|r| r.risk_level == "high")
            .take(5)
            .map(|r| r.contract_id.clone())
            .collect();

        recommendations.push(ActionRecommendation {
            priority: 1,
            category: "risk".to_string(),
            title: format!("处理 {} 个高风险合同", risk_result.high_risk_count),
            description: "建议立即关注高风险合同，优先处理交期延迟和客户降级风险".to_string(),
            related_contracts: high_risk_contracts,
            related_kpis: vec!["L03_DELIVERY_FORECAST".to_string(), "S01_CUSTOMER_RISK".to_string()],
            suggested_department: Some("生产计划部".to_string()),
        });
    }

    // 2. 基于 KPI 状态生成建议
    for kpi in &kpi_summary.leadership {
        if kpi.status == "danger" {
            recommendations.push(ActionRecommendation {
                priority: 2,
                category: "kpi".to_string(),
                title: format!("{} 指标异常", kpi.kpi_name),
                description: format!(
                    "当前值 {}，低于警戒线。{}",
                    kpi.display_value,
                    kpi.business_meaning.as_deref().unwrap_or("建议关注并采取措施")
                ),
                related_contracts: Vec::new(),
                related_kpis: vec![kpi.kpi_code.clone()],
                suggested_department: None,
            });
        }
    }

    // 3. 基于客户分析生成建议
    if customer_analysis.risk_count > 0 {
        // 获取风险客户 ID 用于日志记录（如需要可在 description 中展示）
        let _risk_customer_ids: Vec<String> = customer_analysis.risk_customers.iter()
            .take(3)
            .map(|c| c.customer_id.clone())
            .collect();

        recommendations.push(ActionRecommendation {
            priority: 2,
            category: "customer".to_string(),
            title: format!("{} 个重点客户保障不足", customer_analysis.risk_count),
            description: format!(
                "A级客户覆盖率 {:.1}%，B级客户覆盖率 {:.1}%，建议关注高等级客户的合同优先级",
                customer_analysis.level_a_coverage,
                customer_analysis.level_b_coverage
            ),
            related_contracts: Vec::new(),
            related_kpis: vec!["S02_VIP_COVERAGE".to_string()],
            suggested_department: Some("销售部".to_string()),
        });
    }

    // 4. 基于节拍分析生成建议
    if rhythm_analysis.congested_days > 0 {
        recommendations.push(ActionRecommendation {
            priority: 3,
            category: "rhythm".to_string(),
            title: format!("{} 天节拍拥堵", rhythm_analysis.congested_days),
            description: format!(
                "整体匹配率 {:.1}%，存在 {} 天节拍拥堵。{}",
                rhythm_analysis.overall_match_rate,
                rhythm_analysis.congested_days,
                rhythm_analysis.recommendations.first().cloned().unwrap_or_default()
            ),
            related_contracts: Vec::new(),
            related_kpis: vec!["P01_RHYTHM_MATCH".to_string()],
            suggested_department: Some("生产调度".to_string()),
        });
    }

    // 按优先级排序
    recommendations.sort_by_key(|r| r.priority);

    recommendations
}

/// 生成包 ID（简化版 UUID）
fn generate_package_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("CP-{:016X}", now)
}

/// 获取共识包（Tauri 命令辅助函数）
pub fn get_consensus_package(
    strategy: &str,
    meeting_type: &str,
    meeting_date: &str,
    user: &str,
) -> Result<ConsensusPackage, String> {
    let config = ConsensusPackageConfig::default();
    generate_consensus_package(strategy, meeting_type, meeting_date, user, &config)
}
