use super::types::ValidationError;
use calamine::{Reader, Xlsx};
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::io::Cursor;

/// 通用行类型：字段名 → 字符串值
pub type RawRow = HashMap<String, String>;

/// 通用解析结果
#[derive(Debug, Clone)]
pub struct GenericParseResult {
    pub headers: Vec<String>,
    pub rows: Vec<RawRow>,
    pub parse_errors: Vec<ValidationError>,
}

pub struct GenericParser;

impl GenericParser {
    /// 统一入口：根据格式解析文件内容为通用行
    pub fn parse(
        content: &[u8],
        format: super::types::FileFormat,
    ) -> Result<GenericParseResult, String> {
        match format {
            super::types::FileFormat::Csv => Self::parse_csv(content),
            super::types::FileFormat::Excel => Self::parse_excel(content),
            super::types::FileFormat::Json => Self::parse_json(content),
        }
    }

    /// 仅解析文件头（用于前端获取源列名）
    pub fn parse_headers(
        content: &[u8],
        format: super::types::FileFormat,
    ) -> Result<Vec<String>, String> {
        match format {
            super::types::FileFormat::Csv => Self::parse_csv_headers(content),
            super::types::FileFormat::Excel => Self::parse_excel_headers(content),
            super::types::FileFormat::Json => Self::parse_json_headers(content),
        }
    }

    /// 解析 CSV 为通用行
    pub fn parse_csv(content: &[u8]) -> Result<GenericParseResult, String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .flexible(true)
            .from_reader(Cursor::new(content));

        let headers: Vec<String> = reader
            .headers()
            .map_err(|e| format!("读取CSV表头失败: {}", e))?
            .iter()
            .map(|h| h.trim().to_string())
            .collect();

        let mut rows = Vec::new();
        let mut parse_errors = Vec::new();

        for (idx, result) in reader.records().enumerate() {
            let row_num = idx + 2; // +1 for 0-index, +1 for header
            match result {
                Ok(record) => {
                    let mut row = RawRow::new();
                    for (col_idx, value) in record.iter().enumerate() {
                        if let Some(header) = headers.get(col_idx) {
                            row.insert(header.clone(), value.trim().to_string());
                        }
                    }
                    rows.push(row);
                }
                Err(e) => {
                    parse_errors.push(ValidationError {
                        row_number: row_num,
                        field: "row".to_string(),
                        value: "".to_string(),
                        message: format!("CSV行解析错误: {}", e),
                    });
                }
            }
        }

