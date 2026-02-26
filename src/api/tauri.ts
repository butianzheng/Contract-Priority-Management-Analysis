import { invoke } from "@tauri-apps/api";

// 数据类型定义
export interface Contract {
  contract_id: string;
  customer_id: string;
  steel_grade: string;
  thickness: number;
  width: number;
  spec_family: string;
  pdd: string;
  days_to_pdd: number;
  margin: number;
}

export interface ContractPriority extends Contract {
  s_score: number;
  p_score: number;
  priority: number;
  alpha?: number;
}

export interface InterventionLog {
  id?: number;
  contract_id: string;
  alpha_value: number;
  reason: string;
  user: string;
  timestamp?: string;
}

// ============================================
// Phase 2: 配置管理相关数据类型
// ============================================

export interface ScoringConfigItem {
  config_key: string;
  config_value: string;
  value_type: string;      // number, string, json_array, json_object
  category: string;         // s_score, p_score, general
  description?: string;
  default_value?: string;
  min_value?: number;
  max_value?: number;
}

export interface ConfigChangeLog {
  id?: number;
  config_key: string;
  old_value: string;
  new_value: string;
  change_reason?: string;
  changed_by: string;
  changed_at?: string;
}

export interface StrategyScoringWeights {
  strategy_name: string;
  w1: number;  // 客户等级权重
  w2: number;  // 毛利权重
  w3: number;  // 紧急度权重
}

export interface StrategyWeights {
  strategy_name: string;
  ws: number;  // S-Score 权重
  wp: number;  // P-Score 权重
  description?: string;
}

// ============================================
// Phase 4: 筛选器相关数据类型
// ============================================

export interface FilterPreset {
  preset_id?: number;
  preset_name: string;
  filter_json: string;  // JSON字符串
  description?: string;
  created_by: string;
  created_at?: string;
  is_default: number;   // 0 or 1
}

export interface FilterCriteria {
  // 合同编号
  contract_id?: string;

  // 客户筛选
  customer_ids?: string[];

  // 钢种筛选
  steel_grades?: string[];

  // 规格族筛选
  spec_families?: string[];

  // 厚度范围
  thickness_min?: number;
  thickness_max?: number;

  // 宽度范围
  width_min?: number;
  width_max?: number;

  // 交期剩余天数范围
  days_to_pdd_min?: number;
  days_to_pdd_max?: number;

  // S分数范围
  s_score_min?: number;
  s_score_max?: number;

  // P分数范围
  p_score_min?: number;
  p_score_max?: number;

  // 优先级范围
  priority_min?: number;
  priority_max?: number;

  // 是否有人工调整
  has_alpha?: boolean;
}

// ============================================
// Phase 5: 批量操作相关数据类型
// ============================================

export interface BatchOperation {
  batch_id?: number;
  operation_type: string;  // 'adjust' 或 'restore'
  contract_count: number;
  reason: string;
  user: string;
  created_at?: string;
}

// ============================================
// 统一历史记录（整合所有变更）
// ============================================

export interface UnifiedHistoryEntry {
  id: string;                      // 唯一标识：类型前缀 + ID
  entry_type: string;              // config_change | alpha_adjust | batch_operation
  timestamp: string;               // 时间戳
  user: string;                    // 操作人
  description: string;             // 操作描述
  reason?: string;                 // 操作原因
  details: any;                    // 详细信息（JSON）
}

// ============================================
// 导入/导出相关数据类型
// ============================================

export type FileFormat = 'csv' | 'json' | 'excel';

// ============================================
// Phase 8: 清洗规则相关数据类型
// ============================================

export type RuleCategory = 'standardization' | 'extraction' | 'normalization' | 'mapping';

export interface TransformRule {
  rule_id?: number;
  rule_name: string;
  category: RuleCategory;
  description?: string;
  enabled: number;           // 0=禁用, 1=启用
  priority: number;
  config_json: string;       // JSON格式存储规则配置
  created_by: string;
  created_at?: string;
  updated_at?: string;
}

export interface TransformRuleChangeLog {
  change_id?: number;
  rule_id: number;
  rule_name?: string;
  change_type: string;       // create, update, delete, enable, disable
  old_value?: string;
  new_value?: string;
  change_reason?: string;
  changed_by: string;
  changed_at?: string;
}

