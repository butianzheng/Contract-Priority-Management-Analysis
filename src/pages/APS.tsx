import { useState, useEffect, useMemo } from "react";
import { api, ContractPriority } from "../api/tauri";
import "./APS.css";

interface APILog {
  id: string;
  method: "GET" | "POST" | "PUT";
  endpoint: string;
  status: "success" | "error" | "pending";
  statusCode?: number;
  duration?: number;
  timestamp: string;
  message?: string;
}

interface ComparisonItem {
  contractId: string;
  customer: string;
  dpmRank: number;
  dpmPriority: number;
  apsRank: number;
  apsPriority: number;
  match: "match" | "mismatch" | "partial";
}

type LogFilter = "all" | "success" | "error";

export function APS() {
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [logFilter, setLogFilter] = useState<LogFilter>("all");

  // 模拟 API 日志
  const [logs] = useState<APILog[]>([
    {
      id: "log-1",
      method: "POST",
      endpoint: "/api/aps/sync-priorities",
      status: "success",
      statusCode: 200,
      duration: 245,
      timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
      message: "优先级同步成功",
    },
    {
      id: "log-2",
      method: "GET",
      endpoint: "/api/aps/schedule-status",
      status: "success",
      statusCode: 200,
      duration: 89,
      timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
      message: "获取排产状态",
    },
    {
      id: "log-3",
      method: "POST",
      endpoint: "/api/aps/push-contracts",
      status: "error",
      statusCode: 500,
      duration: 1023,
      timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      message: "连接超时",
    },
    {
      id: "log-4",
      method: "GET",
      endpoint: "/api/aps/comparison",
      status: "success",
      statusCode: 200,
      duration: 156,
      timestamp: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
      message: "获取对比数据",
    },
    {
      id: "log-5",
      method: "PUT",
      endpoint: "/api/aps/update-mapping",
      status: "success",
      statusCode: 200,
      duration: 312,
      timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
      message: "更新映射关系",
    },
  ]);

  // 加载数据
  useEffect(() => {
    api.getStrategies().then((strats) => {
      setStrategies(strats);
      if (strats.length > 0) {
        setSelectedStrategy(strats[0]);
      }
    });
  }, []);

  useEffect(() => {
    if (selectedStrategy) {
      loadData();
    }
  }, [selectedStrategy]);

  const loadData = async () => {
    setLoading(true);
    try {
      const data = await api.computeAllPriorities(selectedStrategy);
      setContracts(data);
    } catch (err) {
      console.error("加载数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 接口状态
  const interfaceStatus = useMemo(() => {
    const successLogs = logs.filter((l) => l.status === "success");
    const errorLogs = logs.filter((l) => l.status === "error");
    const lastSync = logs.find((l) => l.endpoint.includes("sync"));
    const avgDuration =
      successLogs.length > 0
        ? successLogs.reduce((sum, l) => sum + (l.duration || 0), 0) / successLogs.length
        : 0;

    return {
      status: errorLogs.length > 0 ? "warning" : "online",
      successRate: logs.length > 0 ? (successLogs.length / logs.length) * 100 : 100,
      lastSync: lastSync?.timestamp,
      avgDuration,
    };
  }, [logs]);

  // 过滤后的日志
  const filteredLogs = useMemo(() => {
    if (logFilter === "all") return logs;
    return logs.filter((l) => l.status === logFilter);
  }, [logs, logFilter]);

  // 生成排产对比数据（模拟 APS 数据）
  const comparisonData: ComparisonItem[] = useMemo(() => {
    return contracts.slice(0, 20).map((c, index) => {
      // 模拟 APS 排名（添加一些随机偏差）
      const deviation = Math.floor(Math.random() * 5) - 2;
      const apsRank = Math.max(1, index + 1 + deviation);
      const apsPriority = c.priority + (Math.random() - 0.5) * 10;

      const rankDiff = Math.abs(index + 1 - apsRank);
      const match: "match" | "mismatch" | "partial" =
        rankDiff === 0 ? "match" : rankDiff <= 2 ? "partial" : "mismatch";

      return {
        contractId: c.contract_id,
        customer: c.customer_id,
        dpmRank: index + 1,
        dpmPriority: c.priority,
        apsRank,
        apsPriority,
        match,
      };
    });
  }, [contracts]);

  // 对比统计
  const comparisonStats = useMemo(() => {
    const total = comparisonData.length;
    const matched = comparisonData.filter((c) => c.match === "match").length;
    const partial = comparisonData.filter((c) => c.match === "partial").length;
    const mismatched = comparisonData.filter((c) => c.match === "mismatch").length;

    return { total, matched, partial, mismatched };
  }, [comparisonData]);

  const formatTime = (isoString: string) => {
    const date = new Date(isoString);
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  return (
    <div className="aps-page">
      <div className="page-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h1 className="page-header__title">APS/调度前置评分视图</h1>
          <p className="page-header__subtitle">APS 接口监控与排产对比</p>
        </div>
        <div style={{ display: "flex", gap: "var(--spacing-sm)", alignItems: "center" }}>
          <select
            value={selectedStrategy}
            onChange={(e) => setSelectedStrategy(e.target.value)}
            disabled={loading}
            style={{
              padding: "var(--spacing-xs) var(--spacing-sm)",
              borderRadius: "var(--border-radius-sm)",
              border: "1px solid var(--color-border)",
            }}
          >
            {strategies.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
          <button
            onClick={loadData}
            disabled={loading}
            className="btn-refresh"
          >
            {loading ? "刷新中..." : "刷新数据"}
          </button>
        </div>
      </div>

      {/* 接口状态卡片 */}
      <div className="aps-status-cards">
        <div className="status-card">
          <div className={`status-indicator ${interfaceStatus.status}`}>
            {interfaceStatus.status === "online" ? "✓" : "⚠"}
          </div>
          <div className="status-info">
            <div className="status-label">接口状态</div>
            <div className={`status-value ${interfaceStatus.status}`}>
              {interfaceStatus.status === "online" ? "正常" : "异常"}
            </div>
          </div>
        </div>

        <div className="status-card">
          <div className="status-indicator online">
            📊
          </div>
          <div className="status-info">
            <div className="status-label">调用成功率</div>
            <div className={`status-value ${interfaceStatus.successRate >= 90 ? "online" : "offline"}`}>
              {interfaceStatus.successRate.toFixed(1)}%
            </div>
          </div>
        </div>

        <div className="status-card">
          <div className="status-indicator pending">
            ⏱️
          </div>
          <div className="status-info">
            <div className="status-label">平均响应时间</div>
            <div className="status-value">
              {interfaceStatus.avgDuration.toFixed(0)} ms
            </div>
          </div>
        </div>

        <div className="status-card">
          <div className="status-indicator online">
            🔄
          </div>
          <div className="status-info">
            <div className="status-label">最后同步</div>
            <div className="status-value" style={{ fontSize: "var(--font-size-base)" }}>
              {interfaceStatus.lastSync ? formatTime(interfaceStatus.lastSync) : "-"}
            </div>
          </div>
        </div>
      </div>

      {/* 主内容区域 */}
      <div className="aps-main">
        {/* 调用日志面板 */}
        <div className="logs-panel">
          <div className="logs-header">
            <h3>接口调用日志</h3>
            <div className="logs-filters">
              <button
                className={`logs-filter-btn ${logFilter === "all" ? "active" : ""}`}
                onClick={() => setLogFilter("all")}
              >
                全部
              </button>
              <button
                className={`logs-filter-btn ${logFilter === "success" ? "active" : ""}`}
                onClick={() => setLogFilter("success")}
              >
                成功
              </button>
              <button
                className={`logs-filter-btn ${logFilter === "error" ? "active" : ""}`}
                onClick={() => setLogFilter("error")}
              >
                失败
              </button>
            </div>
          </div>
          <div className="logs-list">
            {filteredLogs.length === 0 ? (
              <div className="aps-empty">
                <div className="aps-empty-icon">📝</div>
                <div className="aps-empty-text">暂无日志记录</div>
              </div>
            ) : (
              filteredLogs.map((log) => (
                <div key={log.id} className={`log-item ${log.status}`}>
                  <div className="log-item-header">
                    <span className={`log-method ${log.method}`}>{log.method}</span>
                    <span className="log-time">{formatTime(log.timestamp)}</span>
                  </div>
                  <div className="log-endpoint">{log.endpoint}</div>
                  <div className="log-meta">
                    <span className={`log-status ${log.status}`}>
                      {log.statusCode} {log.status === "success" ? "OK" : "Error"}
                    </span>
                    <span>{log.duration}ms</span>
                    {log.message && <span>{log.message}</span>}
                  </div>
                </div>
              ))
            )}
          </div>
        </div>

        {/* 排产对比面板 */}
        <div className="comparison-panel">
          <div className="comparison-header">
            <h3>DPM vs APS 排产对比</h3>
            <div className="comparison-actions">
              <button className="btn-refresh" disabled={loading}>
                同步 APS
              </button>
            </div>
          </div>
          <div className="comparison-body">
            {comparisonData.length === 0 ? (
              <div className="aps-empty">
                <div className="aps-empty-icon">📈</div>
                <div className="aps-empty-text">暂无对比数据</div>
                <div className="aps-empty-hint">加载合同数据后可查看排产对比</div>
              </div>
            ) : (
              <table className="comparison-table">
                <thead>
                  <tr>
                    <th>合同编号</th>
                    <th>客户</th>
                    <th>DPM排名</th>
                    <th>APS排名</th>
                    <th>DPM优先级</th>
                    <th>APS优先级</th>
                    <th>匹配状态</th>
                  </tr>
                </thead>
                <tbody>
                  {comparisonData.map((item) => (
                    <tr key={item.contractId}>
                      <td>{item.contractId}</td>
                      <td>{item.customer}</td>
                      <td>
                        <span className="priority-badge dpm">#{item.dpmRank}</span>
                      </td>
                      <td>
                        <span className="priority-badge aps">#{item.apsRank}</span>
                      </td>
                      <td>{item.dpmPriority.toFixed(2)}</td>
                      <td>{item.apsPriority.toFixed(2)}</td>
                      <td>
                        <div className="diff-cell">
                          <span className={`diff-indicator ${item.match}`}></span>
                          <span className={`match-badge ${item.match}`}>
                            {item.match === "match"
                              ? "一致"
                              : item.match === "partial"
                              ? "接近"
                              : "偏差"}
                          </span>
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
          <div className="aps-summary">
            <div className="summary-item">
              <div className="summary-label">完全匹配</div>
              <div className="summary-value success">{comparisonStats.matched}</div>
            </div>
            <div className="summary-item">
              <div className="summary-label">接近匹配</div>
              <div className="summary-value warning">{comparisonStats.partial}</div>
            </div>
            <div className="summary-item">
              <div className="summary-label">存在偏差</div>
              <div className="summary-value error">{comparisonStats.mismatched}</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
