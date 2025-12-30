/// 计算综合优先级
///
/// # 公式
/// ```text
/// Base Priority = ws × S-Score + wp × P-Score
/// ```
///
/// # 参数
/// - `s_score`: S-Score（战略价值评分）
/// - `p_score`: P-Score（生产难度评分）
/// - `ws`: S-Score 权重（来自策略配置）
/// - `wp`: P-Score 权重（来自策略配置）
///
/// # 返回
/// - 基础优先级值（未应用 Alpha 调整）
pub fn calc_priority(s_score: f64, p_score: f64, ws: f64, wp: f64) -> f64 {
    ws * s_score + wp * p_score
}

/// 应用人工调整系数 Alpha
///
/// # 公式
/// ```text
/// Final Priority = Base Priority × Alpha
/// ```
///
/// # 参数
/// - `priority`: 基础优先级（ws × S + wp × P）
/// - `alpha`: 人工调整系数
///   - 范围：[0.5, 2.0]
///   - 默认值：1.0（无调整）
///   - alpha < 1.0：降低优先级
///   - alpha = 1.0：保持不变
///   - alpha > 1.0：提升优先级
///
/// # 返回
/// - 调整后的最终优先级
///
/// # 验收标准
/// - Alpha = 1.0 时，返回值等于输入的 priority（无变化）
/// - Alpha = 2.0 时，返回值为 priority × 2
/// - Alpha = 0.5 时，返回值为 priority × 0.5
pub fn apply_alpha(priority: f64, alpha: f64) -> f64 {
    priority * alpha
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_priority() {
        // Base Priority = ws × S + wp × P
        let s = 80.0;
        let p = 60.0;
        let ws = 0.6;
        let wp = 0.4;

        let priority = calc_priority(s, p, ws, wp);
        assert_eq!(priority, 72.0); // 80×0.6 + 60×0.4 = 48 + 24 = 72
    }

    #[test]
    fn test_apply_alpha_increase() {
        // Final Priority = Base Priority × Alpha
        let priority = 70.0;
        let alpha = 1.2; // 提升 20%
        let adjusted = apply_alpha(priority, alpha);
        assert_eq!(adjusted, 84.0); // 70 × 1.2 = 84
    }

    #[test]
    fn test_apply_alpha_decrease() {
        // Final Priority = Base Priority × Alpha
        let priority = 80.0;
        let alpha = 0.8; // 降低 20%
        let adjusted = apply_alpha(priority, alpha);
        assert_eq!(adjusted, 64.0); // 80 × 0.8 = 64
    }

    /// 验收标准：Alpha = 1.0 时结果不变
    #[test]
    fn test_apply_alpha_no_change() {
        let priority = 75.0;
        let alpha = 1.0; // 默认值，无调整
        let adjusted = apply_alpha(priority, alpha);
        assert_eq!(adjusted, priority); // 结果必须等于原值
    }

    /// 验收标准：Alpha 边界值测试
    #[test]
    fn test_apply_alpha_boundaries() {
        let priority = 100.0;

        // Alpha 最小值 0.5：优先级降低 50%
        assert_eq!(apply_alpha(priority, 0.5), 50.0);

        // Alpha 最大值 2.0：优先级翻倍
        assert_eq!(apply_alpha(priority, 2.0), 200.0);
    }

    /// 验收标准：Alpha = 1.0 时对任意优先级值都无变化
    #[test]
    fn test_apply_alpha_identity() {
        // 测试多个不同的优先级值
        let test_values = [0.0, 50.0, 75.5, 100.0, 150.0];
        for priority in test_values {
            let adjusted = apply_alpha(priority, 1.0);
            assert_eq!(adjusted, priority, "Alpha=1.0 应对 priority={} 无变化", priority);
        }
    }
}
