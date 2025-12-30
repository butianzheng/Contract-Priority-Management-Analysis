-- ============================================
-- Phase 8: 清洗规则管理表
-- ============================================

-- 清洗规则表
CREATE TABLE IF NOT EXISTS transform_rules (
    rule_id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,           -- standardization, extraction, normalization, mapping
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,  -- 0=禁用, 1=启用
    priority INTEGER NOT NULL DEFAULT 1, -- 同类规则中的执行优先级
    config_json TEXT NOT NULL,        -- JSON格式存储规则配置
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_transform_rules_category ON transform_rules(category);
CREATE INDEX IF NOT EXISTS idx_transform_rules_enabled ON transform_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_transform_rules_priority ON transform_rules(category, priority);

-- 清洗规则执行日志表（记录每次执行的结果）
CREATE TABLE IF NOT EXISTS transform_execution_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    execution_time TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    records_processed INTEGER NOT NULL DEFAULT 0,
    records_modified INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL,              -- success, failed, partial
    error_message TEXT,
    executed_by TEXT NOT NULL,
    FOREIGN KEY (rule_id) REFERENCES transform_rules(rule_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_transform_exec_rule ON transform_execution_log(rule_id);
CREATE INDEX IF NOT EXISTS idx_transform_exec_time ON transform_execution_log(execution_time);

-- 清洗规则变更日志表
CREATE TABLE IF NOT EXISTS transform_rule_change_log (
    change_id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    change_type TEXT NOT NULL,         -- create, update, delete, enable, disable
    old_value TEXT,                    -- JSON格式的旧配置
    new_value TEXT,                    -- JSON格式的新配置
    change_reason TEXT,
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES transform_rules(rule_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_transform_change_rule ON transform_rule_change_log(rule_id);
CREATE INDEX IF NOT EXISTS idx_transform_change_time ON transform_rule_change_log(changed_at);

-- ============================================
-- 插入默认清洗规则
-- ============================================

-- 1. 字段标准化规则
INSERT INTO transform_rules (rule_name, category, description, enabled, priority, config_json, created_by)
VALUES
('钢种名称标准化', 'standardization', '将不同格式的钢种名称统一为标准格式（如 Q235B -> Q235-B）', 1, 1,
 '{"type": "regex_replace", "field": "steel_grade", "pattern": "^([A-Z]+)(\\d+)([A-Z]?)$", "replacement": "$1$2-$3", "trim_dash": true}', 'system'),

('客户编号格式化', 'standardization', '统一客户编号为 CUST000 格式', 1, 2,
 '{"type": "format_id", "field": "customer_id", "prefix": "CUST", "digits": 3, "pad_char": "0"}', 'system'),

('合同编号格式化', 'standardization', '统一合同编号格式', 1, 3,
 '{"type": "format_id", "field": "contract_id", "prefix": "CT", "digits": 4, "pad_char": "0"}', 'system');

-- 2. 规格段提取规则
INSERT INTO transform_rules (rule_name, category, description, enabled, priority, config_json, created_by)
VALUES
('厚度规格段提取', 'extraction', '根据厚度值提取规格段分类（薄规格/中规格/厚规格）', 1, 1,
 '{"type": "range_classify", "source_field": "thickness", "target_field": "thickness_class", "ranges": [{"min": 0, "max": 0.8, "label": "薄规格"}, {"min": 0.8, "max": 1.5, "label": "中规格"}, {"min": 1.5, "max": 999, "label": "厚规格"}]}', 'system'),

('宽度规格段提取', 'extraction', '根据宽度值提取规格段分类（窄幅/中幅/宽幅）', 1, 2,
 '{"type": "range_classify", "source_field": "width", "target_field": "width_class", "ranges": [{"min": 0, "max": 1000, "label": "窄幅"}, {"min": 1000, "max": 1500, "label": "中幅"}, {"min": 1500, "max": 9999, "label": "宽幅"}]}', 'system'),

('规格族自动分类', 'extraction', '根据钢种和规格自动计算规格族', 1, 3,
 '{"type": "spec_family_classify", "steel_grade_field": "steel_grade", "thickness_field": "thickness", "width_field": "width", "target_field": "spec_family", "special_grades": ["304", "316L", "310S", "Q550", "Q690"], "ultra_special_grades": ["Inconel", "Hastelloy"]}', 'system');

-- 3. 等级归一化规则
INSERT INTO transform_rules (rule_name, category, description, enabled, priority, config_json, created_by)
VALUES
('客户等级归一化', 'normalization', '将客户等级转换为数值分数（A=1.0, B=0.7, C=0.4）', 1, 1,
 '{"type": "value_mapping", "source_field": "customer_level", "target_field": "customer_score", "mapping": {"A": 1.0, "B": 0.7, "C": 0.4, "D": 0.1}, "default": 0.5}', 'system'),

('信用等级归一化', 'normalization', '将信用等级转换为数值分数', 1, 2,
 '{"type": "value_mapping", "source_field": "credit_level", "target_field": "credit_score", "mapping": {"AAA": 1.0, "AA": 0.8, "A": 0.6, "BBB": 0.4, "BB": 0.2, "B": 0.1}, "default": 0.5}', 'system'),

('毛利归一化', 'normalization', '将毛利值归一化到0-100范围', 1, 3,
 '{"type": "range_normalize", "source_field": "margin", "target_field": "margin_score", "min_value": 0, "max_value": 1000, "output_min": 0, "output_max": 100}', 'system');

-- 4. 标签映射规则
INSERT INTO transform_rules (rule_name, category, description, enabled, priority, config_json, created_by)
VALUES
('节奏日标签映射', 'mapping', '根据剩余天数映射节奏日标签（D+1/D+2/D+3）', 1, 1,
 '{"type": "condition_mapping", "source_field": "days_to_pdd", "target_field": "rhythm_label", "conditions": [{"operator": "<=", "value": 1, "label": "D+1"}, {"operator": "<=", "value": 2, "label": "D+2"}, {"operator": "<=", "value": 3, "label": "D+3"}], "default": "D+N"}', 'system'),

('紧急度标签映射', 'mapping', '根据剩余天数映射紧急度标签', 0, 2,
 '{"type": "condition_mapping", "source_field": "days_to_pdd", "target_field": "urgency_label", "conditions": [{"operator": "<=", "value": 3, "label": "紧急"}, {"operator": "<=", "value": 7, "label": "优先"}, {"operator": "<=", "value": 14, "label": "关注"}], "default": "正常"}', 'system'),

('优先级等级映射', 'mapping', '将计算出的优先级分数映射为等级标签', 1, 3,
 '{"type": "condition_mapping", "source_field": "priority", "target_field": "priority_level", "conditions": [{"operator": ">=", "value": 80, "label": "高"}, {"operator": ">=", "value": 60, "label": "中"}, {"operator": ">=", "value": 40, "label": "低"}], "default": "待定"}', 'system');
