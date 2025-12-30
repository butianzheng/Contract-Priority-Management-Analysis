-- DPM v2.0 配置初始数据
-- Phase 1: 核心配置化

-- ============================================
-- 1. S-Score 配置参数
-- ============================================

-- 客户等级评分
INSERT INTO scoring_config (config_key, config_value, value_type, category, description, default_value, min_value, max_value)
VALUES
('customer_level_a_score', '100.0', 'number', 's_score', 'A级客户评分', '100.0', 0, 100),
('customer_level_b_score', '70.0', 'number', 's_score', 'B级客户评分', '70.0', 0, 100),
('customer_level_c_score', '40.0', 'number', 's_score', 'C级客户评分', '40.0', 0, 100),
('customer_level_default_score', '50.0', 'number', 's_score', '默认客户评分', '50.0', 0, 100);

-- 毛利评分参数
INSERT INTO scoring_config (config_key, config_value, value_type, category, description, default_value, min_value, max_value)
VALUES
('margin_min', '0.0', 'number', 's_score', '毛利最小值', '0.0', 0, NULL),
('margin_max', '1000.0', 'number', 's_score', '毛利最大值', '1000.0', 0, NULL),
('margin_conversion_factor', '10.0', 'number', 's_score', '毛利转换系数（毛利/系数=分数）', '10.0', 0.1, 100);

-- 紧急度评分参数
INSERT INTO scoring_config (config_key, config_value, value_type, category, description, default_value)
VALUES
('urgency_thresholds', '[0, 3, 7, 14, 30]', 'json_array', 's_score', '紧急度天数阈值（递增）', '[0, 3, 7, 14, 30]'),
('urgency_scores', '[100, 95, 80, 60, 40, 20]', 'json_array', 's_score', '紧急度对应分数（递减，比阈值数组多1个元素）', '[100, 95, 80, 60, 40, 20]');

-- ============================================
-- 2. P-Score 配置参数
-- ============================================

-- 规格族系数
INSERT INTO scoring_config (config_key, config_value, value_type, category, description, default_value, min_value, max_value)
VALUES
('spec_family_regular', '1.0', 'number', 'p_score', '常规规格系数', '1.0', 0.1, 5.0),
('spec_family_special', '1.2', 'number', 'p_score', '特殊规格系数', '1.2', 0.1, 5.0),
('spec_family_ultra', '1.5', 'number', 'p_score', '超特规格系数', '1.5', 0.1, 5.0),
('spec_family_default', '1.0', 'number', 'p_score', '默认规格系数', '1.0', 0.1, 5.0);

-- ============================================
-- 3. 通用配置参数
-- ============================================

-- Alpha 系数范围
INSERT INTO scoring_config (config_key, config_value, value_type, category, description, default_value, min_value, max_value)
VALUES
('alpha_min', '0.5', 'number', 'general', 'Alpha最小值', '0.5', 0.1, 1.0),
('alpha_max', '2.0', 'number', 'general', 'Alpha最大值', '2.0', 1.0, 10.0),
('alpha_default', '1.0', 'number', 'general', 'Alpha默认值', '1.0', 0.5, 2.0);

-- ============================================
-- 4. 策略级评分子权重配置
-- ============================================

-- 为每个策略配置 S-Score 的子权重
INSERT INTO strategy_scoring_weights (strategy_name, w1, w2, w3, description)
VALUES
('均衡', 0.4, 0.3, 0.3, '均衡考虑客户等级、毛利、紧急度'),
('客户优先', 0.5, 0.4, 0.1, '强化客户等级和毛利，弱化紧急度'),
('生产优先', 0.2, 0.2, 0.6, '强化紧急度，弱化客户等级和毛利');

-- ============================================
-- 验证数据
-- ============================================

-- 查询配置总数（应该有 16 条）
-- SELECT category, COUNT(*) as count FROM scoring_config GROUP BY category;

-- 查询策略权重（应该有 3 条）
-- SELECT * FROM strategy_scoring_weights;
