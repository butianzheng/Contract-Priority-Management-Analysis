import { useState, useEffect, useMemo } from "react";
import { Link } from "react-router-dom";
import { api, UnifiedHistoryEntry } from "../../api/tauri";
import "./Settings.css";

type SystemTab = "roles" | "environment" | "history";

interface Role {
  id: string;
  name: string;
  description: string;
  permissions: string[];
  userCount: number;
}

interface EnvConfig {
  key: string;
  value: string;
  description: string;
  category: string;
  editable: boolean;
}

export function System() {
  const [activeTab, setActiveTab] = useState<SystemTab>("roles");
  const [history, setHistory] = useState<UnifiedHistoryEntry[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);

  // 模拟角色数据
  const [roles] = useState<Role[]>([
    {
      id: "admin",
      name: "管理员",
      description: "系统管理员，拥有所有权限",
      permissions: ["全部权限"],
      userCount: 2,
    },
    {
      id: "manager",
      name: "生产经理",
      description: "可以管理合同优先级和策略配置",
      permissions: ["查看合同", "调整优先级", "配置策略", "导出数据"],
      userCount: 5,
    },
    {
      id: "operator",
      name: "操作员",
      description: "可以查看和调整合同优先级",
      permissions: ["查看合同", "调整优先级"],
      userCount: 12,
    },
    {
      id: "viewer",
      name: "查看者",
      description: "只读访问权限",
      permissions: ["查看合同", "查看报表"],
      userCount: 8,
    },
  ]);

  // 模拟环境配置
  const [envConfigs] = useState<EnvConfig[]>([
    { key: "APP_NAME", value: "DPM 合同优先级系统", description: "应用名称", category: "基础", editable: false },
    { key: "APP_VERSION", value: "1.0.0", description: "应用版本", category: "基础", editable: false },
    { key: "DB_PATH", value: "./data/dpm.db", description: "数据库路径", category: "数据库", editable: true },
    { key: "BACKUP_PATH", value: "./backups", description: "备份文件路径", category: "数据库", editable: true },
    { key: "EXPORT_PATH", value: "./exports", description: "导出文件路径", category: "文件", editable: true },
    { key: "IMPORT_PATH", value: "./imports", description: "导入文件路径", category: "文件", editable: true },
    { key: "LOG_LEVEL", value: "INFO", description: "日志级别", category: "日志", editable: true },
    { key: "LOG_PATH", value: "./logs", description: "日志文件路径", category: "日志", editable: true },
    { key: "AUTO_BACKUP", value: "true", description: "自动备份开关", category: "数据库", editable: true },
    { key: "BACKUP_INTERVAL", value: "24", description: "备份间隔（小时）", category: "数据库", editable: true },
  ]);

  // 加载变更历史
  useEffect(() => {
    if (activeTab === "history") {
      loadHistory();
    }
  }, [activeTab]);

  const loadHistory = async () => {
    setHistoryLoading(true);
    try {
      const data = await api.getUnifiedHistory(undefined, undefined, 50);
      setHistory(data);
    } catch (err) {
      console.error("加载历史失败:", err);
    } finally {
      setHistoryLoading(false);
    }
  };

  // 按分类分组环境配置
  const envByCategory = useMemo(() => {
    const grouped: Record<string, EnvConfig[]> = {};
    envConfigs.forEach((config) => {
      if (!grouped[config.category]) {
        grouped[config.category] = [];
      }
      grouped[config.category].push(config);
    });
    return grouped;
  }, [envConfigs]);

  const formatTime = (timestamp: string) => {
    const date = new Date(timestamp);
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getEntryTypeLabel = (type: string) => {
    switch (type) {
      case "config_change":
        return { label: "配置变更", color: "primary" };
      case "alpha_adjust":
        return { label: "Alpha 调整", color: "warning" };
      case "batch_operation":
        return { label: "批量操作", color: "success" };
      default:
        return { label: type, color: "default" };
    }
  };

  const renderRolesTab = () => (
    <div className="data-cards">
      {roles.map((role) => (
        <div key={role.id} className="data-card">
          <div className="data-card-header">
            <span className="data-card-title">{role.name}</span>
            <span className="data-card-badge active">{role.userCount} 人</span>
          </div>
          <div className="data-card-body">
            <p style={{ margin: "0 0 var(--spacing-sm) 0" }}>{role.description}</p>
            <div style={{ display: "flex", gap: "4px", flexWrap: "wrap" }}>
              {role.permissions.map((perm) => (
                <span key={perm} className="tag default">{perm}</span>
              ))}
            </div>
          </div>
        </div>
      ))}
    </div>
  );

  const renderEnvironmentTab = () => (
    <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-lg)" }}>
      {Object.entries(envByCategory).map(([category, configs]) => (
        <div key={category} className="settings-section">
          <div className="settings-section-header">
            <h3>{category}配置</h3>
          </div>
          <div className="settings-section-body">
            <div className="data-table-container">
              <table className="data-table">
                <thead>
                  <tr>
                    <th>配置项</th>
                    <th>值</th>
                    <th>描述</th>
                    <th>操作</th>
                  </tr>
                </thead>
                <tbody>
                  {configs.map((config) => (
                    <tr key={config.key}>
                      <td>
                        <code style={{ background: "var(--color-bg-layout)", padding: "2px 6px", borderRadius: "4px" }}>
                          {config.key}
                        </code>
                      </td>
                      <td>
                        <input
                          type="text"
                          className="form-input"
                          value={config.value}
                          disabled={!config.editable}
                          style={{ width: "200px" }}
                          readOnly
                        />
                      </td>
                      <td style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
                        {config.description}
                      </td>
                      <td>
                        {config.editable ? (
                          <button className="settings-btn" style={{ fontSize: "12px" }}>编辑</button>
                        ) : (
                          <span style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
                            系统配置
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </div>
      ))}
    </div>
  );

  const renderHistoryTab = () => (
    <div className="settings-section">
      <div className="settings-section-header">
        <h3>系统变更记录</h3>
        <button
          className="settings-btn"
          onClick={loadHistory}
          disabled={historyLoading}
        >
          {historyLoading ? "加载中..." : "刷新"}
        </button>
      </div>
      <div className="settings-section-body">
        {historyLoading ? (
          <div className="settings-empty">
            <div className="settings-empty-text">加载中...</div>
          </div>
        ) : history.length === 0 ? (
          <div className="settings-empty">
            <div className="settings-empty-icon">📜</div>
            <div className="settings-empty-text">暂无变更记录</div>
          </div>
        ) : (
          <div className="data-table-container" style={{ maxHeight: "500px", overflow: "auto" }}>
            <table className="data-table">
              <thead>
                <tr>
                  <th>时间</th>
                  <th>类型</th>
                  <th>操作描述</th>
                  <th>操作人</th>
                  <th>原因</th>
                </tr>
              </thead>
              <tbody>
                {history.map((entry) => {
                  const typeInfo = getEntryTypeLabel(entry.entry_type);
                  return (
                    <tr key={entry.id}>
                      <td style={{ whiteSpace: "nowrap" }}>{formatTime(entry.timestamp)}</td>
                      <td>
                        <span className={`tag ${typeInfo.color}`}>{typeInfo.label}</span>
                      </td>
                      <td>{entry.description}</td>
                      <td>{entry.user}</td>
                      <td style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
                        {entry.reason || "-"}
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );

  return (
    <div className="system-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <Link to="/settings">设置</Link>
          <span>/</span>
          <span>系统参数</span>
        </div>
        <h1 className="page-header__title">系统参数</h1>
        <p className="page-header__subtitle">系统级配置与权限管理</p>
      </div>

      <div className="settings-content">
        {/* 标签切换 */}
        <div className="stats-row">
          <div
            className="stat-item"
            style={{ cursor: "pointer" }}
            onClick={() => setActiveTab("roles")}
          >
            <div className="stat-item-label">👤 角色权限</div>
            <div className={`stat-item-value ${activeTab === "roles" ? "primary" : ""}`}>
              {roles.length}
            </div>
          </div>
          <div
            className="stat-item"
            style={{ cursor: "pointer" }}
            onClick={() => setActiveTab("environment")}
          >
            <div className="stat-item-label">🌐 环境配置</div>
            <div className={`stat-item-value ${activeTab === "environment" ? "primary" : ""}`}>
              {envConfigs.length}
            </div>
          </div>
          <div
            className="stat-item"
            style={{ cursor: "pointer" }}
            onClick={() => setActiveTab("history")}
          >
            <div className="stat-item-label">📜 变更历史</div>
            <div className={`stat-item-value ${activeTab === "history" ? "primary" : ""}`}>
              {history.length}
            </div>
          </div>
        </div>

        {/* 内容区域 */}
        {activeTab === "roles" && renderRolesTab()}
        {activeTab === "environment" && renderEnvironmentTab()}
        {activeTab === "history" && renderHistoryTab()}

        {/* 系统信息 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>系统信息</h3>
          </div>
          <div className="settings-section-body">
            <div className="form-row">
              <div className="form-group">
                <label className="form-label">应用名称</label>
                <input type="text" className="form-input" value="DPM 合同优先级系统" readOnly />
              </div>
              <div className="form-group">
                <label className="form-label">版本号</label>
                <input type="text" className="form-input" value="1.0.0" readOnly />
              </div>
              <div className="form-group">
                <label className="form-label">运行环境</label>
                <input type="text" className="form-input" value="Tauri Desktop" readOnly />
              </div>
              <div className="form-group">
                <label className="form-label">数据库版本</label>
                <input type="text" className="form-input" value="SQLite 3.x" readOnly />
              </div>
            </div>
          </div>
        </div>

        {/* 数据维护 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>数据维护</h3>
          </div>
          <div className="settings-section-body">
            <div className="import-export-section">
              <div className="import-area">
                <div className="import-area-icon">💾</div>
                <div className="import-area-text">
                  备份数据库<br />
                  <span style={{ fontSize: "11px", color: "var(--color-text-tertiary)" }}>
                    创建当前数据库的完整备份
                  </span>
                </div>
              </div>
              <div className="export-area">
                <div className="export-area-icon">🔄</div>
                <div className="export-area-text">
                  恢复数据库<br />
                  <span style={{ fontSize: "11px", color: "var(--color-text-tertiary)" }}>
                    从备份文件恢复数据
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <style>{`
        .system-page {
          max-width: 1200px;
        }
      `}</style>
    </div>
  );
}