export interface TransformExecutionLog {
  log_id?: number;
  rule_id: number;
  rule_name?: string;
  execution_time?: string;
  records_processed: number;
  records_modified: number;
  status: string;            // success, failed, partial
  error_message?: string;
  executed_by: string;
}

export interface RuleTestResult {
  success: boolean;
  input_sample: any;
  output_sample: any;
  records_matched: number;
  error_message?: string;
}

// ============================================
// Phase 9: 规格族管理相关数据类型
// ============================================

export interface SpecFamily {
  family_id?: number;
  family_name: string;
  family_code: string;
  description?: string;
  factor: number;                    // P-Score 系数
  steel_grades?: string;             // JSON数组格式
  thickness_min?: number;
  thickness_max?: number;
  width_min?: number;
  width_max?: number;
  enabled: number;                   // 0=禁用, 1=启用
  sort_order: number;
  created_by: string;
  created_at?: string;
  updated_at?: string;
}

export interface SpecFamilyChangeLog {
  change_id?: number;
  family_id: number;
  family_name?: string;
  change_type: string;               // create, update, delete, enable, disable
  old_value?: string;                // JSON格式
  new_value?: string;                // JSON格式
  change_reason?: string;
  changed_by: string;
  changed_at?: string;
}

// ============================================
// Phase 11: 评分透明化 - Explain 数据类型
// ============================================

/** S-Score 子项详情 */
export interface SScoreComponent {
  name: string;
  input_value: string;
  score: number;
  weight: number;
  contribution: number;
  rule_description: string;
}

/** S-Score 评分详情 */
export interface SScoreExplain {
  s1_customer_level: SScoreComponent;
  s2_margin: SScoreComponent;
  s3_urgency: SScoreComponent;
  total_score: number;
  verification_passed: boolean;
}

/** P-Score 子项详情 */
export interface PScoreComponent {
  name: string;
  input_value: string;
  score: number;
  weight: number;
  contribution: number;
  rule_description: string;
}

/** P-Score 评分详情 */
export interface PScoreExplain {
  p1_difficulty: PScoreComponent;
  p2_aggregation: PScoreComponent;
  p3_rhythm: PScoreComponent;
  total_score: number;
  verification_passed: boolean;
}

/** 综合优先级详情 */
export interface PriorityExplain {
  contract_id: string;
  strategy_name: string;
  s_score_explain: SScoreExplain;
  p_score_explain: PScoreExplain;
  s_score: number;
  p_score: number;
  ws: number;
  wp: number;
  base_priority: number;
  alpha?: number;
  final_priority: number;
  formula_summary: string;
  all_verifications_passed: boolean;
}

export type ImportDataType = 'contracts' | 'customers' | 'process_difficulty' | 'strategy_weights';
export type ConflictStrategy = 'skip' | 'overwrite';

export interface Customer {
  customer_id: string;
  customer_name?: string;
  customer_level: string;
  credit_level?: string;
  customer_group?: string;
}

export interface ProcessDifficulty {
  id: number;
  steel_grade: string;
  thickness_min: number;
  thickness_max: number;
  width_min: number;
  width_max: number;
  difficulty_level: string;
  difficulty_score: number;
}

export interface ValidationError {
  row_number: number;
  field: string;
  value: string;
  message: string;
}

export interface ConflictRecord {
  row_number: number;
  primary_key: string;
  existing_data: any;
  new_data: any;
  action?: ConflictStrategy;
}

export type FieldType = "string" | "float" | "integer" | "date";

export interface TargetFieldDef {
  name: string;
  display_name: string;
  required: boolean;
  field_type: FieldType;
  default_value?: string | null;
}

export interface FieldMappingResult {
  mappings: Record<string, string>;
  unmapped_targets: string[];
  unmapped_sources: string[];
  confidence: Record<string, number>;
}

export interface TransformConfig {
  field_transforms: Record<string, any[]>;
  default_values?: Record<string, {
    value: string;
    condition: "when_empty" | "when_missing" | "always";
  }>;
}

export interface FieldAlignmentRule {
  rule_id?: number;
  rule_name: string;
  data_type: string;
  source_type?: string | null;
  description?: string | null;
  enabled: number;
  priority: number;
  field_mapping: string;
  value_transform?: string | null;
  default_values?: string | null;
  created_by: string;
  created_at?: string | null;
  updated_at?: string | null;
}

