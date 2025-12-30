# DPM 项目快速启动指南
**更新时间**: 2024-12-09 20:45
**项目状态**: ✅ 开发完成，测试通过，可正常使用

---

## 🚀 快速启动

### 开发模式（推荐）
```bash
cd /Users/butianzheng/Documents/trae_projects/DPM/dpm-app
npm run tauri dev
```

⏱️ **编译时间**: 约 2-3 秒（增量编译）
✅ **状态**: 已验证，应用正常运行

---

## ✅ 项目完成情况

### 1. 开发完成 (100%)
- ✅ **后端评分引擎**: S-Score + P-Score + Priority
- ✅ **前端界面**: 主表格 + 3个对话框
- ✅ **数据管理**: 80条测试合同 + 完整配置
- ✅ **关键 Bug 修复**:
  - Tauri IPC 导入错误
  - Alpha 系数计算逻辑
  - 应用图标生成

### 2. 测试通过 (100%)
- ✅ **基础功能**: 应用启动、数据加载、排序
- ✅ **策略切换**: 均衡/客户优先/生产优先
- ✅ **Alpha 调整**: 计算正确，标识显示
- ✅ **对话框**: 调整/详情/历史全部正常
- ✅ **核心逻辑**: alpha=1.2提升20%, alpha=0.8降低20%

### 3. 性能验证 (通过)
- ⭐ **加载速度**: 流畅
- ⭐ **切换响应**: 即时
- ⭐ **交互体验**: 良好

---

## 📁 关键文件位置

### 应用文件
- **数据库**: `~/Library/Application Support/dpm.db` (macOS)
- **源代码**: `/Users/butianzheng/Documents/trae_projects/DPM/dpm-app`
- **编译产物**: `src-tauri/target/debug/dpm-app`

### 文档
| 文档 | 说明 |
|------|------|
| `README.md` | 项目说明 |
| `PROGRESS.md` | 详细开发进度 (⭐ 推荐阅读) |
| `PROJECT_COMPLETION.md` | 项目完成报告 (⭐ 推荐阅读) |
| `QUICKSTART.md` | 本文档 |
| `FRONTEND_FEATURES.md` | 前端功能说明 |
| `TEST_DATA.md` | 测试数据说明 |
| `CLAUDE.md` | 开发指引 |

---

## 🎯 使用指南

### 基本操作
1. **查看合同列表**: 应用启动后自动显示所有合同
2. **切换策略**: 点击策略选择器（均衡/客户优先/生产优先）
3. **查看详情**: 点击任意合同行
4. **调整优先级**: 点击"调整"按钮，设置 Alpha 值（0.5-2.0）
5. **查看历史**: 点击"历史"按钮，查看该合同的所有调整记录

### Alpha 系数说明
- **alpha = 1.0**: 优先级不变（默认值）
- **alpha > 1.0**: 提升优先级（如 1.2 = 提升20%）
- **alpha < 1.0**: 降低优先级（如 0.8 = 降低20%）
- **范围限制**: 0.5 ~ 2.0

### 策略说明
- **均衡策略** (ws=0.5, wp=0.5): 战略价值和生产难度权重相同
- **客户优先** (ws=0.7, wp=0.3): 侧重客户等级和利润
- **生产优先** (ws=0.3, wp=0.7): 侧重工艺难度和规格复杂度

---

## 🔧 故障排除

### 问题1: 应用无法启动
**解决方案**:
```bash
# 清理并重新编译
cd /Users/butianzheng/Documents/trae_projects/DPM/dpm-app
rm -rf node_modules package-lock.json
npm install
npm run tauri dev
```

### 问题2: 数据库错误
**解决方案**:
```bash
# 删除数据库文件，重启应用自动重新初始化
rm ~/Library/Application\ Support/dpm.db
npm run tauri dev
```

### 问题3: 编译错误
**解决方案**:
```bash
# 清理 Rust 编译缓存
cd src-tauri
cargo clean
cd ..
npm run tauri dev
```

### 问题4: 端口占用
**现象**: Vite 报错端口 1420 被占用
**解决方案**:
```bash
# 查找并终止占用进程
lsof -ti:1420 | xargs kill -9
npm run tauri dev
```

---

## 📦 生产部署

### 构建安装包
```bash
cd /Users/butianzheng/Documents/trae_projects/DPM/dpm-app
npm run tauri build
```

### 输出位置
- **macOS**: `src-tauri/target/release/bundle/dmg/DPM_0.1.0_aarch64.dmg`
- **Windows**: `src-tauri/target/release/bundle/msi/DPM_0.1.0_x64.msi`
- **Linux**: `src-tauri/target/release/bundle/appimage/DPM_0.1.0_amd64.AppImage`

### 安装说明
1. 双击生成的安装包
2. 按照系统提示完成安装
3. 首次启动会自动创建数据库并加载测试数据

---

## 💡 开发提示

### 修改代码后
```bash
# 前端代码修改：Vite 会自动热重载，无需重启
# 后端代码修改：需要重新编译，但增量编译很快（2-3秒）
```

### 查看日志
- **前端日志**: 浏览器开发者工具 Console（如果 Tauri 支持）
- **后端日志**: 终端输出（stdout/stderr）
- **数据库**: 使用 SQLite 工具查看 `~/Library/Application Support/dpm.db`

### 推荐工具
- **SQLite 查看器**: [DB Browser for SQLite](https://sqlitebrowser.org/)
- **Rust 格式化**: `cargo fmt`
- **Rust 检查**: `cargo check`
- **Rust 测试**: `cargo test`

---

## 🚀 下一步建议

### 立即可做
✅ 项目已可正常使用，可以：
1. 继续测试各种业务场景
2. 导入实际业务数据
3. 部署到生产环境

### 功能增强（可选）
参考 `FRONTEND_FEATURES.md` 和 `PROJECT_COMPLETION.md` 的建议：
1. 📊 批量 Alpha 调整
2. 📤 数据导出（Excel/CSV）
3. 🔍 搜索和筛选
4. 📈 图表可视化
5. 👥 用户权限管理

---

## 📌 重要提示

1. ✅ **项目已完成**: 所有核心功能开发和测试完成
2. ✅ **可正常使用**: 应用稳定，性能良好
3. 📁 **数据备份**: 建议定期备份数据库文件
4. 🔄 **增量编译**: 后续修改编译很快（2-3秒）
5. 📖 **详细文档**: 查看 `PROGRESS.md` 了解完整开发过程

---

**项目状态**: ✅ 开发完成，测试通过，可交付使用
**启动命令**: `npm run tauri dev`
**数据库位置**: `~/Library/Application Support/dpm.db`

**祝使用愉快！** 🎉
