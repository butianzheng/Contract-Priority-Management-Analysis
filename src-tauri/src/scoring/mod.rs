pub mod s_score;
pub mod p_score;
pub mod priority;

pub use s_score::*;
pub use p_score::*;
pub use priority::*;

// 导出 Explain 生成函数（仅保留实际使用的）
pub use s_score::generate_s_score_explain;
pub use p_score::{
    generate_p_score_explain_with_aggregation,
    calc_p_score_with_aggregation,
};
