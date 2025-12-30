-- ============================================
-- Phase 5: 批量操作相关表
-- ============================================

-- 批量操作记录表
CREATE TABLE IF NOT EXISTS batch_operations (
    batch_id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type TEXT NOT NULL,  -- 'adjust' 或 'restore'
    contract_count INTEGER NOT NULL,  -- 受影响的合同数量
    reason TEXT NOT NULL,
    user TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- 批量操作索引
CREATE INDEX IF NOT EXISTS idx_batch_operations_type ON batch_operations(operation_type);
CREATE INDEX IF NOT EXISTS idx_batch_operations_user ON batch_operations(user);
CREATE INDEX IF NOT EXISTS idx_batch_operations_created_at ON batch_operations(created_at);

-- 修改 intervention_log 表,添加 batch_id 字段
-- 注意:SQLite 不支持 ALTER TABLE ADD COLUMN IF NOT EXISTS
-- 使用 PRAGMA 检查字段是否存在,如果已存在则忽略错误
-- 这里直接尝试添加,如果失败说明字段已存在(正常情况)
-- ALTER TABLE intervention_log ADD COLUMN batch_id INTEGER REFERENCES batch_operations(batch_id);

-- 如果字段不存在才添加(通过检查 schema)
-- 由于 SQLite 限制,此语句在字段已存在时会失败,但不影响表结构
-- 建议:首次运行会成功,后续运行会被 is_new_db 条件跳过

-- 为 batch_id 创建索引
CREATE INDEX IF NOT EXISTS idx_intervention_log_batch_id ON intervention_log(batch_id);
