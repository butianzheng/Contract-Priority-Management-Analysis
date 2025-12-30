use calamine::{Reader, Xlsx, Data, DataType};
use rust_xlsxwriter::{Workbook, Format, Color};
use std::io::Cursor;
use crate::db::schema::{Contract, Customer, ContractPriority, ProcessDifficulty};
use super::types::ValidationError;
use super::validator::Validator;

pub struct ExcelHandler;

impl ExcelHandler {
    /// 解析 Excel 为合同数据
    pub fn parse_contracts(content: &[u8]) -> Result<(Vec<Contract>, Vec<ValidationError>), String> {
        let cursor = Cursor::new(content);
        let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
            .map_err(|e| format!("无法读取Excel文件: {}", e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names.first()
            .ok_or_else(|| "Excel文件没有工作表".to_string())?;

        let range = workbook.worksheet_range(sheet_name)
            .map_err(|e| format!("无法读取工作表: {}", e))?;

        let mut contracts = Vec::new();
        let mut errors = Vec::new();

        // 跳过表头，从第2行开始
        for (row_idx, row) in range.rows().skip(1).enumerate() {
            let row_num = row_idx + 2;

            match Self::parse_contract_row(row, row_num) {
                Ok(contract) => {
                    match Validator::validate_contract(&contract, row_num) {
                        Ok(_) => contracts.push(contract),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        Ok((contracts, errors))
    }

    fn parse_contract_row(row: &[Data], row_num: usize) -> Result<Contract, ValidationError> {
        let get_string = |idx: usize, field: &str| -> Result<String, ValidationError> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(s) = v.get_string() {
                        Some(s.to_string())
                    } else if let Some(f) = v.get_float() {
                        Some(f.to_string())
                    } else if let Some(i) = v.get_int() {
                        Some(i.to_string())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列缺失或格式错误", field),
                })
        };

        let get_float = |idx: usize, field: &str| -> Result<f64, ValidationError> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(f) = v.get_float() {
                        Some(f)
                    } else if let Some(i) = v.get_int() {
                        Some(i as f64)
                    } else if let Some(s) = v.get_string() {
                        s.parse().ok()
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列必须是数字", field),
                })
        };

        let get_int = |idx: usize, field: &str| -> Result<i64, ValidationError> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(i) = v.get_int() {
                        Some(i)
                    } else if let Some(f) = v.get_float() {
                        Some(f as i64)
                    } else if let Some(s) = v.get_string() {
                        s.parse().ok()
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列必须是整数", field),
                })
        };

        Ok(Contract {
            contract_id: get_string(0, "contract_id")?,
            customer_id: get_string(1, "customer_id")?,
            steel_grade: get_string(2, "steel_grade")?,
            thickness: get_float(3, "thickness")?,
            width: get_float(4, "width")?,
            spec_family: get_string(5, "spec_family")?,
            pdd: get_string(6, "pdd")?,
            days_to_pdd: get_int(7, "days_to_pdd")?,
            margin: get_float(8, "margin")?,
        })
    }

