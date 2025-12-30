//! 数据校验模块
//!
//! # 设计原则
//!
//! 1. **不允许"悄悄算错"**：所有字段缺失/异常都有明确处理策略
//! 2. **数据问题可见**：每条合同带 validation_status 和 warnings
//! 3. **可解释**：详细记录使用了哪些默认值
//! 4. **可修复**：提供修复建议
//! 5. **系统不崩溃**：即使数据有问题也能给出结果
//!
//! # 校验流程
//!
//! 1. 加载缺失值策略配置
//! 2. 对每个合同进行字段校验
//! 3. 根据策略应用默认值或标记错误
//! 4. 返回校验结果（包含警告列表）
//! 5. 可选：生成数据质量报告

use crate::db;
use crate::db::schema::{
    Contract, Customer, ContractValidationResult, ValidationIssue,
    ValidationSeverity, ValidationIssueType, MissingValueStrategy,
    DataQualityReport, ValidationSummary, FieldIssueDetail, DefaultValueUsed,
};
use std::collections::HashMap;

/// 校验后的合同数据（已应用默认值）
#[derive(Debug, Clone)]
pub struct ValidatedContract {
    /// 原始合同数据（已填充默认值）
    pub contract: Contract,
    /// 校验结果
    pub validation: ContractValidationResult,
    /// 使用的默认值列表
    pub defaults_used: Vec<DefaultValueUsed>,
}

/// 校验后的客户数据（已应用默认值）
#[derive(Debug, Clone)]
pub struct ValidatedCustomer {
    /// 原始客户数据（已填充默认值）
    pub customer: Customer,
    /// 校验结果
    #[allow(dead_code)]
    pub validation: ContractValidationResult,
    /// 使用的默认值列表
    pub defaults_used: Vec<DefaultValueUsed>,
}

/// 加载缺失值策略配置
fn load_strategies() -> Result<HashMap<String, MissingValueStrategy>, String> {
    let strategies = db::list_missing_value_strategies()?;
    let mut map = HashMap::new();
    for s in strategies {
        map.insert(s.field_name.clone(), s);
    }
    Ok(map)
}

/// 解析 JSON 格式的默认值
fn parse_default_value<T: std::str::FromStr>(json_value: &str) -> Option<T> {
    // 去除 JSON 引号
    let cleaned = json_value.trim_matches('"');
    cleaned.parse().ok()
}

