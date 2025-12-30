import { useState, useEffect, useMemo } from "react";
import { api, ContractPriority } from "../api/tauri";

interface Issue {
  id: string;
  type: "critical" | "warning" | "info";
  title: string;
  description: string;
  suggestion: string;
  relatedContracts: ContractPriority[];
  time: string;
}

export function CockpitIssues() {
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState("");
  const [loading, setLoading] = useState(false);
  const [filterType, setFilterType] = useState<"all" | "critical" | "warning" | "info">("all");
  const [expandedIssue, setExpandedIssue] = useState<string | null>(null);

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
      console.error("Failed to load data:", err);
    } finally {
      setLoading(false);
    }
  };

  // Generate issues based on contract data
  const issues: Issue[] = useMemo(() => {
    const result: Issue[] = [];

    // Issue 1: Contracts expiring within 2 days
    const urgentContracts = contracts.filter((c) => c.days_to_pdd <= 2);
    if (urgentContracts.length > 0) {
      result.push({
        id: "issue-urgent",
        type: "critical",
        title: `${urgentContracts.length} 份合同即将超期`,
        description: "这些合同将在2天内到达计划交货日期，需要优先处理",
        suggestion: "建议立即安排紧急排产，确保按时交付",
        relatedContracts: urgentContracts,
        time: "实时检测",
      });
    }

    // Issue 2: High priority backlog
    const highPriorityContracts = contracts.filter((c) => c.priority >= 85);
    if (highPriorityContracts.length > 10) {
      result.push({
        id: "issue-backlog",
        type: "warning",
        title: "高优先级合同堆积",
        description: `${highPriorityContracts.length} 份高优先级合同待处理，可能影响客户满意度`,
        suggestion: "建议评估产能，考虑加班或调配资源",
        relatedContracts: highPriorityContracts.slice(0, 20),
        time: "30分钟前",
      });
    }

    // Issue 3: Manual adjustments need review
    const alphaContracts = contracts.filter((c) => c.alpha && c.alpha !== 1.0);
    if (alphaContracts.length > 0) {
      result.push({
        id: "issue-alpha",
        type: "info",
        title: `${alphaContracts.length} 份合同有人工调整`,
        description: "这些合同的优先级已被人工干预，可能需要定期审核",
        suggestion: "建议定期检查调整是否仍然有效，避免过期调整影响排产",
        relatedContracts: alphaContracts,
        time: "1小时前",
      });
    }

    // Issue 4: Single spec family overload
    const specFamilyCounts = new Map<string, number>();
    contracts.filter((c) => c.days_to_pdd <= 3).forEach((c) => {
      specFamilyCounts.set(c.spec_family, (specFamilyCounts.get(c.spec_family) || 0) + 1);
    });
    const overloadedFamilies = Array.from(specFamilyCounts.entries())
      .filter(([_, count]) => count > 10);
    if (overloadedFamilies.length > 0) {
      const familyContracts = contracts.filter(
        (c) => c.days_to_pdd <= 3 && overloadedFamilies.some(([f]) => f === c.spec_family)
      );
      result.push({
        id: "issue-overload",
        type: "warning",
        title: "规格族产能超载预警",
        description: `${overloadedFamilies.map(([f, c]) => `${f}(${c}份)`).join(", ")} 近3日订单集中`,
        suggestion: "建议协调产线资源，或与客户沟通调整交期",
        relatedContracts: familyContracts.slice(0, 15),
        time: "实时检测",
      });
    }

    // Issue 5: Low priority contracts approaching deadline
    const lowPriorityUrgent = contracts.filter(
      (c) => c.priority < 50 && c.days_to_pdd <= 5
    );
    if (lowPriorityUrgent.length > 5) {
      result.push({
        id: "issue-low-priority-urgent",
        type: "info",
        title: "低优先级合同临近交期",
        description: `${lowPriorityUrgent.length} 份低优先级合同5天内到期，需关注`,
        suggestion: "虽然优先级低，但仍需确保按时交付，避免客户投诉",
        relatedContracts: lowPriorityUrgent,
        time: "2小时前",
      });
    }

    return result;
  }, [contracts]);

  // Filter issues
  const filteredIssues = useMemo(() => {
    if (filterType === "all") return issues;
    return issues.filter((i) => i.type === filterType);
  }, [issues, filterType]);

  // Statistics
  const stats = useMemo(() => ({
    critical: issues.filter((i) => i.type === "critical").length,
    warning: issues.filter((i) => i.type === "warning").length,
    info: issues.filter((i) => i.type === "info").length,
  }), [issues]);

  return (
    <div className="cockpit-issues-page">
      <div className="page-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h1 className="page-header__title">问题清单</h1>
          <p className="page-header__subtitle">产销协同问题监控与预警</p>
        </div>
        <div style={{ display: "flex", gap: "var(--spacing-sm)", alignItems: "center" }}>
          <select
            value={selectedStrategy}
            onChange={(e) => setSelectedStrategy(e.target.value)}
            disabled={loading}
            className="strategy-select"
          >
            {strategies.map((s) => (
              <option key={s} value={s}>{s}</option>
            ))}
          </select>
          <button className="btn-refresh" onClick={loadData} disabled={loading}>
            {loading ? "刷新中..." : "刷新"}
          </button>
        </div>
      </div>

      {/* Statistics */}
      <div className="issues-stats">
        <div className={`stat-badge critical ${filterType === "critical" ? "active" : ""}`}
             onClick={() => setFilterType(filterType === "critical" ? "all" : "critical")}>
          <span className="stat-icon">🚨</span>
          <span className="stat-count">{stats.critical}</span>
          <span className="stat-label">紧急</span>
        </div>
        <div className={`stat-badge warning ${filterType === "warning" ? "active" : ""}`}
             onClick={() => setFilterType(filterType === "warning" ? "all" : "warning")}>
          <span className="stat-icon">⚠️</span>
          <span className="stat-count">{stats.warning}</span>
          <span className="stat-label">警告</span>
        </div>
        <div className={`stat-badge info ${filterType === "info" ? "active" : ""}`}
             onClick={() => setFilterType(filterType === "info" ? "all" : "info")}>
          <span className="stat-icon">ℹ️</span>
          <span className="stat-count">{stats.info}</span>
          <span className="stat-label">提示</span>
        </div>
      </div>

      {/* Issues List */}
      <div className="issues-container">
        {loading ? (
          <div className="loading-state">加载中...</div>
        ) : filteredIssues.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">✅</div>
            <div className="empty-state-text">
              {filterType === "all" ? "暂无问题，系统运行正常" : "该类型暂无问题"}
            </div>
          </div>
        ) : (
          <div className="issues-list">
            {filteredIssues.map((issue) => (
              <div key={issue.id} className={`issue-card ${issue.type}`}>
                <div className="issue-header" onClick={() => setExpandedIssue(
                  expandedIssue === issue.id ? null : issue.id
                )}>
                  <div className="issue-type-icon">
                    {issue.type === "critical" ? "🚨" : issue.type === "warning" ? "⚠️" : "ℹ️"}
                  </div>
                  <div className="issue-main">
                    <div className="issue-title">{issue.title}</div>
                    <div className="issue-desc">{issue.description}</div>
                  </div>
                  <div className="issue-meta">
                    <span className="issue-time">{issue.time}</span>
                    <span className="expand-icon">{expandedIssue === issue.id ? "▼" : "▶"}</span>
                  </div>
                </div>

                {expandedIssue === issue.id && (
                  <div className="issue-detail">
                    <div className="suggestion-box">
                      <strong>建议措施：</strong>
                      <p>{issue.suggestion}</p>
                    </div>

                    <div className="related-contracts">
                      <h4>相关合同 ({issue.relatedContracts.length})</h4>
                      <div className="contracts-table-container">
                        <table className="contracts-table">
                          <thead>
                            <tr>
                              <th>合同编号</th>
                              <th>客户</th>
                              <th>规格族</th>
                              <th>到期天数</th>
                              <th>优先级</th>
                              <th>Alpha</th>
                            </tr>
                          </thead>
                          <tbody>
                            {issue.relatedContracts.slice(0, 10).map((c) => (
                              <tr key={c.contract_id}>
                                <td className="contract-id">{c.contract_id}</td>
                                <td>{c.customer_id}</td>
                                <td>{c.spec_family}</td>
                                <td className={c.days_to_pdd <= 2 ? "urgent" : ""}>
                                  {c.days_to_pdd}天
                                </td>
                                <td>{c.priority.toFixed(2)}</td>
                                <td>{c.alpha?.toFixed(2) || "-"}</td>
                              </tr>
                            ))}
                          </tbody>
                        </table>
                        {issue.relatedContracts.length > 10 && (
                          <div className="more-hint">
                            还有 {issue.relatedContracts.length - 10} 份合同...
                          </div>
                        )}
                      </div>
                    </div>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      <style>{`
        .cockpit-issues-page {
          display: flex;
          flex-direction: column;
          height: 100%;
          gap: var(--spacing-md);
        }

        .strategy-select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        .btn-refresh {
          padding: var(--spacing-xs) var(--spacing-md);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .issues-stats {
          display: flex;
          gap: var(--spacing-md);
        }

        .stat-badge {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
          padding: var(--spacing-sm) var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-md);
          cursor: pointer;
          transition: all var(--transition-fast);
          border: 2px solid transparent;
        }

        .stat-badge:hover {
          transform: translateY(-2px);
        }

        .stat-badge.active {
          border-color: currentColor;
        }

        .stat-badge.critical { color: #ff4d4f; }
        .stat-badge.warning { color: #faad14; }
        .stat-badge.info { color: #1890ff; }

        .stat-count {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .stat-label {
          font-size: var(--font-size-sm);
        }

        .issues-container {
          flex: 1;
          overflow: auto;
        }

        .issues-list {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-md);
        }

        .issue-card {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          overflow: hidden;
          border-left: 4px solid;
        }

        .issue-card.critical { border-left-color: #ff4d4f; }
        .issue-card.warning { border-left-color: #faad14; }
        .issue-card.info { border-left-color: #1890ff; }

        .issue-header {
          display: flex;
          align-items: flex-start;
          gap: var(--spacing-md);
          padding: var(--spacing-md);
          cursor: pointer;
        }

        .issue-header:hover {
          background: var(--color-bg-layout);
        }

        .issue-type-icon {
          font-size: 24px;
        }

        .issue-main {
          flex: 1;
        }

        .issue-title {
          font-weight: 600;
          font-size: var(--font-size-base);
          margin-bottom: var(--spacing-xs);
        }

        .issue-desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .issue-meta {
          display: flex;
          flex-direction: column;
          align-items: flex-end;
          gap: var(--spacing-xs);
        }

        .issue-time {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .expand-icon {
          color: var(--color-text-tertiary);
          font-size: 12px;
        }

        .issue-detail {
          padding: var(--spacing-md);
          border-top: 1px solid var(--color-border-light);
          background: var(--color-bg-layout);
        }

        .suggestion-box {
          padding: var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-md);
          margin-bottom: var(--spacing-md);
        }

        .suggestion-box p {
          margin: var(--spacing-xs) 0 0 0;
          color: var(--color-text-secondary);
        }

        .related-contracts h4 {
          margin: 0 0 var(--spacing-sm) 0;
        }

        .contracts-table-container {
          overflow-x: auto;
        }

        .contracts-table {
          width: 100%;
          border-collapse: collapse;
          font-size: var(--font-size-sm);
        }

        .contracts-table th,
        .contracts-table td {
          padding: var(--spacing-xs) var(--spacing-sm);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .contracts-table th {
          background: var(--color-bg-container);
          font-weight: 600;
        }

        .contract-id {
          font-family: monospace;
        }

        .urgent {
          color: var(--color-error);
          font-weight: 600;
        }

        .more-hint {
          text-align: center;
          padding: var(--spacing-sm);
          color: var(--color-text-tertiary);
          font-size: var(--font-size-sm);
        }

        .loading-state, .empty-state {
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
