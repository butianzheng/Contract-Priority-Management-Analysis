use serde::{Deserialize, Serialize};

/// 合同数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    pub contract_id: String,
    pub customer_id: String,
    pub steel_grade: String,
    pub thickness: f64,
    pub width: f64,
    pub spec_family: String,
    pub pdd: String,
    pub days_to_pdd: i64,
    pub margin: f64,
}

/// 客户数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub customer_id: String,
    pub customer_name: Option<String>,
    pub customer_level: String,  // A/B/C
    pub credit_level: Option<String>,
    pub customer_group: Option<String>,
}

/// 工艺难度配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDifficulty {
    pub id: i64,
    pub steel_grade: String,
    pub thickness_min: f64,
    pub thickness_max: f64,
    pub width_min: f64,
    pub width_max: f64,
    pub difficulty_level: String,
    pub difficulty_score: f64,
}

/// 节拍标签
///
/// # 说明
/// 节拍标签用于 P3（节拍匹配评分）计算，支持 n 日可配置周期。
/// 每个标签关联到一个 `rhythm_config`，通过 `config_id` 关联。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmLabel {
    pub id: i64,
    /// 关联的节拍配置 ID
    pub config_id: i64,
    /// 周期日（1 到 cycle_days）
    pub rhythm_day: i32,
    /// 标签名称
    pub label_name: String,
    /// 匹配的规格族（支持通配符 * 或逗号分隔的列表）
    pub match_spec: Option<String>,
    /// P3 加分（0-100）
    pub bonus_score: f64,
    /// 标签描述
    pub description: Option<String>,
}

// ============================================
// Phase 10: n日节拍配置相关数据结构
// ============================================

/// 节拍配置
///
/// # 说明
/// 支持 n 日可配置周期，允许计划人员表达"最近 n 天生产偏好"。
/// 系统中只能有一个配置处于激活状态（is_active = 1）。
///
/// # 设计原则
/// - 灵活性：支持 1-30 天的周期配置
/// - 可切换：可快速切换不同周期配置
/// - 可追溯：所有变更记录在 rhythm_config_change_log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmConfig {
    pub config_id: Option<i64>,
    /// 配置名称（唯一）
    pub config_name: String,
    /// 周期天数（1-30）
    pub cycle_days: i32,
    /// 配置描述
    pub description: Option<String>,
    /// 是否激活（0=禁用, 1=激活）
    pub is_active: i64,
    /// 创建人
    pub created_by: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 节拍配置变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmConfigChangeLog {
    pub change_id: Option<i64>,
    pub config_id: i64,
    pub config_name: Option<String>,
    /// 变更类型：create, update, delete, activate, deactivate
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_reason: Option<String>,
    pub changed_by: Option<String>,
    pub changed_at: Option<String>,
}

/// 节拍匹配结果
///
/// # 用途
/// 用于 P3 评分详情展示，包含命中/未命中状态和匹配信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhythmMatchResult {
    /// 是否命中节拍标签
    pub is_hit: bool,
    /// 当前周期日（1 到 cycle_days）
    pub rhythm_day: i32,
    /// 当前激活的周期天数
    pub cycle_days: i32,
    /// 命中的标签名称（未命中时为 None）
    pub label_name: Option<String>,
    /// P3 得分
    pub rhythm_score: f64,
    /// 匹配说明（用于 Explain 输出）
    pub match_description: String,
}

/// 策略权重
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyWeights {
    pub strategy_name: String,
    pub ws: f64,  // S-Score 权重
    pub wp: f64,  // P-Score 权重
    pub description: Option<String>,
}

/// 人工干预记录
///
/// # 用途
/// 记录对合同优先级的人工调整历史，实现"可控、可追溯、可复盘"的干预管理。
///
/// # Alpha 规范
/// - 公式：`Final Priority = Base Priority × Alpha`
/// - 范围：[0.5, 2.0]
/// - 默认值：1.0（无调整）
///
/// # 字段说明
/// - `id`: 自增主键
/// - `contract_id`: 关联的合同编号
/// - `alpha_value`: 人工调整系数（0.5 ~ 2.0）
/// - `reason`: 调整原因（必填，用于复盘追溯）
/// - `user`: 操作人
/// - `timestamp`: 操作时间
///
/// # 生效规则
/// 每个合同可有多条干预记录，仅最新记录的 alpha_value 生效。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterventionLog {
    pub id: Option<i64>,
    pub contract_id: String,
    /// Alpha 值：[0.5, 2.0]，1.0 表示无调整
    pub alpha_value: f64,
    /// 调整原因（必填），用于可追溯性
    pub reason: String,
    /// 操作人
    pub user: String,
    /// 操作时间（自动生成）
    pub timestamp: Option<String>,
}

/// 合同优先级结果（包含评分详情）
///
/// # 字段说明
/// - `s_score`: S-Score（战略价值评分）
/// - `p_score`: P-Score（生产难度评分）
/// - `priority`: **最终优先级**（已应用 Alpha 调整）
/// - `alpha`: 当前生效的 Alpha 值（None 表示无调整，等同于 1.0）
///
/// # 一致性保证
/// 前端显示、后端计算、数据导出三处的 priority 值完全一致，
/// 均为应用 Alpha 后的最终值：`Final Priority = Base Priority × Alpha`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractPriority {
    #[serde(flatten)]
    pub contract: Contract,
    /// S-Score（战略价值评分）
    pub s_score: f64,
    /// P-Score（生产难度评分）
    pub p_score: f64,
    /// 最终优先级 = (ws × S + wp × P) × Alpha
    pub priority: f64,
    /// 当前生效的 Alpha 值，None 表示无调整（等同于 1.0）
    pub alpha: Option<f64>,
}

// ============================================
// Phase 2: 配置管理相关数据结构
// ============================================

/// 评分配置项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfigItem {
    pub config_key: String,
    pub config_value: String,
    pub value_type: String,      // number, string, json_array, json_object
    pub category: String,         // s_score, p_score, general
    pub description: Option<String>,
    pub default_value: Option<String>,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
}

/// 配置变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeLog {
    pub id: Option<i64>,
    pub config_key: String,
    pub old_value: String,
    pub new_value: String,
    pub change_reason: Option<String>,
    pub changed_by: String,
    pub changed_at: Option<String>,
}

// ============================================
// Phase 4: 筛选器预设数据结构
// ============================================

