-- ============================================
-- Phase 16: 会议驾驶舱 KPI 固化
-- Meeting Cockpit KPI Solidification
-- ============================================
-- 功能：
-- 1. 会议快照表（保存每次会议的完整状态）
-- 2. KPI 指标配置表（四视角 KPI 定义）
-- 3. 风险合同标记表（标记需关注的合同）
-- 4. 排名变化明细表（Explain 拆解）
-- 5. 共识包模板表（会议输出格式）
-- ============================================

-- 1. 会议快照表
-- 保存每次会议的完整状态快照，支持历史对比和复盘
CREATE TABLE IF NOT EXISTS meeting_snapshot (
    snapshot_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 会议基本信息
    meeting_type TEXT NOT NULL,              -- production_sales: 产销例会, business: 经营例会
    meeting_date TEXT NOT NULL,              -- 会议日期 YYYY-MM-DD
    snapshot_name TEXT NOT NULL,             -- 快照名称（如：2024-01-15产销例会）

    -- 关联策略版本
    strategy_version_id INTEGER,             -- 关联的策略版本 ID
    strategy_name TEXT,                      -- 策略名称（冗余存储便于查询）

    -- KPI 汇总（JSON 格式）
    -- 格式: {"leadership": [...], "sales": [...], "production": [...], "finance": [...]}
    kpi_summary TEXT NOT NULL,

    -- 风险汇总（JSON 格式）
    -- 格式: {"total_risk_count": N, "by_type": {...}, "top_risks": [...]}
    risk_summary TEXT NOT NULL,

    -- 策略推荐说明
    recommendation TEXT,                      -- 本次会议的策略推荐理由

    -- 合同排名快照（JSON 格式）
    -- 格式: [{"contract_id": "...", "priority": 0.85, "rank": 1, ...}, ...]
    contract_rankings TEXT NOT NULL,

    -- 排名变化汇总（JSON 格式，与上次会议对比）
    -- 格式: {"total_changes": N, "up_count": N, "down_count": N, "avg_change": 0.05}
    ranking_changes TEXT,

    -- 共识状态
    consensus_status TEXT NOT NULL DEFAULT 'draft',  -- draft, pending_approval, approved, archived
    approved_by TEXT,                        -- 审批人
    approved_at TEXT,                        -- 审批时间

    -- 审计信息
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (strategy_version_id) REFERENCES strategy_version(version_id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_meeting_snapshot_type ON meeting_snapshot(meeting_type);
CREATE INDEX IF NOT EXISTS idx_meeting_snapshot_date ON meeting_snapshot(meeting_date);
CREATE INDEX IF NOT EXISTS idx_meeting_snapshot_status ON meeting_snapshot(consensus_status);
CREATE INDEX IF NOT EXISTS idx_meeting_snapshot_strategy ON meeting_snapshot(strategy_version_id);

-- 2. KPI 指标配置表
-- 定义四视角（领导/销售/生产/财务）的 KPI 指标
CREATE TABLE IF NOT EXISTS meeting_kpi_config (
    kpi_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 指标基本信息
    kpi_code TEXT NOT NULL UNIQUE,           -- 指标代码（如：LEADERSHIP_01）
    kpi_name TEXT NOT NULL,                  -- 指标名称（如：高优合同占比）
    kpi_category TEXT NOT NULL,              -- 指标分类：leadership, sales, production, finance

    -- 计算定义
    calculation_type TEXT NOT NULL,          -- count, sum, avg, ratio, custom
    calculation_formula TEXT,                -- 计算公式（SQL 或表达式）
    data_source TEXT,                        -- 数据来源表/视图
    filter_condition TEXT,                   -- 筛选条件

    -- 显示设置
    display_format TEXT NOT NULL DEFAULT 'number',  -- number, percent, currency, text
    display_unit TEXT,                       -- 显示单位（如：个、%、万元）
    decimal_places INTEGER DEFAULT 2,        -- 小数位数

    -- 阈值设置（用于红绿灯状态）
    threshold_good REAL,                     -- 良好阈值（绿灯）
    threshold_warning REAL,                  -- 警告阈值（黄灯）
    threshold_danger REAL,                   -- 危险阈值（红灯）
    threshold_direction TEXT DEFAULT 'higher_better',  -- higher_better, lower_better

    -- 排序和启用
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,

    -- 说明
    description TEXT,                        -- 指标说明
    business_meaning TEXT,                   -- 业务含义（给管理层看）

    -- 审计信息
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_kpi_config_category ON meeting_kpi_config(kpi_category);
CREATE INDEX IF NOT EXISTS idx_kpi_config_enabled ON meeting_kpi_config(enabled);
CREATE INDEX IF NOT EXISTS idx_kpi_config_sort ON meeting_kpi_config(sort_order);

-- 3. 风险合同标记表
-- 标记需要在会议上重点关注的风险合同
CREATE TABLE IF NOT EXISTS risk_contract_flag (
    flag_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 关联信息
    snapshot_id INTEGER,                     -- 关联的会议快照
    contract_id TEXT NOT NULL,               -- 合同 ID

    -- 风险信息
    risk_type TEXT NOT NULL,                 -- 风险类型
    -- delivery_delay: 交期延迟风险
    -- customer_downgrade: 客户降级风险（高优客户被降低优先级）
    -- margin_loss: 毛利损失风险
    -- rhythm_mismatch: 节拍不匹配风险
    -- capacity_conflict: 产能冲突风险
    -- quality_concern: 质量关注风险

    risk_level TEXT NOT NULL DEFAULT 'medium',  -- high, medium, low
    risk_score REAL,                         -- 风险评分（0-100）

    -- 风险详情
    risk_description TEXT NOT NULL,          -- 风险描述
    risk_factors TEXT,                       -- 风险因素（JSON 数组）

    -- 影响分析
    affected_kpis TEXT,                      -- 受影响的 KPI（JSON 数组）
    potential_loss REAL,                     -- 潜在损失金额
    potential_loss_unit TEXT DEFAULT 'CNY',  -- 损失单位

    -- 建议措施
    suggested_action TEXT,                   -- 建议措施
    action_priority INTEGER DEFAULT 2,       -- 措施优先级 1-3

    -- 处理状态
    status TEXT NOT NULL DEFAULT 'open',     -- open, acknowledged, mitigated, closed
    handled_by TEXT,                         -- 处理人
    handled_at TEXT,                         -- 处理时间
    handling_note TEXT,                      -- 处理备注

    -- 审计信息
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (snapshot_id) REFERENCES meeting_snapshot(snapshot_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_risk_flag_snapshot ON risk_contract_flag(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_risk_flag_contract ON risk_contract_flag(contract_id);
CREATE INDEX IF NOT EXISTS idx_risk_flag_type ON risk_contract_flag(risk_type);
CREATE INDEX IF NOT EXISTS idx_risk_flag_level ON risk_contract_flag(risk_level);
CREATE INDEX IF NOT EXISTS idx_risk_flag_status ON risk_contract_flag(status);

-- 4. 排名变化明细表
-- 记录合同排名变化及 Explain 拆解
CREATE TABLE IF NOT EXISTS ranking_change_detail (
    change_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 关联信息
    snapshot_id INTEGER NOT NULL,            -- 当前会议快照
    compare_snapshot_id INTEGER,             -- 对比的会议快照（NULL 表示与实时对比）
    contract_id TEXT NOT NULL,               -- 合同 ID

    -- 排名变化
    old_rank INTEGER,                        -- 旧排名
    new_rank INTEGER NOT NULL,               -- 新排名
    rank_change INTEGER,                     -- 排名变化（正数上升，负数下降）

    -- 优先级变化
    old_priority REAL,                       -- 旧优先级
    new_priority REAL NOT NULL,              -- 新优先级
    priority_change REAL,                    -- 优先级变化

    -- S-Score 拆解
    old_s_score REAL,
    new_s_score REAL NOT NULL,
    s_score_change REAL,

    -- S-Score 子项变化
    s1_change REAL,                          -- 客户等级分变化
    s1_old REAL,
    s1_new REAL,
    s2_change REAL,                          -- 毛利分变化
    s2_old REAL,
    s2_new REAL,
    s3_change REAL,                          -- 紧急度分变化
    s3_old REAL,
    s3_new REAL,

    -- P-Score 拆解
    old_p_score REAL,
    new_p_score REAL NOT NULL,
    p_score_change REAL,

    -- P-Score 子项变化
    p1_change REAL,                          -- 工艺难度分变化
    p1_old REAL,
    p1_new REAL,
    p2_change REAL,                          -- 聚合度分变化
    p2_old REAL,
    p2_new REAL,
    p3_change REAL,                          -- 节拍分变化
    p3_old REAL,
    p3_new REAL,

    -- Explain 分析
    primary_factor TEXT,                     -- 主要变化因素代码
    -- s1_customer: 客户等级变化
    -- s2_margin: 毛利变化
    -- s3_urgency: 紧急度变化
    -- p1_difficulty: 工艺难度变化
    -- p2_aggregation: 聚合度变化
    -- p3_rhythm: 节拍变化
    -- strategy_weight: 策略权重变化

    primary_factor_name TEXT,                -- 主要变化因素名称（中文）
    explain_text TEXT,                       -- 完整的变化解释文本

    -- 权重信息（用于理解策略影响）
    ws_used REAL,                            -- 使用的 S 权重
    wp_used REAL,                            -- 使用的 P 权重

    -- 审计信息
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (snapshot_id) REFERENCES meeting_snapshot(snapshot_id) ON DELETE CASCADE,
    FOREIGN KEY (compare_snapshot_id) REFERENCES meeting_snapshot(snapshot_id) ON DELETE SET NULL
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_ranking_change_snapshot ON ranking_change_detail(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_ranking_change_contract ON ranking_change_detail(contract_id);
CREATE INDEX IF NOT EXISTS idx_ranking_change_factor ON ranking_change_detail(primary_factor);
CREATE INDEX IF NOT EXISTS idx_ranking_change_rank ON ranking_change_detail(rank_change);

-- 5. 共识包模板表
-- 定义会议输出的模板格式
CREATE TABLE IF NOT EXISTS consensus_template (
    template_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 模板基本信息
    template_code TEXT NOT NULL UNIQUE,      -- 模板代码
    template_name TEXT NOT NULL,             -- 模板名称
    meeting_type TEXT NOT NULL,              -- 适用会议类型：production_sales, business, all

    -- 模板内容定义（JSON 格式）
    -- 格式: {"sections": [{"id": "...", "title": "...", "type": "...", "config": {...}}]}
    template_config TEXT NOT NULL,

    -- 输出格式
    output_formats TEXT NOT NULL DEFAULT 'pdf,csv',  -- 支持的输出格式

    -- 状态
    is_default INTEGER NOT NULL DEFAULT 0,   -- 是否默认模板
    enabled INTEGER NOT NULL DEFAULT 1,

    -- 说明
    description TEXT,

    -- 审计信息
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_consensus_template_type ON consensus_template(meeting_type);
CREATE INDEX IF NOT EXISTS idx_consensus_template_default ON consensus_template(is_default);

-- 6. 会议行动项表
-- 记录会议产生的行动项和跟踪状态
CREATE TABLE IF NOT EXISTS meeting_action_item (
    action_id INTEGER PRIMARY KEY AUTOINCREMENT,

    -- 关联信息
    snapshot_id INTEGER NOT NULL,            -- 关联的会议快照

    -- 行动项内容
    action_title TEXT NOT NULL,              -- 行动项标题
    action_description TEXT,                 -- 详细描述
    action_category TEXT,                    -- 分类：strategy, risk, customer, production

    -- 优先级和截止日期
    priority INTEGER NOT NULL DEFAULT 2,     -- 1: 高, 2: 中, 3: 低
    due_date TEXT,                           -- 截止日期

    -- 责任人
    assignee TEXT,                           -- 责任人
    department TEXT,                         -- 责任部门

    -- 关联的合同（可选）
    related_contracts TEXT,                  -- JSON 数组，关联的合同 ID

    -- 状态跟踪
    status TEXT NOT NULL DEFAULT 'open',     -- open, in_progress, completed, cancelled
    completion_rate INTEGER DEFAULT 0,       -- 完成率 0-100
    completed_at TEXT,

    -- 备注
    notes TEXT,

    -- 审计信息
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (snapshot_id) REFERENCES meeting_snapshot(snapshot_id) ON DELETE CASCADE
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_action_item_snapshot ON meeting_action_item(snapshot_id);
CREATE INDEX IF NOT EXISTS idx_action_item_status ON meeting_action_item(status);
CREATE INDEX IF NOT EXISTS idx_action_item_assignee ON meeting_action_item(assignee);
CREATE INDEX IF NOT EXISTS idx_action_item_priority ON meeting_action_item(priority);
CREATE INDEX IF NOT EXISTS idx_action_item_due ON meeting_action_item(due_date);

-- ============================================
-- 插入默认 KPI 配置
-- ============================================

-- 领导视角 KPI
INSERT OR IGNORE INTO meeting_kpi_config (
    kpi_code, kpi_name, kpi_category, calculation_type,
    display_format, display_unit, threshold_good, threshold_warning, threshold_danger,
    threshold_direction, sort_order, description, business_meaning
) VALUES
-- L01: 高优合同占比
('L01_HIGH_PRIORITY_RATIO', '高优合同占比', 'leadership', 'ratio',
 'percent', '%', 30, 20, 10, 'higher_better', 1,
 '优先级 Top 20% 的合同占总合同的比例',
 '反映当前合同池的整体质量，越高说明优质合同越多'),

-- L02: 客户覆盖率
('L02_CUSTOMER_COVERAGE', '重点客户覆盖率', 'leadership', 'ratio',
 'percent', '%', 95, 85, 75, 'higher_better', 2,
 'A/B级客户有在产合同的比例',
 '确保重点客户都有订单在生产，避免客户流失'),

-- L03: 交期达成预测
('L03_DELIVERY_FORECAST', '交期达成率预测', 'leadership', 'ratio',
 'percent', '%', 95, 90, 85, 'higher_better', 3,
 '预计能按期交付的合同比例',
 '提前预警交付风险，便于采取补救措施'),

-- L04: 毛利保障指数
('L04_MARGIN_INDEX', '毛利保障指数', 'leadership', 'custom',
 'number', '', 80, 60, 40, 'higher_better', 4,
 '加权平均毛利水平（考虑优先级权重）',
 '综合反映当前排产的盈利能力');

-- 销售视角 KPI
INSERT OR IGNORE INTO meeting_kpi_config (
    kpi_code, kpi_name, kpi_category, calculation_type,
    display_format, display_unit, threshold_good, threshold_warning, threshold_danger,
    threshold_direction, sort_order, description, business_meaning
) VALUES
-- S01: 客户满意度风险
('S01_CUSTOMER_RISK', '客户满意风险数', 'sales', 'count',
 'number', '个', 5, 10, 20, 'lower_better', 1,
 '高优客户被降低优先级的合同数',
 '这些客户可能因交期延迟而不满，需重点关注'),

-- S02: VIP 客户保障率
('S02_VIP_COVERAGE', 'VIP客户保障率', 'sales', 'ratio',
 'percent', '%', 100, 95, 90, 'higher_better', 2,
 'VIP客户合同进入 Top 30% 的比例',
 '确保最重要客户的订单得到优先保障'),

-- S03: 新客户订单优先级
('S03_NEW_CUSTOMER_PRIORITY', '新客户平均排名', 'sales', 'avg',
 'number', '名', 50, 100, 150, 'lower_better', 3,
 '新客户订单的平均排名',
 '新客户体验很重要，排名太靠后会影响合作'),

-- S04: 紧急订单处理率
('S04_URGENT_HANDLING', '紧急订单响应率', 'sales', 'ratio',
 'percent', '%', 95, 85, 75, 'higher_better', 4,
 '7天内交期订单进入 Top 50 的比例',
 '紧急订单必须快速响应，否则必然延期');

-- 生产/计划视角 KPI
INSERT OR IGNORE INTO meeting_kpi_config (
    kpi_code, kpi_name, kpi_category, calculation_type,
    display_format, display_unit, threshold_good, threshold_warning, threshold_danger,
    threshold_direction, sort_order, description, business_meaning
) VALUES
-- P01: 节拍匹配度
('P01_RHYTHM_MATCH', '节拍匹配度', 'production', 'ratio',
 'percent', '%', 85, 70, 60, 'higher_better', 1,
 '符合当日节拍标签的合同比例',
 '节拍匹配越高，生产越顺畅，效率越高'),

-- P02: 规格聚合度
('P02_SPEC_AGGREGATION', '规格聚合度', 'production', 'custom',
 'number', '', 70, 50, 30, 'higher_better', 2,
 'Top 100 合同的规格聚合程度',
 '聚合度高意味着换规次数少，生产效率高'),

-- P03: 工艺难度分布
('P03_DIFFICULTY_BALANCE', '工艺难度均衡度', 'production', 'custom',
 'number', '', 80, 60, 40, 'higher_better', 3,
 '高难度合同在各时段的分布均衡程度',
 '难度分布均匀，避免某时段全是难单'),

-- P04: 产能利用预测
('P04_CAPACITY_FORECAST', '产能利用率', 'production', 'ratio',
 'percent', '%', 90, 80, 70, 'higher_better', 4,
 '预计产能利用率',
 '太低浪费产能，太高则无弹性应对紧急订单');

-- 财务/经营视角 KPI
INSERT OR IGNORE INTO meeting_kpi_config (
    kpi_code, kpi_name, kpi_category, calculation_type,
    display_format, display_unit, threshold_good, threshold_warning, threshold_danger,
    threshold_direction, sort_order, description, business_meaning
) VALUES
-- F01: 毛利贡献
('F01_MARGIN_CONTRIBUTION', '预计毛利贡献', 'finance', 'sum',
 'currency', '万元', 500, 300, 200, 'higher_better', 1,
 'Top 100 合同的预计毛利总额',
 '直接反映当前排产的盈利能力'),

-- F02: 高毛利占比
('F02_HIGH_MARGIN_RATIO', '高毛利合同占比', 'finance', 'ratio',
 'percent', '%', 40, 30, 20, 'higher_better', 2,
 '毛利 > 15% 的合同占 Top 100 的比例',
 '高毛利合同越多，整体盈利水平越高'),

-- F03: 风险敞口
('F03_RISK_EXPOSURE', '风险敞口金额', 'finance', 'sum',
 'currency', '万元', 50, 100, 200, 'lower_better', 3,
 '风险合同的潜在损失总额',
 '需要关注的潜在损失金额'),

-- F04: 账期风险
('F04_CREDIT_RISK', '账期风险合同数', 'finance', 'count',
 'number', '个', 5, 10, 20, 'lower_better', 4,
 '信用等级低且金额大的合同数',
 '这些合同可能存在回款风险');

-- ============================================
-- 插入默认共识包模板
-- ============================================

-- 产销例会模板
INSERT INTO consensus_template (
    template_code, template_name, meeting_type,
    template_config, output_formats, is_default, description, created_by
) VALUES
('PSM_STANDARD', '产销例会标准模板', 'production_sales',
'{
  "sections": [
    {
      "id": "header",
      "title": "会议信息",
      "type": "info",
      "fields": ["meeting_date", "strategy_name", "total_contracts"]
    },
    {
      "id": "kpi_summary",
      "title": "KPI 总览",
      "type": "kpi_cards",
      "categories": ["leadership", "production"]
    },
    {
      "id": "ranking_changes",
      "title": "排名变化 Top 10",
      "type": "ranking_table",
      "limit": 10,
      "show_explain": true
    },
    {
      "id": "risk_contracts",
      "title": "风险合同清单",
      "type": "risk_list",
      "risk_types": ["delivery_delay", "rhythm_mismatch"]
    },
    {
      "id": "action_items",
      "title": "行动项",
      "type": "action_list",
      "categories": ["production", "strategy"]
    }
  ]
}',
'pdf,csv', 1, '产销例会标准输出模板，聚焦生产和交付', 'system'),

-- 经营例会模板
('BM_STANDARD', '经营例会标准模板', 'business',
'{
  "sections": [
    {
      "id": "header",
      "title": "会议信息",
      "type": "info",
      "fields": ["meeting_date", "strategy_name", "total_contracts"]
    },
    {
      "id": "kpi_summary",
      "title": "经营 KPI 总览",
      "type": "kpi_cards",
      "categories": ["leadership", "sales", "finance"]
    },
    {
      "id": "customer_analysis",
      "title": "客户保障分析",
      "type": "customer_table",
      "group_by": "customer_level"
    },
    {
      "id": "risk_contracts",
      "title": "风险合同清单",
      "type": "risk_list",
      "risk_types": ["customer_downgrade", "margin_loss"]
    },
    {
      "id": "margin_analysis",
      "title": "毛利分析",
      "type": "margin_chart",
      "show_trend": true
    },
    {
      "id": "action_items",
      "title": "行动项",
      "type": "action_list",
      "categories": ["customer", "strategy"]
    }
  ]
}',
'pdf,csv', 1, '经营例会标准输出模板，聚焦客户和盈利', 'system');

-- ============================================
-- 辅助视图
-- ============================================

-- 会议快照汇总视图
CREATE VIEW IF NOT EXISTS v_meeting_snapshot_summary AS
SELECT
    s.snapshot_id,
    s.meeting_type,
    s.meeting_date,
    s.snapshot_name,
    s.strategy_name,
    s.consensus_status,
    s.created_by,
    s.created_at,
    sv.version_number as strategy_version_number,
    (SELECT COUNT(*) FROM risk_contract_flag r WHERE r.snapshot_id = s.snapshot_id) as risk_count,
    (SELECT COUNT(*) FROM ranking_change_detail rc WHERE rc.snapshot_id = s.snapshot_id) as change_count,
    (SELECT COUNT(*) FROM meeting_action_item a WHERE a.snapshot_id = s.snapshot_id) as action_count
FROM meeting_snapshot s
LEFT JOIN strategy_version sv ON s.strategy_version_id = sv.version_id
ORDER BY s.meeting_date DESC, s.created_at DESC;

-- 风险合同汇总视图
CREATE VIEW IF NOT EXISTS v_risk_summary_by_type AS
SELECT
    snapshot_id,
    risk_type,
    risk_level,
    COUNT(*) as count,
    SUM(COALESCE(potential_loss, 0)) as total_potential_loss
FROM risk_contract_flag
GROUP BY snapshot_id, risk_type, risk_level
ORDER BY snapshot_id, risk_type, risk_level;

-- 排名变化统计视图
CREATE VIEW IF NOT EXISTS v_ranking_change_stats AS
SELECT
    snapshot_id,
    COUNT(*) as total_contracts,
    SUM(CASE WHEN rank_change > 0 THEN 1 ELSE 0 END) as up_count,
    SUM(CASE WHEN rank_change < 0 THEN 1 ELSE 0 END) as down_count,
    SUM(CASE WHEN rank_change = 0 THEN 1 ELSE 0 END) as unchanged_count,
    AVG(ABS(rank_change)) as avg_change,
    MAX(rank_change) as max_up,
    MIN(rank_change) as max_down,
    primary_factor,
    COUNT(*) as factor_count
FROM ranking_change_detail
GROUP BY snapshot_id, primary_factor
ORDER BY snapshot_id, factor_count DESC;
