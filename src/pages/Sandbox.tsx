import { useState, useEffect, useMemo } from "react";
import { api, ContractPriority, StrategyScoringWeights } from "../api/tauri";
import { SandboxCharts } from "../components/SandboxCharts";
import "./Sandbox.css";

interface SimulatedContract extends ContractPriority {
  originalRank: number;
  newPriority: number;
  newRank: number;
  rankChange: number;
  priorityChange: number;
}

type ViewMode = "table" | "chart";

export function Sandbox() {
  // 基础数据
  const [strategies, setStrategies] = useState<string[]>([]);
  const [, setStrategyWeights] = useState<StrategyScoringWeights[]>([]);
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [loading, setLoading] = useState(false);
  const [simulating, setSimulating] = useState(false);

  // 参数设置
  const [baseStrategy, setBaseStrategy] = useState<string>("");
  const [ws, setWs] = useState<number>(0.6); // S-Score 权重
  const [wp, setWp] = useState<number>(0.4); // P-Score 权重

  // 筛选条件
  const [filterCustomer, setFilterCustomer] = useState<string>("all");
  const [filterSpecFamily, setFilterSpecFamily] = useState<string>("all");
  const [filterUrgent, setFilterUrgent] = useState<boolean>(false);

  // 模拟结果
  const [simulatedContracts, setSimulatedContracts] = useState<SimulatedContract[]>([]);
  const [hasSimulated, setHasSimulated] = useState(false);

  // 视图模式
  const [viewMode, setViewMode] = useState<ViewMode>("table");

  // 加载策略和权重
  useEffect(() => {
    Promise.all([api.getStrategies(), api.getAllStrategyWeights()])
      .then(([strats, weights]) => {
        setStrategies(strats);
        setStrategyWeights(weights);
        if (strats.length > 0) {
          setBaseStrategy(strats[0]);
          // 设置默认权重
          const defaultWeight = weights.find(w => w.strategy_name === strats[0]);
          if (defaultWeight) {
            // 这里简化：使用 ws=0.6, wp=0.4 作为默认
            setWs(0.6);
            setWp(0.4);
          }
        }
      })
      .catch(console.error);
  }, []);

  // 加载合同数据
  useEffect(() => {
    if (baseStrategy) {
      loadContracts();
    }
  }, [baseStrategy]);

  const loadContracts = async () => {
    setLoading(true);
    try {
      const data = await api.computeAllPriorities(baseStrategy);
      setContracts(data);
      setHasSimulated(false);
      setSimulatedContracts([]);
    } catch (err) {
      console.error("加载合同失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 获取唯一的客户和规格族列表
  const uniqueCustomers = useMemo(() =>
    Array.from(new Set(contracts.map(c => c.customer_id))).sort(),
    [contracts]
  );

  const uniqueSpecFamilies = useMemo(() =>
    Array.from(new Set(contracts.map(c => c.spec_family))).sort(),
    [contracts]
  );

  // 筛选后的合同
  const filteredContracts = useMemo(() => {
    return contracts.filter(c => {
      if (filterCustomer !== "all" && c.customer_id !== filterCustomer) return false;
      if (filterSpecFamily !== "all" && c.spec_family !== filterSpecFamily) return false;
      if (filterUrgent && c.days_to_pdd > 7) return false;
      return true;
    });
  }, [contracts, filterCustomer, filterSpecFamily, filterUrgent]);

  // 执行模拟计算
  const runSimulation = () => {
    setSimulating(true);

    // 使用 setTimeout 模拟异步计算，让 UI 有机会更新
    setTimeout(() => {
      // 计算原始排名（基于 priority）
      const sortedOriginal = [...filteredContracts].sort((a, b) => b.priority - a.priority);
      const originalRankMap = new Map<string, number>();
      sortedOriginal.forEach((c, i) => originalRankMap.set(c.contract_id, i + 1));

      // 计算新优先级
      const withNewPriority = filteredContracts.map(c => ({
        ...c,
        originalRank: originalRankMap.get(c.contract_id) || 0,
        newPriority: ws * c.s_score + wp * c.p_score,
        newRank: 0,
        rankChange: 0,
        priorityChange: 0,
      }));

      // 按新优先级排序并计算新排名
      withNewPriority.sort((a, b) => b.newPriority - a.newPriority);
      withNewPriority.forEach((c, i) => {
        c.newRank = i + 1;
        c.rankChange = c.originalRank - c.newRank; // 正数表示排名上升
        c.priorityChange = c.newPriority - c.priority;
      });

      setSimulatedContracts(withNewPriority);
      setHasSimulated(true);
      setSimulating(false);
    }, 300);
  };

  // 重置参数
  const resetParams = () => {
    setWs(0.6);
    setWp(0.4);
    setFilterCustomer("all");
    setFilterSpecFamily("all");
    setFilterUrgent(false);
    setHasSimulated(false);
    setSimulatedContracts([]);
  };

  // 统计数据
  const stats = useMemo(() => {
    if (!hasSimulated || simulatedContracts.length === 0) {
      return {
        totalContracts: filteredContracts.length,
        rankUpCount: 0,
        rankDownCount: 0,
        avgPriorityChange: 0,
      };
    }

    const rankUp = simulatedContracts.filter(c => c.rankChange > 0).length;
    const rankDown = simulatedContracts.filter(c => c.rankChange < 0).length;
    const avgChange = simulatedContracts.reduce((sum, c) => sum + c.priorityChange, 0) / simulatedContracts.length;

    return {
      totalContracts: simulatedContracts.length,
      rankUpCount: rankUp,
      rankDownCount: rankDown,
      avgPriorityChange: avgChange,
    };
  }, [simulatedContracts, hasSimulated, filteredContracts]);

  // 权重和校验
  const weightSum = ws + wp;
  const isWeightValid = Math.abs(weightSum - 1.0) < 0.01;

  return (
    <div className="sandbox-page">
      {/* 左侧参数面板 */}
      <div className="sandbox-params">
        {/* 策略与权重 */}
        <div className="param-card">
          <div className="param-card-header">
            <h3>基准策略</h3>
          </div>
          <div className="param-card-body">
            <div className="strategy-selector">
              <label>选择基准策略</label>
              <select
                value={baseStrategy}
                onChange={e => setBaseStrategy(e.target.value)}
                disabled={loading}
              >
                {strategies.map(s => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
            </div>
          </div>
        </div>

        {/* 权重调节 */}
        <div className="param-card">
          <div className="param-card-header">
            <h3>权重调节</h3>
          </div>
          <div className="param-card-body">
            <div className="weight-slider">
              <div className="weight-slider-header">
                <span className="weight-slider-label">S-Score 权重 (ws)</span>
                <span className="weight-slider-value">{ws.toFixed(2)}</span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={ws}
                onChange={e => setWs(parseFloat(e.target.value))}
              />
            </div>

            <div className="weight-slider">
              <div className="weight-slider-header">
                <span className="weight-slider-label">P-Score 权重 (wp)</span>
                <span className="weight-slider-value">{wp.toFixed(2)}</span>
              </div>
              <input
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={wp}
                onChange={e => setWp(parseFloat(e.target.value))}
              />
            </div>

            <div className="weight-sum">
              <span>权重和</span>
              <span className={isWeightValid ? "weight-sum-valid" : "weight-sum-invalid"}>
                {weightSum.toFixed(2)} {isWeightValid ? "✓" : "✗"}
              </span>
            </div>
          </div>
        </div>

        {/* 合同筛选 */}
        <div className="param-card">
          <div className="param-card-header">
            <h3>合同筛选</h3>
          </div>
          <div className="param-card-body">
            <div className="filter-group">
              <label>客户</label>
              <select
                value={filterCustomer}
                onChange={e => setFilterCustomer(e.target.value)}
              >
                <option value="all">全部客户</option>
                {uniqueCustomers.map(c => (
                  <option key={c} value={c}>{c}</option>
                ))}
              </select>
            </div>

            <div className="filter-group">
              <label>规格族</label>
              <select
                value={filterSpecFamily}
                onChange={e => setFilterSpecFamily(e.target.value)}
              >
                <option value="all">全部规格族</option>
                {uniqueSpecFamilies.map(s => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
            </div>

            <label className="filter-checkbox">
              <input
                type="checkbox"
                checked={filterUrgent}
                onChange={e => setFilterUrgent(e.target.checked)}
              />
              <span>仅显示紧急合同 (≤7天)</span>
            </label>

            <div className="filter-count">
              筛选后: {filteredContracts.length} / {contracts.length} 条
            </div>
          </div>
        </div>

        {/* 操作按钮 */}
        <div className="param-card">
          <div className="param-actions">
            <button
              className="btn-simulate"
              onClick={runSimulation}
              disabled={loading || simulating || filteredContracts.length === 0}
            >
              {simulating ? "模拟计算中..." : "运行模拟"}
            </button>
            <button className="btn-reset" onClick={resetParams}>
              重置参数
            </button>
          </div>
        </div>
      </div>

      {/* 右侧结果区域 */}
      <div className="sandbox-results">
        {/* 统计卡片 */}
        <div className="stats-cards">
          <div className="stat-card">
            <div className="stat-card-label">模拟合同数</div>
            <div className="stat-card-value">{stats.totalContracts}</div>
          </div>
          <div className="stat-card">
            <div className="stat-card-label">排名上升</div>
            <div className="stat-card-value">{stats.rankUpCount}</div>
            {hasSimulated && (
              <div className="stat-card-change positive">
                {stats.totalContracts > 0
                  ? `${((stats.rankUpCount / stats.totalContracts) * 100).toFixed(0)}%`
                  : "-"}
              </div>
            )}
          </div>
          <div className="stat-card">
            <div className="stat-card-label">排名下降</div>
            <div className="stat-card-value">{stats.rankDownCount}</div>
            {hasSimulated && (
              <div className="stat-card-change negative">
                {stats.totalContracts > 0
                  ? `${((stats.rankDownCount / stats.totalContracts) * 100).toFixed(0)}%`
                  : "-"}
              </div>
            )}
          </div>
          <div className="stat-card">
            <div className="stat-card-label">平均优先级变化</div>
            <div className="stat-card-value">
              {hasSimulated ? stats.avgPriorityChange.toFixed(2) : "-"}
            </div>
            {hasSimulated && (
              <div className={`stat-card-change ${stats.avgPriorityChange > 0 ? "positive" : stats.avgPriorityChange < 0 ? "negative" : "neutral"}`}>
                {stats.avgPriorityChange > 0 ? "↑" : stats.avgPriorityChange < 0 ? "↓" : "→"}
              </div>
            )}
          </div>
        </div>

        {/* 对比视图 */}
        <div className="comparison-section">
          <div className="comparison-header">
            <h3>模拟结果对比</h3>
            <div className="comparison-tabs">
              <button
                className={`comparison-tab ${viewMode === "table" ? "active" : ""}`}
                onClick={() => setViewMode("table")}
              >
                列表视图
              </button>
              <button
                className={`comparison-tab ${viewMode === "chart" ? "active" : ""}`}
                onClick={() => setViewMode("chart")}
              >
                图表视图
              </button>
            </div>
          </div>

          <div className="comparison-body">
            {!hasSimulated ? (
              <div className="empty-state">
                <div className="empty-state-icon">🎯</div>
                <div className="empty-state-text">调整参数后点击"运行模拟"</div>
                <div className="empty-state-hint">
                  系统将计算新权重下的优先级排名变化
                </div>
              </div>
            ) : viewMode === "table" ? (
              <table className="comparison-table">
                <thead>
                  <tr>
                    <th>原排名</th>
                    <th>新排名</th>
                    <th>变化</th>
                    <th>合同编号</th>
                    <th>客户</th>
                    <th>规格族</th>
                    <th>S-Score</th>
                    <th>P-Score</th>
                    <th>原优先级</th>
                    <th>新优先级</th>
                    <th>优先级变化</th>
                  </tr>
                </thead>
                <tbody>
                  {simulatedContracts.map(c => (
                    <tr key={c.contract_id}>
                      <td>{c.originalRank}</td>
                      <td>{c.newRank}</td>
                      <td>
                        <span className={`rank-change ${c.rankChange > 0 ? "up" : c.rankChange < 0 ? "down" : "same"}`}>
                          {c.rankChange > 0 ? `↑${c.rankChange}` : c.rankChange < 0 ? `↓${Math.abs(c.rankChange)}` : "-"}
                        </span>
                      </td>
                      <td>{c.contract_id}</td>
                      <td>{c.customer_id}</td>
                      <td>{c.spec_family}</td>
                      <td>{c.s_score.toFixed(1)}</td>
                      <td>{c.p_score.toFixed(1)}</td>
                      <td>{c.priority.toFixed(2)}</td>
                      <td style={{ color: "var(--color-primary)", fontWeight: 600 }}>
                        {c.newPriority.toFixed(2)}
                      </td>
                      <td>
                        <span className={`priority-diff ${c.priorityChange > 0 ? "positive" : c.priorityChange < 0 ? "negative" : ""}`}>
                          {c.priorityChange > 0 ? "+" : ""}{c.priorityChange.toFixed(2)}
                        </span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            ) : (
              <SandboxCharts data={simulatedContracts} ws={ws} wp={wp} />
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
