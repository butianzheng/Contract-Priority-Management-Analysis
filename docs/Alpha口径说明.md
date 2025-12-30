# Alpha（人工干预系数）口径说明

## 1. 定义与公式

### 1.1 数学定义

Alpha 是用于人工调整合同优先级的乘数系数。

**统一公式**：

```
Final Priority = Base Priority × Alpha
```

其中：
- `Base Priority = ws × S-Score + wp × P-Score`
- `Alpha` 为人工调整系数

### 1.2 参数规范

| 参数 | 值 | 说明 |
|------|------|------|
| 默认值 | 1.0 | 无调整状态 |
| 最小值 | 0.5 | 最大降低 50% |
| 最大值 | 2.0 | 最大提升 100% |
| 精度 | 0.01 | 保留两位小数 |

### 1.3 Alpha 效果说明

| Alpha 值 | 效果 | 典型场景 |
|----------|------|----------|
| 0.5 | 优先级降低 50% | 产能受限、暂缓处理 |
| 0.8 | 优先级降低 20% | 设备维护期间降级 |
| 1.0 | 保持不变（默认） | 无人工干预 |
| 1.2 | 优先级提升 20% | 一般紧急需求 |
| 1.5 | 优先级提升 50% | 重要客户特殊要求 |
| 2.0 | 优先级翻倍 | 极端紧急情况 |

## 2. 完整评分公式

### 2.1 公式链

```
1. S-Score = S1×w1 + S2×w2 + S3×w3
   - S1: 客户等级评分 (customer_level)
   - S2: 毛利评分 (margin)
   - S3: 紧急度评分 (days_to_pdd)

2. P-Score = difficulty_score × spec_factor
   - difficulty_score: 工艺难度分
   - spec_factor: 规格族系数

3. Base Priority = ws × S-Score + wp × P-Score
   - ws, wp: 策略权重（从 strategy_weights 表加载）

4. Final Priority = Base Priority × Alpha
   - Alpha: 人工调整系数（默认 1.0）
```

### 2.2 计算示例

```
假设：
- S-Score = 85
- P-Score = 70
- ws = 0.6, wp = 0.4
- Alpha = 1.3（紧急订单提升）

计算：
Base Priority = 0.6 × 85 + 0.4 × 70 = 51 + 28 = 79
Final Priority = 79 × 1.3 = 102.7
```

## 3. 数据库规范

### 3.1 intervention_log 表结构

```sql
CREATE TABLE intervention_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id TEXT NOT NULL,      -- 合同编号
    alpha_value REAL NOT NULL,      -- Alpha 值（0.5 ~ 2.0）
    reason TEXT NOT NULL,           -- 调整原因（必填）
    user TEXT NOT NULL,             -- 操作人
    timestamp TEXT DEFAULT (datetime('now','localtime')),
    FOREIGN KEY (contract_id) REFERENCES contract_master(contract_id)
);
```

### 3.2 字段约束

| 字段 | 约束 | 说明 |
|------|------|------|
| alpha_value | NOT NULL, CHECK(0.5 <= alpha_value <= 2.0) | 必须在有效范围内 |
| reason | NOT NULL, LENGTH >= 5 | 调整原因必填且不少于 5 字符 |
| user | NOT NULL | 操作人必填 |

### 3.3 查询最新 Alpha

只有最新的 Alpha 记录生效：

```sql
SELECT alpha_value
FROM intervention_log
WHERE contract_id = ?
ORDER BY timestamp DESC
LIMIT 1;
```

## 4. 前端展示规范

### 4.1 Alpha 值展示

- 显示格式：保留 2 位小数（如 `1.30`）
- 当 `alpha !== 1.0` 时，显示 "Alpha" 标识徽章
- 颜色编码：
  - `alpha < 1.0`：降级（橙色/黄色）
  - `alpha = 1.0`：无调整（默认）
  - `alpha > 1.0`：提升（绿色）

### 4.2 优先级展示

展示的优先级值 **始终是应用 Alpha 后的最终优先级**：

```
显示值 = Base Priority × Alpha
```

### 4.3 历史记录展示

每条记录应显示：
- Alpha 值（2 位小数）
- 调整时间（本地时间格式）
- 调整原因
- 操作人

## 5. 后端实现规范

### 5.1 Rust 函数签名

```rust
/// 应用人工调整系数 alpha
/// Final Priority = Base Priority × Alpha
///
/// # 参数
/// - priority: 基础优先级（ws × S + wp × P）
/// - alpha: 人工调整系数，范围 [0.5, 2.0]，默认 1.0
///
/// # 返回
/// - 调整后的最终优先级
pub fn apply_alpha(priority: f64, alpha: f64) -> f64 {
    priority * alpha
}
```

### 5.2 Alpha 生效逻辑

```rust
// 计算基础优先级
let base_priority = calc_priority(s_score, p_score, ws, wp);

// 获取最新 Alpha（如果存在）
let alpha = db::get_latest_alpha(&contract_id)?.unwrap_or(1.0);

// 应用 Alpha
let final_priority = apply_alpha(base_priority, alpha);
```

## 6. 验收标准

### 6.1 核心验证

| 验证项 | 期望结果 |
|--------|----------|
| Alpha = 1.0 时 | Final Priority = Base Priority（无变化） |
| Alpha = 2.0 时 | Final Priority = Base Priority × 2 |
| Alpha = 0.5 时 | Final Priority = Base Priority × 0.5 |

### 6.2 一致性验证

| 验证场景 | 验证方式 |
|----------|----------|
| 前端显示 | 检查 priority 字段已包含 Alpha 调整 |
| 后端计算 | 验证 apply_alpha 函数计算正确 |
| 数据导出 | 确保导出的优先级与界面显示一致 |
| 历史记录 | 可追溯每次 Alpha 变更及原因 |

### 6.3 边界测试

```rust
#[test]
fn test_alpha_no_change() {
    let priority = 75.0;
    let result = apply_alpha(priority, 1.0);
    assert_eq!(result, 75.0); // Alpha=1.0 时结果不变
}

#[test]
fn test_alpha_boundaries() {
    let priority = 100.0;
    assert_eq!(apply_alpha(priority, 0.5), 50.0);  // 最小值
    assert_eq!(apply_alpha(priority, 2.0), 200.0); // 最大值
}
```

## 7. 常见问题

### Q1: 为什么使用乘法而不是加法？

使用 `Priority × Alpha` 而非 `Priority + Alpha` 或 `Priority × (1 + Alpha)`：

- **比例调整**：Alpha 代表调整比例，更符合业务语义
- **默认无变化**：Alpha = 1.0 时结果不变，符合直觉
- **范围对称**：0.5~2.0 对应 -50%~+100% 调整幅度

### Q2: 为什么 Alpha 范围是 0.5~2.0？

- **0.5**：允许最多降低 50%，防止优先级过低
- **2.0**：允许最多提升 100%，避免极端干预
- 此范围确保人工调整在合理范围内，不会完全覆盖算法判断

### Q3: 如何恢复默认？

将 Alpha 设置为 1.0 即可恢复默认优先级：

```typescript
await api.setAlpha(contractId, 1.0, "重置 Alpha 值", userName);
```

## 8. 变更历史

| 日期 | 版本 | 变更内容 |
|------|------|----------|
| 2024-12-19 | 1.0 | 统一 Alpha 口径定义 |