/// 校验单个合同
///
/// # 参数
/// - `contract`: 待校验的合同
/// - `strategies`: 缺失值策略配置（可选，不传则自动加载）
///
/// # 返回
/// 校验后的合同（已应用默认值）和校验结果
pub fn validate_contract(
    contract: &Contract,
    strategies: Option<&HashMap<String, MissingValueStrategy>>,
) -> Result<ValidatedContract, String> {
    // 加载策略配置
    let owned_strategies: HashMap<String, MissingValueStrategy>;
    let strats = match strategies {
        Some(s) => s,
        None => {
            owned_strategies = load_strategies()?;
            &owned_strategies
        }
    };

    let mut issues: Vec<ValidationIssue> = Vec::new();
    let mut defaults_used: Vec<DefaultValueUsed> = Vec::new();
    let mut validated = contract.clone();

    // ============================================
    // 校验 steel_grade（钢种）
    // ============================================
    if validated.steel_grade.trim().is_empty() || validated.steel_grade == "UNKNOWN" {
        if let Some(strategy) = strats.get("steel_grade") {
            let (severity, default_val) = match strategy.strategy.as_str() {
                "default" => {
                    let default = strategy.default_value.as_deref().unwrap_or("\"UNKNOWN\"");
                    let default_str: String = parse_default_value(default).unwrap_or_else(|| "UNKNOWN".to_string());
                    validated.steel_grade = default_str.clone();
                    defaults_used.push(DefaultValueUsed {
                        field_name: "steel_grade".to_string(),
                        field_label: strategy.field_label.clone(),
                        default_value: default_str.clone(),
                        description: strategy.default_description.clone().unwrap_or_default(),
                    });
                    (ValidationSeverity::Warning, Some(default_str))
                }
                "skip" => (ValidationSeverity::Error, None),
                "error" => (ValidationSeverity::Error, None),
                _ => (ValidationSeverity::Warning, None),
            };

            issues.push(ValidationIssue {
                field_name: "steel_grade".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: ValidationIssueType::Missing,
                severity,
                original_value: Some(contract.steel_grade.clone()),
                default_value_used: default_val,
                message: "钢种缺失，工艺难度评分(P1)可能不准确".to_string(),
                suggested_fix: Some("请补充钢种代码".to_string()),
            });
        }
    }

    // ============================================
    // 校验 thickness（厚度）
    // ============================================
    if validated.thickness <= 0.0 {
        if let Some(strategy) = strats.get("thickness") {
            let default = strategy.default_value.as_deref().unwrap_or("2.0");
            let default_val: f64 = parse_default_value(default).unwrap_or(2.0);
            validated.thickness = default_val;

            defaults_used.push(DefaultValueUsed {
                field_name: "thickness".to_string(),
                field_label: strategy.field_label.clone(),
                default_value: default_val.to_string(),
                description: strategy.default_description.clone().unwrap_or_default(),
            });

            issues.push(ValidationIssue {
                field_name: "thickness".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: if contract.thickness == 0.0 { ValidationIssueType::Missing } else { ValidationIssueType::Invalid },
                severity: ValidationSeverity::Warning,
                original_value: Some(contract.thickness.to_string()),
                default_value_used: Some(default_val.to_string()),
                message: format!("厚度值无效({}mm)，使用默认值{}mm", contract.thickness, default_val),
                suggested_fix: Some("请检查厚度数据".to_string()),
            });
        }
    }

    // ============================================
    // 校验 width（宽度）
    // ============================================
    if validated.width <= 0.0 {
        if let Some(strategy) = strats.get("width") {
            let default = strategy.default_value.as_deref().unwrap_or("1200");
            let default_val: f64 = parse_default_value(default).unwrap_or(1200.0);
            validated.width = default_val;

            defaults_used.push(DefaultValueUsed {
                field_name: "width".to_string(),
                field_label: strategy.field_label.clone(),
                default_value: default_val.to_string(),
                description: strategy.default_description.clone().unwrap_or_default(),
            });

            issues.push(ValidationIssue {
                field_name: "width".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: if contract.width == 0.0 { ValidationIssueType::Missing } else { ValidationIssueType::Invalid },
                severity: ValidationSeverity::Warning,
                original_value: Some(contract.width.to_string()),
                default_value_used: Some(default_val.to_string()),
                message: format!("宽度值无效({}mm)，使用默认值{}mm", contract.width, default_val),
                suggested_fix: Some("请检查宽度数据".to_string()),
            });
        }
    }

    // ============================================
    // 校验 spec_family（规格族）
    // ============================================
    if validated.spec_family.trim().is_empty() {
        if let Some(strategy) = strats.get("spec_family") {
            let default = strategy.default_value.as_deref().unwrap_or("\"常规\"");
            let default_str: String = parse_default_value(default).unwrap_or_else(|| "常规".to_string());
            validated.spec_family = default_str.clone();

            defaults_used.push(DefaultValueUsed {
                field_name: "spec_family".to_string(),
                field_label: strategy.field_label.clone(),
                default_value: default_str.clone(),
                description: strategy.default_description.clone().unwrap_or_default(),
            });

            issues.push(ValidationIssue {
                field_name: "spec_family".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: ValidationIssueType::Missing,
                severity: ValidationSeverity::Warning,
                original_value: Some(contract.spec_family.clone()),
                default_value_used: Some(default_str),
                message: "规格族缺失，聚合度评分(P2)可能不准确".to_string(),
                suggested_fix: Some("请补充规格族".to_string()),
            });
        }
    }

    // ============================================
    // 校验 days_to_pdd（距交期天数）
    // ============================================
    // 注意：days_to_pdd 可以为 0 或负数（已逾期），但如果过于异常需要警告
    if validated.days_to_pdd < -365 || validated.days_to_pdd > 365 {
        if let Some(strategy) = strats.get("days_to_pdd") {
            issues.push(ValidationIssue {
                field_name: "days_to_pdd".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: ValidationIssueType::OutOfRange,
                severity: ValidationSeverity::Info,
                original_value: Some(contract.days_to_pdd.to_string()),
                default_value_used: None,
                message: format!("距交期天数({})超出常规范围(-365~365)", contract.days_to_pdd),
                suggested_fix: Some("请确认交期数据是否正确".to_string()),
            });
        }
    }

    // ============================================
    // 校验 margin（毛利）
    // ============================================
    // margin 是可选字段，缺失时使用 0.0
    if validated.margin < -100.0 || validated.margin > 100.0 {
        if let Some(strategy) = strats.get("margin") {
            issues.push(ValidationIssue {
                field_name: "margin".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: ValidationIssueType::OutOfRange,
                severity: ValidationSeverity::Info,
                original_value: Some(contract.margin.to_string()),
                default_value_used: None,
                message: format!("毛利率({:.2}%)超出常规范围(-100%~100%)", contract.margin),
                suggested_fix: Some("请确认毛利数据是否正确".to_string()),
            });
        }
    }

    // ============================================
    // 校验 customer_id（客户编号）
    // ============================================
    if validated.customer_id.trim().is_empty() {
        if let Some(strategy) = strats.get("customer_id") {
            let default = strategy.default_value.as_deref().unwrap_or("\"UNKNOWN\"");
            let default_str: String = parse_default_value(default).unwrap_or_else(|| "UNKNOWN".to_string());
            validated.customer_id = default_str.clone();

            defaults_used.push(DefaultValueUsed {
                field_name: "customer_id".to_string(),
                field_label: strategy.field_label.clone(),
                default_value: default_str.clone(),
                description: strategy.default_description.clone().unwrap_or_default(),
            });

            issues.push(ValidationIssue {
                field_name: "customer_id".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: ValidationIssueType::Missing,
                severity: ValidationSeverity::Warning,
                original_value: Some(contract.customer_id.clone()),
                default_value_used: Some(default_str),
                message: "客户编号缺失，客户等级评分(S1)将使用默认值".to_string(),
                suggested_fix: Some("请补充客户编号".to_string()),
            });
        }
    }

    // 计算统计
    let error_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Error).count() as i64;
    let warning_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count() as i64;
    let can_calculate = error_count == 0;
    let status = if error_count > 0 {
        "error".to_string()
    } else if warning_count > 0 {
        "warning".to_string()
    } else {
        "valid".to_string()
    };

    let validation = ContractValidationResult {
        contract_id: validated.contract_id.clone(),
        can_calculate,
        status,
        issues,
        error_count,
        warning_count,
    };

    Ok(ValidatedContract {
        contract: validated,
        validation,
        defaults_used,
    })
}

