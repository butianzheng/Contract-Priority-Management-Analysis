use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 配置值枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Number(f64),
    String(String),
    JsonArray(Vec<serde_json::Value>),
    JsonObject(serde_json::Map<String, serde_json::Value>),
}

impl ConfigValue {
    /// 转换为 f64
    #[allow(dead_code)]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ConfigValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 转换为 String
    #[allow(dead_code)]
    pub fn as_string(&self) -> Option<String> {
        match self {
            ConfigValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    /// 转换为数组
    #[allow(dead_code)]
    pub fn as_array(&self) -> Option<&Vec<serde_json::Value>> {
        match self {
            ConfigValue::JsonArray(arr) => Some(arr),
            _ => None,
        }
    }
}

/// 评分配置集合（预加载）
#[derive(Debug, Clone)]
pub struct ScoringConfig {
    // S-Score 配置
    pub customer_level_scores: HashMap<String, f64>,
    pub margin_range: (f64, f64),
    pub margin_factor: f64,
    pub urgency_thresholds: Vec<i64>,
    pub urgency_scores: Vec<f64>,

    // P-Score 配置
    pub spec_family_factors: HashMap<String, f64>,

    // 通用配置
    #[allow(dead_code)]
    pub alpha_range: (f64, f64),
    #[allow(dead_code)]
    pub alpha_default: f64,
}

impl Default for ScoringConfig {
    /// 创建默认配置（硬编码值，作为后备）
    fn default() -> Self {
        let mut customer_level_scores = HashMap::new();
        customer_level_scores.insert("A".to_string(), 100.0);
        customer_level_scores.insert("B".to_string(), 70.0);
        customer_level_scores.insert("C".to_string(), 40.0);
        customer_level_scores.insert("default".to_string(), 50.0);

        let mut spec_family_factors = HashMap::new();
        spec_family_factors.insert("常规".to_string(), 1.0);
        spec_family_factors.insert("特殊".to_string(), 1.2);
        spec_family_factors.insert("超特".to_string(), 1.5);
        spec_family_factors.insert("default".to_string(), 1.0);

        ScoringConfig {
            customer_level_scores,
            margin_range: (0.0, 1000.0),
            margin_factor: 10.0,
            urgency_thresholds: vec![0, 3, 7, 14, 30],
            urgency_scores: vec![100.0, 95.0, 80.0, 60.0, 40.0, 20.0],
            spec_family_factors,
            alpha_range: (0.5, 2.0),
            alpha_default: 1.0,
        }
    }
}

/// 策略评分权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyScoringWeights {
    pub strategy_name: String,
    pub w1: f64,  // 客户等级权重
    pub w2: f64,  // 毛利权重
    pub w3: f64,  // 紧急度权重
}

impl Default for StrategyScoringWeights {
    fn default() -> Self {
        StrategyScoringWeights {
            strategy_name: "default".to_string(),
            w1: 0.4,
            w2: 0.3,
            w3: 0.3,
        }
    }
}
