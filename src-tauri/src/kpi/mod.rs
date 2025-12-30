//! KPI 计算引擎模块
//!
//! # 功能
//! 实现会议驾驶舱的四视角 KPI 计算：
//! - 领导视角 (Leadership): 整体业务健康度
//! - 销售视角 (Sales): 客户保障与满意度
//! - 生产视角 (Production): 生产效率与节拍匹配
//! - 财务视角 (Finance): 盈利能力与风险控制
//!
//! # 设计原则
//! - 每个 KPI 有独立的计算函数
//! - 统一输出 KpiValue 结构
//! - 支持阈值比较生成红黄绿灯状态
//!
//! # P2 扩展模块
//! - customer_analysis: 客户保障分析（客户维度聚合）
//! - rhythm_analysis: 节拍顺行分析（生产维度聚合）
//!
//! # P3 扩展模块
//! - consensus: 共识包生成（会议材料打包）
//! - export: CSV 导出（会议材料导出）

mod calculator;
mod consensus;
mod customer_analysis;
mod export;
mod ranking;
mod rhythm_analysis;
mod risk;

pub use calculator::*;
pub use consensus::*;
pub use customer_analysis::*;
pub use export::*;
pub use ranking::*;
pub use rhythm_analysis::*;
pub use risk::*;
