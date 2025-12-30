import { useState, useEffect, useMemo } from "react";
import { api, ContractPriority, StrategyWeights, Customer, PriorityExplain } from "../api/tauri";

interface CompareResult {
  contract_id: string;
  customer_id: string;
  customer_level: string;
  spec_family: string;
  days_to_pdd: number;
  strategy1Priority: number;
  strategy1Rank: number;
  strategy2Priority: number;
  strategy2Rank: number;
  priorityDiff: number;
  rankDiff: number;
  changeSource: string;  // 变化来源说明
}

export function SandboxCompare() {
  const [strategies, setStrategies] = useState<string[]>([]);
  const [strategyWeights, setStrategyWeights] = useState<StrategyWeights[]>([]);
  const [strategy1, setStrategy1] = useState("");
  const [strategy2, setStrategy2] = useState("");
  const [contracts1, setContracts1] = useState<ContractPriority[]>([]);
  const [contracts2, setContracts2] = useState<ContractPriority[]>([]);
  const [customers, setCustomers] = useState<Map<string, Customer>>(new Map());
  const [loading, setLoading] = useState(false);
  const [compared, setCompared] = useState(false);

  // 选中合同的 Explain 详情
  const [selectedContract, setSelectedContract] = useState<string | null>(null);
  const [explain1, setExplain1] = useState<PriorityExplain | null>(null);
  const [explain2, setExplain2] = useState<PriorityExplain | null>(null);
  const [loadingExplain, setLoadingExplain] = useState(false);
  const [explainError, setExplainError] = useState<string | null>(null);

  // Top N 筛选
  const [topN, setTopN] = useState<number>(50);

  useEffect(() => {
    Promise.all([
      api.getStrategies(),
      api.getStrategyWeightsList(),
      api.getCustomers()
    ]).then(
      ([strats, weights, customerList]) => {
        setStrategies(strats);
        setStrategyWeights(weights);

        // 构建客户 Map
        const customerMap = new Map<string, Customer>();
        customerList.forEach(c => customerMap.set(c.customer_id, c));
        setCustomers(customerMap);

        if (strats.length >= 2) {
          setStrategy1(strats[0]);
          setStrategy2(strats[1]);
        } else if (strats.length === 1) {
          setStrategy1(strats[0]);
          setStrategy2(strats[0]);
        }
      }
    );
  }, []);

  const runCompare = async () => {
    if (!strategy1 || !strategy2) return;

    setLoading(true);
    setSelectedContract(null);
    setExplain1(null);
    setExplain2(null);

    try {
      const [data1, data2] = await Promise.all([
        api.computeAllPriorities(strategy1),
        api.computeAllPriorities(strategy2),
      ]);
      setContracts1(data1);
      setContracts2(data2);
      setCompared(true);
    } catch (err) {
      console.error("Compare failed:", err);
    } finally {
      setLoading(false);
    }
  };

  // 加载选中合同的 Explain 详情
  const loadExplain = async (contractId: string) => {
    setSelectedContract(contractId);
    setLoadingExplain(true);
    setExplainError(null);

    try {
      const [e1, e2] = await Promise.all([
        api.explainPriority(contractId, strategy1),
        api.explainPriority(contractId, strategy2),
      ]);
      console.log("Explain loaded:", { e1, e2 });
      setExplain1(e1);
      setExplain2(e2);
    } catch (err) {
      console.error("Load explain failed:", err);
      setExplainError(String(err));
      setExplain1(null);
      setExplain2(null);
    } finally {
      setLoadingExplain(false);
    }
  };

  // 计算变化来源
  const calculateChangeSource = (
    c1: ContractPriority,
    c2: ContractPriority,
    w1: StrategyWeights | undefined,
    w2: StrategyWeights | undefined
  ): string => {
    const sources: string[] = [];

    // 检查 S-Score 变化
    const sDiff = c2.s_score - c1.s_score;
    if (Math.abs(sDiff) > 1) {
      sources.push(sDiff > 0 ? "S↑" : "S↓");
    }

    // 检查 P-Score 变化
    const pDiff = c2.p_score - c1.p_score;
    if (Math.abs(pDiff) > 1) {
      sources.push(pDiff > 0 ? "P↑" : "P↓");
    }

    // 检查权重变化
    if (w1 && w2) {
      const wsDiff = w2.ws - w1.ws;
      if (Math.abs(wsDiff) > 0.01) {
        sources.push(wsDiff > 0 ? "ws↑" : "ws↓");
      }
      const wpDiff = w2.wp - w1.wp;
      if (Math.abs(wpDiff) > 0.01) {
        sources.push(wpDiff > 0 ? "wp↑" : "wp↓");
      }
    }

    // 检查 Alpha 干预
    if (c1.alpha && c1.alpha !== 1.0) {
      sources.push(`α=${c1.alpha.toFixed(2)}`);
    }
    if (c2.alpha && c2.alpha !== 1.0) {
      sources.push(`α'=${c2.alpha.toFixed(2)}`);
    }

    return sources.length > 0 ? sources.join(" + ") : "-";
  };

  const compareResults: CompareResult[] = useMemo(() => {
    if (!compared || contracts1.length === 0) return [];

    const w1 = strategyWeights.find(w => w.strategy_name === strategy1);
    const w2 = strategyWeights.find(w => w.strategy_name === strategy2);

    // Sort and rank strategy1
    const sorted1 = [...contracts1].sort((a, b) => b.priority - a.priority);
    const rankMap1 = new Map<string, number>();
    sorted1.forEach((c, i) => rankMap1.set(c.contract_id, i + 1));

    // Sort and rank strategy2
    const sorted2 = [...contracts2].sort((a, b) => b.priority - a.priority);
    const rankMap2 = new Map<string, number>();
    sorted2.forEach((c, i) => rankMap2.set(c.contract_id, i + 1));

    // Create map for strategy2 lookup
    const contractMap2 = new Map<string, ContractPriority>();
    contracts2.forEach((c) => contractMap2.set(c.contract_id, c));

    return sorted1.slice(0, topN).map((c) => {
      const c2 = contractMap2.get(c.contract_id);
      const customer = customers.get(c.customer_id);

      return {
        contract_id: c.contract_id,
        customer_id: c.customer_id,
        customer_level: customer?.customer_level || "-",
        spec_family: c.spec_family,
        days_to_pdd: c.days_to_pdd,
        strategy1Priority: c.priority,
        strategy1Rank: rankMap1.get(c.contract_id) || 0,
        strategy2Priority: c2?.priority || 0,
        strategy2Rank: rankMap2.get(c.contract_id) || 0,
        priorityDiff: (c2?.priority || 0) - c.priority,
        rankDiff: (rankMap1.get(c.contract_id) || 0) - (rankMap2.get(c.contract_id) || 0),
        changeSource: c2 ? calculateChangeSource(c, c2, w1, w2) : "-",
      };
    });
  }, [compared, contracts1, contracts2, customers, strategyWeights, strategy1, strategy2, topN]);

  // Statistics
  const stats = useMemo(() => {
    if (compareResults.length === 0) {
      return { total: 0, rankUp: 0, rankDown: 0, noChange: 0, avgRankDiff: 0, avgPriorityDiff: 0 };
    }
    const rankUp = compareResults.filter((r) => r.rankDiff > 0).length;
    const rankDown = compareResults.filter((r) => r.rankDiff < 0).length;
    const noChange = compareResults.filter((r) => r.rankDiff === 0).length;
    const avgRankDiff =
      compareResults.reduce((sum, r) => sum + r.rankDiff, 0) / compareResults.length;
    const avgPriorityDiff =
      compareResults.reduce((sum, r) => sum + r.priorityDiff, 0) / compareResults.length;

    return { total: compareResults.length, rankUp, rankDown, noChange, avgRankDiff, avgPriorityDiff };
  }, [compareResults]);

  const getWeightInfo = (strategyName: string) => {
    const w = strategyWeights.find((sw) => sw.strategy_name === strategyName);
    return w ? `(${w.ws.toFixed(1)}/${w.wp.toFixed(1)})` : "";
  };

  // 获取客户等级样式
  const getCustomerLevelStyle = (level: string) => {
    switch (level) {
      case "A": return { background: "#fff0f0", color: "#cf1322", fontWeight: 600 };
      case "B": return { background: "#fff7e6", color: "#d46b08", fontWeight: 600 };
      case "C": return { background: "#f6ffed", color: "#389e0d" };
      default: return {};
    }
  };

  // 获取交期紧急度样式
  const getDaysStyle = (days: number) => {
    if (days <= 3) return { color: "#cf1322", fontWeight: 600 };
    if (days <= 7) return { color: "#d46b08" };
    return {};
  };

  return (
    <div className="sandbox-compare-page">
      <div className="page-header">
        <h1 className="page-header__title">排名变化拆解</h1>
        <p className="page-header__subtitle">
          对比不同策略下的优先级排名差异，分析变化来源
        </p>
      </div>

      {/* Strategy Selection */}
      <div className="compare-controls">
        <div className="strategy-select">
          <div className="strategy-box">
            <label>策略 A</label>
            <select value={strategy1} onChange={(e) => setStrategy1(e.target.value)}>
              {strategies.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
            <span className="weight-info">{getWeightInfo(strategy1)}</span>
          </div>
          <div className="vs-divider">VS</div>
          <div className="strategy-box">
            <label>策略 B</label>
            <select value={strategy2} onChange={(e) => setStrategy2(e.target.value)}>
              {strategies.map((s) => (
                <option key={s} value={s}>{s}</option>
              ))}
            </select>
            <span className="weight-info">{getWeightInfo(strategy2)}</span>
          </div>
        </div>
        <div className="control-right">
          <div className="top-n-selector">
            <label>Top</label>
            <select value={topN} onChange={(e) => setTopN(Number(e.target.value))}>
              <option value={20}>20</option>
              <option value={50}>50</option>
              <option value={100}>100</option>
              <option value={200}>200</option>
            </select>
          </div>
          <button className="btn-compare" onClick={runCompare} disabled={loading || !strategy1 || !strategy2}>
            {loading ? "计算中..." : "开始对比"}
          </button>
        </div>
      </div>

      {/* Statistics Cards */}
      {compared && (
        <div className="compare-stats">
          <div className="stat-card">
            <div className="stat-label">统计范围</div>
            <div className="stat-value">Top {stats.total}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">排名上升 (A→B)</div>
            <div className="stat-value up">{stats.rankUp}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">排名下降 (A→B)</div>
            <div className="stat-value down">{stats.rankDown}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">排名不变</div>
            <div className="stat-value">{stats.noChange}</div>
          </div>
          <div className="stat-card">
            <div className="stat-label">平均位次变化</div>
            <div className="stat-value" style={{ color: stats.avgRankDiff > 0 ? "var(--color-success)" : stats.avgRankDiff < 0 ? "var(--color-error)" : "inherit" }}>
              {stats.avgRankDiff > 0 ? "+" : ""}{stats.avgRankDiff.toFixed(1)}
            </div>
          </div>
        </div>
      )}

      {/* Main Content */}
      <div className="compare-main">
        {/* Compare Table */}
        <div className="compare-table-container">
          {!compared ? (
            <div className="empty-state">
              <div className="empty-state-icon">📊</div>
              <div className="empty-state-text">选择两个策略后点击"开始对比"</div>
            </div>
          ) : (
            <table className="compare-table">
              <thead>
                <tr>
                  <th>合同ID</th>
                  <th>客户等级</th>
                  <th>交期T-</th>
                  <th>A位次</th>
                  <th>B位次</th>
                  <th>Δ位次</th>
                  <th>ΔPriority</th>
                  <th>变化来源(Explain)</th>
                </tr>
              </thead>
              <tbody>
                {compareResults.map((r) => (
                  <tr
                    key={r.contract_id}
                    className={selectedContract === r.contract_id ? "selected" : ""}
                    onClick={() => loadExplain(r.contract_id)}
                  >
                    <td className="contract-id">{r.contract_id}</td>
                    <td>
                      <span className="customer-level-badge" style={getCustomerLevelStyle(r.customer_level)}>
                        {r.customer_level === "A" ? "战略" : r.customer_level === "B" ? "重点" : "一般"}
                      </span>
                    </td>
                    <td style={getDaysStyle(r.days_to_pdd)}>{r.days_to_pdd}天</td>
                    <td>#{r.strategy1Rank}</td>
                    <td>#{r.strategy2Rank}</td>
                    <td>
                      <span className={`rank-change ${r.rankDiff > 0 ? "up" : r.rankDiff < 0 ? "down" : "same"}`}>
                        {r.rankDiff > 0 ? `↑${r.rankDiff}` : r.rankDiff < 0 ? `↓${Math.abs(r.rankDiff)}` : "-"}
                      </span>
                    </td>
                    <td>
                      <span className={r.priorityDiff > 0 ? "positive" : r.priorityDiff < 0 ? "negative" : ""}>
                        {r.priorityDiff > 0 ? "+" : ""}{r.priorityDiff.toFixed(1)}
                      </span>
                    </td>
                    <td className="change-source">{r.changeSource}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Explain Detail Panel */}
        {compared && (
          <div className="explain-panel">
            <div className="explain-header">
              <h3>选中合同 Explain</h3>
              {selectedContract && <span className="selected-id">{selectedContract}</span>}
            </div>

            {!selectedContract ? (
              <div className="explain-empty">
                <div className="empty-icon">🔍</div>
                <div>点击表格行查看详细评分拆解</div>
              </div>
            ) : loadingExplain ? (
              <div className="explain-loading">加载中...</div>
            ) : explainError ? (
              <div className="explain-error">
                <div className="error-icon">⚠️</div>
                <div className="error-title">加载失败</div>
                <div className="error-message">{explainError}</div>
              </div>
            ) : explain1 && explain2 ? (
              <div className="explain-content">
                {/* S-Score 拆解 */}
                <div className="explain-section">
                  <div className="section-title">S-Score 拆解</div>
                  <div className="formula-box">
                    <div className="formula-label">策略 A ({strategy1}):</div>
                    <div className="formula">
                      S = {explain1.s_score.toFixed(1)} =
                      S1({explain1.s_score_explain.s1_customer_level.score.toFixed(0)})×{explain1.s_score_explain.s1_customer_level.weight.toFixed(1)} +
                      S2({explain1.s_score_explain.s2_margin.score.toFixed(0)})×{explain1.s_score_explain.s2_margin.weight.toFixed(1)} +
                      S3({explain1.s_score_explain.s3_urgency.score.toFixed(0)})×{explain1.s_score_explain.s3_urgency.weight.toFixed(1)}
                    </div>
                  </div>
                  <div className="formula-box">
                    <div className="formula-label">策略 B ({strategy2}):</div>
                    <div className="formula">
                      S = {explain2.s_score.toFixed(1)} =
                      S1({explain2.s_score_explain.s1_customer_level.score.toFixed(0)})×{explain2.s_score_explain.s1_customer_level.weight.toFixed(1)} +
                      S2({explain2.s_score_explain.s2_margin.score.toFixed(0)})×{explain2.s_score_explain.s2_margin.weight.toFixed(1)} +
                      S3({explain2.s_score_explain.s3_urgency.score.toFixed(0)})×{explain2.s_score_explain.s3_urgency.weight.toFixed(1)}
                    </div>
                  </div>
                </div>

                {/* P-Score 拆解 */}
                <div className="explain-section">
                  <div className="section-title">P-Score 拆解</div>
                  <div className="formula-box">
                    <div className="formula-label">策略 A:</div>
                    <div className="formula">
                      P = {explain1.p_score.toFixed(1)} =
                      P1({explain1.p_score_explain.p1_difficulty.score.toFixed(0)})×{explain1.p_score_explain.p1_difficulty.weight.toFixed(1)} +
                      P2({explain1.p_score_explain.p2_aggregation.score.toFixed(0)})×{explain1.p_score_explain.p2_aggregation.weight.toFixed(1)} +
                      P3({explain1.p_score_explain.p3_rhythm.score.toFixed(0)})×{explain1.p_score_explain.p3_rhythm.weight.toFixed(1)}
                    </div>
                  </div>
                  <div className="formula-box">
                    <div className="formula-label">策略 B:</div>
                    <div className="formula">
                      P = {explain2.p_score.toFixed(1)} =
                      P1({explain2.p_score_explain.p1_difficulty.score.toFixed(0)})×{explain2.p_score_explain.p1_difficulty.weight.toFixed(1)} +
                      P2({explain2.p_score_explain.p2_aggregation.score.toFixed(0)})×{explain2.p_score_explain.p2_aggregation.weight.toFixed(1)} +
                      P3({explain2.p_score_explain.p3_rhythm.score.toFixed(0)})×{explain2.p_score_explain.p3_rhythm.weight.toFixed(1)}
                    </div>
                  </div>
                </div>

                {/* 最终优先级对比 */}
                <div className="explain-section priority-compare">
                  <div className="section-title">最终优先级对比</div>
                  <div className="priority-row">
                    <div className="priority-box strategy-a">
                      <div className="priority-label">A Priority ({strategy1})</div>
                      <div className="priority-value">{explain1.final_priority.toFixed(1)}</div>
                      <div className="priority-formula">
                        = S({explain1.s_score.toFixed(1)})×{explain1.ws.toFixed(1)} + P({explain1.p_score.toFixed(1)})×{explain1.wp.toFixed(1)}
                        {explain1.alpha && explain1.alpha !== 1.0 && ` × α(${explain1.alpha.toFixed(2)})`}
                      </div>
                    </div>
                    <div className="priority-box strategy-b">
                      <div className="priority-label">B Priority ({strategy2})</div>
                      <div className="priority-value">{explain2.final_priority.toFixed(1)}</div>
                      <div className="priority-formula">
                        = S({explain2.s_score.toFixed(1)})×{explain2.ws.toFixed(1)} + P({explain2.p_score.toFixed(1)})×{explain2.wp.toFixed(1)}
                        {explain2.alpha && explain2.alpha !== 1.0 && ` × α(${explain2.alpha.toFixed(2)})`}
                      </div>
                    </div>
                  </div>
                  <div className="priority-diff">
                    <span className={explain2.final_priority - explain1.final_priority > 0 ? "positive" : "negative"}>
                      Δ = {(explain2.final_priority - explain1.final_priority) > 0 ? "+" : ""}
                      {(explain2.final_priority - explain1.final_priority).toFixed(1)}
                    </span>
                  </div>
                </div>
              </div>
            ) : (
              <div className="explain-empty">
                <div className="empty-icon">📭</div>
                <div>无法加载评分详情</div>
              </div>
            )}
          </div>
        )}
      </div>

      <style>{`
        .sandbox-compare-page {
          display: flex;
          flex-direction: column;
          height: 100%;
        }

        .compare-controls {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: var(--spacing-lg);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          margin-bottom: var(--spacing-md);
        }

        .strategy-select {
          display: flex;
          align-items: center;
          gap: var(--spacing-lg);
        }

        .strategy-box {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-xs);
        }

        .strategy-box label {
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .strategy-box select {
          padding: var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          min-width: 150px;
        }

        .weight-info {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .vs-divider {
          font-size: var(--font-size-xl);
          font-weight: 700;
          color: var(--color-text-tertiary);
        }

        .control-right {
          display: flex;
          align-items: center;
          gap: var(--spacing-md);
        }

        .top-n-selector {
          display: flex;
          align-items: center;
          gap: var(--spacing-xs);
        }

        .top-n-selector label {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .top-n-selector select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        .btn-compare {
          padding: var(--spacing-sm) var(--spacing-lg);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-md);
          font-weight: 600;
          cursor: pointer;
        }

        .btn-compare:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .compare-stats {
          display: grid;
          grid-template-columns: repeat(5, 1fr);
          gap: var(--spacing-md);
          margin-bottom: var(--spacing-md);
        }

        .stat-card {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-md);
          padding: var(--spacing-md);
          text-align: center;
        }

        .stat-label {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          margin-bottom: var(--spacing-xs);
        }

        .stat-value {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .stat-value.up { color: var(--color-success); }
        .stat-value.down { color: var(--color-error); }

        .compare-main {
          display: grid;
          grid-template-columns: 1fr 380px;
          gap: var(--spacing-md);
          flex: 1;
          min-height: 0;
        }

        .compare-table-container {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          overflow: auto;
        }

        .compare-table {
          width: 100%;
          border-collapse: collapse;
        }

        .compare-table th,
        .compare-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .compare-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
          z-index: 1;
        }

        .compare-table tbody tr {
          cursor: pointer;
          transition: background var(--transition-fast);
        }

        .compare-table tbody tr:hover {
          background: var(--color-primary-light);
        }

        .compare-table tbody tr.selected {
          background: rgba(24, 144, 255, 0.15);
        }

        .contract-id {
          font-family: monospace;
          font-weight: 500;
        }

        .customer-level-badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: var(--font-size-sm);
        }

        .rank-change {
          font-weight: 600;
        }

        .rank-change.up { color: var(--color-success); }
        .rank-change.down { color: var(--color-error); }
        .rank-change.same { color: var(--color-text-tertiary); }

        .positive { color: var(--color-success); font-weight: 500; }
        .negative { color: var(--color-error); font-weight: 500; }

        .change-source {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
          font-family: monospace;
        }

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: calc(var(--spacing-xl) * 2);
          color: var(--color-text-tertiary);
        }

        .empty-state-icon {
          font-size: 48px;
          margin-bottom: var(--spacing-md);
        }

        /* Explain Panel */
        .explain-panel {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          display: flex;
          flex-direction: column;
          overflow: hidden;
        }

        .explain-header {
          padding: var(--spacing-md);
          border-bottom: 1px solid var(--color-border-light);
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .explain-header h3 {
          margin: 0;
          font-size: var(--font-size-base);
        }

        .selected-id {
          font-family: monospace;
          padding: 2px 8px;
          background: var(--color-primary-light);
          border-radius: 4px;
          font-size: var(--font-size-sm);
        }

        .explain-empty {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          color: var(--color-text-tertiary);
          padding: var(--spacing-xl);
        }

        .explain-empty .empty-icon {
          font-size: 32px;
          margin-bottom: var(--spacing-sm);
        }

        .explain-loading {
          flex: 1;
          display: flex;
          align-items: center;
          justify-content: center;
          color: var(--color-text-tertiary);
        }

        .explain-error {
          flex: 1;
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          color: var(--color-error);
          padding: var(--spacing-lg);
          text-align: center;
        }

        .explain-error .error-icon {
          font-size: 32px;
          margin-bottom: var(--spacing-sm);
        }

        .explain-error .error-title {
          font-weight: 600;
          margin-bottom: var(--spacing-xs);
        }

        .explain-error .error-message {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
          word-break: break-all;
          max-width: 100%;
        }

        .explain-content {
          flex: 1;
          overflow: auto;
          padding: var(--spacing-md);
        }

        .explain-section {
          margin-bottom: var(--spacing-md);
        }

        .section-title {
          font-weight: 600;
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
          margin-bottom: var(--spacing-xs);
          padding-bottom: var(--spacing-xs);
          border-bottom: 1px dashed var(--color-border-light);
        }

        .formula-box {
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
          padding: var(--spacing-sm);
          margin-bottom: var(--spacing-xs);
        }

        .formula-label {
          font-size: 11px;
          color: var(--color-text-tertiary);
          margin-bottom: 2px;
        }

        .formula {
          font-family: monospace;
          font-size: var(--font-size-sm);
          line-height: 1.4;
          word-break: break-all;
        }

        .priority-compare {
          background: linear-gradient(135deg, rgba(24, 144, 255, 0.05), rgba(82, 196, 26, 0.05));
          border-radius: var(--border-radius-md);
          padding: var(--spacing-md);
          margin-top: var(--spacing-md);
        }

        .priority-row {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: var(--spacing-sm);
        }

        .priority-box {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          padding: var(--spacing-sm);
          text-align: center;
        }

        .priority-box.strategy-a {
          border-left: 3px solid #1890ff;
        }

        .priority-box.strategy-b {
          border-left: 3px solid #52c41a;
        }

        .priority-label {
          font-size: 11px;
          color: var(--color-text-tertiary);
        }

        .priority-value {
          font-size: var(--font-size-xl);
          font-weight: 700;
          margin: var(--spacing-xs) 0;
        }

        .priority-formula {
          font-size: 10px;
          font-family: monospace;
          color: var(--color-text-secondary);
        }

        .priority-diff {
          text-align: center;
          margin-top: var(--spacing-sm);
          padding-top: var(--spacing-sm);
          border-top: 1px dashed var(--color-border-light);
        }

        .priority-diff span {
          font-size: var(--font-size-lg);
          font-weight: 700;
        }
      `}</style>
    </div>
  );
}