/// 筛选器预设
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterPreset {
    pub preset_id: Option<i64>,
    pub preset_name: String,
    pub filter_json: String,  // JSON字符串
    pub description: Option<String>,
    pub created_by: String,
    pub created_at: Option<String>,
    pub is_default: i64,  // 0 or 1
}

// ============================================
// Phase 5: 批量操作数据结构
// ============================================

/// 批量操作记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOperation {
    pub batch_id: Option<i64>,
    pub operation_type: String,  // 'adjust' 或 'restore'
    pub contract_count: i64,
    pub reason: String,
    pub user: String,
    pub created_at: Option<String>,
}

// ============================================
// 统一历史记录（整合所有变更）
// ============================================

/// 统一历史记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedHistoryEntry {
    pub id: String,                      // 唯一标识：类型前缀 + ID
    pub entry_type: String,              // config_change | alpha_adjust | batch_operation
    pub timestamp: String,               // 时间戳
    pub user: String,                    // 操作人
    pub description: String,             // 操作描述
    pub reason: Option<String>,          // 操作原因
    pub details: serde_json::Value,      // 详细信息（JSON）
}

// ============================================
// Phase 8: 清洗规则相关数据结构
// ============================================

/// 清洗规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRule {
    pub rule_id: Option<i64>,
    pub rule_name: String,
    pub category: String,              // standardization, extraction, normalization, mapping
    pub description: Option<String>,
    pub enabled: i64,                  // 0=禁用, 1=启用
    pub priority: i64,                 // 同类规则中的执行优先级
    pub config_json: String,           // JSON格式存储规则配置
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 清洗规则执行日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformExecutionLog {
    pub log_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name: Option<String>,     // 关联查询时填充
    pub execution_time: Option<String>,
    pub records_processed: i64,
    pub records_modified: i64,
    pub status: String,                // success, failed, partial
    pub error_message: Option<String>,
    pub executed_by: String,
}

/// 清洗规则变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRuleChangeLog {
    pub change_id: Option<i64>,
    pub rule_id: i64,
    pub rule_name: Option<String>,     // 关联查询时填充
    pub change_type: String,           // create, update, delete, enable, disable
    pub old_value: Option<String>,     // JSON格式的旧配置
    pub new_value: Option<String>,     // JSON格式的新配置
    pub change_reason: Option<String>,
    pub changed_by: String,
    pub changed_at: Option<String>,
}

/// 规则测试结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleTestResult {
    pub success: bool,
    pub input_sample: serde_json::Value,
    pub output_sample: serde_json::Value,
    pub records_matched: i64,
    pub error_message: Option<String>,
}

// ============================================
// Phase 9: 规格族管理相关数据结构
// ============================================

/// 规格族主数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFamily {
    pub family_id: Option<i64>,
    pub family_name: String,
    pub family_code: String,
    pub description: Option<String>,
    pub factor: f64,                    // P-Score 系数
    pub steel_grades: Option<String>,   // JSON数组格式
    pub thickness_min: Option<f64>,
    pub thickness_max: Option<f64>,
    pub width_min: Option<f64>,
    pub width_max: Option<f64>,
    pub enabled: i64,                   // 0=禁用, 1=启用
    pub sort_order: i64,
    pub created_by: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// 规格族变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecFamilyChangeLog {
    pub change_id: Option<i64>,
    pub family_id: i64,
    pub family_name: Option<String>,
    pub change_type: String,            // create, update, delete, enable, disable
    pub old_value: Option<String>,      // JSON格式
    pub new_value: Option<String>,      // JSON格式
    pub change_reason: Option<String>,
    pub changed_by: String,
    pub changed_at: Option<String>,
}

// ============================================
// Phase 11: 评分透明化 - Explain 数据结构
// ============================================

/// S-Score 子项详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SScoreComponent {
    /// 子项名称
    pub name: String,
    /// 原始输入值
    pub input_value: String,
    /// 计算后的分数（0-100）
    pub score: f64,
    /// 权重
    pub weight: f64,
    /// 加权后贡献值 = score × weight
    pub contribution: f64,
    /// 计算规则说明
    pub rule_description: String,
}

/// S-Score 评分详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SScoreExplain {
    /// S1: 客户等级评分
    pub s1_customer_level: SScoreComponent,
    /// S2: 毛利评分
    pub s2_margin: SScoreComponent,
    /// S3: 紧急度评分
    pub s3_urgency: SScoreComponent,
    /// S-Score 总分 = S1×w1 + S2×w2 + S3×w3
    pub total_score: f64,
    /// 验证：各项贡献之和是否等于总分
    pub verification_passed: bool,
}

/// P-Score 子项详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PScoreComponent {
    /// 子项名称
    pub name: String,
    /// 原始输入值
    pub input_value: String,
    /// 计算后的分数（0-100）
    pub score: f64,
    /// 权重
    pub weight: f64,
    /// 加权后贡献值 = score × weight
    pub contribution: f64,
    /// 计算规则说明
    pub rule_description: String,
}

/// P-Score 评分详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PScoreExplain {
    /// P1: 工艺难度评分
    pub p1_difficulty: PScoreComponent,
    /// P2: 聚合度评分
    pub p2_aggregation: PScoreComponent,
    /// P3: 节拍匹配评分
    pub p3_rhythm: PScoreComponent,
    /// P-Score 总分 = P1×w_p1 + P2×w_p2 + P3×w_p3
    pub total_score: f64,
    /// 验证：各项贡献之和是否等于总分
    pub verification_passed: bool,
}

/// 综合优先级详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityExplain {
    /// 合同 ID
    pub contract_id: String,
    /// 策略名称
    pub strategy_name: String,

    /// S-Score 详情
    pub s_score_explain: SScoreExplain,
    /// P-Score 详情
    pub p_score_explain: PScoreExplain,

    /// S-Score 总分
    pub s_score: f64,
    /// P-Score 总分
    pub p_score: f64,
    /// S-Score 权重 (ws)
    pub ws: f64,
    /// P-Score 权重 (wp)
    pub wp: f64,

    /// 基础优先级 = ws × S + wp × P
    pub base_priority: f64,
    /// Alpha 调整系数（None 表示无调整，等同于 1.0）
    pub alpha: Option<f64>,
    /// 最终优先级 = base_priority × alpha
    pub final_priority: f64,

    /// 计算公式汇总
    pub formula_summary: String,
    /// 验证：所有计算是否一致
    pub all_verifications_passed: bool,
}

// ============================================
// Phase 12: 聚合度配置相关数据结构
// ============================================

