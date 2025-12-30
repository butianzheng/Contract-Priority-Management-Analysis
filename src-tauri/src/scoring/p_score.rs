//! P-Score 计算模块
//!
//! # 静态规则声明
//!
//! **重要：本模块严格遵守「静态规则 + 合同属性」原则**
//!
//! ## 允许的数据源
//! - 合同主表字段（steel_grade, thickness, width, spec_family, days_to_pdd）
//! - 工艺难度配置表（process_difficulty）
//! - 规格族配置（scoring_config / spec_family_master）
//! - 节拍标签配置（rhythm_label）
//! - 聚合统计数据（合同池快照，批量预计算）
//!
//! ## 严禁的数据源
//! - 实时设备状态（MES）
//! - 动态库存数据（ERP）
//! - 实时排产结果
//! - 设备负载率
//! - 任何需要网络请求的实时数据
//!
//! ## 结果稳定性保证
//! 同一合同属性 + 同一配置数据 + 同一合同池快照 = 完全相同的 P-Score

use crate::db;
use crate::config::ScoringConfig;
use crate::db::schema::{PScoreComponent, PScoreExplain, AggregationStats, P2CurveConfig, P2AggregationDetail};

// ============================================
// 常量定义：P-Score 子权重默认值
// ============================================

/// P1（工艺难度）默认权重
pub const DEFAULT_W_P1: f64 = 0.5;
/// P2（聚合度）默认权重
pub const DEFAULT_W_P2: f64 = 0.3;
/// P3（节拍匹配）默认权重
pub const DEFAULT_W_P3: f64 = 0.2;

/// P1 默认值（未匹配时）
#[allow(dead_code)]
const P1_DEFAULT: f64 = 50.0;
/// P2 默认系数（未匹配规格族时）
const P2_DEFAULT_FACTOR: f64 = 1.0;
/// P3 默认值（未匹配节拍时）
const P3_DEFAULT: f64 = 50.0;

/// 规格族系数范围（用于归一化）
const FACTOR_MIN: f64 = 1.0;
const FACTOR_MAX: f64 = 1.6;

// ============================================
// 数据结构
// ============================================

/// P-Score 计算输入
///
/// # 字段来源
/// 所有字段均来自 `contract_master` 表，属于静态合同属性。
#[derive(Debug, Clone)]
pub struct PScoreInput {
    /// 钢种代码（P1 输入）
    /// 来源：contract_master.steel_grade
    pub steel_grade: String,

    /// 厚度 mm（P1 输入）
    /// 来源：contract_master.thickness
    pub thickness: f64,

    /// 宽度 mm（P1 输入）
    /// 来源：contract_master.width
    pub width: f64,

    /// 规格族代码（P2 输入）
    /// 来源：contract_master.spec_family
    pub spec_family: String,

    /// 距交期天数（P3 输入）
    /// 来源：contract_master.days_to_pdd
    /// 注意：此字段为合同属性，非实时计算
    pub days_to_pdd: i64,
}

/// P-Score 计算详情
///
/// 用于调试和前端展示各子评分的贡献。
#[derive(Debug, Clone)]
pub struct PScoreDetail {
    /// P1：工艺难度评分（0-100）
    pub p1_difficulty: f64,
    /// P2：聚合度评分（0-100）
    pub p2_aggregation: f64,
    /// P3：节拍匹配评分（0-100）
    pub p3_rhythm: f64,
    /// 最终 P-Score
    pub p_score: f64,
    /// 节拍匹配详情（用于 Explain 输出）
    pub rhythm_match: Option<RhythmMatchInfo>,
    /// P2 聚合度详情（用于 Explain 输出）
    pub p2_detail: Option<P2AggregationDetail>,
}