export interface FieldAlignmentChangeLog {
  log_id?: number;
  rule_id: number;
  change_type: string;
  old_value?: string | null;
  new_value?: string | null;
  change_reason?: string | null;
  changed_by: string;
  changed_at?: string | null;
}

export interface ImportPreview {
  total_rows: number;
  valid_rows: number;
  error_rows: number;
  conflicts: ConflictRecord[];
  validation_errors: ValidationError[];
  sample_data: any[];
}

export interface ImportResult {
  success: boolean;
  imported_count: number;
  skipped_count: number;
  error_count: number;
  errors: ValidationError[];
  message: string;
}

export interface ExportOptions {
  format: FileFormat;
  data_type: ImportDataType;
  include_computed: boolean;
  strategy?: string;
}

export interface ExportResult {
  success: boolean;
  file_path: string;
  row_count: number;
  message: string;
}

// API 调用封装
export const api = {
  /**
   * 计算单个合同的优先级
   */
  async computePriority(
    contract_id: string,
    strategy: string
  ): Promise<number> {
    return invoke("compute_priority", { contractId: contract_id, strategy });
  },

  /**
   * 批量计算所有合同的优先级
   */
  async computeAllPriorities(
    strategy: string
  ): Promise<ContractPriority[]> {
    return invoke("compute_all_priorities", { strategy });
  },

  /**
   * 获取所有合同列表
   */
  async getContracts(): Promise<Contract[]> {
    return invoke("get_contracts");
  },

  /**
   * 获取所有策略列表
   */
  async getStrategies(): Promise<string[]> {
    return invoke("get_strategies");
  },

  /**
   * 设置人工调整系数 alpha
   */
  async setAlpha(
    contract_id: string,
    alpha: number,
    reason: string,
    user: string
  ): Promise<void> {
    return invoke("set_alpha", { contractId: contract_id, alpha, reason, user });
  },

  /**
   * 获取合同的干预历史
   */
  async getInterventionHistory(
    contract_id: string
  ): Promise<InterventionLog[]> {
    return invoke("get_intervention_history", { contractId: contract_id });
  },

  /**
   * 获取所有干预日志（支持分页）
   */
  async getAllInterventionLogs(
    limit?: number
  ): Promise<InterventionLog[]> {
    return invoke("get_all_intervention_logs", { limit });
  },

  // ============================================
  // Phase 2: 配置管理 API
  // ============================================

  /**
   * 获取所有评分配置项
   */
  async getScoringConfigs(): Promise<ScoringConfigItem[]> {
    return invoke("get_scoring_configs");
  },

  /**
   * 更新配置项
   */
  async updateConfig(
    config_key: string,
    new_value: string,
    changed_by: string,
    reason?: string
  ): Promise<void> {
    return invoke("update_config", { configKey: config_key, newValue: new_value, changedBy: changed_by, reason });
  },

  /**
   * 获取配置变更历史
   */
  async getConfigHistory(
    config_key?: string,
    limit?: number
  ): Promise<ConfigChangeLog[]> {
    return invoke("get_config_history", { configKey: config_key, limit });
  },

  /**
   * 回滚配置到指定版本
   */
  async rollbackConfig(
    log_id: number,
    changed_by: string,
    reason: string
  ): Promise<void> {
    return invoke("rollback_config", { logId: log_id, changedBy: changed_by, reason });
  },

  /**
   * 获取所有策略的评分权重 (w1, w2, w3)
   */
  async getAllStrategyWeights(): Promise<StrategyScoringWeights[]> {
    return invoke("get_all_strategy_weights");
  },

  /**
   * 获取所有策略权重 (ws, wp)
   */
  async getStrategyWeightsList(): Promise<StrategyWeights[]> {
    return invoke("get_strategy_weights_list");
  },

  /**
   * 更新策略的评分权重
   */
  async updateStrategyWeights(
    strategy_name: string,
    w1: number,
    w2: number,
    w3: number
  ): Promise<void> {
    return invoke("update_strategy_weights", { strategyName: strategy_name, w1, w2, w3 });
  },

  /**
   * 创建或更新策略权重 (ws, wp)
   */
  async upsertStrategyWeight(params: {
    strategy_name: string;
    ws: number;
    wp: number;
    description?: string;
  }): Promise<void> {
    return invoke("upsert_strategy_weight", {
      strategyName: params.strategy_name,
      ws: params.ws,
      wp: params.wp,
      description: params.description,
    });
  },

  /**
   * 删除策略权重
   */
  async deleteStrategyWeight(strategy_name: string): Promise<void> {
    return invoke("delete_strategy_weight", { strategyName: strategy_name });
  },

  // ============================================
  // Phase 4: 筛选器预设 API
  // ============================================

  /**
   * 获取所有筛选器预设
   */
  async getFilterPresets(): Promise<FilterPreset[]> {
    return invoke("get_filter_presets");
  },

  /**
   * 保存筛选器预设
   */
  async saveFilterPreset(
    preset_name: string,
    filter_json: string,
    description: string,
    created_by: string
  ): Promise<void> {
    return invoke("save_filter_preset", { presetName: preset_name, filterJson: filter_json, description, createdBy: created_by });
  },

  /**
   * 删除筛选器预设
   */
  async deleteFilterPreset(preset_id: number): Promise<void> {
    return invoke("delete_filter_preset", { presetId: preset_id });
  },

  /**
   * 设置默认筛选器预设
   */
  async setDefaultFilterPreset(preset_id: number): Promise<void> {
    return invoke("set_default_filter_preset", { presetId: preset_id });
  },

  // ============================================
  // Phase 5: 批量操作 API
  // ============================================

  /**
   * 批量调整合同的 alpha 值
   */
  async batchAdjustAlpha(
    contract_ids: string[],
    alpha: number,
    reason: string,
    user: string
  ): Promise<number> {
    return invoke("batch_adjust_alpha", { contractIds: contract_ids, alpha, reason, user });
  },

  /**
   * 批量恢复合同的 alpha 值
   */
  async batchRestoreAlpha(
    contract_ids: string[],
    reason: string,
    user: string
  ): Promise<number> {
    return invoke("batch_restore_alpha", { contractIds: contract_ids, reason, user });
  },

  /**
   * 获取批量操作历史
   */
  async getBatchOperations(limit?: number): Promise<BatchOperation[]> {
    return invoke("get_batch_operations", { limit });
  },

  /**
   * 获取批量操作涉及的合同列表
   */
  async getBatchOperationContracts(batchId: number): Promise<string[]> {
    return invoke("get_batch_operation_contracts", { batchId });
  },

  // ============================================
  // 统一历史记录 API
  // ============================================

  /**
   * 获取统一的变更历史（整合配置变更、Alpha调整、批量操作）
   */
  async getUnifiedHistory(
    entry_type?: string,
    user?: string,
    limit?: number
  ): Promise<UnifiedHistoryEntry[]> {
    return invoke("get_unified_history", { entryType: entry_type, user, limit });
  },

  // ============================================
  // 导入/导出 API
  // ============================================

  /**
   * 预览导入文件
   */
  async previewImport(
    file_path: string,
    data_type: ImportDataType,
    format: FileFormat,
    field_mapping?: Record<string, string>,
    value_transforms?: Record<string, any>
  ): Promise<ImportPreview> {
    return invoke("preview_import", {
      filePath: file_path,
      dataType: data_type,
      format,
      fieldMapping: field_mapping,
      valueTransforms: value_transforms,
    });
  },

  /**
   * 执行导入
   */
  async executeImport(
    file_path: string,
    data_type: ImportDataType,
    format: FileFormat,
    conflict_strategy: ConflictStrategy,
    conflict_decisions?: ConflictRecord[],
    field_mapping?: Record<string, string>,
    value_transforms?: Record<string, any>
  ): Promise<ImportResult> {
    return invoke("execute_import", {
      filePath: file_path,
      dataType: data_type,
      format,
      conflictStrategy: conflict_strategy,
      conflictDecisions: conflict_decisions,
      fieldMapping: field_mapping,
      valueTransforms: value_transforms,
    });
  },

  /**
   * 解析文件头
   */
  async parseFileHeaders(
    file_path: string,
    format: FileFormat
  ): Promise<string[]> {
    return invoke("parse_file_headers", { filePath: file_path, format });
  },

  /**
   * 获取目标字段定义
   */
  async getTargetFields(
    data_type: ImportDataType
  ): Promise<TargetFieldDef[]> {
    return invoke("get_target_fields", { dataType: data_type });
  },

  /**
   * 自动检测字段映射
   */
  async autoDetectMapping(
    file_path: string,
    format: FileFormat,
    data_type: ImportDataType
  ): Promise<FieldMappingResult> {
    return invoke("auto_detect_mapping", { filePath: file_path, format, dataType: data_type });
  },

  /**
   * 验证表达式语法
   */
  async validateExpression(expression: string): Promise<boolean> {
    return invoke("validate_expression", { expression });
  },

  /**
   * 测试表达式
   */
  async testExpression(
    expression: string,
    sample_row: Record<string, string>
  ): Promise<string> {
    return invoke("test_expression", { expression, sampleRow: sample_row });
  },

  /**
   * 获取字段映射规则
   */
  async getFieldAlignmentRules(
    data_type?: string,
    include_disabled?: boolean
  ): Promise<FieldAlignmentRule[]> {
    return invoke("get_field_alignment_rules", { dataType: data_type, includeDisabled: include_disabled });
  },

  /**
   * 获取字段映射规则变更日志
   */
  async getFieldAlignmentChangeLogs(
    rule_id?: number,
    limit?: number
  ): Promise<FieldAlignmentChangeLog[]> {
    return invoke("get_field_alignment_change_logs", { ruleId: rule_id, limit });
  },

  /**
   * 保存字段映射规则（新建/更新）
   */
  async saveFieldAlignmentRule(rule: FieldAlignmentRule, user: string): Promise<number> {
    return invoke("save_field_alignment_rule", { rule, user });
  },

  /**
   * 删除字段映射规则
   */
  async deleteFieldAlignmentRule(rule_id: number, user: string): Promise<void> {
    return invoke("delete_field_alignment_rule", { ruleId: rule_id, user });
  },

  /**
   * 导出数据
   */
  async exportData(
    file_path: string,
    options: ExportOptions
  ): Promise<ExportResult> {
    return invoke("export_data", { filePath: file_path, options });
  },

  /**
   * 获取客户列表
   */
  async getCustomers(): Promise<Customer[]> {
    return invoke("get_customers");
  },

  /**
   * 获取工艺难度配置
   */
  async getProcessDifficulty(): Promise<ProcessDifficulty[]> {
    return invoke("get_process_difficulty");
  },

  // ============================================
  // Phase 8: 清洗规则管理 API
  // ============================================

  /**
   * 获取所有清洗规则
   */
  async getTransformRules(): Promise<TransformRule[]> {
    return invoke("get_transform_rules");
  },

  /**
   * 按分类获取清洗规则
   */
  async getTransformRulesByCategory(category: string): Promise<TransformRule[]> {
    return invoke("get_transform_rules_by_category", { category });
  },

  /**
   * 获取单个清洗规则
   */
  async getTransformRule(rule_id: number): Promise<TransformRule> {
    return invoke("get_transform_rule", { ruleId: rule_id });
  },

  /**
   * 创建清洗规则
   */
  async createTransformRule(
    rule_name: string,
    category: string,
    description: string | null,
    priority: number,
    config_json: string,
    user: string
  ): Promise<number> {
    return invoke("create_transform_rule", {
      ruleName: rule_name,
      category,
      description,
      priority,
      configJson: config_json,
      user,
    });
  },

  /**
   * 更新清洗规则
   */
  async updateTransformRule(
    rule_id: number,
    rule_name: string,
    description: string | null,
    priority: number,
    config_json: string,
    user: string,
    reason?: string
  ): Promise<void> {
    return invoke("update_transform_rule", {
      ruleId: rule_id,
      ruleName: rule_name,
      description,
      priority,
      configJson: config_json,
      user,
      reason,
    });
  },

  /**
   * 删除清洗规则
   */
  async deleteTransformRule(
    rule_id: number,
    user: string,
    reason?: string
  ): Promise<void> {
    return invoke("delete_transform_rule", { ruleId: rule_id, user, reason });
  },

  /**
   * 切换规则启用/禁用状态
   */
  async toggleTransformRule(
    rule_id: number,
    enabled: boolean,
    user: string
  ): Promise<void> {
    return invoke("toggle_transform_rule", { ruleId: rule_id, enabled, user });
  },

  /**
   * 获取规则变更历史
   */
  async getTransformRuleHistory(
    rule_id?: number,
    limit?: number
  ): Promise<TransformRuleChangeLog[]> {
    return invoke("get_transform_rule_history", { ruleId: rule_id, limit });
  },

  /**
   * 获取规则执行历史
   */
  async getTransformExecutionHistory(
    rule_id?: number,
    limit?: number
  ): Promise<TransformExecutionLog[]> {
    return invoke("get_transform_execution_history", { ruleId: rule_id, limit });
  },

  /**
   * 测试规则（预览模式）
   */
  async testTransformRule(
    rule_id: number,
    sample_size?: number
  ): Promise<RuleTestResult> {
    return invoke("test_transform_rule", { ruleId: rule_id, sampleSize: sample_size });
  },

  /**
   * 执行清洗规则
   */
  async executeTransformRule(
    rule_id: number,
    user: string
  ): Promise<TransformExecutionLog> {
    return invoke("execute_transform_rule", { ruleId: rule_id, user });
  },

  // ============================================
  // Phase 9: 规格族管理 API
  // ============================================

  /**
   * 获取所有规格族
   */
  async getSpecFamilies(): Promise<SpecFamily[]> {
    return invoke("get_spec_families");
  },

  /**
   * 获取启用的规格族
   */
  async getEnabledSpecFamilies(): Promise<SpecFamily[]> {
    return invoke("get_enabled_spec_families");
  },

  /**
   * 获取单个规格族
   */
  async getSpecFamily(family_id: number): Promise<SpecFamily> {
    return invoke("get_spec_family", { familyId: family_id });
  },

  /**
   * 创建规格族
   */
  async createSpecFamily(
    family_name: string,
    family_code: string,
    description: string | null,
    factor: number,
    steel_grades: string | null,
    thickness_min: number | null,
    thickness_max: number | null,
    width_min: number | null,
    width_max: number | null,
    sort_order: number,
    user: string
  ): Promise<number> {
    return invoke("create_spec_family", {
      familyName: family_name,
      familyCode: family_code,
      description,
      factor,
      steelGrades: steel_grades,
      thicknessMin: thickness_min,
      thicknessMax: thickness_max,
      widthMin: width_min,
      widthMax: width_max,
      sortOrder: sort_order,
      user,
    });
  },

  /**
   * 更新规格族
   */
  async updateSpecFamily(
    family_id: number,
    family_name: string,
    family_code: string,
    description: string | null,
    factor: number,
    steel_grades: string | null,
    thickness_min: number | null,
    thickness_max: number | null,
    width_min: number | null,
    width_max: number | null,
    sort_order: number,
    user: string,
    reason?: string
  ): Promise<void> {
    return invoke("update_spec_family", {
      familyId: family_id,
      familyName: family_name,
      familyCode: family_code,
      description,
      factor,
      steelGrades: steel_grades,
      thicknessMin: thickness_min,
      thicknessMax: thickness_max,
      widthMin: width_min,
      widthMax: width_max,
      sortOrder: sort_order,
      user,
      reason,
    });
  },

  /**
   * 删除规格族
   */
  async deleteSpecFamily(
    family_id: number,
    user: string,
    reason?: string
  ): Promise<void> {
    return invoke("delete_spec_family", { familyId: family_id, user, reason });
  },

  /**
   * 切换规格族启用/禁用状态
   */
  async toggleSpecFamily(
    family_id: number,
    enabled: boolean,
    user: string
  ): Promise<void> {
    return invoke("toggle_spec_family", { familyId: family_id, enabled, user });
  },

  /**
   * 获取规格族变更历史
   */
  async getSpecFamilyHistory(
    family_id?: number,
    limit?: number
  ): Promise<SpecFamilyChangeLog[]> {
    return invoke("get_spec_family_history", { familyId: family_id, limit });
  },

  // ============================================
  // Phase 11: 评分透明化 API
  // ============================================

  /**
   * 获取合同优先级的详细评分拆分（Explain）
   * 用于让业务人员复算、复核任何一笔合同的优先级
   */
  async explainPriority(
    contract_id: string,
    strategy: string
  ): Promise<PriorityExplain> {
    return invoke("explain_priority", { contractId: contract_id, strategy });
  },

  // ============================================
  // Phase 16: 会议驾驶舱 KPI 计算引擎
  // ============================================

  /**
   * 计算所有 KPI（四视角）
   * 返回领导、销售、生产、财务四个维度的指标
   */
  async calculateMeetingKpis(strategy: string): Promise<KpiSummary> {
    return invoke("calculate_meeting_kpis", { strategy });
  },

  /**
   * 计算单个 KPI
   */
  async calculateSingleKpi(
    kpi_code: string,
    strategy: string
  ): Promise<KpiValue> {
    return invoke("calculate_single_kpi", { kpiCode: kpi_code, strategy });
  },

  /**
   * 识别风险合同
   */
  async identifyRiskContracts(
    strategy: string,
    snapshot_id?: number,
    auto_save: boolean = false
  ): Promise<RiskIdentificationResult> {
    return invoke("identify_risk_contracts", { strategy, snapshotId: snapshot_id, autoSave: auto_save });
  },

  /**
   * 计算排名变化（与历史快照对比）
   */
  async calculateRankingChanges(
    current_snapshot_id: number,
    previous_snapshot_id: number,
    strategy: string,
    auto_save: boolean = false
  ): Promise<RankingChangesResult> {
    return invoke("calculate_ranking_changes", {
      currentSnapshotId: current_snapshot_id,
      previousSnapshotId: previous_snapshot_id,
      strategy,
      autoSave: auto_save,
    });
  },

  // ============================================
  // Phase 16 P2: 维度聚合分析
  // ============================================

  /**
   * 客户保障分析
   * 返回每个客户的保障评分和风险客户列表
   */
  async analyzeCustomerProtection(
    strategy: string
  ): Promise<CustomerProtectionAnalysis> {
    return invoke("analyze_customer_protection", { strategy });
  },

  /**
   * 节拍顺行分析
   * 返回每日节拍匹配情况和规格族分布
   */
  async analyzeRhythmFlow(strategy: string): Promise<RhythmFlowAnalysis> {
    return invoke("analyze_rhythm_flow", { strategy });
  },

  // ============================================
  // Phase 16 P3: 共识包生成与导出
  // ============================================

  /**
   * 生成会议共识包
   */
  async generateConsensusPackage(
    strategy: string,
    meeting_type: MeetingType,
    meeting_date: string,
    user: string
  ): Promise<ConsensusPackage> {
    return invoke("generate_consensus_package", {
      strategy,
      meetingType: meeting_type,
      meetingDate: meeting_date,
      user,
    });
  },

  /**
   * 导出共识包为 CSV 文件（多文件）
   */
  async exportConsensusCsv(
    strategy: string,
    meeting_type: MeetingType,
    meeting_date: string,
    output_dir: string,
    file_prefix: string | null,
    user: string
  ): Promise<CsvExportResult> {
    return invoke("export_consensus_csv", {
      strategy,
      meetingType: meeting_type,
      meetingDate: meeting_date,
      outputDir: output_dir,
      filePrefix: file_prefix,
      user,
    });
  },

  /**
   * 导出合同排名表为单个 CSV 文件
   */
  async exportContractsRankingCsv(
    strategy: string,
    meeting_type: MeetingType,
    meeting_date: string,
    file_path: string,
    user: string
  ): Promise<CsvExportResult> {
    return invoke("export_contracts_ranking_csv", {
      strategy,
      meetingType: meeting_type,
      meetingDate: meeting_date,
      filePath: file_path,
      user,
    });
  },
};

