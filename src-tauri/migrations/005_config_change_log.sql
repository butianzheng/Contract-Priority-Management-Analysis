-- ======================================
-- Phase 2: 配置变更日志表
-- ======================================

-- 表: config_change_log (配置变更日志)
CREATE TABLE IF NOT EXISTS config_change_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL,                 -- 配置键（对应 scoring_config.config_key）
    old_value TEXT NOT NULL,                  -- 旧值
    new_value TEXT NOT NULL,                  -- 新值
    change_reason TEXT,                       -- 变更原因
    changed_by TEXT NOT NULL,                 -- 操作人
    changed_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (config_key) REFERENCES scoring_config(config_key)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_config_change_log_key ON config_change_log(config_key);
CREATE INDEX IF NOT EXISTS idx_config_change_log_changed_at ON config_change_log(changed_at);
CREATE INDEX IF NOT EXISTS idx_config_change_log_changed_by ON config_change_log(changed_by);

-- 注释
-- 用途: 记录所有配置修改历史，支持配置审计和回滚