    /// 解析 Excel 为客户数据
    pub fn parse_customers(content: &[u8]) -> Result<(Vec<Customer>, Vec<ValidationError>), String> {
        let cursor = Cursor::new(content);
        let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
            .map_err(|e| format!("无法读取Excel文件: {}", e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names.first()
            .ok_or_else(|| "Excel文件没有工作表".to_string())?;

        let range = workbook.worksheet_range(sheet_name)
            .map_err(|e| format!("无法读取工作表: {}", e))?;

        let mut customers = Vec::new();
        let mut errors = Vec::new();

        for (row_idx, row) in range.rows().skip(1).enumerate() {
            let row_num = row_idx + 2;

            match Self::parse_customer_row(row, row_num) {
                Ok(customer) => {
                    match Validator::validate_customer(&customer, row_num) {
                        Ok(_) => customers.push(customer),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        Ok((customers, errors))
    }

    fn parse_customer_row(row: &[Data], row_num: usize) -> Result<Customer, ValidationError> {
        let get_string = |idx: usize, field: &str| -> Result<String, ValidationError> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(s) = v.get_string() {
                        Some(s.to_string())
                    } else if let Some(i) = v.get_int() {
                        Some(i.to_string())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列缺失或格式错误", field),
                })
        };

        let get_optional_string = |idx: usize| -> Option<String> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(s) = v.get_string() {
                        if !s.is_empty() { Some(s.to_string()) } else { None }
                    } else if let Some(i) = v.get_int() {
                        Some(i.to_string())
                    } else {
                        None
                    }
                })
        };

        Ok(Customer {
            customer_id: get_string(0, "customer_id")?,
            customer_name: get_optional_string(1),
            customer_level: get_string(2, "customer_level")?,
            credit_level: get_optional_string(3),
            customer_group: get_optional_string(4),
        })
    }

    /// 解析 Excel 为工艺难度数据
    #[allow(dead_code)]
    pub fn parse_process_difficulty(content: &[u8]) -> Result<(Vec<ProcessDifficulty>, Vec<ValidationError>), String> {
        let cursor = Cursor::new(content);
        let mut workbook: Xlsx<_> = calamine::open_workbook_from_rs(cursor)
            .map_err(|e| format!("无法读取Excel文件: {}", e))?;

        let sheet_names = workbook.sheet_names().to_vec();
        let sheet_name = sheet_names.first()
            .ok_or_else(|| "Excel文件没有工作表".to_string())?;

        let range = workbook.worksheet_range(sheet_name)
            .map_err(|e| format!("无法读取工作表: {}", e))?;

        let mut items = Vec::new();
        let mut errors = Vec::new();

        for (row_idx, row) in range.rows().skip(1).enumerate() {
            let row_num = row_idx + 2;

            match Self::parse_process_difficulty_row(row, row_num) {
                Ok(item) => {
                    match Validator::validate_process_difficulty(&item, row_num) {
                        Ok(_) => items.push(item),
                        Err(e) => errors.push(e),
                    }
                }
                Err(e) => errors.push(e),
            }
        }

        Ok((items, errors))
    }

    #[allow(dead_code)]
    fn parse_process_difficulty_row(row: &[Data], row_num: usize) -> Result<ProcessDifficulty, ValidationError> {
        let get_string = |idx: usize, field: &str| -> Result<String, ValidationError> {
            row.get(idx)
                .and_then(|v| v.get_string().map(|s| s.to_string()))
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列缺失或格式错误", field),
                })
        };

        let get_float = |idx: usize, field: &str| -> Result<f64, ValidationError> {
            row.get(idx)
                .and_then(|v| {
                    if let Some(f) = v.get_float() {
                        Some(f)
                    } else if let Some(i) = v.get_int() {
                        Some(i as f64)
                    } else if let Some(s) = v.get_string() {
                        s.parse().ok()
                    } else {
                        None
                    }
                })
                .ok_or_else(|| ValidationError {
                    row_number: row_num,
                    field: field.to_string(),
                    value: "".to_string(),
                    message: format!("{}列必须是数字", field),
                })
        };

        let get_int = |idx: usize, _field: &str| -> i64 {
            row.get(idx)
                .and_then(|v| {
                    if let Some(i) = v.get_int() {
                        Some(i)
                    } else if let Some(f) = v.get_float() {
                        Some(f as i64)
                    } else {
                        None
                    }
                })
                .unwrap_or(0)
        };