/// 聚合区间配置
///
/// # 用途
/// 定义宽度段和厚度段的分界点，支持灵活配置。
/// 用于将合同的宽度/厚度映射到对应的区间代码。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationBin {
    pub bin_id: Option<i64>,
    /// 维度类型：'width' 或 'thickness'
    pub dimension: String,
    /// 区间名称，如 '薄规格', '窄幅'
    pub bin_name: String,
    /// 区间代码，如 'THIN', 'NARROW'
    pub bin_code: String,
    /// 区间下限（含）
    pub min_value: f64,
    /// 区间上限（不含，最后一档用 999999）
    pub max_value: f64,
    /// 排序顺序
    pub sort_order: i64,
    /// 是否启用
    pub enabled: i64,
    /// 描述说明
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// P2 曲线参数配置
///
/// # 用途
/// 配置 P2 聚合度的计算参数，支持对数曲线、线性、阶梯等计算方式。
///
/// # 计算公式
/// - 对数曲线：`聚合度分数 = min(max_score, log_scale × ln(count + 1))`
/// - 加权组合：`P2 = α × 聚合度分数 + β × factor归一化分数`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2CurveConfig {
    pub config_id: i64,
    /// 曲线类型：'logarithmic' | 'linear' | 'step'
    pub curve_type: String,
    /// 对数底数（默认自然对数 e）
    pub log_base: f64,
    /// 缩放系数，控制曲线斜率
    pub log_scale: f64,
    /// 最低分
    pub min_score: f64,
    /// 最高分
    pub max_score: f64,
    /// 达到满分所需的最小数量
    pub min_count_for_max: i64,
    /// 聚合度分数权重（α）
    pub alpha: f64,
    /// factor归一化分数权重（β）
    pub beta: f64,
    /// 描述
    pub description: Option<String>,
    pub updated_by: Option<String>,
    pub updated_at: Option<String>,
}

impl Default for P2CurveConfig {
    fn default() -> Self {
        Self {
            config_id: 1,
            curve_type: "logarithmic".to_string(),
            log_base: std::f64::consts::E,
            log_scale: 25.0,
            min_score: 0.0,
            max_score: 100.0,
            min_count_for_max: 50,
            alpha: 0.7,
            beta: 0.3,
            description: None,
            updated_by: None,
            updated_at: None,
        }
    }
}

/// 聚合统计缓存
///
/// # 用途
/// 缓存合同池中各聚合键的统计信息，避免每次计算都全表扫描。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStatsCache {
    pub cache_id: Option<i64>,
    /// 聚合键：spec_family|steel_grade|thickness_bin|width_bin
    pub aggregation_key: String,
    pub spec_family: String,
    pub steel_grade: String,
    /// 厚度段代码
    pub thickness_bin: String,
    /// 宽度段代码
    pub width_bin: String,
    /// 同类合同数量
    pub contract_count: i64,
    /// 合同ID列表（JSON数组）
    pub contract_ids: Option<String>,
    pub last_updated: Option<String>,
}

/// 聚合统计结果（用于 P2 计算）
///
/// # 用途
/// 返回单个合同的聚合统计信息，用于 P2 评分计算。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregationStats {
    /// 聚合键
    pub aggregation_key: String,
    /// 规格族
    pub spec_family: String,
    /// 钢种
    pub steel_grade: String,
    /// 厚度段代码
    pub thickness_bin_code: String,
    /// 厚度段名称
    pub thickness_bin_name: String,
    /// 宽度段代码
    pub width_bin_code: String,
    /// 宽度段名称
    pub width_bin_name: String,
    /// 同类合同数量
    pub contract_count: i64,
}

/// P2 聚合度详情（用于 Explain 展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2AggregationDetail {
    /// 聚合键
    pub aggregation_key: String,
    /// 规格族
    pub spec_family: String,
    /// 钢种
    pub steel_grade: String,
    /// 厚度段
    pub thickness_bin: String,
    /// 宽度段
    pub width_bin: String,
    /// 同类合同数量
    pub contract_count: i64,
    /// 聚合度分数（对数曲线计算结果）
    pub aggregation_score: f64,
    /// 规格族 factor 值
    pub factor_value: f64,
    /// factor 归一化分数
    pub factor_score: f64,
    /// α（聚合度权重）
    pub alpha: f64,
    /// β（factor权重）
    pub beta: f64,
    /// 最终 P2 分数 = α×聚合度分数 + β×factor分数
    pub final_p2_score: f64,
    /// 计算公式说明
    pub formula: String,
}

// ============================================
// Phase 13: 数据校验与缺失值策略
// ============================================

/// 校验问题严重程度
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    /// 错误：无法计算，必须修复
    Error,
    /// 警告：使用默认值计算，建议修复
    Warning,
    /// 信息：仅提示，不影响计算
    Info,
}

impl Default for ValidationSeverity {
    fn default() -> Self {
        ValidationSeverity::Info
    }
}

/// 校验问题类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationIssueType {
    /// 字段缺失
    Missing,
    /// 值无效
    Invalid,
    /// 值超出范围
    OutOfRange,
    /// 格式错误
    FormatError,
    /// 关联数据不存在
    ReferenceNotFound,
}

/// 缺失值处理策略
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MissingValueAction {
    /// 使用默认值
    UseDefault,
    /// 跳过该合同
    Skip,
    /// 阻断计算
    Error,
}

/// 缺失值策略配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissingValueStrategy {
    pub strategy_id: Option<i64>,
    /// 字段名
    pub field_name: String,
    /// 字段中文标签
    pub field_label: String,
    /// 所属模块：s_score, p_score, contract, customer
    pub module: String,
    /// 是否必填
    pub is_required: bool,
    /// 缺失时策略
    pub strategy: String,
    /// 默认值（JSON格式）
    pub default_value: Option<String>,
    /// 默认值说明
    pub default_description: Option<String>,
    /// 影响的评分项
    pub affects_score: Option<String>,
    /// 字段描述
    pub description: Option<String>,
    /// 排序
    pub sort_order: i64,
}

/// 单个校验问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// 字段名
    pub field_name: String,
    /// 字段中文标签
    pub field_label: String,
    /// 问题类型
    pub issue_type: ValidationIssueType,
    /// 严重程度
    pub severity: ValidationSeverity,
    /// 原始值（如果有）
    pub original_value: Option<String>,
    /// 使用的默认值
    pub default_value_used: Option<String>,
    /// 问题描述
    pub message: String,
    /// 建议修复方案
    pub suggested_fix: Option<String>,
}

