-- ============================================================================
-- 聚合度配置表 - P2 聚合度计算支持
--
-- 功能：
-- 1. aggregation_bins: 定义宽度段和厚度段的分界点
-- 2. p2_curve_config: P2 对数曲线参数配置
-- 3. aggregation_stats_cache: 聚合统计缓存（可选，用于性能优化）
-- ============================================================================

-- 1. 聚合区间配置表
-- 用于定义宽度段和厚度段的分界点，支持灵活配置
CREATE TABLE IF NOT EXISTS aggregation_bins (
    bin_id INTEGER PRIMARY KEY AUTOINCREMENT,
    dimension TEXT NOT NULL,                    -- 'width' 或 'thickness'
    bin_name TEXT NOT NULL,                     -- 区间名称，如 '薄规格', '窄幅'
    bin_code TEXT NOT NULL,                     -- 区间代码，如 'THIN', 'NARROW'
    min_value REAL NOT NULL,                    -- 区间下限（含）
    max_value REAL NOT NULL,                    -- 区间上限（不含，最后一档用 999999）
    sort_order INTEGER NOT NULL DEFAULT 0,      -- 排序顺序
    enabled INTEGER NOT NULL DEFAULT 1,         -- 是否启用
    description TEXT,                           -- 描述说明
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime')),

    UNIQUE(dimension, bin_code)
);

-- 2. P2 曲线参数配置表
-- 支持对数曲线、线性、阶梯等多种计算方式
CREATE TABLE IF NOT EXISTS p2_curve_config (
    config_id INTEGER PRIMARY KEY DEFAULT 1,    -- 单行配置
    curve_type TEXT NOT NULL DEFAULT 'logarithmic',  -- 'logarithmic' | 'linear' | 'step'

    -- 对数曲线参数
    log_base REAL NOT NULL DEFAULT 2.718281828, -- 对数底数（默认自然对数 e）
    log_scale REAL NOT NULL DEFAULT 25.0,       -- 缩放系数，控制曲线斜率

    -- 分数范围
    min_score REAL NOT NULL DEFAULT 0.0,        -- 最低分
    max_score REAL NOT NULL DEFAULT 100.0,      -- 最高分
    min_count_for_max INTEGER NOT NULL DEFAULT 50, -- 达到满分所需的最小数量

    -- 加权组合参数（聚合度分数 vs factor归一化分数）
    alpha REAL NOT NULL DEFAULT 0.7,            -- 聚合度分数权重
    beta REAL NOT NULL DEFAULT 0.3,             -- factor归一化分数权重

    -- 元数据
    description TEXT,
    updated_by TEXT DEFAULT 'system',
    updated_at TEXT DEFAULT (datetime('now','localtime')),

    -- 约束：alpha + beta = 1.0
    CHECK (abs(alpha + beta - 1.0) < 0.001)
);

-- 3. 聚合统计缓存表（可选，用于性能优化）
-- 避免每次计算都全表扫描
CREATE TABLE IF NOT EXISTS aggregation_stats_cache (
    cache_id INTEGER PRIMARY KEY AUTOINCREMENT,
    aggregation_key TEXT NOT NULL UNIQUE,       -- 聚合键：spec_family|steel_grade|thickness_bin|width_bin
    spec_family TEXT NOT NULL,
    steel_grade TEXT NOT NULL,
    thickness_bin TEXT NOT NULL,                -- 厚度段代码
    width_bin TEXT NOT NULL,                    -- 宽度段代码
    contract_count INTEGER NOT NULL DEFAULT 0,  -- 同类合同数量
    contract_ids TEXT,                          -- 合同ID列表（JSON数组）
    last_updated TEXT DEFAULT (datetime('now','localtime'))
);

-- 4. 聚合配置变更日志
CREATE TABLE IF NOT EXISTS aggregation_config_change_log (
    log_id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,                   -- 'aggregation_bins' | 'p2_curve_config'
    record_id INTEGER,
    change_type TEXT NOT NULL,                  -- 'create' | 'update' | 'delete'
    old_value TEXT,                             -- 变更前的JSON值
    new_value TEXT,                             -- 变更后的JSON值
    change_reason TEXT,
    changed_by TEXT NOT NULL DEFAULT 'system',
    changed_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_aggregation_bins_dimension ON aggregation_bins(dimension);
CREATE INDEX IF NOT EXISTS idx_aggregation_bins_enabled ON aggregation_bins(enabled);
CREATE INDEX IF NOT EXISTS idx_aggregation_stats_cache_key ON aggregation_stats_cache(aggregation_key);
CREATE INDEX IF NOT EXISTS idx_aggregation_stats_cache_spec ON aggregation_stats_cache(spec_family, steel_grade);

-- ============================================================================
-- 预设数据：厚度段和宽度段的常用分界点
-- ============================================================================

-- 厚度段配置（基于冷轧产线能力）
INSERT OR IGNORE INTO aggregation_bins (dimension, bin_name, bin_code, min_value, max_value, sort_order, description)
VALUES
    ('thickness', '薄规格', 'THIN', 0, 1.5, 1, '厚度 < 1.5mm，轧制难度较高'),
    ('thickness', '常规厚度', 'REGULAR', 1.5, 3.0, 2, '厚度 1.5-3.0mm，标准生产'),
    ('thickness', '厚规格', 'THICK', 3.0, 999999, 3, '厚度 > 3.0mm，大压下量');

-- 宽度段配置（基于轧机宽度能力）
INSERT OR IGNORE INTO aggregation_bins (dimension, bin_name, bin_code, min_value, max_value, sort_order, description)
VALUES
    ('width', '窄幅', 'NARROW', 0, 1200, 1, '宽度 < 1200mm'),
    ('width', '标准幅宽', 'STANDARD', 1200, 1500, 2, '宽度 1200-1500mm，主力规格'),
    ('width', '宽幅', 'WIDE', 1500, 999999, 3, '宽度 > 1500mm');

-- P2 曲线参数初始配置
INSERT OR IGNORE INTO p2_curve_config (config_id, curve_type, log_base, log_scale, min_score, max_score, min_count_for_max, alpha, beta, description)
VALUES (
    1,
    'logarithmic',
    2.718281828,    -- 自然对数
    25.0,           -- 缩放系数
    0.0,            -- 最低分
    100.0,          -- 最高分
    50,             -- 50个同类合同达到满分
    0.7,            -- 70% 权重给聚合度
    0.3,            -- 30% 权重给规格族系数
    'P2聚合度计算参数：对数曲线，α=0.7聚合度 + β=0.3规格族系数'
);