/// 节拍匹配信息
///
/// 用于展示节拍命中状态和详细信息
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct RhythmMatchInfo {
    /// 是否命中节拍标签
    pub is_hit: bool,
    /// 当前周期日（1 到 cycle_days）
    pub rhythm_day: i32,
    /// 当前激活的周期天数
    pub cycle_days: i32,
    /// 命中的标签名称（未命中时为 None）
    pub label_name: Option<String>,
    /// 匹配说明（用于 Explain 输出）
    pub description: String,
}

// ============================================
// P-Score 计算函数
// ============================================

/// 计算 P-Score（生产难度评分）- 兼容旧接口
///
/// # 注意
/// 此函数使用静态 factor 计算 P2，不支持基于合同池的聚合度计算。
/// 对于新功能，请使用 `calc_p_score_with_aggregation`。
#[allow(dead_code)]
pub fn calc_p_score(input: PScoreInput, config: &ScoringConfig) -> Result<f64, String> {
    calc_p_score_weighted(input, config, DEFAULT_W_P1, DEFAULT_W_P2, DEFAULT_W_P3)
}

/// 计算 P-Score（带聚合统计数据）
///
/// # 公式
/// ```text
/// P-Score = w_p1 × P1 + w_p2 × P2 + w_p3 × P3
/// ```
///
/// 其中 P2 使用新的聚合度计算：
/// ```text
/// P2 = α × 聚合度分数 + β × factor归一化分数
/// 聚合度分数 = min(100, log_scale × ln(count + 1))
/// ```
///
/// # 参数
/// - `input`: P-Score 计算所需的合同属性
/// - `config`: 评分配置（包含规格族系数）
/// - `aggregation_stats`: 预计算的聚合统计（可选，None 时退化为静态 factor）
/// - `curve_config`: P2 曲线配置（可选，None 时使用默认配置）
pub fn calc_p_score_with_aggregation(
    input: PScoreInput,
    config: &ScoringConfig,
    aggregation_stats: Option<&AggregationStats>,
    curve_config: Option<&P2CurveConfig>,
) -> Result<f64, String> {
    calc_p_score_with_aggregation_weighted(
        input,
        config,
        aggregation_stats,
        curve_config,
        DEFAULT_W_P1,
        DEFAULT_W_P2,
        DEFAULT_W_P3,
    )
}

/// 计算 P-Score（带自定义权重）
///
/// 允许调用者指定 P1/P2/P3 的权重，用于策略调优。
///
/// # 约束
/// `w_p1 + w_p2 + w_p3` 应等于 1.0（函数内不强制校验，由调用者保证）
#[allow(dead_code)]
pub fn calc_p_score_weighted(
    input: PScoreInput,
    config: &ScoringConfig,
    w_p1: f64,
    w_p2: f64,
    w_p3: f64,
) -> Result<f64, String> {
    let detail = calc_p_score_detail(input, config)?;

    // 加权求和
    let p_score = w_p1 * detail.p1_difficulty
        + w_p2 * detail.p2_aggregation
        + w_p3 * detail.p3_rhythm;

    // 归一化到 0-100 范围
    Ok(p_score.clamp(0.0, 100.0))
}

/// 计算 P-Score（带聚合统计 + 自定义权重）
pub fn calc_p_score_with_aggregation_weighted(
    input: PScoreInput,
    config: &ScoringConfig,
    aggregation_stats: Option<&AggregationStats>,
    curve_config: Option<&P2CurveConfig>,
    w_p1: f64,
    w_p2: f64,
    w_p3: f64,
) -> Result<f64, String> {
    let detail = calc_p_score_detail_with_aggregation(input, config, aggregation_stats, curve_config)?;

    // 加权求和
    let p_score = w_p1 * detail.p1_difficulty
        + w_p2 * detail.p2_aggregation
        + w_p3 * detail.p3_rhythm;

    // 归一化到 0-100 范围
    Ok(p_score.clamp(0.0, 100.0))
}

/// 计算 P-Score 详情（返回各子评分）- 兼容旧接口
///
/// 用于调试和前端展示。
#[allow(dead_code)]
pub fn calc_p_score_detail(input: PScoreInput, config: &ScoringConfig) -> Result<PScoreDetail, String> {
    calc_p_score_detail_with_aggregation(input, config, None, None)
}

