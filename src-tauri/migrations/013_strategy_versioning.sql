-- ============================================
-- Phase 14: 策略版本化（可回放、可复盘）
-- ============================================
--
-- 功能说明：
-- 1. 每次策略参数变更自动生成版本快照
-- 2. 沙盘结果引用特定版本，确保可复现
-- 3. 支持会议复盘、责任追溯
-- 4. 历史策略可复算，结果一致
--
-- 设计原则：
-- - 版本快照是不可变的（immutable）
-- - 包含计算所需的所有配置参数
-- - 支持版本间对比
-- ============================================

-- 策略版本快照表
-- 记录策略参数的完整快照，确保历史计算可复现
CREATE TABLE IF NOT EXISTS strategy_version (
    version_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 版本标识
    version_number INTEGER NOT NULL,          -- 版本序号（同策略内递增）
    version_tag TEXT,                         -- 版本标签（如 v1.0.0, 2024Q1）
    strategy_name TEXT NOT NULL,              -- 策略名称

    -- 策略权重快照（strategy_weights）
    ws REAL NOT NULL,                         -- S-Score 权重
    wp REAL NOT NULL,                         -- P-Score 权重

    -- S-Score 子权重快照（strategy_scoring_weights）
    w1 REAL NOT NULL,                         -- 客户等级权重
    w2 REAL NOT NULL,                         -- 毛利权重
    w3 REAL NOT NULL,                         -- 紧急度权重

    -- P-Score 子权重（当前使用默认值，未来可配置化）
    w_p1 REAL NOT NULL DEFAULT 0.5,           -- 工艺难度权重
    w_p2 REAL NOT NULL DEFAULT 0.3,           -- 聚合度权重
    w_p3 REAL NOT NULL DEFAULT 0.2,           -- 节拍匹配权重

    -- 完整配置快照（JSON 格式）
    scoring_config_snapshot TEXT NOT NULL,    -- scoring_config 表的完整快照
    p2_curve_config_snapshot TEXT,            -- P2 曲线配置快照
    aggregation_bins_snapshot TEXT,           -- 聚合区间配置快照
    rhythm_config_snapshot TEXT,              -- 节拍配置快照（包含当前激活的配置）

    -- 版本元数据
    description TEXT,                         -- 版本描述
    change_reason TEXT,                       -- 变更原因
    created_by TEXT NOT NULL,                 -- 创建人
    created_at TEXT DEFAULT (datetime('now','localtime')),

    -- 版本状态
    is_active INTEGER NOT NULL DEFAULT 0,     -- 是否为当前激活版本
    is_locked INTEGER NOT NULL DEFAULT 0,     -- 是否锁定（锁定后不可删除）

    -- 索引优化
    UNIQUE(strategy_name, version_number)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_strategy_version_name ON strategy_version(strategy_name);
CREATE INDEX IF NOT EXISTS idx_strategy_version_active ON strategy_version(strategy_name, is_active);
CREATE INDEX IF NOT EXISTS idx_strategy_version_created ON strategy_version(created_at);

-- 沙盘计算会话表
-- 记录每次沙盘计算的上下文，确保可复现
CREATE TABLE IF NOT EXISTS sandbox_session (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 会话标识
    session_name TEXT NOT NULL,               -- 会话名称（如"2024年12月排产预演"）
    session_type TEXT NOT NULL DEFAULT 'sandbox',  -- 类型：sandbox（沙盘）、production（生产）

    -- 版本引用（核心：确保可复现）
    strategy_version_id INTEGER NOT NULL,     -- 使用的策略版本
    contract_snapshot_time TEXT,              -- 合同池快照时间点

    -- 会话状态
    status TEXT NOT NULL DEFAULT 'draft',     -- draft, running, completed, archived

    -- 结果摘要
    total_contracts INTEGER,                  -- 参与计算的合同数
    result_summary TEXT,                      -- 结果摘要 JSON

    -- 元数据
    description TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now','localtime')),
    completed_at TEXT,

    FOREIGN KEY (strategy_version_id) REFERENCES strategy_version(version_id)
);

-- 沙盘计算结果表
-- 存储每次计算的详细结果，支持历史对比
CREATE TABLE IF NOT EXISTS sandbox_result (
    result_id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,

    -- 合同信息（快照）
    contract_id TEXT NOT NULL,
    contract_snapshot TEXT NOT NULL,          -- 合同数据快照 JSON
    customer_snapshot TEXT,                   -- 客户数据快照 JSON

    -- 计算结果
    s_score REAL NOT NULL,
    p_score REAL NOT NULL,
    priority REAL NOT NULL,
    alpha REAL,

    -- 详细评分（用于复盘）
    s1_score REAL,                            -- 客户等级分
    s2_score REAL,                            -- 毛利分
    s3_score REAL,                            -- 紧急度分
    p1_score REAL,                            -- 工艺难度分
    p2_score REAL,                            -- 聚合度分
    p3_score REAL,                            -- 节拍匹配分

    -- 聚合信息（用于复盘）
    aggregation_key TEXT,
    aggregation_count INTEGER,

    -- 排名
    priority_rank INTEGER,                    -- 在本次会话中的排名

    FOREIGN KEY (session_id) REFERENCES sandbox_session(session_id)
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_sandbox_result_session ON sandbox_result(session_id);
CREATE INDEX IF NOT EXISTS idx_sandbox_result_contract ON sandbox_result(contract_id);
CREATE INDEX IF NOT EXISTS idx_sandbox_result_priority ON sandbox_result(session_id, priority DESC);

-- 版本对比记录表
-- 记录版本间的对比结果
CREATE TABLE IF NOT EXISTS version_comparison (
    comparison_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 对比的两个版本
    version_a_id INTEGER NOT NULL,
    version_b_id INTEGER NOT NULL,

    -- 对比结果摘要
    contracts_compared INTEGER,               -- 对比的合同数
    rank_changes INTEGER,                     -- 排名变化的合同数
    avg_priority_diff REAL,                   -- 平均优先级差异
    max_rank_change INTEGER,                  -- 最大排名变化

    -- 详细结果
    comparison_details TEXT,                  -- 详细对比结果 JSON

    -- 元数据
    created_by TEXT NOT NULL,
    created_at TEXT DEFAULT (datetime('now','localtime')),

    FOREIGN KEY (version_a_id) REFERENCES strategy_version(version_id),
    FOREIGN KEY (version_b_id) REFERENCES strategy_version(version_id)
);

-- 策略版本变更日志
-- 记录版本创建、激活、锁定等操作
CREATE TABLE IF NOT EXISTS strategy_version_change_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    version_id INTEGER NOT NULL,

    change_type TEXT NOT NULL,                -- create, activate, deactivate, lock, unlock
    old_value TEXT,
    new_value TEXT,
    change_reason TEXT,

    changed_by TEXT NOT NULL,
    changed_at TEXT DEFAULT (datetime('now','localtime')),

    FOREIGN KEY (version_id) REFERENCES strategy_version(version_id)
);

-- ============================================
-- 初始化：为现有策略创建初始版本
-- ============================================

-- 注意：这个初始化脚本会在应用首次运行时执行
-- 它会为每个现有策略创建一个 v1 版本快照

-- 此处不自动执行，由应用启动时的 Rust 代码处理
-- 因为需要读取多个表并组装 JSON

-- ============================================
-- 视图：方便查询当前激活版本
-- ============================================

CREATE VIEW IF NOT EXISTS v_active_strategy_versions AS
SELECT
    sv.*,
    (SELECT COUNT(*) FROM strategy_version sv2
     WHERE sv2.strategy_name = sv.strategy_name) as total_versions
FROM strategy_version sv
WHERE sv.is_active = 1;

-- ============================================
-- 视图：沙盘会话详情（含版本信息）
-- ============================================

CREATE VIEW IF NOT EXISTS v_sandbox_session_details AS
SELECT
    ss.*,
    sv.version_number,
    sv.version_tag,
    sv.ws,
    sv.wp,
    sv.created_by as version_created_by
FROM sandbox_session ss
JOIN strategy_version sv ON ss.strategy_version_id = sv.version_id;