/// 校验客户数据
///
/// # 参数
/// - `customer`: 待校验的客户
/// - `strategies`: 缺失值策略配置（可选）
pub fn validate_customer(
    customer: &Customer,
    strategies: Option<&HashMap<String, MissingValueStrategy>>,
) -> Result<ValidatedCustomer, String> {
    let owned_strategies: HashMap<String, MissingValueStrategy>;
    let strats = match strategies {
        Some(s) => s,
        None => {
            owned_strategies = load_strategies()?;
            &owned_strategies
        }
    };

    let mut issues: Vec<ValidationIssue> = Vec::new();
    let mut defaults_used: Vec<DefaultValueUsed> = Vec::new();
    let mut validated = customer.clone();

    // ============================================
    // 校验 customer_level（客户等级）
    // ============================================
    if validated.customer_level.trim().is_empty() || !["A", "B", "C"].contains(&validated.customer_level.as_str()) {
        if let Some(strategy) = strats.get("customer_level") {
            let default = strategy.default_value.as_deref().unwrap_or("\"C\"");
            let default_str: String = parse_default_value(default).unwrap_or_else(|| "C".to_string());
            validated.customer_level = default_str.clone();

            defaults_used.push(DefaultValueUsed {
                field_name: "customer_level".to_string(),
                field_label: strategy.field_label.clone(),
                default_value: default_str.clone(),
                description: strategy.default_description.clone().unwrap_or_default(),
            });

            issues.push(ValidationIssue {
                field_name: "customer_level".to_string(),
                field_label: strategy.field_label.clone(),
                issue_type: if customer.customer_level.trim().is_empty() {
                    ValidationIssueType::Missing
                } else {
                    ValidationIssueType::Invalid
                },
                severity: ValidationSeverity::Warning,
                original_value: Some(customer.customer_level.clone()),
                default_value_used: Some(default_str),
                message: format!("客户等级'{}'无效，使用默认值C级", customer.customer_level),
                suggested_fix: Some("请设置正确的客户等级(A/B/C)".to_string()),
            });
        }
    }

    // 计算统计
    let error_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Error).count() as i64;
    let warning_count = issues.iter().filter(|i| i.severity == ValidationSeverity::Warning).count() as i64;
    let can_calculate = error_count == 0;
    let status = if error_count > 0 {
        "error".to_string()
    } else if warning_count > 0 {
        "warning".to_string()
    } else {
        "valid".to_string()
    };

    let validation = ContractValidationResult {
        contract_id: customer.customer_id.clone(),
        can_calculate,
        status,
        issues,
        error_count,
        warning_count,
    };

    Ok(ValidatedCustomer {
        customer: validated,
        validation,
        defaults_used,
    })
}