/// 合同校验结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractValidationResult {
    /// 合同编号
    pub contract_id: String,
    /// 是否可计算（无阻断性错误）
    pub can_calculate: bool,
    /// 校验状态：valid, warning, error
    pub status: String,
    /// 校验问题列表
    pub issues: Vec<ValidationIssue>,
    /// 错误数
    pub error_count: i64,
    /// 警告数
    pub warning_count: i64,
}

impl ContractValidationResult {
    /// 创建一个有效的校验结果（无问题）
    #[allow(dead_code)]
    pub fn valid(contract_id: String) -> Self {
        Self {
            contract_id,
            can_calculate: true,
            status: "valid".to_string(),
            issues: vec![],
            error_count: 0,
            warning_count: 0,
        }
    }
}

/// 批量校验汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// 校验批次ID
    pub batch_id: String,
    /// 校验时间
    pub validation_time: String,
    /// 总合同数
    pub total_contracts: i64,
    /// 完全有效的合同数
    pub valid_contracts: i64,
    /// 有警告但可计算的合同数
    pub warning_contracts: i64,
    /// 有错误无法计算的合同数
    pub error_contracts: i64,
    /// 按字段统计的问题数
    pub issues_by_field: std::collections::HashMap<String, i64>,
}

/// 数据质量报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityReport {
    /// 校验汇总
    pub summary: ValidationSummary,
    /// 各字段问题详情
    pub field_issues: Vec<FieldIssueDetail>,
    /// 问题合同列表（仅包含有问题的合同）
    pub problem_contracts: Vec<ContractValidationResult>,
    /// 修复建议
    pub recommendations: Vec<String>,
}

/// 字段问题详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldIssueDetail {
    /// 字段名
    pub field_name: String,
    /// 字段中文标签
    pub field_label: String,
    /// 影响的评分项
    pub affects_score: Option<String>,
    /// 缺失/问题数量
    pub issue_count: i64,
    /// 使用的默认值
    pub default_value: Option<String>,
    /// 默认值说明
    pub default_description: Option<String>,
    /// 受影响的合同ID列表
    pub affected_contract_ids: Vec<String>,
}

/// 带校验状态的合同优先级结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractPriorityWithValidation {
    /// 原有的优先级结果
    #[serde(flatten)]
    pub priority_result: ContractPriority,
    /// 校验状态：valid, warning, error
    pub validation_status: String,
    /// 警告信息列表
    pub warnings: Vec<String>,
    /// 是否使用了默认值
    pub used_defaults: bool,
    /// 使用的默认值详情
    pub default_values_used: Vec<DefaultValueUsed>,
}

/// 使用的默认值详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultValueUsed {
    /// 字段名
    pub field_name: String,
    /// 字段标签
    pub field_label: String,
    /// 默认值
    pub default_value: String,
    /// 说明
    pub description: String,
}

// ============================================
// Phase 14: 策略版本化（可回放、可复盘）
// ============================================

/// 策略版本快照
///
/// # 用途
/// 记录策略参数的完整快照，确保历史计算可复现。
/// 每次策略参数变更时自动创建新版本。
///
/// # 设计原则
/// - **不可变性**：版本创建后参数不可修改
/// - **完整性**：包含计算所需的所有配置参数
/// - **可追溯**：记录创建人和变更原因
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersion {
    pub version_id: Option<i64>,
    /// 版本序号（同策略内递增）
    pub version_number: i64,
    /// 版本标签（如 v1.0.0, 2024Q1）
    pub version_tag: Option<String>,
    /// 策略名称
    pub strategy_name: String,

    // 策略权重快照（strategy_weights）
    /// S-Score 权重
    pub ws: f64,
    /// P-Score 权重
    pub wp: f64,

    // S-Score 子权重快照（strategy_scoring_weights）
    /// 客户等级权重
    pub w1: f64,
    /// 毛利权重
    pub w2: f64,
    /// 紧急度权重
    pub w3: f64,

    // P-Score 子权重
    /// 工艺难度权重
    pub w_p1: f64,
    /// 聚合度权重
    pub w_p2: f64,
    /// 节拍匹配权重
    pub w_p3: f64,

    // 完整配置快照（JSON 格式）
    /// scoring_config 表的完整快照
    pub scoring_config_snapshot: String,
    /// P2 曲线配置快照
    pub p2_curve_config_snapshot: Option<String>,
    /// 聚合区间配置快照
    pub aggregation_bins_snapshot: Option<String>,
    /// 节拍配置快照
    pub rhythm_config_snapshot: Option<String>,

    // 版本元数据
    /// 版本描述
    pub description: Option<String>,
    /// 变更原因
    pub change_reason: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,

    // 版本状态
    /// 是否为当前激活版本
    pub is_active: i64,
    /// 是否锁定（锁定后不可删除）
    pub is_locked: i64,
}

/// 沙盘计算会话
///
/// # 用途
/// 记录每次沙盘计算的上下文，确保结果可复现。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxSession {
    pub session_id: Option<i64>,
    /// 会话名称（如"2024年12月排产预演"）
    pub session_name: String,
    /// 类型：sandbox（沙盘）、production（生产）
    pub session_type: String,

    /// 使用的策略版本 ID
    pub strategy_version_id: i64,
    /// 合同池快照时间点
    pub contract_snapshot_time: Option<String>,

    /// 会话状态：draft, running, completed, archived
    pub status: String,

    /// 参与计算的合同数
    pub total_contracts: Option<i64>,
    /// 结果摘要 JSON
    pub result_summary: Option<String>,

    /// 描述
    pub description: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 完成时间
    pub completed_at: Option<String>,
}

/// 沙盘计算结果
///
/// # 用途
/// 存储每次计算的详细结果，支持历史对比。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    pub result_id: Option<i64>,
    pub session_id: i64,

    /// 合同编号
    pub contract_id: String,
    /// 合同数据快照 JSON
    pub contract_snapshot: String,
    /// 客户数据快照 JSON
    pub customer_snapshot: Option<String>,

    /// S-Score
    pub s_score: f64,
    /// P-Score
    pub p_score: f64,
    /// 最终优先级
    pub priority: f64,
    /// Alpha 调整系数
    pub alpha: Option<f64>,

    /// S1: 客户等级分
    pub s1_score: Option<f64>,
    /// S2: 毛利分
    pub s2_score: Option<f64>,
    /// S3: 紧急度分
    pub s3_score: Option<f64>,
    /// P1: 工艺难度分
    pub p1_score: Option<f64>,
    /// P2: 聚合度分
    pub p2_score: Option<f64>,
    /// P3: 节拍匹配分
    pub p3_score: Option<f64>,

    /// 聚合键
    pub aggregation_key: Option<String>,
    /// 聚合数量
    pub aggregation_count: Option<i64>,

    /// 在本次会话中的排名
    pub priority_rank: Option<i64>,
}