/// 计算 P-Score 详情（带聚合统计）
///
/// # 参数
/// - `input`: P-Score 计算所需的合同属性
/// - `config`: 评分配置
/// - `aggregation_stats`: 预计算的聚合统计（可选）
/// - `curve_config`: P2 曲线配置（可选）
pub fn calc_p_score_detail_with_aggregation(
    input: PScoreInput,
    config: &ScoringConfig,
    aggregation_stats: Option<&AggregationStats>,
    curve_config: Option<&P2CurveConfig>,
) -> Result<PScoreDetail, String> {
    // ============================================
    // P1: 工艺难度评分
    // 来源：process_difficulty 表（静态配置）
    // ============================================
    let p1_difficulty = calc_p1_difficulty(&input.steel_grade, input.thickness, input.width)?;

    // ============================================
    // P2: 聚合度评分
    // 新逻辑：结合合同池统计 + 规格族系数
    // ============================================
    let (p2_aggregation, p2_detail) = calc_p2_aggregation_new(
        &input.spec_family,
        &input.steel_grade,
        config,
        aggregation_stats,
        curve_config,
    );

    // ============================================
    // P3: 节拍匹配评分（支持 n 日可配置周期）
    // 来源：rhythm_config + rhythm_label 表（静态配置）
    // ============================================
    let (p3_rhythm, rhythm_match) = calc_p3_rhythm_with_detail(input.days_to_pdd, &input.spec_family)?;

    // 使用默认权重计算最终分数
    let p_score = DEFAULT_W_P1 * p1_difficulty
        + DEFAULT_W_P2 * p2_aggregation
        + DEFAULT_W_P3 * p3_rhythm;

    Ok(PScoreDetail {
        p1_difficulty,
        p2_aggregation,
        p3_rhythm,
        p_score: p_score.clamp(0.0, 100.0),
        rhythm_match: Some(rhythm_match),
        p2_detail: Some(p2_detail),
    })
}

// ============================================
// P1: 工艺难度评分
// ============================================

/// 计算 P1：工艺难度评分
///
/// # 数据来源
/// `process_difficulty` 表（静态配置表）
///
/// # 匹配逻辑
/// 1. 精确匹配钢种
/// 2. 范围匹配厚度（thickness_min <= thickness <= thickness_max）
/// 3. 范围匹配宽度（width_min <= width <= width_max）
///
/// # 返回
/// - 匹配成功：返回 difficulty_score（0-100）
/// - 未匹配：返回默认值 50.0
fn calc_p1_difficulty(steel_grade: &str, thickness: f64, width: f64) -> Result<f64, String> {
    // 从配置表查询（静态数据，非实时）
    let score = db::get_process_difficulty_score(steel_grade, thickness, width)?;
    Ok(score)
}

// ============================================
// P2: 聚合度评分（新逻辑）
// ============================================