        Ok(ProcessDifficulty {
            id: get_int(0, "id"),
            steel_grade: get_string(1, "steel_grade")?,
            thickness_min: get_float(2, "thickness_min")?,
            thickness_max: get_float(3, "thickness_max")?,
            width_min: get_float(4, "width_min")?,
            width_max: get_float(5, "width_max")?,
            difficulty_level: get_string(6, "difficulty_level")?,
            difficulty_score: get_float(7, "difficulty_score")?,
        })
    }

    /// 生成优先级结果 Excel
    pub fn generate_priorities(priorities: &[ContractPriority]) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        // 表头格式
        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x1890FF))
            .set_font_color(Color::White);

        // 写入表头
        let headers = [
            "合同编号", "客户编号", "钢种", "厚度(mm)", "宽度(mm)",
            "规格族", "交期", "剩余天数", "毛利",
            "S分数", "P分数", "优先级", "调整系数"
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // 写入数据
        for (row_idx, p) in priorities.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_string(row, 0, &p.contract.contract_id).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 1, &p.contract.customer_id).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 2, &p.contract.steel_grade).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 3, p.contract.thickness).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 4, p.contract.width).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 5, &p.contract.spec_family).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 6, &p.contract.pdd).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 7, p.contract.days_to_pdd as f64).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 8, p.contract.margin).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 9, p.s_score).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 10, p.p_score).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 11, p.priority).map_err(|e| e.to_string())?;
            if let Some(alpha) = p.alpha {
                worksheet.write_number(row, 12, alpha).map_err(|e| e.to_string())?;
            }
        }

        // 设置列宽
        for col in 0..13u16 {
            worksheet.set_column_width(col, 12.0).map_err(|e| e.to_string())?;
        }

        workbook.save_to_buffer().map_err(|e| e.to_string())
    }

    /// 生成合同数据 Excel
    pub fn generate_contracts(contracts: &[Contract]) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x1890FF))
            .set_font_color(Color::White);

        let headers = [
            "合同编号", "客户编号", "钢种", "厚度(mm)", "宽度(mm)",
            "规格族", "交期", "剩余天数", "毛利"
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, c) in contracts.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_string(row, 0, &c.contract_id).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 1, &c.customer_id).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 2, &c.steel_grade).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 3, c.thickness).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 4, c.width).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 5, &c.spec_family).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 6, &c.pdd).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 7, c.days_to_pdd as f64).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 8, c.margin).map_err(|e| e.to_string())?;
        }

        for col in 0..9u16 {
            worksheet.set_column_width(col, 12.0).map_err(|e| e.to_string())?;
        }

        workbook.save_to_buffer().map_err(|e| e.to_string())
    }

    /// 生成客户数据 Excel
    pub fn generate_customers(customers: &[Customer]) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x1890FF))
            .set_font_color(Color::White);

        let headers = ["客户编号", "客户名称", "客户等级", "信用等级", "客户分组"];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, c) in customers.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_string(row, 0, &c.customer_id).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 1, c.customer_name.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 2, &c.customer_level).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 3, c.credit_level.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 4, c.customer_group.as_deref().unwrap_or("")).map_err(|e| e.to_string())?;
        }

        for col in 0..5u16 {
            worksheet.set_column_width(col, 15.0).map_err(|e| e.to_string())?;
        }

        workbook.save_to_buffer().map_err(|e| e.to_string())
    }

    /// 生成工艺难度数据 Excel
    pub fn generate_process_difficulty(items: &[ProcessDifficulty]) -> Result<Vec<u8>, String> {
        let mut workbook = Workbook::new();
        let worksheet = workbook.add_worksheet();

        let header_format = Format::new()
            .set_bold()
            .set_background_color(Color::RGB(0x1890FF))
            .set_font_color(Color::White);

        let headers = ["ID", "钢种", "最小厚度", "最大厚度", "最小宽度", "最大宽度", "难度等级", "难度分数"];

        for (col, header) in headers.iter().enumerate() {
            worksheet.write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, item) in items.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet.write_number(row, 0, item.id as f64).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 1, &item.steel_grade).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 2, item.thickness_min).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 3, item.thickness_max).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 4, item.width_min).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 5, item.width_max).map_err(|e| e.to_string())?;
            worksheet.write_string(row, 6, &item.difficulty_level).map_err(|e| e.to_string())?;
            worksheet.write_number(row, 7, item.difficulty_score).map_err(|e| e.to_string())?;
        }

        for col in 0..8u16 {
            worksheet.set_column_width(col, 12.0).map_err(|e| e.to_string())?;
        }

        workbook.save_to_buffer().map_err(|e| e.to_string())
    }
}
