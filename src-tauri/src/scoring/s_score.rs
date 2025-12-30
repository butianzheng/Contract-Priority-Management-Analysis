use crate::config::ScoringConfig;
use crate::db::schema::{SScoreComponent, SScoreExplain};

/// S-Score 计算输入
#[derive(Clone)]
pub struct SScoreInput {
    pub customer_level: String,  // A/B/C
    pub margin: f64,              // 毛利
    pub days_to_pdd: i64,         // 距离交期天数
}

/// S-Score 详细计算结果
#[derive(Debug, Clone)]
pub struct SScoreDetail {
    /// S1: 客户等级评分
    pub s1_score: f64,
    /// S2: 毛利评分
    pub s2_score: f64,
    /// S3: 紧急度评分
    pub s3_score: f64,
    /// S-Score 总分
    pub total_score: f64,
}

/// 计算 S-Score（战略价值评分）- 配置化版本
/// S = S1*w1 + S2*w2 + S3*w3
/// S1: 客户等级评分
/// S2: 毛利评分
/// S3: 紧急度评分
pub fn calc_s_score(input: SScoreInput, w1: f64, w2: f64, w3: f64, config: &ScoringConfig) -> f64 {
    // S1: 客户等级评分（从配置读取）
    let s1 = config
        .customer_level_scores
        .get(&input.customer_level)
        .copied()
        .unwrap_or_else(|| {
            config
                .customer_level_scores
                .get("default")
                .copied()
                .unwrap_or(50.0)
        });

    // S2: 毛利评分（从配置读取参数）
    let (margin_min, margin_max) = config.margin_range;
    let s2 = if input.margin < margin_min {
        0.0
    } else if input.margin > margin_max {
        100.0
    } else {
        input.margin / config.margin_factor
    };

    // S3: 紧急度评分（从配置读取阈值和分数）
    let s3 = calculate_urgency_score(
        input.days_to_pdd,
        &config.urgency_thresholds,
        &config.urgency_scores,
    );

    // 加权求和
    s1 * w1 + s2 * w2 + s3 * w3
}

/// 计算 S-Score 详情（返回各子评分）
///
/// # 用途
/// 用于 Explain API，返回 S1/S2/S3 的拆分贡献。
pub fn calc_s_score_detail(
    input: &SScoreInput,
    w1: f64,
    w2: f64,
    w3: f64,
    config: &ScoringConfig,
) -> SScoreDetail {
    // S1: 客户等级评分（从配置读取）
    let s1_score = config
        .customer_level_scores
        .get(&input.customer_level)
        .copied()
        .unwrap_or_else(|| {
            config
                .customer_level_scores
                .get("default")
                .copied()
                .unwrap_or(50.0)
        });

    // S2: 毛利评分（从配置读取参数）
    let (margin_min, margin_max) = config.margin_range;
    let s2_score = if input.margin < margin_min {
        0.0
    } else if input.margin > margin_max {
        100.0
    } else {
        input.margin / config.margin_factor
    };

    // S3: 紧急度评分（从配置读取阈值和分数）
    let s3_score = calculate_urgency_score(
        input.days_to_pdd,
        &config.urgency_thresholds,
        &config.urgency_scores,
    );

    // 加权求和
    let total_score = s1_score * w1 + s2_score * w2 + s3_score * w3;

    SScoreDetail {
        s1_score,
        s2_score,
        s3_score,
        total_score,
    }
}

/// 生成 S-Score 完整 Explain 结构
///
/// # 用途
/// 用于 Explain API，返回可序列化的 S-Score 拆分详情。
pub fn generate_s_score_explain(
    input: &SScoreInput,
    w1: f64,
    w2: f64,
    w3: f64,
    config: &ScoringConfig,
) -> SScoreExplain {
    let detail = calc_s_score_detail(input, w1, w2, w3, config);

    // S1: 客户等级
    let s1_contribution = detail.s1_score * w1;
    let s1_rule = format!(
        "客户等级 '{}' → 配置表映射得分 {:.1}",
        input.customer_level, detail.s1_score
    );
    let s1_component = SScoreComponent {
        name: "S1 客户等级".to_string(),
        input_value: input.customer_level.clone(),
        score: detail.s1_score,
        weight: w1,
        contribution: s1_contribution,
        rule_description: s1_rule,
    };

    // S2: 毛利
    let s2_contribution = detail.s2_score * w2;
    let (margin_min, margin_max) = config.margin_range;
    let s2_rule = format!(
        "毛利 {:.2} → 归一化 (范围 {}-{}, 因子 {}) → {:.1}",
        input.margin, margin_min, margin_max, config.margin_factor, detail.s2_score
    );
    let s2_component = SScoreComponent {
        name: "S2 毛利".to_string(),
        input_value: format!("{:.2}", input.margin),
        score: detail.s2_score,
        weight: w2,
        contribution: s2_contribution,
        rule_description: s2_rule,
    };

    // S3: 紧急度
    let s3_contribution = detail.s3_score * w3;
    let s3_rule = format!(
        "距交期 {} 天 → 阈值分段 {:?} → {:.1}",
        input.days_to_pdd, config.urgency_thresholds, detail.s3_score
    );
    let s3_component = SScoreComponent {
        name: "S3 紧急度".to_string(),
        input_value: format!("{} 天", input.days_to_pdd),
        score: detail.s3_score,
        weight: w3,
        contribution: s3_contribution,
        rule_description: s3_rule,
    };

    // 验证：贡献之和是否等于总分
    let contribution_sum = s1_contribution + s2_contribution + s3_contribution;
    let verification_passed = (contribution_sum - detail.total_score).abs() < 0.001;

    SScoreExplain {
        s1_customer_level: s1_component,
        s2_margin: s2_component,
        s3_urgency: s3_component,
        total_score: detail.total_score,
        verification_passed,
    }
}

/// 根据配置计算紧急度评分
fn calculate_urgency_score(days: i64, thresholds: &[i64], scores: &[f64]) -> f64 {
    // 遍历阈值，找到对应的分数
    for (i, &threshold) in thresholds.iter().enumerate() {
        if days <= threshold {
            return scores.get(i).copied().unwrap_or(0.0);
        }
    }
    // 如果超过所有阈值，返回最后一个分数
    scores.last().copied().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s_score_a_customer() {
        let config = ScoringConfig::default();
        let input = SScoreInput {
            customer_level: "A".to_string(),
            margin: 500.0,
            days_to_pdd: 5,
        };
        let score = calc_s_score(input, 0.4, 0.3, 0.3, &config);
        assert!(score > 50.0);
    }

    #[test]
    fn test_s_score_urgent() {
        let config = ScoringConfig::default();
        let input = SScoreInput {
            customer_level: "B".to_string(),
            margin: 300.0,
            days_to_pdd: 1,
        };
        let score = calc_s_score(input, 0.3, 0.3, 0.4, &config);
        // 紧急订单应该得分较高
        assert!(score > 60.0);
    }

    #[test]
    fn test_calculate_urgency_score() {
        let thresholds = vec![0, 3, 7, 14, 30];
        let scores = vec![100.0, 95.0, 80.0, 60.0, 40.0, 20.0];

        assert_eq!(calculate_urgency_score(0, &thresholds, &scores), 100.0);
        assert_eq!(calculate_urgency_score(2, &thresholds, &scores), 95.0);
        assert_eq!(calculate_urgency_score(5, &thresholds, &scores), 80.0);
        assert_eq!(calculate_urgency_score(10, &thresholds, &scores), 60.0);
        assert_eq!(calculate_urgency_score(50, &thresholds, &scores), 20.0);
    }
}

