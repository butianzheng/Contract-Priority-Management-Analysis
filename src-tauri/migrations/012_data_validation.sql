-- ============================================
-- Phase 13: 数据校验与缺失值策略
--
-- 目标：
-- 1. 明确字段缺失/异常时如何处理
-- 2. 不允许"悄悄算错"
-- 3. 数据问题可见、可解释、可修复
-- ============================================

-- ============================================
-- 缺失值策略配置表
-- ============================================
CREATE TABLE IF NOT EXISTS missing_value_strategy (
    strategy_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 字段标识
    field_name TEXT NOT NULL UNIQUE,
    -- 字段中文名
    field_label TEXT NOT NULL,
    -- 所属模块：s_score, p_score, contract, customer
    module TEXT NOT NULL,
    -- 是否必填：1=必填, 0=可选
    is_required INTEGER NOT NULL DEFAULT 1,
    -- 缺失时策略：'default'=使用默认值, 'skip'=跳过该合同, 'error'=阻断计算
    strategy TEXT NOT NULL DEFAULT 'default',
    -- 默认值（JSON格式，支持数值、字符串等）
    default_value TEXT,
    -- 默认值说明
    default_description TEXT,
    -- 字段描述
    description TEXT,
    -- 影响的评分项
    affects_score TEXT,
    -- 排序
    sort_order INTEGER DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 预置校验字段配置
INSERT OR IGNORE INTO missing_value_strategy (field_name, field_label, module, is_required, strategy, default_value, default_description, affects_score, sort_order) VALUES
-- S-Score 必填字段
('customer_level', '客户等级', 's_score', 1, 'default', '"C"', '缺失时默认为C级客户（最低等级）', 'S1', 1),
('margin', '毛利率', 's_score', 0, 'default', '0.0', '缺失时默认为0（无毛利贡献）', 'S2', 2),
('days_to_pdd', '距交期天数', 's_score', 1, 'default', '30', '缺失时默认为30天（中等紧急度）', 'S3,P3', 3),

-- P-Score 必填字段
('steel_grade', '钢种', 'p_score', 1, 'default', '"UNKNOWN"', '缺失时标记为UNKNOWN，工艺难度使用默认分50', 'P1', 4),
('thickness', '厚度(mm)', 'p_score', 1, 'default', '2.0', '缺失时默认2.0mm（常规厚度）', 'P1,P2', 5),
('width', '宽度(mm)', 'p_score', 1, 'default', '1200', '缺失时默认1200mm（标准幅宽）', 'P1,P2', 6),
('spec_family', '规格族', 'p_score', 1, 'default', '"常规"', '缺失时默认为常规规格族', 'P2,P3', 7),

-- 合同基础字段
('contract_id', '合同编号', 'contract', 1, 'error', NULL, '合同编号为主键，不可缺失', NULL, 8),
('customer_id', '客户编号', 'contract', 1, 'default', '"UNKNOWN"', '缺失时标记为UNKNOWN客户', 'S1', 9),
('pdd', '计划交期', 'contract', 0, 'default', NULL, '可从days_to_pdd推算', 'S3,P3', 10);

-- ============================================
-- 数据校验日志表
-- ============================================
CREATE TABLE IF NOT EXISTS validation_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 校验批次ID
    batch_id TEXT NOT NULL,
    -- 校验时间
    validation_time TEXT DEFAULT (datetime('now','localtime')),
    -- 总合同数
    total_contracts INTEGER NOT NULL,
    -- 有效合同数（通过校验）
    valid_contracts INTEGER NOT NULL,
    -- 警告合同数（有缺失但可计算）
    warning_contracts INTEGER NOT NULL,
    -- 错误合同数（无法计算）
    error_contracts INTEGER NOT NULL,
    -- 校验详情（JSON）
    details_json TEXT,
    -- 校验人
    validated_by TEXT
);

-- ============================================
-- 合同校验问题明细表
-- ============================================
CREATE TABLE IF NOT EXISTS contract_validation_issues (
    issue_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 关联的校验批次
    batch_id TEXT NOT NULL,
    -- 合同编号
    contract_id TEXT NOT NULL,
    -- 字段名
    field_name TEXT NOT NULL,
    -- 问题类型：missing, invalid, out_of_range, format_error
    issue_type TEXT NOT NULL,
    -- 严重程度：error, warning, info
    severity TEXT NOT NULL DEFAULT 'warning',
    -- 原始值（如果有）
    original_value TEXT,
    -- 使用的默认值
    default_value_used TEXT,
    -- 问题描述
    message TEXT NOT NULL,
    -- 建议修复方案
    suggested_fix TEXT,
    -- 是否已修复
    is_resolved INTEGER DEFAULT 0,
    -- 记录时间
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_validation_log_batch ON validation_log(batch_id);
CREATE INDEX IF NOT EXISTS idx_validation_log_time ON validation_log(validation_time);
CREATE INDEX IF NOT EXISTS idx_contract_issues_batch ON contract_validation_issues(batch_id);
CREATE INDEX IF NOT EXISTS idx_contract_issues_contract ON contract_validation_issues(contract_id);
CREATE INDEX IF NOT EXISTS idx_contract_issues_severity ON contract_validation_issues(severity);

-- ============================================
-- 数据质量汇总视图
-- ============================================
CREATE VIEW IF NOT EXISTS v_data_quality_summary AS
SELECT
    field_name,
    field_label,
    module,
    is_required,
    strategy,
    default_value,
    COUNT(DISTINCT cvi.contract_id) as affected_contracts,
    SUM(CASE WHEN cvi.severity = 'error' THEN 1 ELSE 0 END) as error_count,
    SUM(CASE WHEN cvi.severity = 'warning' THEN 1 ELSE 0 END) as warning_count
FROM missing_value_strategy mvs
LEFT JOIN contract_validation_issues cvi ON mvs.field_name = cvi.field_name AND cvi.is_resolved = 0
GROUP BY mvs.field_name, mvs.field_label, mvs.module, mvs.is_required, mvs.strategy, mvs.default_value;