/// 计算 P2：聚合度评分（新逻辑）
///
/// # 新公式
/// ```text
/// P2 = α × 聚合度分数 + β × factor归一化分数
///
/// 其中：
/// - 聚合度分数 = min(max_score, log_scale × ln(count + 1))
/// - factor归一化分数 = (factor - 1.0) / (1.6 - 1.0) × 100
/// - α + β = 1.0
/// ```
///
/// # 参数
/// - `spec_family`: 规格族代码
/// - `steel_grade`: 钢种代码
/// - `config`: 评分配置
/// - `aggregation_stats`: 预计算的聚合统计（可选）
/// - `curve_config`: P2 曲线配置（可选）
///
/// # 返回
/// (P2 分数, P2 详情)
fn calc_p2_aggregation_new(
    spec_family: &str,
    steel_grade: &str,
    config: &ScoringConfig,
    aggregation_stats: Option<&AggregationStats>,
    curve_config: Option<&P2CurveConfig>,
) -> (f64, P2AggregationDetail) {
    // 获取曲线配置（使用传入的或默认值）
    let default_curve = P2CurveConfig::default();
    let curve = curve_config.unwrap_or(&default_curve);

    // 获取规格族系数
    let factor = config
        .spec_family_factors
        .get(spec_family)
        .copied()
        .unwrap_or_else(|| {
            config
                .spec_family_factors
                .get("default")
                .copied()
                .unwrap_or(P2_DEFAULT_FACTOR)
        });

    // 计算 factor 归一化分数
    let factor_score = ((factor - FACTOR_MIN) / (FACTOR_MAX - FACTOR_MIN) * 100.0).clamp(0.0, 100.0);

    // 计算聚合度分数
    let (aggregation_score, contract_count, aggregation_key, thickness_bin, width_bin) =
        if let Some(stats) = aggregation_stats {
            // 使用预计算的聚合统计
            let count = stats.contract_count;
            let score = calc_aggregation_score(count, curve);
            (
                score,
                count,
                stats.aggregation_key.clone(),
                format!("{} ({})", stats.thickness_bin_name, stats.thickness_bin_code),
                format!("{} ({})", stats.width_bin_name, stats.width_bin_code),
            )
        } else {
            // 无聚合统计时，使用 0 个同类合同（退化为纯 factor 模式）
            let score = calc_aggregation_score(1, curve);  // 至少有自己 1 个
            (
                score,
                1,
                format!("{}|{}|UNKNOWN|UNKNOWN", spec_family, steel_grade),
                "未知".to_string(),
                "未知".to_string(),
            )
        };

    // 加权组合
    let alpha = curve.alpha;
    let beta = curve.beta;
    let final_p2 = (alpha * aggregation_score + beta * factor_score).clamp(0.0, 100.0);

    // 构建公式说明
    let formula = format!(
        "P2 = {:.2}×{:.1}(聚合度:{}个同类) + {:.2}×{:.1}(factor:{:.2}) = {:.1}",
        alpha, aggregation_score, contract_count,
        beta, factor_score, factor,
        final_p2
    );

    let detail = P2AggregationDetail {
        aggregation_key,
        spec_family: spec_family.to_string(),
        steel_grade: steel_grade.to_string(),
        thickness_bin,
        width_bin,
        contract_count,
        aggregation_score,
        factor_value: factor,
        factor_score,
        alpha,
        beta,
        final_p2_score: final_p2,
        formula,
    };

    (final_p2, detail)
}

/// 计算聚合度分数（对数曲线）
///
/// # 公式
/// ```text
/// score = min(max_score, log_scale × ln(count + 1))
/// ```
///
/// # 效果示例（log_scale=25）
/// | count | ln(count+1) | score |
/// |-------|-------------|-------|
/// | 1     | 0.69        | 17    |
/// | 3     | 1.39        | 35    |
/// | 5     | 1.79        | 45    |
/// | 10    | 2.40        | 60    |
/// | 20    | 3.04        | 76    |
/// | 50    | 3.93        | 98    |
fn calc_aggregation_score(count: i64, curve: &P2CurveConfig) -> f64 {
    if count <= 0 {
        return curve.min_score;
    }

    match curve.curve_type.as_str() {
        "logarithmic" => {
            // 对数曲线：log_scale × ln(count + 1)
            let ln_value = ((count + 1) as f64).ln() / curve.log_base.ln();  // 转换为指定底数
            let score = curve.log_scale * ln_value;
            score.clamp(curve.min_score, curve.max_score)
        }
        "linear" => {
            // 线性：count / min_count_for_max × max_score
            let score = (count as f64 / curve.min_count_for_max as f64) * curve.max_score;
            score.clamp(curve.min_score, curve.max_score)
        }
        "step" => {
            // 阶梯：根据数量分档
            let score: f64 = match count {
                0..=2 => 20.0,
                3..=5 => 40.0,
                6..=10 => 60.0,
                11..=20 => 80.0,
                _ => 100.0,
            };
            score.clamp(curve.min_score, curve.max_score)
        }
        _ => {
            // 默认使用对数曲线
            let ln_value = ((count + 1) as f64).ln();
            let score = curve.log_scale * ln_value;
            score.clamp(curve.min_score, curve.max_score)
        }
    }
}

