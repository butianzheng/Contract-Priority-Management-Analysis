//! 共识包导出模块
//!
//! # 功能
//! 将共识包导出为 CSV 格式，便于：
//! - 在 Excel 中进一步分析
//! - 邮件分发会议材料
//! - 归档备查
//!
//! # 导出内容
//! 1. 合同排名表（Top N）
//! 2. KPI 汇总表（四视角 16 个 KPI）
//! 3. 风险合同列表
//! 4. 客户保障分析
//! 5. 节拍顺行分析

use crate::kpi::{
    ConsensusPackage, ContractRankingSummary,
    CustomerProtectionSummary, DailyRhythmSummary,
    ActionRecommendation,
};
use crate::db::{KpiValue, RiskContractFlag};
use serde::Serialize;

/// CSV 导出结果
#[derive(Debug, Clone, Serialize)]
pub struct CsvExportResult {
    /// 是否成功
    pub success: bool,
    /// 导出文件路径（如果是单文件）
    pub file_path: Option<String>,
    /// 导出的多个文件路径（如果是多文件模式）
    pub file_paths: Vec<CsvFileInfo>,
    /// 导出的记录总数
    pub total_rows: i64,
    /// 消息
    pub message: String,
}

/// 单个 CSV 文件信息
#[derive(Debug, Clone, Serialize)]
pub struct CsvFileInfo {
    /// 文件名
    pub file_name: String,
    /// 文件路径
    pub file_path: String,
    /// 数据类型
    pub data_type: String,
    /// 行数
    pub row_count: i64,
}

/// CSV 导出配置
pub struct CsvExportConfig {
    /// 输出目录
    pub output_dir: String,
    /// 文件名前缀
    pub file_prefix: String,
    /// 是否添加 UTF-8 BOM（Excel 兼容）
    pub add_bom: bool,
    /// 是否导出合同排名
    pub export_contracts: bool,
    /// 是否导出 KPI
    pub export_kpis: bool,
    /// 是否导出风险合同
    pub export_risks: bool,
    /// 是否导出客户分析
    pub export_customers: bool,
    /// 是否导出节拍分析
    pub export_rhythm: bool,
    /// 是否导出行动建议
    pub export_recommendations: bool,
}

impl Default for CsvExportConfig {
    fn default() -> Self {
        Self {
            output_dir: ".".to_string(),
            file_prefix: "consensus".to_string(),
            add_bom: true,
            export_contracts: true,
            export_kpis: true,
            export_risks: true,
            export_customers: true,
            export_rhythm: true,
            export_recommendations: true,
        }
    }
}

