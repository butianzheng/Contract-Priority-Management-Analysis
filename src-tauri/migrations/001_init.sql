-- DPM 数据库初始化脚本
-- 合同动态优先级管理系统

-- 1. 合同主表
CREATE TABLE IF NOT EXISTS contract_master (
    contract_id TEXT PRIMARY KEY,
    customer_id TEXT NOT NULL,
    steel_grade TEXT,
    thickness REAL,
    width REAL,
    spec_family TEXT,
    pdd DATE,
    days_to_pdd INTEGER,
    margin REAL DEFAULT 0.0,
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 2. 客户主表
CREATE TABLE IF NOT EXISTS customer_master (
    customer_id TEXT PRIMARY KEY,
    customer_name TEXT,
    customer_level TEXT CHECK(customer_level IN ('A', 'B', 'C')),
    credit_level TEXT,
    customer_group TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 3. 工艺难度配置表
CREATE TABLE IF NOT EXISTS process_difficulty (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    steel_grade TEXT,
    thickness_min REAL,
    thickness_max REAL,
    width_min REAL,
    width_max REAL,
    difficulty_level TEXT,
    difficulty_score REAL,
    UNIQUE(steel_grade, thickness_min, thickness_max, width_min, width_max)
);

-- 4. 节拍标签表（3日周期）
CREATE TABLE IF NOT EXISTS rhythm_label (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rhythm_day INTEGER CHECK(rhythm_day BETWEEN 1 AND 3),
    label_name TEXT,
    match_spec TEXT,
    bonus_score REAL DEFAULT 0.0
);

-- 5. 策略权重表
CREATE TABLE IF NOT EXISTS strategy_weights (
    strategy_name TEXT PRIMARY KEY,
    ws REAL NOT NULL CHECK(ws >= 0),
    wp REAL NOT NULL CHECK(wp >= 0),
    description TEXT,
    created_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 6. 人工干预日志表
CREATE TABLE IF NOT EXISTS intervention_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id TEXT NOT NULL,
    alpha_value REAL,
    reason TEXT,
    user TEXT,
    timestamp TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (contract_id) REFERENCES contract_master(contract_id)
);

-- 创建索引以提升查询性能
CREATE INDEX IF NOT EXISTS idx_contract_customer ON contract_master(customer_id);
CREATE INDEX IF NOT EXISTS idx_contract_pdd ON contract_master(pdd);
CREATE INDEX IF NOT EXISTS idx_difficulty_grade ON process_difficulty(steel_grade);
CREATE INDEX IF NOT EXISTS idx_intervention_contract ON intervention_log(contract_id);