        Ok(GenericParseResult {
            headers,
            rows,
            parse_errors,
        })
    }

    /// 解析 Excel 为通用行
    pub fn parse_excel(content: &[u8]) -> Result<GenericParseResult, String> {
        let cursor = Cursor::new(content);
        let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
            .map_err(|e| format!("无法读取Excel文件: {}", e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names
            .first()
            .ok_or_else(|| "Excel文件没有工作表".to_string())?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("无法读取工作表: {}", e))?;

        let mut row_iter = range.rows();

        // 读取表头
        let header_row = row_iter
            .next()
            .ok_or_else(|| "Excel文件没有数据行".to_string())?;
        let headers: Vec<String> = header_row
            .iter()
            .map(|cell| Self::cell_to_string(cell).trim().to_string())
            .collect();

        let mut rows = Vec::new();
        let mut parse_errors = Vec::new();

        for (row_idx, row) in row_iter.enumerate() {
            let row_num = row_idx + 2;
            let mut raw_row = RawRow::new();
            let mut has_data = false;

            for (col_idx, cell) in row.iter().enumerate() {
                if let Some(header) = headers.get(col_idx) {
                    let value = Self::cell_to_string(cell);
                    if !value.is_empty() {
                        has_data = true;
                    }
                    raw_row.insert(header.clone(), value);
                }
            }

            // 跳过完全空行
            if has_data {
                rows.push(raw_row);
            } else if row_idx < 1000 {
                // 只对前1000行报告空行警告，避免大文件尾部空行产生大量警告
                parse_errors.push(ValidationError {
                    row_number: row_num,
                    field: "row".to_string(),
                    value: "".to_string(),
                    message: "空行已跳过".to_string(),
                });
            }
        }

        Ok(GenericParseResult {
            headers,
            rows,
            parse_errors,
        })
    }

    /// 解析 JSON 为通用行
    pub fn parse_json(content: &[u8]) -> Result<GenericParseResult, String> {
        let content_str =
            std::str::from_utf8(content).map_err(|e| format!("UTF-8 解码错误: {}", e))?;

        let json_value: serde_json::Value =
            serde_json::from_str(content_str).map_err(|e| format!("JSON 解析错误: {}", e))?;

        let array = match &json_value {
            serde_json::Value::Array(arr) => arr,
            _ => return Err("JSON 根元素必须是数组".to_string()),
        };

        if array.is_empty() {
            return Ok(GenericParseResult {
                headers: Vec::new(),
                rows: Vec::new(),
                parse_errors: Vec::new(),
            });
        }

        // 从所有对象中收集所有字段名作为 headers
        let mut header_set = indexmap::IndexSet::new();
        for item in array {
            if let serde_json::Value::Object(obj) = item {
                for key in obj.keys() {
                    header_set.insert(key.clone());
                }
            }
        }
        let headers: Vec<String> = header_set.into_iter().collect();

        let mut rows = Vec::new();
        let mut parse_errors = Vec::new();

        for (idx, item) in array.iter().enumerate() {
            let row_num = idx + 1;
            match item {
                serde_json::Value::Object(obj) => {
                    let mut row = RawRow::new();
                    for header in &headers {
                        let value = match obj.get(header.as_str()) {
                            Some(serde_json::Value::String(s)) => s.clone(),
                            Some(serde_json::Value::Number(n)) => n.to_string(),
                            Some(serde_json::Value::Bool(b)) => b.to_string(),
                            Some(serde_json::Value::Null) => String::new(),
                            Some(other) => other.to_string(),
                            None => String::new(),
                        };
                        row.insert(header.clone(), value);
                    }
                    rows.push(row);
                }
                _ => {
                    parse_errors.push(ValidationError {
                        row_number: row_num,
                        field: "row".to_string(),
                        value: "".to_string(),
                        message: "JSON数组元素必须是对象".to_string(),
                    });
                }
            }
        }

        Ok(GenericParseResult {
            headers,
            rows,
            parse_errors,
        })
    }

    // === 仅解析表头的辅助方法 ===

    fn parse_csv_headers(content: &[u8]) -> Result<Vec<String>, String> {
        let mut reader = ReaderBuilder::new()
            .has_headers(true)
            .from_reader(Cursor::new(content));

        let headers = reader
            .headers()
            .map_err(|e| format!("读取CSV表头失败: {}", e))?
            .iter()
            .map(|h| h.trim().to_string())
            .collect();

        Ok(headers)
    }

    fn parse_excel_headers(content: &[u8]) -> Result<Vec<String>, String> {
        let cursor = Cursor::new(content);
        let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
            .map_err(|e| format!("无法读取Excel文件: {}", e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names
            .first()
            .ok_or_else(|| "Excel文件没有工作表".to_string())?;

        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| format!("无法读取工作表: {}", e))?;

        let header_row = range
            .rows()
            .next()
            .ok_or_else(|| "Excel文件没有数据行".to_string())?;

        Ok(header_row
            .iter()
            .map(|cell| Self::cell_to_string(cell).trim().to_string())
            .collect())
    }

    fn parse_json_headers(content: &[u8]) -> Result<Vec<String>, String> {
        let content_str =
            std::str::from_utf8(content).map_err(|e| format!("UTF-8 解码错误: {}", e))?;

        let json_value: serde_json::Value =
            serde_json::from_str(content_str).map_err(|e| format!("JSON 解析错误: {}", e))?;

        let array = match &json_value {
            serde_json::Value::Array(arr) => arr,
            _ => return Err("JSON 根元素必须是数组".to_string()),
        };

        let mut header_set = indexmap::IndexSet::new();
        for item in array.iter().take(10) {
            if let serde_json::Value::Object(obj) = item {
                for key in obj.keys() {
                    header_set.insert(key.clone());
                }
            }
        }

        Ok(header_set.into_iter().collect())
    }

    /// Excel 单元格转字符串
    fn cell_to_string(cell: &calamine::Data) -> String {
        match cell {
            calamine::Data::Empty => String::new(),
            calamine::Data::String(s) => s.clone(),
            calamine::Data::Float(f) => {
                // 如果是整数值，去掉小数点
                if *f == (*f as i64) as f64 {
                    format!("{}", *f as i64)
                } else {
                    f.to_string()
                }
            }
            calamine::Data::Int(i) => i.to_string(),
            calamine::Data::Bool(b) => b.to_string(),
            calamine::Data::Error(e) => format!("#ERR:{:?}", e),
            calamine::Data::DateTime(dt) => {
                // calamine DateTime 转字符串
                if let Some(s) = (*dt).as_datetime() {
                    s.format("%Y-%m-%d").to_string()
                } else {
                    format!("{}", dt)
                }
            }
            calamine::Data::DateTimeIso(s) => s.clone(),
            calamine::Data::DurationIso(s) => s.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_basic() {
        let csv_content = b"name,age,city\nAlice,30,Beijing\nBob,25,Shanghai";
        let result = GenericParser::parse_csv(csv_content).unwrap();
        assert_eq!(result.headers, vec!["name", "age", "city"]);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("name").unwrap(), "Alice");
        assert_eq!(result.rows[1].get("age").unwrap(), "25");
    }

    #[test]
    fn test_parse_json_basic() {
        let json_content = br#"[{"name":"Alice","age":30},{"name":"Bob","age":25}]"#;
        let result = GenericParser::parse_json(json_content).unwrap();
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.rows.len(), 2);
        assert_eq!(result.rows[0].get("name").unwrap(), "Alice");
        assert_eq!(result.rows[1].get("age").unwrap(), "25");
    }

    #[test]
    fn test_parse_csv_headers_only() {
        let csv_content = b"contract_id,customer_id,steel_grade\nC001,K001,Q235";
        let headers = GenericParser::parse_csv_headers(csv_content).unwrap();
        assert_eq!(headers, vec!["contract_id", "customer_id", "steel_grade"]);
    }
}