/// 批量校验所有合同
///
/// # 返回
/// (校验后的合同列表, 数据质量报告)
pub fn validate_all_contracts() -> Result<(Vec<ValidatedContract>, DataQualityReport), String> {
    // 加载策略配置（批量校验只需加载一次）
    let strategies = load_strategies()?;

    // 获取所有合同
    let contracts = db::list_contracts()?;

    let batch_id = chrono::Local::now().format("%Y%m%d%H%M%S").to_string();
    let validation_time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut validated_contracts: Vec<ValidatedContract> = Vec::new();
    let mut problem_contracts: Vec<ContractValidationResult> = Vec::new();
    let mut issues_by_field: HashMap<String, i64> = HashMap::new();
    let mut field_affected_contracts: HashMap<String, Vec<String>> = HashMap::new();

    let mut valid_count = 0i64;
    let mut warning_count = 0i64;
    let mut error_count = 0i64;

    for contract in &contracts {
        let validated = validate_contract(contract, Some(&strategies))?;

        // 统计问题
        match validated.validation.status.as_str() {
            "valid" => valid_count += 1,
            "warning" => warning_count += 1,
            "error" => error_count += 1,
            _ => {}
        }

        // 按字段统计问题
        for issue in &validated.validation.issues {
            *issues_by_field.entry(issue.field_name.clone()).or_insert(0) += 1;
            field_affected_contracts
                .entry(issue.field_name.clone())
                .or_default()
                .push(validated.contract.contract_id.clone());
        }

        // 记录有问题的合同
        if validated.validation.status != "valid" {
            problem_contracts.push(validated.validation.clone());
        }

        validated_contracts.push(validated);
    }

    // 构建字段问题详情
    let mut field_issues: Vec<FieldIssueDetail> = Vec::new();
    for (field_name, count) in &issues_by_field {
        if let Some(strategy) = strategies.get(field_name) {
            field_issues.push(FieldIssueDetail {
                field_name: field_name.clone(),
                field_label: strategy.field_label.clone(),
                affects_score: strategy.affects_score.clone(),
                issue_count: *count,
                default_value: strategy.default_value.clone(),
                default_description: strategy.default_description.clone(),
                affected_contract_ids: field_affected_contracts.get(field_name).cloned().unwrap_or_default(),
            });
        }
    }
    // 按问题数量排序
    field_issues.sort_by(|a, b| b.issue_count.cmp(&a.issue_count));

    // 生成修复建议
    let mut recommendations: Vec<String> = Vec::new();
    if !field_issues.is_empty() {
        let top_issue = &field_issues[0];
        recommendations.push(format!(
            "优先修复「{}」字段问题（共{}条合同受影响）",
            top_issue.field_label, top_issue.issue_count
        ));
    }
    if warning_count > contracts.len() as i64 / 2 {
        recommendations.push("超过50%的合同有数据质量问题，建议批量核查数据源".to_string());
    }
    if error_count > 0 {
        recommendations.push(format!("{}条合同因关键字段缺失无法计算，请先修复", error_count));
    }

    let summary = ValidationSummary {
        batch_id,
        validation_time,
        total_contracts: contracts.len() as i64,
        valid_contracts: valid_count,
        warning_contracts: warning_count,
        error_contracts: error_count,
        issues_by_field,
    };

    let report = DataQualityReport {
        summary,
        field_issues,
        problem_contracts,
        recommendations,
    };

    Ok((validated_contracts, report))
}

/// 获取简化的警告信息列表
///
/// 用于在前端显示简洁的警告信息
pub fn get_warning_messages(validation: &ContractValidationResult) -> Vec<String> {
    validation.issues.iter()
        .filter(|i| i.severity == ValidationSeverity::Warning)
        .map(|i| format!("{}: {}", i.field_label, i.message))
        .collect()
}
