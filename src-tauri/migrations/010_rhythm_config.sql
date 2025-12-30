-- ============================================
-- 010_rhythm_config.sql
-- n日节拍可配置方案
-- ============================================

-- 1. 创建节拍配置表
-- 支持多套节拍配置，但只有一个处于激活状态
CREATE TABLE IF NOT EXISTS rhythm_config (
    config_id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_name TEXT NOT NULL UNIQUE,
    cycle_days INTEGER NOT NULL CHECK(cycle_days >= 1 AND cycle_days <= 30),
    description TEXT,
    is_active INTEGER DEFAULT 0,  -- 0=禁用, 1=激活（系统中只能有一个激活配置）
    created_by TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 2. 创建新的节拍标签表（移除固定3日约束）
-- 注意：SQLite 不支持 ALTER TABLE DROP CONSTRAINT，需要重建表
CREATE TABLE IF NOT EXISTS rhythm_label_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    rhythm_day INTEGER NOT NULL CHECK(rhythm_day >= 1),  -- 周期日，从1开始
    label_name TEXT NOT NULL,
    match_spec TEXT,  -- 匹配的规格族（支持通配符 * 或逗号分隔的列表）
    bonus_score REAL DEFAULT 0.0,  -- P3 加分（0-100）
    description TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (config_id) REFERENCES rhythm_config(config_id)
);

-- 3. 创建节拍配置变更日志表
CREATE TABLE IF NOT EXISTS rhythm_config_change_log (
    change_id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_id INTEGER NOT NULL,
    config_name TEXT,
    change_type TEXT NOT NULL,  -- create, update, delete, activate, deactivate
    old_value TEXT,             -- JSON 格式
    new_value TEXT,             -- JSON 格式
    change_reason TEXT,
    changed_by TEXT,
    changed_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 4. 迁移旧数据
-- 首先插入默认的3日周期配置
INSERT OR IGNORE INTO rhythm_config (config_name, cycle_days, description, is_active, created_by)
VALUES ('默认3日周期', 3, '系统默认的3日生产周期配置', 1, 'system');

-- 获取刚插入的config_id并迁移旧的rhythm_label数据
INSERT OR IGNORE INTO rhythm_label_new (config_id, rhythm_day, label_name, match_spec, bonus_score)
SELECT
    (SELECT config_id FROM rhythm_config WHERE config_name = '默认3日周期'),
    rhythm_day,
    label_name,
    match_spec,
    bonus_score
FROM rhythm_label;

-- 5. 替换表
DROP TABLE IF EXISTS rhythm_label;
ALTER TABLE rhythm_label_new RENAME TO rhythm_label;

-- 6. 创建索引
CREATE INDEX IF NOT EXISTS idx_rhythm_label_config ON rhythm_label(config_id);
CREATE INDEX IF NOT EXISTS idx_rhythm_label_day ON rhythm_label(rhythm_day);
CREATE INDEX IF NOT EXISTS idx_rhythm_config_active ON rhythm_config(is_active);

-- 7. 插入示例：5日周期配置
INSERT OR IGNORE INTO rhythm_config (config_name, cycle_days, description, is_active, created_by)
VALUES ('5日周期', 5, '5日生产周期配置，适用于特殊生产节奏', 0, 'system');

-- 插入5日周期的节拍标签示例
INSERT OR IGNORE INTO rhythm_label (config_id, rhythm_day, label_name, match_spec, bonus_score, description)
SELECT
    (SELECT config_id FROM rhythm_config WHERE config_name = '5日周期'),
    day_num,
    label,
    spec,
    score,
    desc_text
FROM (
    SELECT 1 AS day_num, '首日启动' AS label, '*' AS spec, 15.0 AS score, '周期首日，大批量常规品' AS desc_text
    UNION SELECT 2, '二日过渡', '*', 10.0, '过渡批次'
    UNION SELECT 3, '中期稳定', '*', 8.0, '稳定生产期'
    UNION SELECT 4, '四日收尾', '*', 5.0, '准备收尾'
    UNION SELECT 5, '末日紧急', '*', 20.0, '周期末日，紧急订单优先'
);

-- 8. 插入示例：7日周期配置
INSERT OR IGNORE INTO rhythm_config (config_name, cycle_days, description, is_active, created_by)
VALUES ('7日周期', 7, '7日生产周期配置，与自然周同步', 0, 'system');

-- 插入7日周期的节拍标签（简化版，按工作日设置）
INSERT OR IGNORE INTO rhythm_label (config_id, rhythm_day, label_name, match_spec, bonus_score, description)
SELECT
    (SELECT config_id FROM rhythm_config WHERE config_name = '7日周期'),
    day_num,
    label,
    spec,
    score,
    desc_text
FROM (
    SELECT 1 AS day_num, '周一启动' AS label, '*' AS spec, 12.0 AS score, '新周开始' AS desc_text
    UNION SELECT 2, '周二常规', '*', 8.0, '常规生产日'
    UNION SELECT 3, '周三常规', '*', 8.0, '常规生产日'
    UNION SELECT 4, '周四常规', '*', 8.0, '常规生产日'
    UNION SELECT 5, '周五赶工', '*', 15.0, '周末前赶工'
    UNION SELECT 6, '周六加班', '*', 5.0, '加班日（产能受限）'
    UNION SELECT 7, '周日休息', '*', 3.0, '休息日（最低产能）'
);
