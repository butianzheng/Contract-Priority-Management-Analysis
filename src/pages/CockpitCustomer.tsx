import { useState, useEffect, useMemo } from "react";
import {
  api,
  ContractPriority,
  CustomerProtectionAnalysis,
  CustomerProtectionSummary,
  RiskIdentificationResult,
  RiskContractFlag,
  Customer,
} from "../api/tauri";

type TabType = "customer" | "deadline" | "risk" | "consensus";

export function CockpitCustomer() {
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState("");
  const [loading, setLoading] = useState(false);

  // 数据状态
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [customerAnalysis, setCustomerAnalysis] = useState<CustomerProtectionAnalysis | null>(null);
  const [riskResult, setRiskResult] = useState<RiskIdentificationResult | null>(null);
  const [customers, setCustomers] = useState<Map<string, Customer>>(new Map());

  // UI 状态
  const [activeTab, setActiveTab] = useState<TabType>("customer");
  const [deadlineDays, setDeadlineDays] = useState<number>(3);

  useEffect(() => {
    Promise.all([api.getStrategies(), api.getCustomers()]).then(
      ([strats, customerList]) => {
        setStrategies(strats);
        const customerMap = new Map<string, Customer>();
        customerList.forEach((c) => customerMap.set(c.customer_id, c));
        setCustomers(customerMap);

        if (strats.length > 0) {
          setSelectedStrategy(strats[0]);
        }
      }
    );
  }, []);

  useEffect(() => {
    if (selectedStrategy) {
      loadData();
    }
  }, [selectedStrategy]);

  const loadData = async () => {
    setLoading(true);
    try {
      const [contractsData, customerData, riskData] = await Promise.all([
        api.computeAllPriorities(selectedStrategy),
        api.analyzeCustomerProtection(selectedStrategy),
        api.identifyRiskContracts(selectedStrategy),
      ]);
      setContracts(contractsData);
      setCustomerAnalysis(customerData);
      setRiskResult(riskData);
    } catch (err) {
      console.error("加载数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 临期合同（按 days_to_pdd 筛选）
  const deadlineContracts = useMemo(() => {
    return contracts
      .filter((c) => c.days_to_pdd <= deadlineDays)
      .sort((a, b) => a.days_to_pdd - b.days_to_pdd);
  }, [contracts, deadlineDays]);

  // 统计概览
  const stats = useMemo(() => {
    return {
      totalCustomers: customerAnalysis?.total_customers || 0,
      protectedCount: customerAnalysis?.well_protected_count || 0,
      riskCustomerCount: customerAnalysis?.risk_count || 0,
      deadlineCount: deadlineContracts.length,
      highRiskCount: riskResult?.high_risk_count || 0,
      mediumRiskCount: riskResult?.medium_risk_count || 0,
    };
  }, [customerAnalysis, riskResult, deadlineContracts]);

  // 客户等级样式
  const getCustomerLevelStyle = (level: string) => {
    switch (level) {
      case "A":
        return { background: "#fff0f0", color: "#cf1322", fontWeight: 600 };
      case "B":
        return { background: "#fff7e6", color: "#d46b08", fontWeight: 600 };
      case "C":
        return { background: "#f6ffed", color: "#389e0d" };
      default:
        return {};
    }
  };

  // 保障状态样式
  const getProtectionStyle = (status: string) => {
    switch (status) {
      case "good":
        return { background: "#f6ffed", color: "#389e0d" };
      case "warning":
        return { background: "#fff7e6", color: "#d46b08" };
      case "risk":
        return { background: "#fff0f0", color: "#cf1322" };
      default:
        return {};
    }
  };

  // 风险等级样式
  const getRiskLevelStyle = (level: string) => {
    switch (level) {
      case "high":
        return { background: "#fff0f0", color: "#cf1322", fontWeight: 600 };
      case "medium":
        return { background: "#fff7e6", color: "#d46b08" };
      case "low":
        return { background: "#f6ffed", color: "#389e0d" };
      default:
        return {};
    }
  };

  // 生成共识模板文本
  const generateConsensusTemplate = () => {
    const lines: string[] = [];
    lines.push(`【客户保障与交付风险共识】`);
    lines.push(`策略：${selectedStrategy}`);
    lines.push(`生成时间：${new Date().toLocaleString()}`);
    lines.push("");

    lines.push(`一、客户保障情况`);
    lines.push(`  - 客户总数：${stats.totalCustomers}`);
    lines.push(`  - 保障良好：${stats.protectedCount}`);
    lines.push(`  - 风险客户：${stats.riskCustomerCount}`);
    lines.push("");

    if (customerAnalysis && customerAnalysis.risk_customers.length > 0) {
      lines.push(`  风险客户清单：`);
      customerAnalysis.risk_customers.slice(0, 5).forEach((c, i) => {
        lines.push(
          `    ${i + 1}. ${c.customer_id}（${c.customer_level}级）- 平均排名 #${c.avg_rank.toFixed(0)}，${c.risk_description || "需关注"}`
        );
      });
      lines.push("");
    }

    lines.push(`二、临期合同（${deadlineDays}天内）`);
    lines.push(`  - 合同数量：${stats.deadlineCount}`);
    if (deadlineContracts.length > 0) {
      lines.push(`  重点关注：`);
      deadlineContracts.slice(0, 5).forEach((c, i) => {
        const customer = customers.get(c.customer_id);
        lines.push(
          `    ${i + 1}. ${c.contract_id}（${customer?.customer_level || "-"}级客户）- T-${c.days_to_pdd}天`
        );
      });
    }
    lines.push("");

    lines.push(`三、风险合同`);
    lines.push(`  - 高风险：${stats.highRiskCount}`);
    lines.push(`  - 中风险：${stats.mediumRiskCount}`);
    if (riskResult && riskResult.risk_contracts.length > 0) {
      lines.push(`  高风险清单：`);
      riskResult.risk_contracts
        .filter((r) => r.risk_level === "high")
        .slice(0, 5)
        .forEach((r, i) => {
          lines.push(`    ${i + 1}. ${r.contract_id} - ${r.risk_description}`);
        });
    }
    lines.push("");

    lines.push(`四、行动建议`);
    lines.push(`  1. 优先处理高风险合同，确保交付`);
    lines.push(`  2. 关注风险客户，提升保障排名`);
    lines.push(`  3. 临期合同加急排产`);

    return lines.join("\n");
  };

  const copyConsensus = () => {
    const text = generateConsensusTemplate();
    navigator.clipboard.writeText(text).then(() => {
      alert("已复制到剪贴板");
    });
  };

  return (
    <div className="cockpit-customer-page">
      <div className="page-header">
        <div>
          <h1 className="page-header__title">客户保障与交付风险</h1>
          <p className="page-header__subtitle">
            重点客户保障分析、临期合同跟踪、风险清单管理
          </p>
        </div>
        <div className="header-controls">
          <select
            value={selectedStrategy}
            onChange={(e) => setSelectedStrategy(e.target.value)}
            disabled={loading}
            className="strategy-select"
          >
            {strategies.map((s) => (
              <option key={s} value={s}>
                {s}
              </option>
            ))}
          </select>
          <button className="btn-refresh" onClick={loadData} disabled={loading}>
            {loading ? "加载中..." : "刷新"}
          </button>
        </div>
      </div>

      {/* 统计概览 */}
      <div className="stats-overview">
        <div className="stat-card">
          <div className="stat-value">{stats.totalCustomers}</div>
          <div className="stat-label">客户总数</div>
        </div>
        <div className="stat-card good">
          <div className="stat-value">{stats.protectedCount}</div>
          <div className="stat-label">保障良好</div>
        </div>
        <div className="stat-card warning">
          <div className="stat-value">{stats.riskCustomerCount}</div>
          <div className="stat-label">风险客户</div>
        </div>
        <div className="stat-card danger">
          <div className="stat-value">{stats.deadlineCount}</div>
          <div className="stat-label">临期合同</div>
        </div>
        <div className="stat-card danger">
          <div className="stat-value">{stats.highRiskCount}</div>
          <div className="stat-label">高风险</div>
        </div>
      </div>

      {/* Tab 导航 */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === "customer" ? "active" : ""}`}
          onClick={() => setActiveTab("customer")}
        >
          重点客户保障
        </button>
        <button
          className={`tab-btn ${activeTab === "deadline" ? "active" : ""}`}
          onClick={() => setActiveTab("deadline")}
        >
          临期合同 ({stats.deadlineCount})
        </button>
        <button
          className={`tab-btn ${activeTab === "risk" ? "active" : ""}`}
          onClick={() => setActiveTab("risk")}
        >
          风险清单 ({riskResult?.total_count || 0})
        </button>
        <button
          className={`tab-btn ${activeTab === "consensus" ? "active" : ""}`}
          onClick={() => setActiveTab("consensus")}
        >
          共识模板
        </button>
      </div>

      {/* Tab 内容 */}
      <div className="tab-content">
        {/* 重点客户保障 */}
        {activeTab === "customer" && (
          <div className="customer-protection-panel">
            {!customerAnalysis ? (
              <div className="empty-state">
                <div className="empty-icon">📊</div>
                <div>加载中...</div>
              </div>
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>客户ID</th>
                    <th>等级</th>
                    <th>合同数</th>
                    <th>平均排名</th>
                    <th>最优排名</th>
                    <th>Top50占比</th>
                    <th>保障状态</th>
                    <th>风险说明</th>
                  </tr>
                </thead>
                <tbody>
                  {customerAnalysis.customers.map((c: CustomerProtectionSummary) => (
                    <tr
                      key={c.customer_id}
                      className={c.protection_status === "risk" ? "risk-row" : ""}
                    >
                      <td className="customer-id">{c.customer_id}</td>
                      <td>
                        <span
                          className="level-badge"
                          style={getCustomerLevelStyle(c.customer_level)}
                        >
                          {c.customer_level === "A"
                            ? "战略"
                            : c.customer_level === "B"
                              ? "重点"
                              : "一般"}
                        </span>
                      </td>
                      <td>{c.contract_count}</td>
                      <td>#{c.avg_rank.toFixed(0)}</td>
                      <td>#{c.best_rank}</td>
                      <td>
                        {c.contract_count > 0
                          ? ((c.top_n_count / c.contract_count) * 100).toFixed(0)
                          : 0}
                        %
                      </td>
                      <td>
                        <span
                          className="status-badge"
                          style={getProtectionStyle(c.protection_status)}
                        >
                          {c.protection_status === "good"
                            ? "良好"
                            : c.protection_status === "warning"
                              ? "警告"
                              : "风险"}
                        </span>
                      </td>
                      <td className="risk-desc">{c.risk_description || "-"}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* 临期合同 */}
        {activeTab === "deadline" && (
          <div className="deadline-panel">
            <div className="deadline-controls">
              <label>到期天数 ≤</label>
              <select
                value={deadlineDays}
                onChange={(e) => setDeadlineDays(Number(e.target.value))}
              >
                <option value={1}>1天</option>
                <option value={2}>2天</option>
                <option value={3}>3天</option>
                <option value={5}>5天</option>
                <option value={7}>7天</option>
              </select>
            </div>

            {deadlineContracts.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">✅</div>
                <div>无临期合同</div>
              </div>
            ) : (
              <table className="data-table">
                <thead>
                  <tr>
                    <th>合同ID</th>
                    <th>客户</th>
                    <th>等级</th>
                    <th>规格族</th>
                    <th>钢种</th>
                    <th>剩余天数</th>
                    <th>优先级</th>
                    <th>当前排名</th>
                  </tr>
                </thead>
                <tbody>
                  {deadlineContracts.map((c, idx) => {
                    const customer = customers.get(c.customer_id);
                    return (
                      <tr
                        key={c.contract_id}
                        className={c.days_to_pdd <= 1 ? "urgent-row" : ""}
                      >
                        <td className="contract-id">{c.contract_id}</td>
                        <td>{c.customer_id}</td>
                        <td>
                          <span
                            className="level-badge"
                            style={getCustomerLevelStyle(
                              customer?.customer_level || ""
                            )}
                          >
                            {customer?.customer_level || "-"}
                          </span>
                        </td>
                        <td>{c.spec_family}</td>
                        <td>{c.steel_grade}</td>
                        <td
                          className={c.days_to_pdd <= 1 ? "urgent-text" : "warning-text"}
                        >
                          T-{c.days_to_pdd}天
                        </td>
                        <td>{c.priority.toFixed(1)}</td>
                        <td>#{idx + 1}</td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            )}
          </div>
        )}

        {/* 风险清单 */}
        {activeTab === "risk" && (
          <div className="risk-panel">
            {!riskResult || riskResult.risk_contracts.length === 0 ? (
              <div className="empty-state">
                <div className="empty-icon">✅</div>
                <div>无风险合同</div>
              </div>
            ) : (
              <>
                <div className="risk-summary">
                  <div className="risk-stat high">
                    <span className="risk-count">{riskResult.high_risk_count}</span>
                    <span className="risk-label">高风险</span>
                  </div>
                  <div className="risk-stat medium">
                    <span className="risk-count">{riskResult.medium_risk_count}</span>
                    <span className="risk-label">中风险</span>
                  </div>
                  <div className="risk-stat low">
                    <span className="risk-count">{riskResult.low_risk_count}</span>
                    <span className="risk-label">低风险</span>
                  </div>
                </div>

                <table className="data-table">
                  <thead>
                    <tr>
                      <th>合同ID</th>
                      <th>风险等级</th>
                      <th>风险类型</th>
                      <th>风险描述</th>
                      <th>建议措施</th>
                      <th>状态</th>
                    </tr>
                  </thead>
                  <tbody>
                    {riskResult.risk_contracts.map((r: RiskContractFlag) => (
                      <tr key={r.contract_id}>
                        <td className="contract-id">{r.contract_id}</td>
                        <td>
                          <span
                            className="level-badge"
                            style={getRiskLevelStyle(r.risk_level)}
                          >
                            {r.risk_level === "high"
                              ? "高"
                              : r.risk_level === "medium"
                                ? "中"
                                : "低"}
                          </span>
                        </td>
                        <td>{r.risk_type}</td>
                        <td className="risk-desc">{r.risk_description}</td>
                        <td>{r.suggested_action || "-"}</td>
                        <td>
                          <span
                            className={`status-tag ${r.status === "resolved" ? "resolved" : r.status === "in_progress" ? "progress" : "open"}`}
                          >
                            {r.status === "resolved"
                              ? "已解决"
                              : r.status === "in_progress"
                                ? "处理中"
                                : "待处理"}
                          </span>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </>
            )}
          </div>
        )}

        {/* 共识模板 */}
        {activeTab === "consensus" && (
          <div className="consensus-panel">
            <div className="consensus-header">
              <h3>会议共识模板</h3>
              <button className="btn-copy" onClick={copyConsensus}>
                复制到剪贴板
              </button>
            </div>
            <pre className="consensus-content">{generateConsensusTemplate()}</pre>
          </div>
        )}
      </div>

      <style>{`
        .cockpit-customer-page {
          display: flex;
          flex-direction: column;
          height: 100%;
          gap: var(--spacing-md);
        }

        .page-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .header-controls {
          display: flex;
          gap: var(--spacing-sm);
          align-items: center;
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

        .btn-refresh:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        /* 统计概览 */
        .stats-overview {
          display: grid;
          grid-template-columns: repeat(5, 1fr);
          gap: var(--spacing-md);
        }

        .stat-card {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          text-align: center;
        }

        .stat-card.good { border-left: 4px solid #52c41a; }
        .stat-card.warning { border-left: 4px solid #faad14; }
        .stat-card.danger { border-left: 4px solid #ff4d4f; }

        .stat-value {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .stat-label {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        /* Tab 导航 */
        .tab-nav {
          display: flex;
          gap: var(--spacing-xs);
          background: var(--color-bg-container);
          padding: var(--spacing-xs);
          border-radius: var(--border-radius-md);
        }

        .tab-btn {
          padding: var(--spacing-sm) var(--spacing-lg);
          border: none;
          background: transparent;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
          font-weight: 500;
          transition: all var(--transition-fast);
        }

        .tab-btn:hover {
          background: var(--color-bg-layout);
        }

        .tab-btn.active {
          background: var(--color-primary);
          color: #fff;
        }

        /* Tab 内容 */
        .tab-content {
          flex: 1;
          min-height: 0;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          overflow: hidden;
        }

        .customer-protection-panel,
        .deadline-panel,
        .risk-panel,
        .consensus-panel {
          height: 100%;
          overflow: auto;
          padding: var(--spacing-md);
        }

        /* 数据表格 */
        .data-table {
          width: 100%;
          border-collapse: collapse;
        }

        .data-table th,
        .data-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .data-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
          z-index: 1;
        }

        .data-table tbody tr:hover {
          background: var(--color-primary-light);
        }

        .data-table tbody tr.risk-row {
          background: rgba(255, 77, 79, 0.05);
        }

        .data-table tbody tr.urgent-row {
          background: rgba(255, 77, 79, 0.1);
        }

        .contract-id, .customer-id {
          font-family: monospace;
          font-weight: 500;
        }

        .level-badge, .status-badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: var(--font-size-sm);
        }

        .risk-desc {
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
          white-space: nowrap;
        }

        .urgent-text {
          color: #cf1322;
          font-weight: 600;
        }

        .warning-text {
          color: #d46b08;
        }

        /* 临期合同控制 */
        .deadline-controls {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
          margin-bottom: var(--spacing-md);
          padding-bottom: var(--spacing-md);
          border-bottom: 1px solid var(--color-border-light);
        }

        .deadline-controls select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        /* 风险摘要 */
        .risk-summary {
          display: flex;
          gap: var(--spacing-lg);
          margin-bottom: var(--spacing-md);
          padding-bottom: var(--spacing-md);
          border-bottom: 1px solid var(--color-border-light);
        }

        .risk-stat {
          display: flex;
          align-items: center;
          gap: var(--spacing-xs);
        }

        .risk-stat.high .risk-count { color: #cf1322; }
        .risk-stat.medium .risk-count { color: #d46b08; }
        .risk-stat.low .risk-count { color: #389e0d; }

        .risk-count {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .risk-label {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .status-tag {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: var(--font-size-sm);
        }

        .status-tag.open {
          background: #fff0f0;
          color: #cf1322;
        }

        .status-tag.progress {
          background: #fff7e6;
          color: #d46b08;
        }

        .status-tag.resolved {
          background: #f6ffed;
          color: #389e0d;
        }

        /* 共识模板 */
        .consensus-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: var(--spacing-md);
        }

        .consensus-header h3 {
          margin: 0;
        }

        .btn-copy {
          padding: var(--spacing-xs) var(--spacing-md);
          background: var(--color-success);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .consensus-content {
          background: var(--color-bg-layout);
          padding: var(--spacing-lg);
          border-radius: var(--border-radius-md);
          font-family: monospace;
          font-size: var(--font-size-sm);
          line-height: 1.6;
          white-space: pre-wrap;
          overflow: auto;
          max-height: calc(100% - 60px);
        }

        /* 空状态 */
        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 200px;
          color: var(--color-text-tertiary);
        }

        .empty-icon {
          font-size: 48px;
          margin-bottom: var(--spacing-md);
        }
      `}</style>
    </div>
  );
}