// ============================================
// Phase 16: 会议驾驶舱相关数据类型
// ============================================

/** 会议类型 */
export type MeetingType = 'production_sales' | 'business';

/** KPI 值 */
export interface KpiValue {
  kpi_code: string;
  kpi_name: string;
  value: number;
  display_value: string;
  status: string;  // 'good' | 'warning' | 'danger'
  change?: number;
  change_direction?: string;  // 'up' | 'down' | 'unchanged'
  business_meaning?: string;
}

/** KPI 汇总（四视角） */
export interface KpiSummary {
  leadership: KpiValue[];
  sales: KpiValue[];
  production: KpiValue[];
  finance: KpiValue[];
}

/** 风险合同标记 */
export interface RiskContractFlag {
  flag_id?: number;
  snapshot_id?: number;
  contract_id: string;
  risk_type: string;  // 'delivery_delay' | 'customer_downgrade' | 'margin_loss' | 'rhythm_mismatch' | 'other'
  risk_level: string;  // 'high' | 'medium' | 'low'
  risk_score?: number;
  risk_description: string;
  risk_factors?: string;
  affected_kpis?: string;
  potential_loss?: number;
  suggested_action?: string;
  action_priority?: number;
  status: string;  // 'open' | 'in_progress' | 'resolved' | 'accepted'
  handled_by?: string;
  handled_at?: string;
  handling_note?: string;
  created_at?: string;
}

