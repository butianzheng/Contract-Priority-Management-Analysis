-- ============================================
-- Phase 4: 筛选器预设表
-- ============================================

CREATE TABLE IF NOT EXISTS filter_presets (
    preset_id INTEGER PRIMARY KEY AUTOINCREMENT,
    preset_name TEXT NOT NULL UNIQUE,
    filter_json TEXT NOT NULL,  -- JSON格式存储筛选条件
    description TEXT,
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_default INTEGER NOT NULL DEFAULT 0  -- 0=非默认, 1=默认预设
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_filter_presets_name ON filter_presets(preset_name);
CREATE INDEX IF NOT EXISTS idx_filter_presets_default ON filter_presets(is_default);

-- 插入默认筛选器预设
INSERT INTO filter_presets (preset_name, filter_json, description, created_by, is_default)
VALUES
('全部合同', '{}', '显示所有合同，不应用任何筛选条件', 'system', 1),
('紧急订单', '{"days_to_pdd_max": 7}', '显示剩余天数<=7天的紧急订单', 'system', 0),
('高优先级', '{"priority_min": 80}', '显示优先级>=80的高优先级合同', 'system', 0),
('特殊钢种', '{"steel_grades": ["304", "316L", "310S"]}', '显示特殊不锈钢钢种', 'system', 0);