/// 计算 P2：聚合度评分（旧接口，向后兼容）
///
/// # 数据来源
/// `scoring_config` 表中的 spec_family_factors（静态配置）
///
/// # 计算逻辑
/// 1. 查找规格族对应的系数（factor）
/// 2. 归一化到 0-100 范围
///
/// # 归一化公式
/// ```text
/// P2 = (factor - FACTOR_MIN) / (FACTOR_MAX - FACTOR_MIN) × 100
/// ```
#[allow(dead_code)]
fn calc_p2_aggregation(spec_family: &str, config: &ScoringConfig) -> f64 {
    // 从配置获取规格族系数（静态数据）
    let factor = config
        .spec_family_factors
        .get(spec_family)
        .copied()
        .unwrap_or_else(|| {
            config
                .spec_family_factors
                .get("default")
                .copied()
                .unwrap_or(P2_DEFAULT_FACTOR)
        });

    // 归一化到 0-100
    let normalized = (factor - FACTOR_MIN) / (FACTOR_MAX - FACTOR_MIN) * 100.0;
    normalized.clamp(0.0, 100.0)
}

// ============================================
// P3: 节拍匹配评分（支持 n 日可配置周期）
// ============================================

/// 计算 P3：节拍匹配评分（带详细信息）
///
/// # 数据来源
/// - `rhythm_config` 表：获取当前激活的周期配置
/// - `rhythm_label` 表：查询匹配的节拍标签
///
/// # 计算逻辑
/// 1. 从 rhythm_config 获取激活配置的 cycle_days
/// 2. 计算周期日：`(days_to_pdd % cycle_days) + 1`
/// 3. 查询匹配的节拍标签
/// 4. 返回 bonus_score 和匹配详情
///
/// # n 日周期说明
/// 支持 1-30 天的可配置周期，计划人员可通过切换配置
/// 来表达"最近 n 天生产偏好"。
///
/// # 注意
/// days_to_pdd 是合同属性（来自 contract_master），
/// 不是实时计算的"当前日期到交期的天数"。
fn calc_p3_rhythm_with_detail(days_to_pdd: i64, spec_family: &str) -> Result<(f64, RhythmMatchInfo), String> {
    // 从数据库查询节拍匹配结果（支持 n 日周期）
    match db::get_rhythm_bonus_with_config(days_to_pdd, spec_family) {
        Ok((score, rhythm_day, cycle_days, label_name)) => {
            let is_hit = label_name.is_some();
            let description = if is_hit {
                format!(
                    "命中 {}日周期 第{}天: {} (得分: {:.1})",
                    cycle_days, rhythm_day, label_name.as_ref().unwrap(), score
                )
            } else {
                format!(
                    "未命中 {}日周期 第{}天 (默认分数: {:.1})",
                    cycle_days, rhythm_day, score
                )
            };

            Ok((score, RhythmMatchInfo {
                is_hit,
                rhythm_day,
                cycle_days,
                label_name,
                description,
            }))
        }
        Err(e) => {
            // 节拍配置未初始化时的后备逻辑
            let rhythm_day = ((days_to_pdd.rem_euclid(3)) + 1) as i32;
            Ok((P3_DEFAULT, RhythmMatchInfo {
                is_hit: false,
                rhythm_day,
                cycle_days: 3,
                label_name: None,
                description: format!("节拍配置未初始化: {} (默认分数: {:.1})", e, P3_DEFAULT),
            }))
        }
    }
}

