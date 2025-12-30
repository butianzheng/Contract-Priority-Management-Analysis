import { useState, useMemo } from "react";

interface APILog {
  id: string;
  method: "GET" | "POST" | "PUT" | "DELETE";
  endpoint: string;
  status: "success" | "error" | "pending";
  statusCode?: number;
  duration?: number;
  timestamp: string;
  message?: string;
  requestBody?: string;
  responseBody?: string;
}

type LogFilter = "all" | "success" | "error";
type TimeRange = "1h" | "6h" | "24h" | "7d";

export function APSLogs() {
  const [logFilter, setLogFilter] = useState<LogFilter>("all");
  const [timeRange, setTimeRange] = useState<TimeRange>("24h");
  const [searchKeyword, setSearchKeyword] = useState("");
  const [selectedLog, setSelectedLog] = useState<APILog | null>(null);

  // Mock API logs
  const [logs] = useState<APILog[]>([
    {
      id: "log-1",
      method: "POST",
      endpoint: "/api/aps/sync-priorities",
      status: "success",
      statusCode: 200,
      duration: 245,
      timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(),
      message: "优先级同步成功，共同步 80 条记录",
      requestBody: '{"strategy": "均衡", "contracts": [...]}',
      responseBody: '{"success": true, "synced": 80}',
    },
    {
      id: "log-2",
      method: "GET",
      endpoint: "/api/aps/schedule-status",
      status: "success",
      statusCode: 200,
      duration: 89,
      timestamp: new Date(Date.now() - 15 * 60 * 1000).toISOString(),
      message: "获取排产状态成功",
      responseBody: '{"status": "running", "progress": 85}',
    },
    {
      id: "log-3",
      method: "POST",
      endpoint: "/api/aps/push-contracts",
      status: "error",
      statusCode: 500,
      duration: 1023,
      timestamp: new Date(Date.now() - 30 * 60 * 1000).toISOString(),
      message: "连接超时：无法连接到 APS 服务器",
      requestBody: '{"contracts": [...]}',
      responseBody: '{"error": "Connection timeout"}',
    },
    {
      id: "log-4",
      method: "GET",
      endpoint: "/api/aps/comparison",
      status: "success",
      statusCode: 200,
      duration: 156,
      timestamp: new Date(Date.now() - 45 * 60 * 1000).toISOString(),
      message: "获取对比数据成功",
    },
    {
      id: "log-5",
      method: "PUT",
      endpoint: "/api/aps/update-mapping",
      status: "success",
      statusCode: 200,
      duration: 312,
      timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(),
      message: "更新字段映射关系成功",
    },
    {
      id: "log-6",
      method: "POST",
      endpoint: "/api/aps/sync-priorities",
      status: "success",
      statusCode: 200,
      duration: 198,
      timestamp: new Date(Date.now() - 2 * 60 * 60 * 1000).toISOString(),
      message: "优先级同步成功",
    },
    {
      id: "log-7",
      method: "GET",
      endpoint: "/api/aps/health",
      status: "success",
      statusCode: 200,
      duration: 23,
      timestamp: new Date(Date.now() - 3 * 60 * 60 * 1000).toISOString(),
      message: "健康检查通过",
    },
    {
      id: "log-8",
      method: "POST",
      endpoint: "/api/aps/push-contracts",
      status: "error",
      statusCode: 503,
      duration: 5000,
      timestamp: new Date(Date.now() - 5 * 60 * 60 * 1000).toISOString(),
      message: "APS 服务不可用",
    },
  ]);

  // Filter logs
  const filteredLogs = useMemo(() => {
    let result = logs;

    // Filter by status
    if (logFilter !== "all") {
      result = result.filter((l) => l.status === logFilter);
    }

    // Filter by keyword
    if (searchKeyword) {
      const keyword = searchKeyword.toLowerCase();
      result = result.filter(
        (l) =>
          l.endpoint.toLowerCase().includes(keyword) ||
          l.message?.toLowerCase().includes(keyword)
      );
    }

    // Filter by time range
    const now = Date.now();
    const ranges: Record<TimeRange, number> = {
      "1h": 60 * 60 * 1000,
      "6h": 6 * 60 * 60 * 1000,
      "24h": 24 * 60 * 60 * 1000,
      "7d": 7 * 24 * 60 * 60 * 1000,
    };
    result = result.filter(
      (l) => now - new Date(l.timestamp).getTime() <= ranges[timeRange]
    );

    return result;
  }, [logs, logFilter, searchKeyword, timeRange]);

  // Statistics
  const stats = useMemo(() => {
    const total = filteredLogs.length;
    const success = filteredLogs.filter((l) => l.status === "success").length;
    const error = filteredLogs.filter((l) => l.status === "error").length;
    const avgDuration =
      filteredLogs.length > 0
        ? filteredLogs.reduce((sum, l) => sum + (l.duration || 0), 0) / filteredLogs.length
        : 0;

    return { total, success, error, avgDuration };
  }, [filteredLogs]);

  const formatTime = (isoString: string) => {
    return new Date(isoString).toLocaleString("zh-CN");
  };

  const getMethodColor = (method: string) => {
    switch (method) {
      case "GET": return "#52c41a";
      case "POST": return "#1890ff";
      case "PUT": return "#faad14";
      case "DELETE": return "#ff4d4f";
      default: return "#666";
    }
  };

  return (
    <div className="aps-logs-page">
      <div className="page-header">
        <h1 className="page-header__title">APS 调用日志</h1>
        <p className="page-header__subtitle">查看接口调用历史和错误详情</p>
      </div>

      {/* Statistics */}
      <div className="logs-stats">
        <div className="stat-card">
          <div className="stat-value">{stats.total}</div>
          <div className="stat-label">总调用次数</div>
        </div>
        <div className="stat-card success">
          <div className="stat-value">{stats.success}</div>
          <div className="stat-label">成功</div>
        </div>
        <div className="stat-card error">
          <div className="stat-value">{stats.error}</div>
          <div className="stat-label">失败</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">{stats.avgDuration.toFixed(0)}ms</div>
          <div className="stat-label">平均响应时间</div>
        </div>
        <div className="stat-card">
          <div className="stat-value">
            {stats.total > 0 ? ((stats.success / stats.total) * 100).toFixed(1) : 0}%
          </div>
          <div className="stat-label">成功率</div>
        </div>
      </div>

      {/* Filters */}
      <div className="logs-filters">
        <div className="filter-group">
          <input
            type="text"
            placeholder="搜索接口或消息..."
            value={searchKeyword}
            onChange={(e) => setSearchKeyword(e.target.value)}
            className="search-input"
          />
        </div>
        <div className="filter-group">
          <select value={timeRange} onChange={(e) => setTimeRange(e.target.value as TimeRange)}>
            <option value="1h">最近1小时</option>
            <option value="6h">最近6小时</option>
            <option value="24h">最近24小时</option>
            <option value="7d">最近7天</option>
          </select>
        </div>
        <div className="filter-buttons">
          <button
            className={`filter-btn ${logFilter === "all" ? "active" : ""}`}
            onClick={() => setLogFilter("all")}
          >
            全部
          </button>
          <button
            className={`filter-btn success ${logFilter === "success" ? "active" : ""}`}
            onClick={() => setLogFilter("success")}
          >
            成功
          </button>
          <button
            className={`filter-btn error ${logFilter === "error" ? "active" : ""}`}
            onClick={() => setLogFilter("error")}
          >
            失败
          </button>
        </div>
      </div>

      {/* Logs Table */}
      <div className="logs-content">
        <div className="logs-table-container">
          {filteredLogs.length === 0 ? (
            <div className="empty-state">
              <div className="empty-state-icon">📝</div>
              <div className="empty-state-text">暂无日志记录</div>
            </div>
          ) : (
            <table className="logs-table">
              <thead>
                <tr>
                  <th>时间</th>
                  <th>方法</th>
                  <th>接口</th>
                  <th>状态</th>
                  <th>耗时</th>
                  <th>消息</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {filteredLogs.map((log) => (
                  <tr key={log.id} className={log.status}>
                    <td className="time-cell">{formatTime(log.timestamp)}</td>
                    <td>
                      <span className="method-badge" style={{ background: getMethodColor(log.method) }}>
                        {log.method}
                      </span>
                    </td>
                    <td className="endpoint-cell">{log.endpoint}</td>
                    <td>
                      <span className={`status-badge ${log.status}`}>
                        {log.statusCode} {log.status === "success" ? "OK" : "Error"}
                      </span>
                    </td>
                    <td>{log.duration}ms</td>
                    <td className="message-cell">{log.message}</td>
                    <td>
                      <button className="btn-detail" onClick={() => setSelectedLog(log)}>
                        详情
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </div>

      {/* Log Detail Modal */}
      {selectedLog && (
        <div className="modal-overlay" onClick={() => setSelectedLog(null)}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>日志详情</h2>
              <button className="close-btn" onClick={() => setSelectedLog(null)}>×</button>
            </div>
            <div className="modal-body">
              <div className="detail-row">
                <span className="detail-label">时间</span>
                <span className="detail-value">{formatTime(selectedLog.timestamp)}</span>
              </div>
              <div className="detail-row">
                <span className="detail-label">方法</span>
                <span className="method-badge" style={{ background: getMethodColor(selectedLog.method) }}>
                  {selectedLog.method}
                </span>
              </div>
              <div className="detail-row">
                <span className="detail-label">接口</span>
                <span className="detail-value mono">{selectedLog.endpoint}</span>
              </div>
              <div className="detail-row">
                <span className="detail-label">状态</span>
                <span className={`status-badge ${selectedLog.status}`}>
                  {selectedLog.statusCode} {selectedLog.status === "success" ? "OK" : "Error"}
                </span>
              </div>
              <div className="detail-row">
                <span className="detail-label">耗时</span>
                <span className="detail-value">{selectedLog.duration}ms</span>
              </div>
              <div className="detail-row">
                <span className="detail-label">消息</span>
                <span className="detail-value">{selectedLog.message}</span>
              </div>
              {selectedLog.requestBody && (
                <div className="detail-section">
                  <div className="detail-label">请求体</div>
                  <pre className="code-block">{selectedLog.requestBody}</pre>
                </div>
              )}
              {selectedLog.responseBody && (
                <div className="detail-section">
                  <div className="detail-label">响应体</div>
                  <pre className="code-block">{selectedLog.responseBody}</pre>
                </div>
              )}
            </div>
            <div className="modal-footer">
              <button className="btn-secondary" onClick={() => setSelectedLog(null)}>关闭</button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .aps-logs-page {
          display: flex;
          flex-direction: column;
          height: 100%;
          gap: var(--spacing-md);
        }

        .logs-stats {
          display: grid;
          grid-template-columns: repeat(5, 1fr);
          gap: var(--spacing-md);
        }

        .stat-card {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-md);
          padding: var(--spacing-md);
          text-align: center;
        }

        .stat-card.success .stat-value { color: var(--color-success); }
        .stat-card.error .stat-value { color: var(--color-error); }

        .stat-value {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .stat-label {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .logs-filters {
          display: flex;
          gap: var(--spacing-md);
          align-items: center;
          padding: var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
        }

        .search-input {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          width: 250px;
        }

        .filter-group select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        .filter-buttons {
          display: flex;
          gap: 4px;
          margin-left: auto;
        }

        .filter-btn {
          padding: var(--spacing-xs) var(--spacing-md);
          border: 1px solid var(--color-border);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .filter-btn.active {
          background: var(--color-primary);
          border-color: var(--color-primary);
          color: #fff;
        }

        .filter-btn.success.active {
          background: var(--color-success);
          border-color: var(--color-success);
        }

        .filter-btn.error.active {
          background: var(--color-error);
          border-color: var(--color-error);
        }

        .logs-content {
          flex: 1;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          overflow: hidden;
        }

        .logs-table-container {
          height: 100%;
          overflow: auto;
        }

        .logs-table {
          width: 100%;
          border-collapse: collapse;
        }

        .logs-table th,
        .logs-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .logs-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
        }

        .logs-table tr.error {
          background: rgba(255, 77, 79, 0.05);
        }

        .time-cell {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          white-space: nowrap;
        }

        .method-badge {
          padding: 2px 8px;
          border-radius: var(--border-radius-sm);
          color: #fff;
          font-size: 12px;
          font-weight: 600;
        }

        .endpoint-cell {
          font-family: monospace;
          font-size: var(--font-size-sm);
        }

        .status-badge {
          padding: 2px 8px;
          border-radius: var(--border-radius-sm);
          font-size: 12px;
        }

        .status-badge.success {
          background: rgba(82, 196, 26, 0.1);
          color: var(--color-success);
        }

        .status-badge.error {
          background: rgba(255, 77, 79, 0.1);
          color: var(--color-error);
        }

        .message-cell {
          max-width: 250px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .btn-detail {
          padding: 2px 8px;
          background: transparent;
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
          font-size: 12px;
        }

        .btn-detail:hover {
          border-color: var(--color-primary);
          color: var(--color-primary);
        }

        .modal-overlay {
          position: fixed;
          inset: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
        }

        .modal-content {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          width: 600px;
          max-width: 90vw;
          max-height: 80vh;
          display: flex;
          flex-direction: column;
        }

        .modal-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: var(--spacing-md) var(--spacing-lg);
          border-bottom: 1px solid var(--color-border-light);
        }

        .modal-header h2 {
          margin: 0;
          font-size: var(--font-size-lg);
        }

        .close-btn {
          background: none;
          border: none;
          font-size: 20px;
          cursor: pointer;
          color: var(--color-text-tertiary);
        }

        .modal-body {
          padding: var(--spacing-lg);
          overflow: auto;
        }

        .detail-row {
          display: flex;
          gap: var(--spacing-md);
          margin-bottom: var(--spacing-sm);
          align-items: center;
        }

        .detail-label {
          min-width: 60px;
          font-weight: 600;
          color: var(--color-text-secondary);
        }

        .detail-value {
          flex: 1;
        }

        .detail-value.mono {
          font-family: monospace;
        }

        .detail-section {
          margin-top: var(--spacing-md);
        }

        .code-block {
          background: var(--color-bg-layout);
          padding: var(--spacing-sm);
          border-radius: var(--border-radius-sm);
          font-family: monospace;
          font-size: var(--font-size-sm);
          overflow-x: auto;
          white-space: pre-wrap;
          word-break: break-all;
        }

        .modal-footer {
          padding: var(--spacing-md) var(--spacing-lg);
          border-top: 1px solid var(--color-border-light);
          display: flex;
          justify-content: flex-end;
        }

        .btn-secondary {
          padding: var(--spacing-sm) var(--spacing-md);
          background: var(--color-bg-container);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: var(--spacing-xl);
          color: var(--color-text-tertiary);
        }

        .empty-state-icon {
          font-size: 48px;
          margin-bottom: var(--spacing-md);
        }
      `}</style>
    </div>
  );
}
