use crate::db::schema::{Contract, ContractPriority, Customer, ProcessDifficulty};
use rust_xlsxwriter::{Color, Format, Workbook};

pub struct ExcelHandler;

impl ExcelHandler {
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
            "合同编号",
            "客户编号",
            "钢种",
            "厚度(mm)",
            "宽度(mm)",
            "规格族",
            "交期",
            "剩余天数",
            "毛利",
            "S分数",
            "P分数",
            "优先级",
            "调整系数",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        // 写入数据
        for (row_idx, p) in priorities.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet
                .write_string(row, 0, &p.contract.contract_id)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &p.contract.customer_id)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 2, &p.contract.steel_grade)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 3, p.contract.thickness)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 4, p.contract.width)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 5, &p.contract.spec_family)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 6, &p.contract.pdd)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 7, p.contract.days_to_pdd as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 8, p.contract.margin)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 9, p.s_score)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 10, p.p_score)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 11, p.priority)
                .map_err(|e| e.to_string())?;
            if let Some(alpha) = p.alpha {
                worksheet
                    .write_number(row, 12, alpha)
                    .map_err(|e| e.to_string())?;
            }
        }

        // 设置列宽
        for col in 0..13u16 {
            worksheet
                .set_column_width(col, 12.0)
                .map_err(|e| e.to_string())?;
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
            "合同编号",
            "客户编号",
            "钢种",
            "厚度(mm)",
            "宽度(mm)",
            "规格族",
            "交期",
            "剩余天数",
            "毛利",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, c) in contracts.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet
                .write_string(row, 0, &c.contract_id)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &c.customer_id)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 2, &c.steel_grade)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 3, c.thickness)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 4, c.width)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 5, &c.spec_family)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 6, &c.pdd)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 7, c.days_to_pdd as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 8, c.margin)
                .map_err(|e| e.to_string())?;
        }

        for col in 0..9u16 {
            worksheet
                .set_column_width(col, 12.0)
                .map_err(|e| e.to_string())?;
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
            worksheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, c) in customers.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet
                .write_string(row, 0, &c.customer_id)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, c.customer_name.as_deref().unwrap_or(""))
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 2, &c.customer_level)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 3, c.credit_level.as_deref().unwrap_or(""))
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 4, c.customer_group.as_deref().unwrap_or(""))
                .map_err(|e| e.to_string())?;
        }

        for col in 0..5u16 {
            worksheet
                .set_column_width(col, 15.0)
                .map_err(|e| e.to_string())?;
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

        let headers = [
            "ID",
            "钢种",
            "最小厚度",
            "最大厚度",
            "最小宽度",
            "最大宽度",
            "难度等级",
            "难度分数",
        ];

        for (col, header) in headers.iter().enumerate() {
            worksheet
                .write_string_with_format(0, col as u16, *header, &header_format)
                .map_err(|e| e.to_string())?;
        }

        for (row_idx, item) in items.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            worksheet
                .write_number(row, 0, item.id as f64)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 1, &item.steel_grade)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 2, item.thickness_min)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 3, item.thickness_max)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 4, item.width_min)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 5, item.width_max)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_string(row, 6, &item.difficulty_level)
                .map_err(|e| e.to_string())?;
            worksheet
                .write_number(row, 7, item.difficulty_score)
                .map_err(|e| e.to_string())?;
        }

        for col in 0..8u16 {
            worksheet
                .set_column_width(col, 12.0)
                .map_err(|e| e.to_string())?;
        }

        workbook.save_to_buffer().map_err(|e| e.to_string())
    }
}
