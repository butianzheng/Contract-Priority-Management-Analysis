-- ============================================
-- Migration 009: 规格族主数据表
-- ============================================

-- 规格族主表
CREATE TABLE IF NOT EXISTS spec_family_master (
    family_id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_name TEXT NOT NULL UNIQUE,            -- 规格族名称
    family_code TEXT NOT NULL UNIQUE,            -- 规格族代码（英文缩写）
    description TEXT,                            -- 描述
    factor REAL NOT NULL DEFAULT 1.0,            -- P-Score 系数
    steel_grades TEXT,                           -- 关联钢种（JSON数组格式）
    thickness_min REAL,                          -- 适用厚度范围-最小
    thickness_max REAL,                          -- 适用厚度范围-最大
    width_min REAL,                              -- 适用宽度范围-最小
    width_max REAL,                              -- 适用宽度范围-最大
    enabled INTEGER NOT NULL DEFAULT 1,          -- 是否启用 (0=禁用, 1=启用)
    sort_order INTEGER NOT NULL DEFAULT 0,       -- 排序顺序
    created_by TEXT NOT NULL DEFAULT 'system',
    created_at TEXT DEFAULT (datetime('now','localtime')),
    updated_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 规格族变更日志
CREATE TABLE IF NOT EXISTS spec_family_change_log (
    change_id INTEGER PRIMARY KEY AUTOINCREMENT,
    family_id INTEGER NOT NULL,
    family_name TEXT,                            -- 冗余存储便于查询
    change_type TEXT NOT NULL,                   -- create, update, delete, enable, disable
    old_value TEXT,                              -- 变更前的JSON值
    new_value TEXT,                              -- 变更后的JSON值
    change_reason TEXT,
    changed_by TEXT NOT NULL,
    changed_at TEXT DEFAULT (datetime('now','localtime'))
);

-- 创建索引
CREATE INDEX IF NOT EXISTS idx_spec_family_code ON spec_family_master(family_code);
CREATE INDEX IF NOT EXISTS idx_spec_family_enabled ON spec_family_master(enabled);
CREATE INDEX IF NOT EXISTS idx_spec_family_change_log_family ON spec_family_change_log(family_id);
CREATE INDEX IF NOT EXISTS idx_spec_family_change_log_time ON spec_family_change_log(changed_at);

-- ============================================
-- 初始数据：预设规格族
-- ============================================

-- 基础规格族（对应现有的 常规/特殊/超特 分类）
INSERT OR IGNORE INTO spec_family_master (family_name, family_code, description, factor, steel_grades, enabled, sort_order, created_by)
VALUES
('常规', 'REG', '常规规格钢材，标准工艺流程', 1.0, '["Q235", "Q345", "SPHC", "SPCC"]', 1, 1, 'system'),
('特殊', 'SPE', '特殊规格钢材，需特殊工艺处理', 1.2, '["304", "316L", "Q550", "Q690"]', 1, 2, 'system'),
('超特', 'ULT', '超特规格钢材，高难度工艺', 1.5, '["310S", "Inconel", "Hastelloy"]', 1, 3, 'system');

-- 扩展规格族（更细分的产品系列）
INSERT OR IGNORE INTO spec_family_master (family_name, family_code, description, factor, steel_grades, thickness_min, thickness_max, enabled, sort_order, created_by)
VALUES
('双相钢', 'DP', '双相钢系列，高强度高延展性', 1.3, '["DP590", "DP780", "DP980", "DP1180"]', 0.5, 3.0, 1, 10, 'system'),
('淬火配分钢', 'QP', '淬火配分钢，超高强度', 1.4, '["QP980", "QP1180", "QP1470"]', 0.8, 2.5, 1, 11, 'system'),
('马氏体钢', 'MS', '马氏体钢，最高强度级别', 1.5, '["MS1180", "MS1500", "MS1700"]', 0.6, 2.0, 1, 12, 'system'),
('高强低合金钢', 'HSLA', '高强低合金钢，良好焊接性', 1.1, '["HSLA340", "HSLA420", "HSLA500"]', 1.0, 6.0, 1, 13, 'system'),
('无间隙原子钢', 'IF', '无间隙原子钢，超深冲性能', 1.2, '["IF260", "IF340", "IF-HS"]', 0.4, 2.5, 1, 14, 'system'),
('烘烤硬化钢', 'BH', '烘烤硬化钢，汽车外板专用', 1.15, '["BH180", "BH220", "BH260"]', 0.5, 1.5, 1, 15, 'system'),
('复相钢', 'CP', '复相钢，高能量吸收', 1.35, '["CP800", "CP1000", "CP1200"]', 0.8, 3.0, 1, 16, 'system'),
('热成形钢', 'PHS', '热成形钢，热冲压用', 1.6, '["22MnB5", "PHS1500", "PHS1800", "PHS2000"]', 0.8, 3.5, 1, 17, 'system');

-- 记录初始创建日志
INSERT INTO spec_family_change_log (family_id, family_name, change_type, new_value, changed_by, change_reason)
SELECT
    family_id,
    family_name,
    'create',
    json_object(
        'family_name', family_name,
        'family_code', family_code,
        'description', description,
        'factor', factor,
        'steel_grades', steel_grades
    ),
    'system',
    '系统初始化'
FROM spec_family_master;