/// 版本对比记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparison {
    pub comparison_id: Option<i64>,
    /// 版本 A 的 ID
    pub version_a_id: i64,
    /// 版本 B 的 ID
    pub version_b_id: i64,

    /// 对比的合同数
    pub contracts_compared: i64,
    /// 排名变化的合同数
    pub rank_changes: i64,
    /// 平均优先级差异
    pub avg_priority_diff: f64,
    /// 最大排名变化
    pub max_rank_change: i64,

    /// 详细对比结果 JSON
    pub comparison_details: Option<String>,

    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 版本对比详情项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionComparisonItem {
    /// 合同编号
    pub contract_id: String,
    /// 版本 A 的优先级
    pub priority_a: f64,
    /// 版本 B 的优先级
    pub priority_b: f64,
    /// 优先级差异
    pub priority_diff: f64,
    /// 版本 A 的排名
    pub rank_a: i64,
    /// 版本 B 的排名
    pub rank_b: i64,
    /// 排名变化（正数表示上升）
    pub rank_change: i64,
}

/// 策略版本变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersionChangeLog {
    pub log_id: Option<i64>,
    pub version_id: i64,
    /// 变更类型：create, activate, deactivate, lock, unlock
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_reason: Option<String>,
    pub changed_by: String,
    pub changed_at: Option<String>,
}

/// 策略版本简要信息（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyVersionSummary {
    pub version_id: i64,
    pub version_number: i64,
    pub version_tag: Option<String>,
    pub strategy_name: String,
    pub ws: f64,
    pub wp: f64,
    pub is_active: bool,
    pub is_locked: bool,
    pub created_by: String,
    pub created_at: String,
    pub description: Option<String>,
}

// ============================================
// Phase 15: 导入/清洗冲突解决机制产品化
// ============================================

/// 导入审计日志
///
/// # 用途
/// 记录每次导入操作的完整信息，用于责任追溯和复盘。
/// 每次导入创建一条审计记录，包含统计信息和处理结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportAuditLog {
    pub audit_id: Option<i64>,
    /// 导入类型：contracts, customers, process_difficulty
    pub import_type: String,
    /// 原始文件名
    pub file_name: String,
    /// 文件格式：csv, json, excel
    pub file_format: String,
    /// 文件哈希值（用于重复导入检测）
    pub file_hash: Option<String>,
    /// 文件大小（字节）
    pub file_size: Option<i64>,

    // 统计信息
    /// 文件总行数
    pub total_rows: i64,
    /// 有效行数
    pub valid_rows: i64,
    /// 错误行数
    pub error_rows: i64,
    /// 冲突行数
    pub conflict_rows: i64,

    // 处理结果
    /// 实际导入数
    pub imported_count: i64,
    /// 更新数（覆盖导入）
    pub updated_count: i64,
    /// 跳过数
    pub skipped_count: i64,

    /// 冲突处理策略：skip, overwrite, manual
    pub conflict_strategy: String,

    /// 执行状态：pending, running, success, partial, failed
    pub status: String,
    /// 失败时的错误信息
    pub error_message: Option<String>,

    /// 应用的清洗规则ID列表（JSON数组）
    pub applied_transform_rules: Option<String>,

    /// 导入人
    pub imported_by: String,
    /// 开始时间
    pub started_at: Option<String>,
    /// 完成时间
    pub completed_at: Option<String>,

    /// 校验错误（JSON数组）
    pub validation_errors: Option<String>,
    /// 校验警告（JSON数组）
    pub validation_warnings: Option<String>,
}

/// 导入冲突明细
///
/// # 用途
/// 记录每条冲突数据的详细信息和处理决策，
/// 确保每条冲突都有完整的决策记录。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportConflictLog {
    pub conflict_id: Option<i64>,
    /// 关联的导入审计ID
    pub audit_id: i64,
    /// 文件中的行号
    pub row_number: i64,
    /// 主键值
    pub primary_key: String,
    /// 数据库中已有的数据（JSON）
    pub existing_data: String,
    /// 导入文件中的新数据（JSON）
    pub new_data: String,
    /// 变更的字段列表（JSON数组）
    pub changed_fields: Option<String>,
    /// 字段级别的差异对比（JSON对象）
    pub field_diffs: Option<String>,
    /// 处理决策：pending, skip, overwrite
    pub action: String,
    /// 决策原因
    pub action_reason: Option<String>,
    /// 决策人（manual 模式下）
    pub decided_by: Option<String>,
    /// 决策时间
    pub decided_at: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 字段对齐规则
///
/// # 用途
/// 定义不同数据源的字段名映射规则，
/// 解决字段名不统一的问题。
///
/// # 示例
/// 将 Excel 中的"合同号"映射到数据库的 contract_id
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAlignmentRule {
    pub rule_id: Option<i64>,
    /// 规则名称
    pub rule_name: String,
    /// 数据类型：contracts, customers, process_difficulty
    pub data_type: String,
    /// 数据来源类型（如 ERP, Excel模板, 第三方）
    pub source_type: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 是否启用
    pub enabled: i64,
    /// 优先级
    pub priority: i64,
    /// 字段映射（JSON格式）
    /// 格式: {"target_field": ["source_field1", "source_field2", ...]}
    pub field_mapping: String,
    /// 值转换规则（JSON格式）
    pub value_transform: Option<String>,
    /// 默认值填充（JSON格式）
    pub default_values: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 字段对齐规则变更日志
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAlignmentChangeLog {
    pub log_id: Option<i64>,
    pub rule_id: i64,
    /// 变更类型：create, update, delete, enable, disable
    pub change_type: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_reason: Option<String>,
    pub changed_by: String,
    pub changed_at: Option<String>,
}

/// 重复检测配置
///
/// # 用途
/// 定义如何判断数据是否重复，支持复合主键和模糊匹配。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuplicateDetectionConfig {
    pub config_id: Option<i64>,
    /// 配置名称
    pub config_name: String,
    /// 数据类型：contracts, customers, process_difficulty
    pub data_type: String,
    /// 描述
    pub description: Option<String>,
    /// 是否激活
    pub is_active: i64,
    /// 主键字段列表（JSON数组）
    pub primary_key_fields: String,
    /// 模糊匹配字段（JSON数组）
    pub fuzzy_match_fields: Option<String>,
    /// 模糊匹配阈值 (0-1)
    pub fuzzy_threshold: Option<f64>,
    /// 时间字段名
    pub time_field: Option<String>,
    /// 时间窗口天数
    pub time_window_days: Option<i64>,
    /// 业务规则（JSON数组）
    pub business_rules: Option<String>,
    /// 默认处理策略：warn, skip, merge, overwrite
    pub default_action: String,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 导入历史快照
///
/// # 用途
/// 保存导入前的数据快照，支持回滚操作。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSnapshot {
    pub snapshot_id: Option<i64>,
    pub audit_id: i64,
    pub data_type: String,
    /// 被影响的记录主键
    pub primary_key: String,
    /// 操作类型：insert, update, delete
    pub action_type: String,
    /// 操作前数据（JSON）
    pub before_data: Option<String>,
    /// 操作后数据（JSON）
    pub after_data: Option<String>,
    pub created_at: Option<String>,
}

