use rusqlite::{Connection, Result};
use std::path::PathBuf;
use std::sync::OnceLock;
use tauri::Config;

// 🔥 性能优化：连接池
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

static DB_PATH: OnceLock<PathBuf> = OnceLock::new();

// 🔥 性能优化：全局连接池
static DB_POOL: OnceLock<Pool<SqliteConnectionManager>> = OnceLock::new();

/// 根据运行时配置解析数据库路径
fn resolve_db_path(config: &Config) -> PathBuf {
    // 使用应用数据目录存储数据库，确保与 tauri.conf.json 的 identifier 一致
    let mut path = tauri::api::path::app_data_dir(config)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    path.push("dpm.db");
    path
}

/// 检查指定表是否存在
fn table_exists(conn: &Connection, table_name: &str) -> bool {
    conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name=?1")
        .and_then(|mut stmt| stmt.exists([table_name]))
        .unwrap_or(false)
}

/// 检查指定表中是否存在某列
fn column_exists(conn: &Connection, table_name: &str, column_name: &str) -> bool {
    conn.prepare(&format!("PRAGMA table_info({})", table_name))
        .and_then(|mut stmt| {
            let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
            for name in rows.flatten() {
                if name == column_name {
                    return Ok(true);
                }
            }
            Ok(false)
        })
        .unwrap_or(false)
}

/// 运行迁移（如果目标表不存在）
/// 返回是否执行了迁移
fn run_migration_if_needed(conn: &Connection, table_name: &str, migration_sql: &str, migration_name: &str) -> Result<bool> {
    if !table_exists(conn, table_name) {
        conn.execute_batch(migration_sql)?;
        println!("  ✓ Migration applied: {}", migration_name);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// 初始化数据库
/// 创建数据库文件并执行迁移脚本
pub fn initialize_database(config: &Config) -> Result<()> {
    let db_path = DB_PATH
        .get_or_init(|| resolve_db_path(config))
        .clone();

    // 确保父目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    let conn = Connection::open(&db_path)?;

    // 检查数据库是否为新建（通过检查表是否存在）
    let is_new_db = conn
        .prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='contract_master'")
        .and_then(|mut stmt| stmt.exists([]))
        .map(|exists| !exists)
        .unwrap_or(true);

    // 执行初始化 SQL（001: 基础表）
    let init_sql = include_str!("../../migrations/001_init.sql");
    conn.execute_batch(init_sql)?;

    // 如果是新数据库，加载测试数据
    if is_new_db {
        let seed_sql = include_str!("../../migrations/002_seed.sql");
        conn.execute_batch(seed_sql)?;
        println!("Test data loaded");
    }

    // Phase 2: 配置表（003: config tables）
    let config_tables_sql = include_str!("../../migrations/003_config_tables.sql");
    conn.execute_batch(config_tables_sql)?;

    // Phase 2: 配置初始数据（004: config seed）- 只在新数据库时执行
    if is_new_db {
        let config_seed_sql = include_str!("../../migrations/004_config_seed.sql");
        conn.execute_batch(config_seed_sql)?;
    }

    // 性能测试数据（016: 10k 合同样本，仅新库时加载）
    if is_new_db {
        let perf_seed_sql = include_str!("../../migrations/016_perf_seed.sql");
        conn.execute_batch(perf_seed_sql)?;
        println!("Performance seed (10k contracts) loaded");
    }

    // Phase 2: 配置变更日志表（005: config change log）
    let config_change_log_sql = include_str!("../../migrations/005_config_change_log.sql");
    conn.execute_batch(config_change_log_sql)?;

    // Phase 4: 筛选预设表（006: filter presets）
    run_migration_if_needed(
        &conn,
        "filter_presets",
        include_str!("../../migrations/006_filter_presets.sql"),
        "006_filter_presets"
    )?;

    // Phase 5: 批量操作表（007: batch operations）
    run_migration_if_needed(
        &conn,
        "batch_operations",
        include_str!("../../migrations/007_batch_operations.sql"),
        "007_batch_operations"
    )?;

    // Phase 8: 清洗规则表（008: transform rules）
    run_migration_if_needed(
        &conn,
        "transform_rules",
        include_str!("../../migrations/008_transform_rules.sql"),
        "008_transform_rules"
    )?;

    // Phase 9: 规格族主数据表（009: spec family）
    run_migration_if_needed(
        &conn,
        "spec_family_master",
        include_str!("../../migrations/009_spec_family.sql"),
        "009_spec_family"
    )?;

    // Phase 10: n日节拍配置表（010: rhythm config）
    run_migration_if_needed(
        &conn,
        "rhythm_config",
        include_str!("../../migrations/010_rhythm_config.sql"),
        "010_rhythm_config"
    )?;

    // Phase 11: 聚合配置表（011: aggregation config）
    // 包含 aggregation_bins, p2_curve_config 等表，P-Score 计算必需
    run_migration_if_needed(
        &conn,
        "aggregation_bins",
        include_str!("../../migrations/011_aggregation_config.sql"),
        "011_aggregation_config"
    )?;

    // Phase 12: 数据校验表（012: data validation）
    run_migration_if_needed(
        &conn,
        "missing_value_strategies",
        include_str!("../../migrations/012_data_validation.sql"),
        "012_data_validation"
    )?;

    // Phase 13: 策略版本化表（013: strategy versioning）
    run_migration_if_needed(
        &conn,
        "strategy_versions",
        include_str!("../../migrations/013_strategy_versioning.sql"),
        "013_strategy_versioning"
    )?;

    // Phase 14: 导入审计表（014: import audit）
    run_migration_if_needed(
        &conn,
        "import_audit_log",
        include_str!("../../migrations/014_import_audit.sql"),
        "014_import_audit"
    )?;

    // Phase 15: 会议驾驶舱表（015: meeting cockpit）
    run_migration_if_needed(
        &conn,
        "meeting_snapshot",
        include_str!("../../migrations/015_meeting_cockpit.sql"),
        "015_meeting_cockpit"
    )?;

    // Phase 5 补丁: 为 intervention_log 添加 batch_id 列（018: intervention batch_id）
    // 修复批量调整功能缺少的列
    if !column_exists(&conn, "intervention_log", "batch_id") {
        let batch_id_sql = include_str!("../../migrations/018_intervention_batch_id.sql");
        conn.execute_batch(batch_id_sql)?;
        println!("  ✓ Migration applied: 018_intervention_batch_id");
    }

    println!("Database initialized at: {:?}", db_path);
    println!("All migrations executed successfully");

    // 🔥 性能优化：初始化连接池（所有迁移完成后）
    drop(conn); // 关闭初始化连接
    let manager = SqliteConnectionManager::file(&db_path);
    let pool = Pool::builder()
        .max_size(10)           // 最大连接数：10
        .min_idle(Some(2))      // 最小空闲连接：2
        .build(manager)
        .map_err(|_| rusqlite::Error::InvalidQuery)?;

    DB_POOL.set(pool).map_err(|_| rusqlite::Error::InvalidQuery)?;
    println!("🔥 Connection pool initialized (max_size=10, min_idle=2)");

    Ok(())
}

/// 获取数据库连接
/// 🔥 性能优化：从连接池获取连接（而非每次创建新连接）
pub fn get_connection() -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
    DB_POOL
        .get()
        .ok_or(rusqlite::Error::InvalidQuery)?
        .get()
        .map_err(|_| rusqlite::Error::InvalidQuery)
}
