import { useState, useEffect, useMemo } from "react";
import { api, InterventionLog } from "../api/tauri";

export function ContractHistory() {
  const [logs, setLogs] = useState<InterventionLog[]>([]);
  const [loading, setLoading] = useState(false);
  const [limit, setLimit] = useState(100);
  const [filterContract, setFilterContract] = useState("");
  const [filterUser, setFilterUser] = useState("");

  useEffect(() => {
    loadHistory();
  }, [limit]);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const data = await api.getAllInterventionLogs(limit);
      setLogs(data);
    } catch (err) {
      console.error("Failed to load history:", err);
    } finally {
      setLoading(false);
    }
  };

  // Unique users for filter
  const uniqueUsers = useMemo(() => {
    return Array.from(new Set(logs.map((l) => l.user))).sort();
  }, [logs]);

  // Filtered logs
  const filteredLogs = useMemo(() => {
    return logs.filter((log) => {
      if (filterContract && !log.contract_id.toLowerCase().includes(filterContract.toLowerCase())) {
        return false;
      }
      if (filterUser && log.user !== filterUser) {
        return false;
      }
      return true;
    });
  }, [logs, filterContract, filterUser]);

  const formatTime = (isoString?: string) => {
    if (!isoString) return "-";
    return new Date(isoString).toLocaleString("zh-CN");
  };

  return (
    <div className="contract-history-page">
      <div className="page-header">
        <h1 className="page-header__title">合同历史记录</h1>
        <p className="page-header__subtitle">查看优先级干预操作历史</p>
      </div>

      {/* Filter Controls */}
      <div className="history-controls">
        <div className="control-group">
          <label>合同编号</label>
          <input
            type="text"
            placeholder="搜索合同编号..."
            value={filterContract}
            onChange={(e) => setFilterContract(e.target.value)}
          />
        </div>
        <div className="control-group">
          <label>操作人</label>
          <select value={filterUser} onChange={(e) => setFilterUser(e.target.value)}>
            <option value="">全部</option>
            {uniqueUsers.map((user) => (
              <option key={user} value={user}>{user}</option>
            ))}
          </select>
        </div>
        <div className="control-group">
          <label>显示条数</label>
          <select value={limit} onChange={(e) => setLimit(Number(e.target.value))}>
            <option value={50}>最近 50 条</option>
            <option value={100}>最近 100 条</option>
            <option value={200}>最近 200 条</option>
            <option value={500}>最近 500 条</option>
          </select>
        </div>
        <button className="btn-refresh" onClick={loadHistory} disabled={loading}>
          {loading ? "加载中..." : "刷新"}
        </button>
      </div>

      {/* History Table */}
      <div className="history-table-container">
        {loading ? (
          <div className="loading-state">加载中...</div>
        ) : filteredLogs.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">📜</div>
            <div className="empty-state-text">暂无历史记录</div>
          </div>
        ) : (
          <table className="history-table">
            <thead>
              <tr>
                <th>合同编号</th>
                <th>Alpha 值</th>
                <th>调整原因</th>
                <th>操作人</th>
                <th>操作时间</th>
              </tr>
            </thead>
            <tbody>
              {filteredLogs.map((log, index) => (
                <tr key={`${log.contract_id}-${index}`}>
                  <td>
                    <span className="contract-id">{log.contract_id}</span>
                  </td>
                  <td>
                    <span className={`alpha-badge ${log.alpha_value > 1 ? "up" : log.alpha_value < 1 ? "down" : "neutral"}`}>
                      {log.alpha_value.toFixed(2)}
                    </span>
                  </td>
                  <td className="reason-cell">{log.reason}</td>
                  <td>{log.user}</td>
                  <td className="time-cell">{formatTime(log.timestamp)}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {/* Summary */}
      <div className="history-summary">
        <span>共 {filteredLogs.length} 条记录</span>
        {filterContract || filterUser ? (
          <span className="filter-hint">（已筛选）</span>
        ) : null}
      </div>

      <style>{`
        .contract-history-page {
          display: flex;
          flex-direction: column;
          height: 100%;
        }

        .history-controls {
          display: flex;
          gap: var(--spacing-md);
          align-items: flex-end;
          padding: var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          margin-bottom: var(--spacing-md);
          flex-wrap: wrap;
        }

        .control-group {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-xs);
        }

        .control-group label {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .control-group input,
        .control-group select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          background: var(--color-bg-container);
          min-width: 150px;
        }

        .btn-refresh {
          padding: var(--spacing-xs) var(--spacing-md);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .btn-refresh:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .history-table-container {
          flex: 1;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          overflow: auto;
        }

        .history-table {
          width: 100%;
          border-collapse: collapse;
        }

        .history-table th,
        .history-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .history-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
        }

        .history-table tbody tr:hover {
          background: var(--color-primary-light);
        }

        .contract-id {
          font-family: monospace;
          font-weight: 500;
        }

        .alpha-badge {
          padding: 2px 8px;
          border-radius: var(--border-radius-sm);
          font-size: var(--font-size-sm);
          font-weight: 600;
        }

        .alpha-badge.up {
          background: rgba(82, 196, 26, 0.1);
          color: var(--color-success);
        }

        .alpha-badge.down {
          background: rgba(255, 77, 79, 0.1);
          color: var(--color-error);
        }

        .alpha-badge.neutral {
          background: var(--color-bg-layout);
          color: var(--color-text-secondary);
        }

        .reason-cell {
          max-width: 300px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .time-cell {
          color: var(--color-text-tertiary);
          font-size: var(--font-size-sm);
        }

        .loading-state,
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

        .history-summary {
          padding: var(--spacing-sm) var(--spacing-md);
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .filter-hint {
          color: var(--color-primary);
          margin-left: var(--spacing-xs);
        }
      `}</style>
    </div>
  );
}
