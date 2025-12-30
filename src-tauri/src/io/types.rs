use serde::{Deserialize, Serialize};

/// 导入文件格式
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileFormat {
    Csv,
    Json,
    Excel,
}

impl FileFormat {
    #[allow(dead_code)]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "csv" => Some(FileFormat::Csv),
            "json" => Some(FileFormat::Json),
            "xlsx" | "xls" => Some(FileFormat::Excel),
            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn extension(&self) -> &'static str {
        match self {
            FileFormat::Csv => "csv",
            FileFormat::Json => "json",
            FileFormat::Excel => "xlsx",
        }
    }
}

/// 导入数据类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ImportDataType {
    Contracts,
    Customers,
    ProcessDifficulty,
    StrategyWeights,
}

impl ImportDataType {
    #[allow(dead_code)]
    pub fn label(&self) -> &'static str {
        match self {
            ImportDataType::Contracts => "合同数据",
            ImportDataType::Customers => "客户数据",
            ImportDataType::ProcessDifficulty => "工艺难度",
            ImportDataType::StrategyWeights => "策略权重",
        }
    }
}

/// 冲突处理策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ConflictStrategy {
    Skip,       // 跳过冲突记录
    Overwrite,  // 覆盖现有记录
}

/// 验证错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub row_number: usize,
    pub field: String,
    pub value: String,
    pub message: String,
}

/// 冲突记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictRecord {
    pub row_number: usize,
    pub primary_key: String,
    pub existing_data: serde_json::Value,
    pub new_data: serde_json::Value,
    pub action: Option<ConflictStrategy>,
}

/// 导入预览结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportPreview {
    pub total_rows: usize,
    pub valid_rows: usize,
    pub error_rows: usize,
    pub conflicts: Vec<ConflictRecord>,
    pub validation_errors: Vec<ValidationError>,
    pub sample_data: Vec<serde_json::Value>,
}

/// 导入结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: usize,
    pub skipped_count: usize,
    pub error_count: usize,
    pub errors: Vec<ValidationError>,
    pub message: String,
}

/// 导出选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: FileFormat,
    pub data_type: ImportDataType,
    pub include_computed: bool,
    pub strategy: Option<String>,
}

/// 导出结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub file_path: String,
    pub row_count: usize,
    pub message: String,
}
