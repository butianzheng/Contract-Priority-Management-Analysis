//! 节拍顺行分析模块
//!
//! # 功能
//! 按生产维度聚合合同数据，分析节拍匹配情况：
//! - 每日/每周期的节拍匹配率
//! - 规格族分布与节拍标签的对应
//! - 识别节拍冲突和产能瓶颈
//!
//! # 使用场景
//! 在会议驾驶舱中展示生产视角的节拍顺行状态

use crate::db::{self, ContractPriority};
use crate::kpi::KpiCalculationInput;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 单日节拍分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRhythmSummary {
    /// 周期日 (1-n)
    pub rhythm_day: i32,
    /// 节拍标签名称
    pub label_name: String,
    /// 匹配的规格族列表
    pub match_specs: Vec<String>,
    /// 匹配的合同数量
    pub matched_contract_count: i64,
    /// 匹配合同的总重量/产能
    pub matched_total_capacity: f64,
    /// 不匹配但排名靠前的合同数量（Top 50 内）
    pub mismatched_top_count: i64,
    /// 匹配率 (%)
    pub match_rate: f64,
    /// 顺行状态：smooth / congested / idle
    pub rhythm_status: String,
    /// 状态说明
    pub status_description: Option<String>,
}

/// 规格族节拍分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFamilyRhythmSummary {
    /// 规格族代码
    pub spec_family: String,
    /// 该规格族的合同数量
    pub contract_count: i64,
    /// 平均排名
    pub avg_rank: f64,
    /// 在 Top N 内的数量
    pub top_n_count: i64,
    /// 匹配的节拍日列表
    pub matched_rhythm_days: Vec<i32>,
    /// 是否为"热门"规格族（合同数多）
    pub is_hot: bool,
    /// 是否存在节拍冲突
    pub has_conflict: bool,
    /// 冲突说明
    pub conflict_description: Option<String>,
}

/// 节拍顺行分析总结
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmFlowAnalysis {
    /// 当前节拍配置名称
    pub rhythm_config_name: String,
    /// 周期天数
    pub cycle_days: i32,
    /// 当前周期日
    pub current_rhythm_day: i32,
    /// 整体匹配率
    pub overall_match_rate: f64,
    /// 顺行天数
    pub smooth_days: i64,
    /// 拥堵天数
    pub congested_days: i64,
    /// 空闲天数
    pub idle_days: i64,
    /// 每日节拍分析
    pub daily_summaries: Vec<DailyRhythmSummary>,
    /// 规格族节拍分析
    pub spec_family_summaries: Vec<SpecFamilyRhythmSummary>,
    /// 节拍建议
    pub recommendations: Vec<String>,
}

/// 节拍分析配置
pub struct RhythmAnalysisConfig {
    /// Top N 定义
    pub top_n: usize,
    /// 匹配率顺行阈值
    pub smooth_threshold: f64,
    /// 匹配率拥堵阈值
    pub congested_threshold: f64,
    /// 热门规格族合同数阈值
    pub hot_spec_threshold: i64,
}

impl Default for RhythmAnalysisConfig {
    fn default() -> Self {
        Self {
            top_n: 50,
            smooth_threshold: 70.0,
            congested_threshold: 40.0,
            hot_spec_threshold: 10,
        }
    }
}