/** 风险识别结果 */
export interface RiskIdentificationResult {
  total_count: number;
  high_risk_count: number;
  medium_risk_count: number;
  low_risk_count: number;
  stats_by_type: Record<string, number>;
  risk_contracts: RiskContractFlag[];
}

/** 排名变化结果 */
export interface RankingChangesResult {
  total_count: number;
  up_count: number;
  down_count: number;
  unchanged_count: number;
  avg_change: number;
  max_up: number;
  max_down: number;
  changes: RankingChangeDetail[];
}

/** 排名变化明细 */
export interface RankingChangeDetail {
  detail_id?: number;
  snapshot_id?: number;
  compare_snapshot_id?: number;
  contract_id: string;
  current_rank: number;
  previous_rank?: number;
  rank_change: number;
  current_priority: number;
  previous_priority?: number;
  priority_change: number;
  change_reason?: string;
  change_factors?: string;
}

/** 客户保障摘要 */
export interface CustomerProtectionSummary {
  customer_id: string;
  customer_name?: string;
  customer_level: string;
  contract_count: number;
  avg_rank: number;
  best_rank: number;
  worst_rank: number;
  top_n_count: number;
  total_margin: number;
  protection_score: number;
  protection_status: string;  // 'good' | 'warning' | 'risk'
  risk_description?: string;
}