/// 计算 P3：节拍匹配评分（简化接口，向后兼容）
///
/// # 数据来源
/// `rhythm_label` 表（静态配置表）
///
/// # 返回
/// - 匹配成功：返回 bonus_score
/// - 未匹配：返回默认值 50.0
#[allow(dead_code)]
fn calc_p3_rhythm(days_to_pdd: i64, spec_family: &str) -> Result<f64, String> {
    let (score, _) = calc_p3_rhythm_with_detail(days_to_pdd, spec_family)?;
    Ok(score)
}

// ============================================
// P-Score Explain 生成
// ============================================

/// 生成 P-Score 完整 Explain 结构
///
/// # 用途
/// 用于 Explain API，返回可序列化的 P-Score 拆分详情。
#[allow(dead_code)]
pub fn generate_p_score_explain(
    input: &PScoreInput,
    config: &ScoringConfig,
    w_p1: f64,
    w_p2: f64,
    w_p3: f64,
) -> Result<PScoreExplain, String> {
    generate_p_score_explain_with_aggregation(input, config, None, None, w_p1, w_p2, w_p3)
}

/// 生成 P-Score 完整 Explain 结构（带聚合统计）
pub fn generate_p_score_explain_with_aggregation(
    input: &PScoreInput,
    config: &ScoringConfig,
    aggregation_stats: Option<&AggregationStats>,
    curve_config: Option<&P2CurveConfig>,
    w_p1: f64,
    w_p2: f64,
    w_p3: f64,
) -> Result<PScoreExplain, String> {
    let detail = calc_p_score_detail_with_aggregation(input.clone(), config, aggregation_stats, curve_config)?;

    // P1: 工艺难度
    let p1_contribution = detail.p1_difficulty * w_p1;
    let p1_rule = format!(
        "钢种 '{}' + 厚度 {:.2}mm + 宽度 {:.0}mm → 工艺难度表查询 → {:.1}",
        input.steel_grade, input.thickness, input.width, detail.p1_difficulty
    );
    let p1_component = PScoreComponent {
        name: "P1 工艺难度".to_string(),
        input_value: format!(
            "钢种={}, 厚度={:.2}mm, 宽度={:.0}mm",
            input.steel_grade, input.thickness, input.width
        ),
        score: detail.p1_difficulty,
        weight: w_p1,
        contribution: p1_contribution,
        rule_description: p1_rule,
    };

    // P2: 聚合度（使用新的详情）
    let p2_contribution = detail.p2_aggregation * w_p2;
    let p2_rule = if let Some(ref p2_detail) = detail.p2_detail {
        p2_detail.formula.clone()
    } else {
        // 旧格式后备
        let factor = config
            .spec_family_factors
            .get(&input.spec_family)
            .copied()
            .unwrap_or(P2_DEFAULT_FACTOR);
        format!(
            "规格族 '{}' → 系数 {:.2} → 归一化 → {:.1}",
            input.spec_family, factor, detail.p2_aggregation
        )
    };
    let p2_input_value = if let Some(ref p2_detail) = detail.p2_detail {
        format!(
            "规格族={}, 钢种={}, 厚度段={}, 宽度段={}, 同类数量={}",
            p2_detail.spec_family, p2_detail.steel_grade,
            p2_detail.thickness_bin, p2_detail.width_bin,
            p2_detail.contract_count
        )
    } else {
        input.spec_family.clone()
    };
    let p2_component = PScoreComponent {
        name: "P2 聚合度".to_string(),
        input_value: p2_input_value,
        score: detail.p2_aggregation,
        weight: w_p2,
        contribution: p2_contribution,
        rule_description: p2_rule,
    };

    // P3: 节拍匹配
    let p3_contribution = detail.p3_rhythm * w_p3;
    let rhythm_info = detail.rhythm_match.as_ref();
    let p3_rule = if let Some(info) = rhythm_info {
        info.description.clone()
    } else {
        format!(
            "距交期 {} 天 → 节拍匹配 → {:.1}",
            input.days_to_pdd, detail.p3_rhythm
        )
    };
    let p3_component = PScoreComponent {
        name: "P3 节拍匹配".to_string(),
        input_value: format!("距交期 {} 天", input.days_to_pdd),
        score: detail.p3_rhythm,
        weight: w_p3,
        contribution: p3_contribution,
        rule_description: p3_rule,
    };

    // 验证：贡献之和是否等于总分
    let contribution_sum = p1_contribution + p2_contribution + p3_contribution;
    let verification_passed = (contribution_sum - detail.p_score).abs() < 0.001;

    Ok(PScoreExplain {
        p1_difficulty: p1_component,
        p2_aggregation: p2_component,
        p3_rhythm: p3_component,
        total_score: detail.p_score,
        verification_passed,
    })
}