/// 执行节拍顺行分析
pub fn analyze_rhythm_flow(
    input: &KpiCalculationInput,
    config: &RhythmAnalysisConfig,
) -> Result<RhythmFlowAnalysis, String> {
    // 1. 获取节拍配置
    let rhythm_config = db::get_active_rhythm_config()
        .map_err(|e| format!("获取节拍配置失败: {}", e))?;

    let rhythm_labels = db::list_rhythm_labels(rhythm_config.config_id.unwrap_or(1))
        .unwrap_or_default();

    // 2. 计算当前周期日
    let today = chrono::Local::now().ordinal() as i32;
    let current_rhythm_day = ((today - 1) % rhythm_config.cycle_days) + 1;

    // 3. 按规格族分组 Top N 合同
    let mut spec_family_contracts: HashMap<String, Vec<(usize, &ContractPriority)>> = HashMap::new();
    for (rank, priority) in input.priorities.iter().enumerate().take(config.top_n) {
        spec_family_contracts
            .entry(priority.contract.spec_family.clone())
            .or_default()
            .push((rank + 1, priority));
    }

    // 4. 分析每日节拍
    let mut daily_summaries: Vec<DailyRhythmSummary> = Vec::new();
    let mut total_matched = 0i64;
    let total_top_n = config.top_n.min(input.priorities.len()) as i64;

    for day in 1..=rhythm_config.cycle_days {
        let day_labels: Vec<_> = rhythm_labels.iter()
            .filter(|l| l.rhythm_day == day)
            .collect();

        if day_labels.is_empty() {
            daily_summaries.push(DailyRhythmSummary {
                rhythm_day: day,
                label_name: "未配置".to_string(),
                match_specs: vec![],
                matched_contract_count: 0,
                matched_total_capacity: 0.0,
                mismatched_top_count: 0,
                match_rate: 0.0,
                rhythm_status: "idle".to_string(),
                status_description: Some("该日无节拍标签配置".to_string()),
            });
            continue;
        }

        // 合并当日所有标签的匹配规格
        let mut all_match_specs: Vec<String> = Vec::new();
        let mut label_names: Vec<String> = Vec::new();

        for label in &day_labels {
            label_names.push(label.label_name.clone());
            if let Some(ref spec) = label.match_spec {
                if spec == "*" {
                    all_match_specs.push("*".to_string());
                } else {
                    all_match_specs.extend(
                        spec.split(',').map(|s| s.trim().to_string())
                    );
                }
            }
        }

        // 计算匹配的合同
        let is_wildcard = all_match_specs.contains(&"*".to_string());
        let mut matched_count = 0i64;
        let mut matched_capacity = 0.0f64;
        let mut mismatched_top = 0i64;

        for (_rank, priority) in input.priorities.iter().enumerate().take(config.top_n) {
            let is_matched = is_wildcard ||
                all_match_specs.contains(&priority.contract.spec_family);

            if is_matched {
                matched_count += 1;
                // 用 width * thickness 近似产能
                matched_capacity += priority.contract.width * priority.contract.thickness / 1000.0;
            } else {
                mismatched_top += 1;
            }
        }

        total_matched += matched_count;

        let match_rate = if total_top_n > 0 {
            matched_count as f64 / total_top_n as f64 * 100.0
        } else {
            0.0
        };

        let (rhythm_status, status_description) = if match_rate >= config.smooth_threshold {
            ("smooth".to_string(), None)
        } else if match_rate >= config.congested_threshold {
            ("congested".to_string(), Some(format!(
                "匹配率 {:.1}%，存在 {} 个不匹配的高优合同",
                match_rate, mismatched_top
            )))
        } else {
            ("idle".to_string(), Some(format!(
                "匹配率仅 {:.1}%，节拍配置可能需要调整",
                match_rate
            )))
        };

        daily_summaries.push(DailyRhythmSummary {
            rhythm_day: day,
            label_name: label_names.join(" + "),
            match_specs: all_match_specs.clone(),
            matched_contract_count: matched_count,
            matched_total_capacity: matched_capacity,
            mismatched_top_count: mismatched_top,
            match_rate,
            rhythm_status,
            status_description,
        });
    }

    // 5. 分析规格族分布
    let mut spec_family_summaries: Vec<SpecFamilyRhythmSummary> = Vec::new();

    for (spec_family, contracts) in &spec_family_contracts {
        let contract_count = contracts.len() as i64;
        let avg_rank = contracts.iter().map(|(r, _)| *r as f64).sum::<f64>() / contract_count as f64;
        let top_n_count = contracts.iter().filter(|(r, _)| *r <= config.top_n).count() as i64;

        // 找出该规格族匹配的节拍日
        let matched_days: Vec<i32> = rhythm_labels.iter()
            .filter(|l| {
                l.match_spec.as_ref().map(|s| {
                    s == "*" || s.split(',').any(|sp| sp.trim() == spec_family)
                }).unwrap_or(false)
            })
            .map(|l| l.rhythm_day)
            .collect();

        let is_hot = contract_count >= config.hot_spec_threshold;
        let has_conflict = is_hot && matched_days.is_empty();
        let conflict_description = if has_conflict {
            Some(format!(
                "热门规格族 {} 有 {} 个合同但无对应节拍标签",
                spec_family, contract_count
            ))
        } else {
            None
        };

        spec_family_summaries.push(SpecFamilyRhythmSummary {
            spec_family: spec_family.clone(),
            contract_count,
            avg_rank,
            top_n_count,
            matched_rhythm_days: matched_days,
            is_hot,
            has_conflict,
            conflict_description,
        });
    }

    // 按合同数降序排序
    spec_family_summaries.sort_by(|a, b| b.contract_count.cmp(&a.contract_count));

    // 6. 统计
    let smooth_days = daily_summaries.iter()
        .filter(|s| s.rhythm_status == "smooth")
        .count() as i64;
    let congested_days = daily_summaries.iter()
        .filter(|s| s.rhythm_status == "congested")
        .count() as i64;
    let idle_days = daily_summaries.iter()
        .filter(|s| s.rhythm_status == "idle")
        .count() as i64;

    let overall_match_rate = if total_top_n > 0 && rhythm_config.cycle_days > 0 {
        total_matched as f64 / (total_top_n * rhythm_config.cycle_days as i64) as f64 * 100.0
    } else {
        0.0
    };

    // 7. 生成建议
    let recommendations = generate_rhythm_recommendations(
        &daily_summaries,
        &spec_family_summaries,
        current_rhythm_day,
    );

    Ok(RhythmFlowAnalysis {
        rhythm_config_name: rhythm_config.config_name,
        cycle_days: rhythm_config.cycle_days,
        current_rhythm_day,
        overall_match_rate,
        smooth_days,
        congested_days,
        idle_days,
        daily_summaries,
        spec_family_summaries,
        recommendations,
    })
}