/// 相似记录关联
///
/// # 用途
/// 记录疑似重复的记录对，用于人工确认。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarRecordPair {
    pub pair_id: Option<i64>,
    pub audit_id: Option<i64>,
    pub data_type: String,
    /// 记录A的主键
    pub record_a_key: String,
    /// 记录B的主键
    pub record_b_key: String,
    /// 综合相似度 (0-1)
    pub similarity_score: f64,
    /// 各字段匹配详情（JSON对象）
    pub matching_fields: Option<String>,
    /// 处理状态：pending, confirmed_same, confirmed_diff, merged, ignored
    pub status: String,
    /// 处理人
    pub resolved_by: Option<String>,
    /// 处理时间
    pub resolved_at: Option<String>,
    /// 处理备注
    pub resolution_note: Option<String>,
    pub created_at: Option<String>,
}

/// 导入统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportStatistics {
    pub import_type: String,
    pub total_imports: i64,
    pub successful_imports: i64,
    pub failed_imports: i64,
    pub total_rows_processed: i64,
    pub total_imported: i64,
    pub total_updated: i64,
    pub total_skipped: i64,
    pub total_conflicts: i64,
    pub last_import_time: Option<String>,
}

/// 冲突详细信息（带差异对比）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictDetail {
    pub conflict_id: i64,
    pub audit_id: i64,
    pub import_type: String,
    pub file_name: String,
    pub row_number: i64,
    pub primary_key: String,
    /// 已有数据
    pub existing_data: serde_json::Value,
    /// 新数据
    pub new_data: serde_json::Value,
    /// 变更字段列表
    pub changed_fields: Vec<String>,
    /// 字段差异对比
    pub field_diffs: Vec<FieldDiff>,
    pub action: String,
    pub imported_by: String,
    pub created_at: String,
}

/// 单个字段的差异
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDiff {
    /// 字段名
    pub field_name: String,
    /// 字段标签
    pub field_label: String,
    /// 旧值
    pub old_value: serde_json::Value,
    /// 新值
    pub new_value: serde_json::Value,
    /// 差异类型：changed, added, removed
    pub diff_type: String,
}

/// 增强的导入预览（含审计信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreviewEnhanced {
    /// 文件信息
    pub file_name: String,
    pub file_format: String,
    pub file_hash: String,
    pub file_size: i64,

    /// 统计信息
    pub total_rows: i64,
    pub valid_rows: i64,
    pub error_rows: i64,
    pub conflict_rows: i64,

    /// 字段映射结果
    pub field_mapping_applied: bool,
    pub mapped_fields: Vec<FieldMappingResult>,
    pub unmapped_source_fields: Vec<String>,

    /// 冲突详情
    pub conflicts: Vec<ConflictDetail>,

    /// 校验错误
    pub validation_errors: Vec<ImportValidationError>,

    /// 样本数据
    pub sample_data: Vec<serde_json::Value>,

    /// 应用的规则
    pub applied_alignment_rules: Vec<String>,
    pub suggested_transform_rules: Vec<String>,
}

/// 字段映射结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// 目标字段名
    pub target_field: String,
    /// 源文件中的字段名
    pub source_field: String,
    /// 映射来源规则
    pub rule_name: String,
}

/// 导入校验错误（增强版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportValidationError {
    pub row_number: i64,
    pub field: String,
    pub field_label: String,
    pub value: String,
    pub error_type: String,
    pub message: String,
    pub suggested_fix: Option<String>,
}

/// 增强的导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResultEnhanced {
    /// 审计ID
    pub audit_id: i64,
    /// 是否成功
    pub success: bool,
    /// 处理结果
    pub imported_count: i64,
    pub updated_count: i64,
    pub skipped_count: i64,
    pub error_count: i64,
    /// 冲突处理结果
    pub conflicts_resolved: i64,
    pub conflicts_skipped: i64,
    /// 错误详情
    pub errors: Vec<ImportValidationError>,
    /// 结果消息
    pub message: String,
    /// 是否可回滚
    pub can_rollback: bool,
}

// ============================================
// Phase 16: 会议驾驶舱 KPI 固化
// Meeting Cockpit KPI Solidification
// ============================================

/// 会议快照
///
/// # 用途
/// 保存每次会议的完整状态快照，支持历史对比和复盘。
/// 是会议驾驶舱的核心数据结构。
///
/// # 设计原则
/// - **完整性**：包含 KPI 汇总、风险汇总、合同排名等完整信息
/// - **可追溯**：记录策略版本、审批状态等信息
/// - **灵活性**：使用 JSON 存储动态结构的数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSnapshot {
    pub snapshot_id: Option<i64>,
    /// 会议类型：production_sales（产销例会）, business（经营例会）
    pub meeting_type: String,
    /// 会议日期 YYYY-MM-DD
    pub meeting_date: String,
    /// 快照名称（如：2024-01-15产销例会）
    pub snapshot_name: String,
    /// 关联的策略版本 ID
    pub strategy_version_id: Option<i64>,
    /// 策略名称（冗余存储便于查询）
    pub strategy_name: Option<String>,
    /// KPI 汇总（JSON 格式）
    /// 格式: {"leadership": [...], "sales": [...], "production": [...], "finance": [...]}
    pub kpi_summary: String,
    /// 风险汇总（JSON 格式）
    /// 格式: {"total_risk_count": N, "by_type": {...}, "top_risks": [...]}
    pub risk_summary: String,
    /// 策略推荐说明
    pub recommendation: Option<String>,
    /// 合同排名快照（JSON 格式）
    pub contract_rankings: String,
    /// 排名变化汇总（JSON 格式）
    pub ranking_changes: Option<String>,
    /// 共识状态：draft, pending_approval, approved, archived
    pub consensus_status: String,
    /// 审批人
    pub approved_by: Option<String>,
    /// 审批时间
    pub approved_at: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// KPI 指标配置
