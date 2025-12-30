# DPM - 合同动态优先级管理系统

基于 Tauri + Rust + React + TypeScript + SQLite 构建的桌面应用。

## 功能特性

- ✅ 合同优先级智能计算（S-Score + P-Score）
- ✅ 多策略动态切换（均衡、客户优先、生产优先）
- ✅ 本地 SQLite 数据库存储
- ✅ 实时优先级排序
- ✅ 跨平台支持（Windows / macOS / Linux）

## 技术栈

- **前端**: React 18 + TypeScript + Vite
- **后端**: Rust + Tauri 1.5
- **数据库**: SQLite + rusqlite
- **评分引擎**: Rust (高性能本地计算)

## 快速开始

### 前置要求

1. Node.js 16+ 和 npm/pnpm
2. Rust 1.70+ 和 Cargo
3. Tauri CLI

### 安装依赖

```bash
cd dpm-app

# 安装前端依赖
npm install

# 或使用 pnpm
pnpm install
```

### 开发模式运行

```bash
# 启动开发服务器（带热重载）
npm run tauri dev
```

### 构建生产版本

```bash
# 构建桌面应用
npm run tauri build

# 输出位置：
# - Windows: src-tauri/target/release/bundle/msi/
# - macOS: src-tauri/target/release/bundle/dmg/
# - Linux: src-tauri/target/release/bundle/appimage/
```

## 项目结构

```
dpm-app/
├─ src/                      # React 前端
│   ├─ App.tsx              # 主页面组件
│   ├─ api/tauri.ts         # Tauri API 调用
│   └─ main.tsx             # 入口文件
├─ src-tauri/               # Rust 后端
│   ├─ src/
│   │   ├─ main.rs         # Tauri 主入口
│   │   ├─ commands.rs     # Tauri 命令（前后端通信）
│   │   ├─ db/             # 数据库层
│   │   │   ├─ init.rs     # 数据库初始化
│   │   │   ├─ schema.rs   # 数据结构定义
│   │   │   └─ repository.rs # 数据访问层
│   │   └─ scoring/        # 评分引擎
│   │       ├─ s_score.rs  # S-Score（战略价值）
│   │       ├─ p_score.rs  # P-Score（生产难度）
│   │       └─ priority.rs # 综合优先级
│   └─ migrations/         # SQL 迁移文件
│       ├─ 001_init.sql    # 表结构
│       └─ 002_seed.sql    # 测试数据
└─ package.json
```

## 评分算法

### S-Score（战略价值评分）

```
S = S1*w1 + S2*w2 + S3*w3
```

- **S1**: 客户等级（A=100, B=70, C=40）
- **S2**: 毛利评分（0-100归一化）
- **S3**: 紧急度评分（基于剩余天数）

### P-Score（生产难度评分）

基于：
- 钢种 + 厚度 + 宽度 → 工艺难度查询
- 规格族系数调整

### 最终优先级

```
Priority = ws * S + wp * P
```

支持人工调整系数 α：
```
Adjusted Priority = Priority * (1 + α)
```

## 数据库

### 核心表

1. **contract_master** - 合同主表
2. **customer_master** - 客户主表
3. **process_difficulty** - 工艺难度配置
4. **rhythm_label** - 节拍标签
5. **strategy_weights** - 策略权重
6. **intervention_log** - 人工干预日志

### 测试数据

首次运行时自动加载：
- 10 条测试合同
- 5 个客户（A/B/C 级）
- 3 种策略（均衡/客户优先/生产优先）
- 工艺难度配置

## 开发指南

### 添加新的 Tauri 命令

1. 在 `src-tauri/src/commands.rs` 中定义命令函数
2. 在 `src-tauri/src/main.rs` 中注册命令
3. 在 `src/api/tauri.ts` 中添加前端调用接口

### 修改评分逻辑

- S-Score: 编辑 `src-tauri/src/scoring/s_score.rs`
- P-Score: 编辑 `src-tauri/src/scoring/p_score.rs`
- 权重计算: 编辑 `src-tauri/src/scoring/priority.rs`

### 数据库迁移

在 `src-tauri/migrations/` 中添加新的 SQL 文件，并在 `init.rs` 中引用。

## 常见问题

### Q: 如何清空数据库重新开始？

A: 删除应用数据目录下的 `dpm.db` 文件，重新启动应用即可。

### Q: 如何修改策略权重？

A: 直接修改数据库中 `strategy_weights` 表，或在 `002_seed.sql` 中修改默认值。

### Q: 如何添加新的钢种？

A: 在 `process_difficulty` 表中添加新的钢种配置行。

## License

MIT

## 参考文档

详细设计文档见：`claude_code_dev_doc.md`