/// 生成节拍优化建议
fn generate_rhythm_recommendations(
    daily_summaries: &[DailyRhythmSummary],
    spec_family_summaries: &[SpecFamilyRhythmSummary],
    current_day: i32,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // 检查当日节拍状态
    if let Some(today) = daily_summaries.iter().find(|s| s.rhythm_day == current_day) {
        match today.rhythm_status.as_str() {
            "congested" => {
                recommendations.push(format!(
                    "今日（第 {} 天）节拍拥堵，有 {} 个高优合同不匹配当前节拍",
                    current_day, today.mismatched_top_count
                ));
            }
            "idle" => {
                recommendations.push(format!(
                    "今日（第 {} 天）节拍空闲，匹配率仅 {:.1}%，考虑调整节拍配置",
                    current_day, today.match_rate
                ));
            }
            _ => {}
        }
    }

    // 检查热门规格族冲突
    for spec in spec_family_summaries {
        if spec.has_conflict {
            recommendations.push(format!(
                "建议为热门规格族 {} 添加专属节拍标签（当前有 {} 个合同）",
                spec.spec_family, spec.contract_count
            ));
        }
    }

    // 检查空闲天数
    let idle_count = daily_summaries.iter()
        .filter(|s| s.rhythm_status == "idle")
        .count();
    if idle_count > 0 {
        recommendations.push(format!(
            "有 {} 天节拍配置空闲，建议补充节拍标签覆盖更多规格族",
            idle_count
        ));
    }

    recommendations
}

/// 获取节拍顺行分析（Tauri 命令辅助函数）
pub fn get_rhythm_flow_analysis(
    strategy: &str,
) -> Result<RhythmFlowAnalysis, String> {
    let input = KpiCalculationInput::load(strategy)?;
    let config = RhythmAnalysisConfig::default();
    analyze_rhythm_flow(&input, &config)
}