// ============================================
// 单元测试
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    /// 创建测试用配置
    fn create_test_config() -> ScoringConfig {
        let mut spec_family_factors = HashMap::new();
        spec_family_factors.insert("常规".to_string(), 1.0);
        spec_family_factors.insert("特殊".to_string(), 1.2);
        spec_family_factors.insert("超特".to_string(), 1.5);
        spec_family_factors.insert("热成形".to_string(), 1.6);
        spec_family_factors.insert("default".to_string(), 1.0);

        ScoringConfig {
            spec_family_factors,
            ..Default::default()
        }
    }

    /// 测试 P2 归一化计算（旧接口）
    #[test]
    fn test_p2_normalization() {
        let config = create_test_config();

        // 常规（factor=1.0）→ P2=0
        let p2_regular = calc_p2_aggregation("常规", &config);
        assert!((p2_regular - 0.0).abs() < 0.01, "常规规格族 P2 应为 0");

        // 特殊（factor=1.2）→ P2=33.3
        let p2_special = calc_p2_aggregation("特殊", &config);
        assert!((p2_special - 33.33).abs() < 1.0, "特殊规格族 P2 应约为 33.3");

        // 超特（factor=1.5）→ P2=83.3
        let p2_ultra = calc_p2_aggregation("超特", &config);
        assert!((p2_ultra - 83.33).abs() < 1.0, "超特规格族 P2 应约为 83.3");

        // 热成形（factor=1.6）→ P2=100
        let p2_hot = calc_p2_aggregation("热成形", &config);
        assert!((p2_hot - 100.0).abs() < 0.01, "热成形规格族 P2 应为 100");

        // 未知规格族 → 使用默认值
        let p2_unknown = calc_p2_aggregation("未知规格", &config);
        assert!((p2_unknown - 0.0).abs() < 0.01, "未知规格族应使用默认值");
    }

    /// 测试聚合度分数对数曲线计算
    #[test]
    fn test_aggregation_score_logarithmic() {
        let curve = P2CurveConfig::default();  // log_scale=25

        // 测试不同数量的分数
        let score_1 = calc_aggregation_score(1, &curve);
        let score_5 = calc_aggregation_score(5, &curve);
        let score_10 = calc_aggregation_score(10, &curve);
        let score_50 = calc_aggregation_score(50, &curve);

        // 分数应该递增
        assert!(score_5 > score_1, "5个应该比1个分数高");
        assert!(score_10 > score_5, "10个应该比5个分数高");
        assert!(score_50 > score_10, "50个应该比10个分数高");

        // 边际递减效应
        let delta_1_to_5 = score_5 - score_1;
        let delta_10_to_50 = score_50 - score_10;
        assert!(delta_1_to_5 > delta_10_to_50 * 0.5, "前期增长应该更快");
    }

    /// 测试新 P2 计算（带聚合统计）
    #[test]
    fn test_p2_with_aggregation() {
        let config = create_test_config();
        let curve = P2CurveConfig::default();

        // 创建聚合统计
        let stats_high = AggregationStats {
            aggregation_key: "常规|DC01|REGULAR|STANDARD".to_string(),
            spec_family: "常规".to_string(),
            steel_grade: "DC01".to_string(),
            thickness_bin_code: "REGULAR".to_string(),
            thickness_bin_name: "常规厚度".to_string(),
            width_bin_code: "STANDARD".to_string(),
            width_bin_name: "标准幅宽".to_string(),
            contract_count: 30,  // 高聚合度
        };

        let stats_low = AggregationStats {
            aggregation_key: "常规|DC01|REGULAR|STANDARD".to_string(),
            spec_family: "常规".to_string(),
            steel_grade: "DC01".to_string(),
            thickness_bin_code: "REGULAR".to_string(),
            thickness_bin_name: "常规厚度".to_string(),
            width_bin_code: "STANDARD".to_string(),
            width_bin_name: "标准幅宽".to_string(),
            contract_count: 2,  // 低聚合度
        };

        let (p2_high, _) = calc_p2_aggregation_new("常规", "DC01", &config, Some(&stats_high), Some(&curve));
        let (p2_low, _) = calc_p2_aggregation_new("常规", "DC01", &config, Some(&stats_low), Some(&curve));

        // 高聚合度应该得分更高
        assert!(p2_high > p2_low, "高聚合度({})应该比低聚合度({})分数高", p2_high, p2_low);
    }

    /// 测试 P3 周期日映射
    #[test]
    fn test_p3_rhythm_day_mapping() {
        // days_to_pdd % 3 的映射
        assert_eq!(1_i64.rem_euclid(3), 1);  // Day 1
        assert_eq!(2_i64.rem_euclid(3), 2);  // Day 2
        assert_eq!(3_i64.rem_euclid(3), 0);  // Day 3
        assert_eq!(4_i64.rem_euclid(3), 1);  // Day 1
        assert_eq!(10_i64.rem_euclid(3), 1); // Day 1
        assert_eq!(0_i64.rem_euclid(3), 0);  // Day 3

        // 负数处理（rem_euclid 保证结果非负）
        assert_eq!((-1_i64).rem_euclid(3), 2);  // Day 2
        assert_eq!((-3_i64).rem_euclid(3), 0);  // Day 3
    }

    /// 测试权重约束
    #[test]
    fn test_weight_sum() {
        let sum = DEFAULT_W_P1 + DEFAULT_W_P2 + DEFAULT_W_P3;
        assert!(
            (sum - 1.0).abs() < 0.001,
            "默认权重之和应为 1.0，实际为 {}",
            sum
        );
    }

    /// 测试 P-Score 结果稳定性（无数据库依赖版本）
    #[test]
    fn test_p2_stability() {
        let config = create_test_config();

        // 同一输入多次计算，结果必须一致
        let p2_1 = calc_p2_aggregation("特殊", &config);
        let p2_2 = calc_p2_aggregation("特殊", &config);
        let p2_3 = calc_p2_aggregation("特殊", &config);

        assert_eq!(p2_1, p2_2, "P2 计算结果不稳定");
        assert_eq!(p2_2, p2_3, "P2 计算结果不稳定");
    }

    /// 测试 P-Score 边界值
    #[test]
    fn test_p2_boundaries() {
        let config = create_test_config();

        // P2 下界
        let p2_min = calc_p2_aggregation("常规", &config);
        assert!(p2_min >= 0.0, "P2 不应小于 0");

        // P2 上界
        let p2_max = calc_p2_aggregation("热成形", &config);
        assert!(p2_max <= 100.0, "P2 不应大于 100");
    }

    /// 验证静态规则：P2 计算不依赖时间
    #[test]
    fn test_p2_no_time_dependency() {
        let config = create_test_config();

        // 在不同"时刻"计算（模拟）
        let result_morning = calc_p2_aggregation("特殊", &config);
        let result_evening = calc_p2_aggregation("特殊", &config);

        assert_eq!(
            result_morning, result_evening,
            "P2 计算不应依赖当前时间"
        );
    }
}