/// 导出共识包为 CSV 文件
pub fn export_consensus_to_csv(
    package: &ConsensusPackage,
    config: &CsvExportConfig,
) -> Result<CsvExportResult, String> {
    let mut file_infos: Vec<CsvFileInfo> = Vec::new();
    let mut total_rows = 0i64;

    // 1. 导出合同排名
    if config.export_contracts {
        let info = export_contracts_csv(
            &package.top_contracts,
            &config.output_dir,
            &format!("{}_contracts.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    // 2. 导出 KPI 汇总
    if config.export_kpis {
        let all_kpis: Vec<&KpiValue> = package.kpi_summary.leadership.iter()
            .chain(package.kpi_summary.sales.iter())
            .chain(package.kpi_summary.production.iter())
            .chain(package.kpi_summary.finance.iter())
            .collect();

        let info = export_kpis_csv(
            &all_kpis,
            &config.output_dir,
            &format!("{}_kpis.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    // 3. 导出风险合同
    if config.export_risks && !package.risk_contracts.is_empty() {
        let info = export_risks_csv(
            &package.risk_contracts,
            &config.output_dir,
            &format!("{}_risks.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    // 4. 导出客户分析
    if config.export_customers && !package.customer_analysis.customers.is_empty() {
        let info = export_customers_csv(
            &package.customer_analysis.customers,
            &config.output_dir,
            &format!("{}_customers.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    // 5. 导出节拍分析
    if config.export_rhythm && !package.rhythm_analysis.daily_summaries.is_empty() {
        let info = export_rhythm_csv(
            &package.rhythm_analysis.daily_summaries,
            &config.output_dir,
            &format!("{}_rhythm.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    // 6. 导出行动建议
    if config.export_recommendations && !package.recommendations.is_empty() {
        let info = export_recommendations_csv(
            &package.recommendations,
            &config.output_dir,
            &format!("{}_recommendations.csv", config.file_prefix),
            config.add_bom,
        )?;
        total_rows += info.row_count;
        file_infos.push(info);
    }

    Ok(CsvExportResult {
        success: true,
        file_path: None,
        file_paths: file_infos.clone(),
        total_rows,
        message: format!("成功导出 {} 个文件，共 {} 行数据", file_infos.len(), total_rows),
    })
}

/// 导出合同排名 CSV
fn export_contracts_csv(
    contracts: &[ContractRankingSummary],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    // 添加 BOM
    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("排名,合同ID,客户ID,客户名称,客户等级,规格族,钢种,厚度,宽度,交期天数,毛利,S评分,P评分,优先级,Alpha,排名变化,风险标记,风险类型\n");

    // 数据行
    for c in contracts {
        content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{:.2},{:.2},{:.2},{:.4},{},{},{},{}\n",
            c.rank,
            c.contract_id,
            c.customer_id,
            c.customer_name.as_deref().unwrap_or(""),
            c.customer_level,
            c.spec_family,
            c.steel_grade,
            c.thickness,
            c.width,
            c.days_to_pdd,
            c.margin,
            c.s_score,
            c.p_score,
            c.priority,
            c.alpha.map(|a| format!("{:.3}", a)).unwrap_or_default(),
            c.rank_change.map(|r| r.to_string()).unwrap_or_default(),
            if c.has_risk { "是" } else { "" },
            c.risk_types.join(";"),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "contracts".to_string(),
        row_count: contracts.len() as i64,
    })
}

/// 导出 KPI CSV
fn export_kpis_csv(
    kpis: &[&KpiValue],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("KPI代码,KPI名称,数值,显示值,状态,变化值,变化方向,业务含义\n");

    // 数据行
    for kpi in kpis {
        content.push_str(&format!(
            "{},{},{:.2},{},{},{},{},{}\n",
            kpi.kpi_code,
            kpi.kpi_name,
            kpi.value,
            kpi.display_value,
            status_to_chinese(&kpi.status),
            kpi.change.map(|c| format!("{:.2}", c)).unwrap_or_default(),
            kpi.change_direction.as_deref().map(direction_to_chinese).unwrap_or(""),
            kpi.business_meaning.as_deref().unwrap_or(""),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "kpis".to_string(),
        row_count: kpis.len() as i64,
    })
}

/// 导出风险合同 CSV
fn export_risks_csv(
    risks: &[RiskContractFlag],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("合同ID,风险类型,风险等级,风险分数,风险描述,潜在损失,建议操作,处理优先级,处理状态\n");

    // 数据行
    for r in risks {
        content.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            r.contract_id,
            risk_type_to_chinese(&r.risk_type),
            risk_level_to_chinese(&r.risk_level),
            r.risk_score.map(|s| format!("{:.1}", s)).unwrap_or_default(),
            escape_csv(&r.risk_description),
            r.potential_loss.map(|l| format!("{:.2}", l)).unwrap_or_default(),
            r.suggested_action.as_deref().map(escape_csv).unwrap_or_default(),
            r.action_priority.map(|p| p.to_string()).unwrap_or_default(),
            status_to_chinese(&r.status),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "risks".to_string(),
        row_count: risks.len() as i64,
    })
}

/// 导出客户分析 CSV
fn export_customers_csv(
    customers: &[CustomerProtectionSummary],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("客户ID,客户名称,客户等级,合同数,平均排名,最佳排名,最差排名,Top50合同数,总毛利,保障评分,保障状态,风险描述\n");

    // 数据行
    for c in customers {
        content.push_str(&format!(
            "{},{},{},{},{:.1},{},{},{},{:.2},{:.1},{},{}\n",
            c.customer_id,
            c.customer_name.as_deref().unwrap_or(""),
            c.customer_level,
            c.contract_count,
            c.avg_rank,
            c.best_rank,
            c.worst_rank,
            c.top_n_count,
            c.total_margin,
            c.protection_score,
            status_to_chinese(&c.protection_status),
            c.risk_description.as_deref().map(escape_csv).unwrap_or_default(),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "customers".to_string(),
        row_count: customers.len() as i64,
    })
}

/// 导出节拍分析 CSV
fn export_rhythm_csv(
    rhythms: &[DailyRhythmSummary],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("节拍日,标签名称,匹配规格,匹配合同数,匹配产能,不匹配Top合同数,匹配率,节拍状态,状态说明\n");

    // 数据行
    for r in rhythms {
        content.push_str(&format!(
            "{},{},{},{},{:.2},{},{:.1}%,{},{}\n",
            r.rhythm_day,
            r.label_name,
            r.match_specs.join(";"),
            r.matched_contract_count,
            r.matched_total_capacity,
            r.mismatched_top_count,
            r.match_rate,
            rhythm_status_to_chinese(&r.rhythm_status),
            r.status_description.as_deref().map(escape_csv).unwrap_or_default(),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "rhythm".to_string(),
        row_count: rhythms.len() as i64,
    })
}

/// 导出行动建议 CSV
fn export_recommendations_csv(
    recommendations: &[ActionRecommendation],
    output_dir: &str,
    file_name: &str,
    add_bom: bool,
) -> Result<CsvFileInfo, String> {
    let file_path = format!("{}/{}", output_dir, file_name);
    let mut content = String::new();

    if add_bom {
        content.push('\u{FEFF}');
    }

    // 表头
    content.push_str("优先级,类别,标题,详情,相关合同,相关KPI,建议部门\n");

    // 数据行
    for r in recommendations {
        content.push_str(&format!(
            "{},{},{},{},{},{},{}\n",
            r.priority,
            category_to_chinese(&r.category),
            escape_csv(&r.title),
            escape_csv(&r.description),
            r.related_contracts.join(";"),
            r.related_kpis.join(";"),
            r.suggested_department.as_deref().unwrap_or(""),
        ));
    }

    std::fs::write(&file_path, content.as_bytes())
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(CsvFileInfo {
        file_name: file_name.to_string(),
        file_path,
        data_type: "recommendations".to_string(),
        row_count: recommendations.len() as i64,
    })
}

// ============================================
// 辅助函数
// ============================================

/// 转义 CSV 字段（处理逗号、引号、换行）
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

/// 状态转中文
fn status_to_chinese(status: &str) -> &'static str {
    match status {
        "good" => "良好",
        "warning" => "警告",
        "danger" => "危险",
        "risk" => "风险",
        "open" => "待处理",
        "in_progress" => "处理中",
        "resolved" => "已解决",
        "accepted" => "已接受",
        _ => "未知",
    }
}

/// 方向转中文
fn direction_to_chinese(direction: &str) -> &'static str {
    match direction {
        "up" => "上升",
        "down" => "下降",
        "unchanged" => "持平",
        _ => "",
    }
}

/// 风险类型转中文
fn risk_type_to_chinese(risk_type: &str) -> &'static str {
    match risk_type {
        "delivery_delay" => "交期延迟",
        "customer_downgrade" => "客户降级",
        "margin_loss" => "毛利损失",
        "rhythm_mismatch" => "节拍不匹配",
        "other" => "其他",
        _ => "未知风险",
    }
}

/// 风险等级转中文
fn risk_level_to_chinese(level: &str) -> &'static str {
    match level {
        "high" => "高",
        "medium" => "中",
        "low" => "低",
        _ => "未知",
    }
}

/// 节拍状态转中文
fn rhythm_status_to_chinese(status: &str) -> &'static str {
    match status {
        "smooth" => "顺行",
        "congested" => "拥堵",
        "idle" => "空闲",
        _ => "未知",
    }
}

/// 类别转中文
fn category_to_chinese(category: &str) -> &'static str {
    match category {
        "risk" => "风险处理",
        "kpi" => "KPI优化",
        "rhythm" => "节拍调整",
        "customer" => "客户保障",
        _ => "其他",
    }
}

/// 导出单个合同排名表为 CSV（简化接口）
pub fn export_contracts_ranking_csv(
    package: &ConsensusPackage,
    file_path: &str,
) -> Result<CsvExportResult, String> {
    let info = export_contracts_csv(
        &package.top_contracts,
        std::path::Path::new(file_path)
            .parent()
            .map(|p| p.to_str().unwrap_or("."))
            .unwrap_or("."),
        std::path::Path::new(file_path)
            .file_name()
            .map(|f| f.to_str().unwrap_or("contracts.csv"))
            .unwrap_or("contracts.csv"),
        true,
    )?;

    Ok(CsvExportResult {
        success: true,
        file_path: Some(file_path.to_string()),
        file_paths: vec![info.clone()],
        total_rows: info.row_count,
        message: format!("成功导出 {} 条合同排名数据", info.row_count),
    })
}