///
/// # 用途
/// 定义四视角（领导/销售/生产/财务）的 KPI 指标，
/// 支持阈值设置和红绿灯状态展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingKpiConfig {
    pub kpi_id: Option<i64>,
    /// 指标代码（唯一）
    pub kpi_code: String,
    /// 指标名称
    pub kpi_name: String,
    /// 指标分类：leadership, sales, production, finance
    pub kpi_category: String,
    /// 计算类型：count, sum, avg, ratio, custom
    pub calculation_type: String,
    /// 计算公式（SQL 或表达式）
    pub calculation_formula: Option<String>,
    /// 数据来源表/视图
    pub data_source: Option<String>,
    /// 筛选条件
    pub filter_condition: Option<String>,
    /// 显示格式：number, percent, currency, text
    pub display_format: String,
    /// 显示单位
    pub display_unit: Option<String>,
    /// 小数位数
    pub decimal_places: Option<i64>,
    /// 良好阈值（绿灯）
    pub threshold_good: Option<f64>,
    /// 警告阈值（黄灯）
    pub threshold_warning: Option<f64>,
    /// 危险阈值（红灯）
    pub threshold_danger: Option<f64>,
    /// 阈值方向：higher_better, lower_better
    pub threshold_direction: String,
    /// 排序顺序
    pub sort_order: i64,
    /// 是否启用
    pub enabled: i64,
    /// 指标说明
    pub description: Option<String>,
    /// 业务含义（给管理层看）
    pub business_meaning: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// KPI 计算值
///
/// # 用途
/// 存储单个 KPI 的计算结果，用于前端展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiValue {
    /// 指标代码
    pub kpi_code: String,
    /// 指标名称
    pub kpi_name: String,
    /// 计算值
    pub value: f64,
    /// 格式化后的显示值
    pub display_value: String,
    /// 状态：good, warning, danger, neutral
    pub status: String,
    /// 与上期对比变化
    pub change: Option<f64>,
    /// 变化方向：up, down, unchanged
    pub change_direction: Option<String>,
    /// 业务含义
    pub business_meaning: Option<String>,
}

/// KPI 汇总（四视角）
///
/// # 用途
/// 按四个视角组织 KPI，便于前端展示。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiSummary {
    /// 领导视角 KPI
    pub leadership: Vec<KpiValue>,
    /// 销售视角 KPI
    pub sales: Vec<KpiValue>,
    /// 生产/计划视角 KPI
    pub production: Vec<KpiValue>,
    /// 财务/经营视角 KPI
    pub finance: Vec<KpiValue>,
}

/// 风险合同标记
///
/// # 用途
/// 标记需要在会议上重点关注的风险合同。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskContractFlag {
    pub flag_id: Option<i64>,
    /// 关联的会议快照
    pub snapshot_id: Option<i64>,
    /// 合同 ID
    pub contract_id: String,
    /// 风险类型
    /// - delivery_delay: 交期延迟风险
    /// - customer_downgrade: 客户降级风险
    /// - margin_loss: 毛利损失风险
    /// - rhythm_mismatch: 节拍不匹配风险
    /// - capacity_conflict: 产能冲突风险
    /// - quality_concern: 质量关注风险
    pub risk_type: String,
    /// 风险等级：high, medium, low
    pub risk_level: String,
    /// 风险评分（0-100）
    pub risk_score: Option<f64>,
    /// 风险描述
    pub risk_description: String,
    /// 风险因素（JSON 数组）
    pub risk_factors: Option<String>,
    /// 受影响的 KPI（JSON 数组）
    pub affected_kpis: Option<String>,
    /// 潜在损失金额
    pub potential_loss: Option<f64>,
    /// 损失单位
    pub potential_loss_unit: Option<String>,
    /// 建议措施
    pub suggested_action: Option<String>,
    /// 措施优先级 1-3
    pub action_priority: Option<i64>,
    /// 处理状态：open, acknowledged, mitigated, closed
    pub status: String,
    /// 处理人
    pub handled_by: Option<String>,
    /// 处理时间
    pub handled_at: Option<String>,
    /// 处理备注
    pub handling_note: Option<String>,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 排名变化明细
///
/// # 用途
/// 记录合同排名变化及 Explain 拆解，
/// 支持详细分析排名变化的原因。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingChangeDetail {
    pub change_id: Option<i64>,
    /// 当前会议快照
    pub snapshot_id: i64,
    /// 对比的会议快照
    pub compare_snapshot_id: Option<i64>,
    /// 合同 ID
    pub contract_id: String,
    /// 旧排名
    pub old_rank: Option<i64>,
    /// 新排名
    pub new_rank: i64,
    /// 排名变化（正数上升，负数下降）
    pub rank_change: Option<i64>,
    /// 旧优先级
    pub old_priority: Option<f64>,
    /// 新优先级
    pub new_priority: f64,
    /// 优先级变化
    pub priority_change: Option<f64>,
    // S-Score 变化
    pub old_s_score: Option<f64>,
    pub new_s_score: f64,
    pub s_score_change: Option<f64>,
    // S-Score 子项变化
    pub s1_change: Option<f64>,
    pub s1_old: Option<f64>,
    pub s1_new: Option<f64>,
    pub s2_change: Option<f64>,
    pub s2_old: Option<f64>,
    pub s2_new: Option<f64>,
    pub s3_change: Option<f64>,
    pub s3_old: Option<f64>,
    pub s3_new: Option<f64>,
    // P-Score 变化
    pub old_p_score: Option<f64>,
    pub new_p_score: f64,
    pub p_score_change: Option<f64>,
    // P-Score 子项变化
    pub p1_change: Option<f64>,
    pub p1_old: Option<f64>,
    pub p1_new: Option<f64>,
    pub p2_change: Option<f64>,
    pub p2_old: Option<f64>,
    pub p2_new: Option<f64>,
    pub p3_change: Option<f64>,
    pub p3_old: Option<f64>,
    pub p3_new: Option<f64>,
    /// 主要变化因素代码
    /// - s1_customer: 客户等级变化
    /// - s2_margin: 毛利变化
    /// - s3_urgency: 紧急度变化
    /// - p1_difficulty: 工艺难度变化
    /// - p2_aggregation: 聚合度变化
    /// - p3_rhythm: 节拍变化
    /// - strategy_weight: 策略权重变化
    pub primary_factor: Option<String>,
    /// 主要变化因素名称（中文）
    pub primary_factor_name: Option<String>,
    /// 完整的变化解释文本
    pub explain_text: Option<String>,
    /// 使用的 S 权重
    pub ws_used: Option<f64>,
    /// 使用的 P 权重
    pub wp_used: Option<f64>,
    /// 创建时间
    pub created_at: Option<String>,
}