/** 客户保障分析结果 */
export interface CustomerProtectionAnalysis {
  total_customers: number;
  well_protected_count: number;
  warning_count: number;
  risk_count: number;
  customers: CustomerProtectionSummary[];
  risk_customers: CustomerProtectionSummary[];
}

/** 每日节拍摘要 */
export interface DailyRhythmSummary {
  rhythm_day: number;
  label_name: string;
  match_specs: string[];
  matched_contract_count: number;
  matched_total_capacity: number;
  mismatched_top_count: number;
  match_rate: number;
  rhythm_status: string;  // 'smooth' | 'congested' | 'idle'
  status_description?: string;
}

/** 规格族节拍摘要 */
export interface SpecFamilyRhythmSummary {
  spec_family: string;
  total_contracts: number;
  matched_contracts: number;
  match_rate: number;
}

/** 节拍顺行分析结果 */
export interface RhythmFlowAnalysis {
  cycle_days: number;
  overall_match_rate: number;
  overall_status: string;  // 'smooth' | 'partial' | 'blocked'
  daily_summaries: DailyRhythmSummary[];
  spec_family_summaries: SpecFamilyRhythmSummary[];
  congestion_days: number[];
  idle_days: number[];
}

/** 合同排名摘要 */
export interface ContractRankingSummary {
  rank: number;
  contract_id: string;
  customer_id: string;
  customer_name?: string;
  customer_level: string;
  spec_family: string;
  steel_grade: string;
  thickness: number;
  width: number;
  days_to_pdd: number;
  margin: number;
  s_score: number;
  p_score: number;
  priority: number;
  alpha?: number;
  rank_change?: number;
  has_risk: boolean;
  risk_types: string[];
}

/** 行动建议 */
export interface ActionRecommendation {
  priority: number;
  category: string;  // 'risk' | 'kpi' | 'rhythm' | 'customer'
  title: string;
  description: string;
  related_contracts: string[];
  related_kpis: string[];
  suggested_department?: string;
}

/** 共识包（完整会议材料） */
export interface ConsensusPackage {
  meeting_type: MeetingType;
  meeting_date: string;
  generated_at: string;
  generated_by: string;
  strategy_name: string;
  kpi_summary: KpiSummary;
  risk_contracts: RiskContractFlag[];
  customer_analysis: CustomerProtectionAnalysis;
  rhythm_analysis: RhythmFlowAnalysis;
  top_contracts: ContractRankingSummary[];
  recommendations: ActionRecommendation[];
  snapshot_id?: number;
}

/** CSV 文件信息 */
export interface CsvFileInfo {
  file_name: string;
  file_path: string;
  data_type: string;
  row_count: number;
}

/** CSV 导出结果 */
export interface CsvExportResult {
  success: boolean;
  file_path?: string;
  file_paths: CsvFileInfo[];
  total_rows: number;
  message: string;
}
