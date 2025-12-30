-- DPM v2.0 配置表迁移脚本
-- Phase 1: 核心配置化

-- 表1: scoring_config (评分参数配置表)
CREATE TABLE IF NOT EXISTS scoring_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL UNIQUE,          -- 配置键
    config_value TEXT NOT NULL,               -- 配置值（支持JSON）
    value_type TEXT NOT NULL,                 -- 值类型: number, string, json_array, json_object
    category TEXT NOT NULL,                   -- 分类: s_score, p_score, general
    description TEXT,                         -- 描述
    default_value TEXT,                       -- 默认值
    min_value REAL,                           -- 最小值（数值类型时）
    max_value REAL,                           -- 最大值（数值类型时）
    is_active INTEGER DEFAULT 1,              -- 是否启用
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_scoring_config_category ON scoring_config(category);
CREATE INDEX IF NOT EXISTS idx_scoring_config_active ON scoring_config(is_active);
CREATE INDEX IF NOT EXISTS idx_scoring_config_key ON scoring_config(config_key);

-- 表2: strategy_scoring_weights (策略级评分子权重表)
CREATE TABLE IF NOT EXISTS strategy_scoring_weights (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    strategy_name TEXT NOT NULL,              -- 策略名称
    w1 REAL NOT NULL DEFAULT 0.4,             -- S1权重（客户等级）
    w2 REAL NOT NULL DEFAULT 0.3,             -- S2权重（毛利）
    w3 REAL NOT NULL DEFAULT 0.3,             -- S3权重（紧急度）
    description TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (strategy_name) REFERENCES strategy_weights(strategy_name) ON DELETE CASCADE,
    UNIQUE(strategy_name)
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_strategy_scoring_weights_name ON strategy_scoring_weights(strategy_name);
