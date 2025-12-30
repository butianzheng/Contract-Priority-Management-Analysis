use super::types::{ScoringConfig, StrategyScoringWeights};
use crate::db;
use std::collections::HashMap;

/// 从数据库加载评分配置
pub fn load_scoring_config() -> Result<ScoringConfig, String> {
    // 获取所有配置
    let configs = db::get_all_scoring_configs()?;

    // 解析客户等级评分
    let mut customer_level_scores = HashMap::new();
    customer_level_scores.insert(
        "A".to_string(),
        configs
            .get("customer_level_a_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(100.0),
    );
    customer_level_scores.insert(
        "B".to_string(),
        configs
            .get("customer_level_b_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(70.0),
    );
    customer_level_scores.insert(
        "C".to_string(),
        configs
            .get("customer_level_c_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(40.0),
    );
    customer_level_scores.insert(
        "default".to_string(),
        configs
            .get("customer_level_default_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(50.0),
    );

    // 解析毛利参数
    let margin_min = configs
        .get("margin_min")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let margin_max = configs
        .get("margin_max")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1000.0);
    let margin_factor = configs
        .get("margin_conversion_factor")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    // 解析紧急度阈值和分数（JSON数组）
    let urgency_thresholds = configs
        .get("urgency_thresholds")
        .and_then(|v| parse_json_array_i64(v).ok())
        .unwrap_or_else(|| vec![0, 3, 7, 14, 30]);

    let urgency_scores = configs
        .get("urgency_scores")
        .and_then(|v| parse_json_array_f64(v).ok())
        .unwrap_or_else(|| vec![100.0, 95.0, 80.0, 60.0, 40.0, 20.0]);

    // 解析规格族系数
    let mut spec_family_factors = HashMap::new();
    spec_family_factors.insert(
        "常规".to_string(),
        configs
            .get("spec_family_regular")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0),
    );
    spec_family_factors.insert(
        "特殊".to_string(),
        configs
            .get("spec_family_special")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.2),
    );
    spec_family_factors.insert(
        "超特".to_string(),
        configs
            .get("spec_family_ultra")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.5),
    );
    spec_family_factors.insert(
        "default".to_string(),
        configs
            .get("spec_family_default")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0),
    );

    // 解析 Alpha 范围
    let alpha_min = configs
        .get("alpha_min")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5);
    let alpha_max = configs
        .get("alpha_max")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(2.0);
    let alpha_default = configs
        .get("alpha_default")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);

    Ok(ScoringConfig {
        customer_level_scores,
        margin_range: (margin_min, margin_max),
        margin_factor,
        urgency_thresholds,
        urgency_scores,
        spec_family_factors,
        alpha_range: (alpha_min, alpha_max),
        alpha_default,
    })
}

/// 从数据库加载策略的评分子权重
pub fn load_strategy_scoring_weights(strategy_name: &str) -> Result<StrategyScoringWeights, String> {
    db::get_strategy_scoring_weights(strategy_name)
}

/// 辅助函数：解析 JSON 数组为 Vec<i64>
fn parse_json_array_i64(json_str: &str) -> Result<Vec<i64>, String> {
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| format!("解析JSON数组失败: {}", e))?;

    let result: Result<Vec<i64>, _> = arr
        .iter()
        .map(|v| {
            v.as_i64()
                .or_else(|| v.as_f64().map(|f| f as i64))
                .ok_or_else(|| "数组元素不是数字".to_string())
        })
        .collect();

    result
}

/// 辅助函数：解析 JSON 数组为 Vec<f64>
fn parse_json_array_f64(json_str: &str) -> Result<Vec<f64>, String> {
    let arr: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|e| format!("解析JSON数组失败: {}", e))?;

    let result: Result<Vec<f64>, _> = arr
        .iter()
        .map(|v| {
            v.as_f64()
                .ok_or_else(|| "数组元素不是数字".to_string())
        })
        .collect();

    result
}

/// 从 HashMap 构建 ScoringConfig
///
/// # 用途
/// 用于从版本快照中恢复 ScoringConfig，实现历史版本的可复现计算。
pub fn build_scoring_config_from_map(configs: &HashMap<String, String>) -> Result<ScoringConfig, String> {
    // 解析客户等级评分
    let mut customer_level_scores = HashMap::new();
    customer_level_scores.insert(
        "A".to_string(),
        configs
            .get("customer_level_a_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(100.0),
    );
    customer_level_scores.insert(
        "B".to_string(),
        configs
            .get("customer_level_b_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(70.0),
    );
    customer_level_scores.insert(
        "C".to_string(),
        configs
            .get("customer_level_c_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(40.0),
    );
    customer_level_scores.insert(
        "default".to_string(),
        configs
            .get("customer_level_default_score")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(50.0),
    );

    // 解析毛利参数
    let margin_min = configs
        .get("margin_min")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.0);
    let margin_max = configs
        .get("margin_max")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1000.0);
    let margin_factor = configs
        .get("margin_conversion_factor")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(10.0);

    // 解析紧急度阈值和分数（JSON数组）
    let urgency_thresholds = configs
        .get("urgency_thresholds")
        .and_then(|v| parse_json_array_i64(v).ok())
        .unwrap_or_else(|| vec![0, 3, 7, 14, 30]);

    let urgency_scores = configs
        .get("urgency_scores")
        .and_then(|v| parse_json_array_f64(v).ok())
        .unwrap_or_else(|| vec![100.0, 95.0, 80.0, 60.0, 40.0, 20.0]);

    // 解析规格族系数
    let mut spec_family_factors = HashMap::new();
    spec_family_factors.insert(
        "常规".to_string(),
        configs
            .get("spec_family_regular")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0),
    );
    spec_family_factors.insert(
        "特殊".to_string(),
        configs
            .get("spec_family_special")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.2),
    );
    spec_family_factors.insert(
        "超特".to_string(),
        configs
            .get("spec_family_ultra")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.5),
    );
    spec_family_factors.insert(
        "default".to_string(),
        configs
            .get("spec_family_default")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(1.0),
    );

    // 解析 Alpha 范围
    let alpha_min = configs
        .get("alpha_min")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(0.5);
    let alpha_max = configs
        .get("alpha_max")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(2.0);
    let alpha_default = configs
        .get("alpha_default")
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(1.0);

    Ok(ScoringConfig {
        customer_level_scores,
        margin_range: (margin_min, margin_max),
        margin_factor,
        urgency_thresholds,
        urgency_scores,
        spec_family_factors,
        alpha_range: (alpha_min, alpha_max),
        alpha_default,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_array_i64() {
        let json = "[0, 3, 7, 14, 30]";
        let result = parse_json_array_i64(json).unwrap();
        assert_eq!(result, vec![0, 3, 7, 14, 30]);
    }

    #[test]
    fn test_parse_json_array_f64() {
        let json = "[100, 95, 80, 60, 40, 20]";
        let result = parse_json_array_f64(json).unwrap();
        assert_eq!(result, vec![100.0, 95.0, 80.0, 60.0, 40.0, 20.0]);
    }
}
