-- ============================================
-- Phase 15: 导入/清洗冲突解决机制产品化
-- ============================================
-- 功能：
-- 1. 导入审计日志（完整记录每次导入操作）
-- 2. 冲突记录表（逐条记录冲突及处理决策）
-- 3. 字段对齐规则（主数据字段映射）
-- 4. 重复检测配置（可配置的主键策略）
-- ============================================

-- 导入审计日志表
-- 记录每次导入操作的完整信息，用于责任追溯和复盘
CREATE TABLE IF NOT EXISTS import_audit_log (
    audit_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 操作信息
    import_type TEXT NOT NULL,              -- contracts, customers, process_difficulty
    file_name TEXT NOT NULL,                -- 原始文件名
    file_format TEXT NOT NULL,              -- csv, json, excel
    file_hash TEXT,                         -- 文件哈希值（用于重复导入检测）
    file_size INTEGER,                      -- 文件大小（字节）

    -- 统计信息
    total_rows INTEGER NOT NULL DEFAULT 0,      -- 文件总行数
    valid_rows INTEGER NOT NULL DEFAULT 0,      -- 有效行数
    error_rows INTEGER NOT NULL DEFAULT 0,      -- 错误行数
    conflict_rows INTEGER NOT NULL DEFAULT 0,   -- 冲突行数

    -- 处理结果
    imported_count INTEGER NOT NULL DEFAULT 0,  -- 实际导入数
    updated_count INTEGER NOT NULL DEFAULT 0,   -- 更新数（覆盖导入）
    skipped_count INTEGER NOT NULL DEFAULT 0,   -- 跳过数

    -- 冲突处理策略
    conflict_strategy TEXT NOT NULL DEFAULT 'skip',  -- skip, overwrite, manual

    -- 执行状态
    status TEXT NOT NULL DEFAULT 'pending',     -- pending, running, success, partial, failed
    error_message TEXT,                         -- 失败时的错误信息

    -- 关联的清洗规则
    applied_transform_rules TEXT,               -- JSON数组，记录应用的清洗规则ID

    -- 审计信息
    imported_by TEXT NOT NULL,                  -- 导入人
    started_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TEXT,

    -- 校验信息
    validation_errors TEXT,                     -- JSON数组，记录校验错误
    validation_warnings TEXT                    -- JSON数组，记录校验警告
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_import_audit_type ON import_audit_log(import_type);
CREATE INDEX IF NOT EXISTS idx_import_audit_status ON import_audit_log(status);
CREATE INDEX IF NOT EXISTS idx_import_audit_time ON import_audit_log(started_at);
CREATE INDEX IF NOT EXISTS idx_import_audit_user ON import_audit_log(imported_by);
CREATE INDEX IF NOT EXISTS idx_import_audit_hash ON import_audit_log(file_hash);

-- 导入冲突明细表
-- 记录每条冲突数据的详细信息和处理决策
CREATE TABLE IF NOT EXISTS import_conflict_log (
    conflict_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 关联的导入审计
    audit_id INTEGER NOT NULL,

    -- 冲突数据
    row_number INTEGER NOT NULL,            -- 文件中的行号
    primary_key TEXT NOT NULL,              -- 主键值（如 contract_id）

    -- 数据对比（JSON格式）
    existing_data TEXT NOT NULL,            -- 数据库中已有的数据
    new_data TEXT NOT NULL,                 -- 导入文件中的新数据

    -- 差异分析
    changed_fields TEXT,                    -- JSON数组，变更的字段列表
    field_diffs TEXT,                       -- JSON对象，字段级别的差异对比

    -- 处理决策
    action TEXT NOT NULL DEFAULT 'pending', -- pending, skip, overwrite
    action_reason TEXT,                     -- 决策原因
    decided_by TEXT,                        -- 决策人（manual 模式下）
    decided_at TEXT,                        -- 决策时间

    -- 时间戳
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (audit_id) REFERENCES import_audit_log(audit_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_conflict_audit ON import_conflict_log(audit_id);
CREATE INDEX IF NOT EXISTS idx_conflict_key ON import_conflict_log(primary_key);
CREATE INDEX IF NOT EXISTS idx_conflict_action ON import_conflict_log(action);

-- 字段对齐规则表
-- 定义不同数据源的字段名映射规则
CREATE TABLE IF NOT EXISTS field_alignment_rule (
    rule_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 规则基础信息
    rule_name TEXT NOT NULL UNIQUE,
    data_type TEXT NOT NULL,                -- contracts, customers, process_difficulty
    source_type TEXT,                       -- 数据来源类型（如 ERP, Excel模板, 第三方）
    description TEXT,
    enabled INTEGER NOT NULL DEFAULT 1,
    priority INTEGER NOT NULL DEFAULT 1,

    -- 字段映射（JSON格式）
    -- 格式: {"target_field": ["source_field1", "source_field2", ...]}
    -- 示例: {"contract_id": ["合同号", "合同编号", "Contract_ID", "ct_id"]}
    field_mapping TEXT NOT NULL,

    -- 值转换规则（JSON格式）
    -- 格式: {"field_name": {"type": "mapping|regex|formula", "config": {...}}}
    value_transform TEXT,

    -- 默认值填充（JSON格式）
    -- 格式: {"field_name": {"value": "default_value", "condition": "when_empty|when_missing"}}
    default_values TEXT,

    -- 审计信息
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_field_align_type ON field_alignment_rule(data_type);
CREATE INDEX IF NOT EXISTS idx_field_align_enabled ON field_alignment_rule(enabled);
CREATE INDEX IF NOT EXISTS idx_field_align_source ON field_alignment_rule(source_type);

-- 字段对齐规则变更日志
CREATE TABLE IF NOT EXISTS field_alignment_change_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id INTEGER NOT NULL,
    change_type TEXT NOT NULL,              -- create, update, delete, enable, disable
    old_value TEXT,                         -- JSON格式的旧配置
    new_value TEXT,                         -- JSON格式的新配置
    change_reason TEXT,
    changed_by TEXT NOT NULL,
    changed_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (rule_id) REFERENCES field_alignment_rule(rule_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_field_align_log_rule ON field_alignment_change_log(rule_id);
CREATE INDEX IF NOT EXISTS idx_field_align_log_time ON field_alignment_change_log(changed_at);

-- 重复检测配置表
-- 定义如何判断数据是否重复（支持复合主键）
CREATE TABLE IF NOT EXISTS duplicate_detection_config (
    config_id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- 配置基础信息
    config_name TEXT NOT NULL UNIQUE,
    data_type TEXT NOT NULL,                -- contracts, customers, process_difficulty
    description TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,   -- 同一 data_type 只能有一个激活配置

    -- 检测规则
    -- 主键字段列表（JSON数组）
    primary_key_fields TEXT NOT NULL,       -- ["contract_id"] 或 ["customer_id", "pdd"]

    -- 模糊匹配字段（用于相似性检测）
    fuzzy_match_fields TEXT,                -- JSON数组，如 ["customer_name"]
    fuzzy_threshold REAL DEFAULT 0.8,       -- 模糊匹配阈值 (0-1)

    -- 时间窗口检测
    time_field TEXT,                        -- 时间字段名（如 pdd）
    time_window_days INTEGER,               -- 时间窗口天数（同一客户N天内的订单视为可能重复）

    -- 业务规则检测
    business_rules TEXT,                    -- JSON数组，业务规则表达式

    -- 处理策略
    default_action TEXT NOT NULL DEFAULT 'warn',  -- warn, skip, merge, overwrite

    -- 审计信息
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_dup_detect_type ON duplicate_detection_config(data_type);
CREATE INDEX IF NOT EXISTS idx_dup_detect_active ON duplicate_detection_config(is_active);

-- 导入历史快照表
-- 保存导入前的数据快照，支持回滚
CREATE TABLE IF NOT EXISTS import_snapshot (
    snapshot_id INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_id INTEGER NOT NULL,
    data_type TEXT NOT NULL,
    primary_key TEXT NOT NULL,              -- 被影响的记录主键
    action_type TEXT NOT NULL,              -- insert, update, delete
    before_data TEXT,                       -- 操作前数据（JSON）
    after_data TEXT,                        -- 操作后数据（JSON）
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audit_id) REFERENCES import_audit_log(audit_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_snapshot_audit ON import_snapshot(audit_id);
CREATE INDEX IF NOT EXISTS idx_snapshot_key ON import_snapshot(primary_key);
CREATE INDEX IF NOT EXISTS idx_snapshot_type ON import_snapshot(data_type, action_type);

-- 相似记录关联表
-- 记录疑似重复的记录对
CREATE TABLE IF NOT EXISTS similar_record_pair (
    pair_id INTEGER PRIMARY KEY AUTOINCREMENT,
    audit_id INTEGER,                       -- 可选，关联的导入审计
    data_type TEXT NOT NULL,

    -- 记录对
    record_a_key TEXT NOT NULL,
    record_b_key TEXT NOT NULL,

    -- 相似度分析
    similarity_score REAL NOT NULL,         -- 综合相似度 (0-1)
    matching_fields TEXT,                   -- JSON对象，各字段匹配详情

    -- 处理状态
    status TEXT NOT NULL DEFAULT 'pending', -- pending, confirmed_same, confirmed_diff, merged, ignored
    resolved_by TEXT,
    resolved_at TEXT,
    resolution_note TEXT,

    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (audit_id) REFERENCES import_audit_log(audit_id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_similar_audit ON similar_record_pair(audit_id);
CREATE INDEX IF NOT EXISTS idx_similar_type ON similar_record_pair(data_type);
CREATE INDEX IF NOT EXISTS idx_similar_status ON similar_record_pair(status);
CREATE INDEX IF NOT EXISTS idx_similar_score ON similar_record_pair(similarity_score);

-- 导入统计视图
-- 汇总导入历史的统计信息
CREATE VIEW IF NOT EXISTS v_import_statistics AS
SELECT
    import_type,
    COUNT(*) as total_imports,
    SUM(CASE WHEN status = 'success' THEN 1 ELSE 0 END) as successful_imports,
    SUM(CASE WHEN status = 'failed' THEN 1 ELSE 0 END) as failed_imports,
    SUM(total_rows) as total_rows_processed,
    SUM(imported_count) as total_imported,
    SUM(updated_count) as total_updated,
    SUM(skipped_count) as total_skipped,
    SUM(conflict_rows) as total_conflicts,
    MAX(started_at) as last_import_time
FROM import_audit_log
GROUP BY import_type;

-- 待处理冲突视图
CREATE VIEW IF NOT EXISTS v_pending_conflicts AS
SELECT
    c.conflict_id,
    c.audit_id,
    a.import_type,
    a.file_name,
    c.row_number,
    c.primary_key,
    c.existing_data,
    c.new_data,
    c.changed_fields,
    c.action,
    a.imported_by,
    c.created_at
FROM import_conflict_log c
JOIN import_audit_log a ON c.audit_id = a.audit_id
WHERE c.action = 'pending'
ORDER BY c.created_at DESC;

-- ============================================
-- 插入默认配置
-- ============================================

-- 默认字段对齐规则（合同数据）
INSERT INTO field_alignment_rule (
    rule_name, data_type, source_type, description, enabled, priority,
    field_mapping, value_transform, default_values, created_by
) VALUES (
    '标准合同字段对齐', 'contracts', 'default', '标准合同导入的字段名映射规则', 1, 1,
    '{
        "contract_id": ["合同号", "合同编号", "Contract_ID", "ct_id", "合同ID"],
        "customer_id": ["客户号", "客户编号", "Customer_ID", "cust_id", "客户ID"],
        "steel_grade": ["钢种", "钢号", "Steel_Grade", "grade", "牌号"],
        "thickness": ["厚度", "Thickness", "thk", "板厚"],
        "width": ["宽度", "Width", "wid", "板宽"],
        "spec_family": ["规格族", "Spec_Family", "spec", "规格类型"],
        "pdd": ["交期", "PDD", "delivery_date", "交货日期", "计划交期"],
        "margin": ["毛利", "Margin", "profit", "利润率"]
    }',
    '{
        "steel_grade": {"type": "regex", "pattern": "^([A-Z]+)(\\d+)([A-Z]?)$", "replacement": "$1$2-$3"},
        "pdd": {"type": "date_format", "input_formats": ["YYYY-MM-DD", "YYYY/MM/DD", "DD-MM-YYYY"]}
    }',
    '{
        "spec_family": {"value": "常规", "condition": "when_empty"},
        "margin": {"value": "0", "condition": "when_missing"}
    }',
    'system'
),
(
    '标准客户字段对齐', 'customers', 'default', '标准客户导入的字段名映射规则', 1, 1,
    '{
        "customer_id": ["客户号", "客户编号", "Customer_ID", "cust_id", "客户ID"],
        "customer_name": ["客户名称", "客户名", "Customer_Name", "name", "公司名称"],
        "customer_level": ["客户等级", "等级", "Customer_Level", "level", "客户级别"],
        "credit_level": ["信用等级", "信用", "Credit_Level", "credit", "信用级别"],
        "customer_group": ["客户组", "客户分组", "Customer_Group", "group", "所属组"]
    }',
    '{
        "customer_level": {"type": "mapping", "values": {"VIP": "A", "重点": "A", "普通": "B", "一般": "C"}}
    }',
    '{
        "customer_level": {"value": "C", "condition": "when_empty"},
        "credit_level": {"value": "BBB", "condition": "when_missing"}
    }',
    'system'
);

-- 默认重复检测配置
INSERT INTO duplicate_detection_config (
    config_name, data_type, description, is_active,
    primary_key_fields, fuzzy_match_fields, fuzzy_threshold,
    time_field, time_window_days, default_action, created_by
) VALUES (
    '合同重复检测', 'contracts', '基于合同号检测重复，同时检测同客户短期内的相似订单', 1,
    '["contract_id"]',
    '["steel_grade", "thickness", "width"]',
    0.9,
    'pdd',
    7,
    'warn',
    'system'
),
(
    '客户重复检测', 'customers', '基于客户编号和名称检测重复客户', 1,
    '["customer_id"]',
    '["customer_name"]',
    0.85,
    NULL,
    NULL,
    'warn',
    'system'
);
