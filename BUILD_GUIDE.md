# GitHub Actions 自动构建指南

## 📦 自动打包说明

DPM 应用已配置 GitHub Actions 自动构建，支持 **Windows、macOS、Linux** 三个平台。

## 🚀 使用方法

### 方式一：推送代码自动构建（开发版本）

每次推送到 `main` 或 `master` 分支时，自动构建所有平台：

```bash
git add .
git commit -m "更新功能"
git push origin main
```

**构建产物位置：**
1. 打开 GitHub 仓库页面
2. 点击 **Actions** 标签页
3. 选择最新的工作流运行记录
4. 在页面底部 **Artifacts** 区域下载：
   - `windows-msi` - Windows 安装包（.msi）
   - `macos-dmg` - macOS 磁盘映像（.dmg）
   - `linux-appimage` - Linux AppImage（.AppImage）
   - `linux-deb` - Linux Debian 包（.deb）

> ⚠️ **注意**：Artifacts 会在 90 天后自动删除

---

### 方式二：创建 Tag 发布正式版本（推荐）

当需要发布正式版本时，创建版本 tag：

```bash
# 创建版本标签（例如 v0.1.0）
git tag v0.1.0

# 推送标签到远程仓库
git push origin v0.1.0
```

**自动发布流程：**
1. GitHub Actions 自动构建所有平台
2. 自动创建 GitHub Release
3. 安装包自动上传到 Release 页面
4. 生成自动化的 Release Notes

**下载发布版本：**
1. 打开 GitHub 仓库页面
2. 点击右侧 **Releases** 区域
3. 选择对应版本（如 `v0.1.0`）
4. 在 **Assets** 区域下载对应平台的安装包

---

### 方式三：手动触发构建

如果需要手动触发构建：

1. 打开 GitHub 仓库页面
2. 点击 **Actions** 标签页
3. 选择 **Build and Release DPM App** 工作流
4. 点击右上角 **Run workflow** 按钮
5. 选择分支并点击 **Run workflow**

---

## 📋 构建产物说明

| 平台 | 文件格式 | 文件名示例 | 用途 |
|------|---------|-----------|------|
| **Windows** | `.msi` | `DPM_0.1.0_x64_en-US.msi` | Windows 安装包，双击安装 |
| **macOS** | `.dmg` | `DPM_0.1.0_x64.dmg` | macOS 磁盘映像，拖拽到 Applications |
| **Linux** | `.AppImage` | `dpm-app_0.1.0_amd64.AppImage` | 便携版，添加执行权限后直接运行 |
| **Linux** | `.deb` | `dpm-app_0.1.0_amd64.deb` | Debian/Ubuntu 包，`sudo dpkg -i` 安装 |

---

## 🔧 本地构建（备选方案）

如果不想使用 GitHub Actions，也可以在本地构建：

### Windows 本地构建
```bash
# 安装依赖
npm install

# 构建应用
npm run tauri build

# 构建产物位置
src-tauri\target\release\bundle\msi\DPM_0.1.0_x64_en-US.msi
```

### macOS 本地构建
```bash
# 安装依赖
npm install

# 构建应用
npm run tauri build

# 构建产物位置
src-tauri/target/release/bundle/dmg/DPM_0.1.0_x64.dmg
```

### Linux 本地构建
```bash
# 安装系统依赖
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev \
  libappindicator3-dev librsvg2-dev patchelf

# 安装依赖
npm install

# 构建应用
npm run tauri build

# 构建产物位置
src-tauri/target/release/bundle/appimage/dpm-app_0.1.0_amd64.AppImage
src-tauri/target/release/bundle/deb/dpm-app_0.1.0_amd64.deb
```

---

## ⚙️ 构建配置说明

### 触发条件
- ✅ 推送到 `main` 或 `master` 分支
- ✅ 提交 Pull Request
- ✅ 创建 `v*` 格式的 tag（如 `v0.1.0`、`v1.0.0`）
- ✅ 手动触发（workflow_dispatch）

### 构建时间
首次构建时间较长（约 10-20 分钟），后续构建会利用缓存加速（约 5-10 分钟）。

### 构建状态
可以在仓库的 **README** 中添加构建状态徽章：

```markdown
![Build Status](https://github.com/你的用户名/dpm-app/workflows/Build%20and%20Release%20DPM%20App/badge.svg)
```

---

## 📝 版本发布流程

**推荐的版本发布工作流：**

1. **开发阶段**：直接 push 到 main，使用 Artifacts 测试
2. **测试完成**：创建 tag 发布正式版本
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```
3. **自动发布**：GitHub Actions 自动创建 Release 并上传安装包
4. **分发应用**：将 Release 页面链接发给用户下载

---

## 🛠️ 故障排查

### 构建失败怎么办？
1. 打开 **Actions** 标签页，查看失败的工作流
2. 点击失败的 job，查看详细日志
3. 常见问题：
   - **依赖安装失败**：检查 `package.json` 和 `Cargo.toml`
   - **Rust 编译错误**：检查 Rust 代码语法
   - **前端构建失败**：检查 TypeScript/React 代码

### Artifacts 找不到？
- Artifacts 仅保留 90 天，过期后会自动删除
- 建议使用 **Release** 方式发布正式版本（永久保存）

### Release 没有自动创建？
- 确保推送的是 `v*` 格式的 tag（如 `v0.1.0`）
- 检查仓库的 **Settings > Actions > General > Workflow permissions** 是否允许写入

---

## ✅ 下一步操作

1. **将代码推送到 GitHub**
   ```bash
   git init
   git add .
   git commit -m "Initial commit with GitHub Actions"
   git branch -M main
   git remote add origin https://github.com/你的用户名/dpm-app.git
   git push -u origin main
   ```

2. **观察自动构建**
   - 打开 GitHub Actions 页面，等待构建完成（约 10-20 分钟）

3. **下载测试**
   - 从 Artifacts 下载 Windows 安装包
   - 在 Windows 电脑上测试安装和运行

4. **发布正式版本**（可选）
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

---

## 🎉 完成！

现在你的 DPM 应用已经配置好自动化构建，每次提交代码都会自动生成多平台安装包！