/// 排名变化解释（Explain）
///
/// # 用途
/// 提供排名变化的详细解释，帮助管理层理解变化原因。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingExplain {
    /// S-Score 变化
    pub s_score_change: f64,
    /// P-Score 变化
    pub p_score_change: f64,
    /// 客户等级分变化
    pub s1_change: f64,
    /// 毛利分变化
    pub s2_change: f64,
    /// 紧急度分变化
    pub s3_change: f64,
    /// 工艺难度分变化
    pub p1_change: f64,
    /// 聚合度分变化
    pub p2_change: f64,
    /// 节拍分变化
    pub p3_change: f64,
    /// 主要变化因素代码
    pub primary_factor: String,
    /// 主要变化因素名称（中文）
    pub primary_factor_name: String,
}

/// 共识包模板
///
/// # 用途
/// 定义会议输出的模板格式，支持 PDF/CSV 导出。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusTemplate {
    pub template_id: Option<i64>,
    /// 模板代码
    pub template_code: String,
    /// 模板名称
    pub template_name: String,
    /// 适用会议类型：production_sales, business, all
    pub meeting_type: String,
    /// 模板内容定义（JSON 格式）
    pub template_config: String,
    /// 支持的输出格式
    pub output_formats: String,
    /// 是否默认模板
    pub is_default: i64,
    /// 是否启用
    pub enabled: i64,
    /// 描述
    pub description: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 会议行动项
///
/// # 用途
/// 记录会议产生的行动项和跟踪状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingActionItem {
    pub action_id: Option<i64>,
    /// 关联的会议快照
    pub snapshot_id: i64,
    /// 行动项标题
    pub action_title: String,
    /// 详细描述
    pub action_description: Option<String>,
    /// 分类：strategy, risk, customer, production
    pub action_category: Option<String>,
    /// 优先级：1 高, 2 中, 3 低
    pub priority: i64,
    /// 截止日期
    pub due_date: Option<String>,
    /// 责任人
    pub assignee: Option<String>,
    /// 责任部门
    pub department: Option<String>,
    /// 关联的合同 ID（JSON 数组）
    pub related_contracts: Option<String>,
    /// 状态：open, in_progress, completed, cancelled
    pub status: String,
    /// 完成率 0-100
    pub completion_rate: Option<i64>,
    /// 完成时间
    pub completed_at: Option<String>,
    /// 备注
    pub notes: Option<String>,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: Option<String>,
    /// 更新时间
    pub updated_at: Option<String>,
}

/// 共识包数据结构
///
/// # 用途
/// 用于导出 PDF/CSV 的完整数据包，包含会议的所有关键信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusPackage {
    /// 会议信息
    pub meeting_info: MeetingInfo,
    /// 策略推荐
    pub strategy_recommendation: StrategyRecommendation,
    /// KPI 对比
    pub kpi_comparison: KpiComparison,
    /// 风险分析
    pub risk_analysis: RiskAnalysis,
    /// 排名亮点
    pub ranking_highlights: RankingHighlights,
    /// 行动项列表
    pub action_items: Vec<MeetingActionItem>,
}

/// 会议信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingInfo {
    /// 会议类型名称
    pub meeting_type_name: String,
    /// 会议日期
    pub meeting_date: String,
    /// 使用的策略
    pub strategy_name: String,
    /// 策略版本
    pub strategy_version: String,
    /// 合同总数
    pub total_contracts: i64,
    /// 创建人
    pub created_by: String,
    /// 创建时间
    pub created_at: String,
}

/// 策略推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyRecommendation {
    /// 推荐策略
    pub recommended_strategy: String,
    /// 推荐理由
    pub recommendation_reason: String,
    /// 预期收益
    pub expected_benefits: Vec<String>,
    /// 潜在风险
    pub potential_risks: Vec<String>,
}

/// KPI 对比
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiComparison {
    /// 当前值
    pub current_values: KpiSummary,
    /// 对比期值（可选）
    pub comparison_values: Option<KpiSummary>,
    /// 变化汇总
    pub change_summary: String,
}

/// 风险分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAnalysis {
    /// 风险合同总数
    pub total_risk_count: i64,
    /// 高风险数量
    pub high_risk_count: i64,
    /// 中风险数量
    pub medium_risk_count: i64,
    /// 低风险数量
    pub low_risk_count: i64,
    /// 按类型统计
    pub by_type: std::collections::HashMap<String, i64>,
    /// Top 风险合同
    pub top_risks: Vec<RiskContractFlag>,
    /// 总潜在损失
    pub total_potential_loss: f64,
}

/// 排名亮点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingHighlights {
    /// 总变化合同数
    pub total_changes: i64,
    /// 上升合同数
    pub up_count: i64,
    /// 下降合同数
    pub down_count: i64,
    /// 平均变化幅度
    pub avg_change: f64,
    /// 最大上升
    pub max_up: i64,
    /// 最大下降
    pub max_down: i64,
    /// Top 上升合同
    pub top_up_contracts: Vec<RankingChangeDetail>,
    /// Top 下降合同
    pub top_down_contracts: Vec<RankingChangeDetail>,
}

/// 会议快照汇总（用于列表展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeetingSnapshotSummary {
    pub snapshot_id: i64,
    pub meeting_type: String,
    pub meeting_date: String,
    pub snapshot_name: String,
    pub strategy_name: Option<String>,
    pub strategy_version_number: Option<i64>,
    pub consensus_status: String,
    pub created_by: String,
    pub created_at: String,
    pub risk_count: i64,
    pub change_count: i64,
    pub action_count: i64,
}
