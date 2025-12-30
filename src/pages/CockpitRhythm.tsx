import { useState, useEffect, useMemo } from "react";
import { api, ContractPriority, RhythmFlowAnalysis, StrategyWeights } from "../api/tauri";

const RHYTHM_DAYS = ["D+1", "D+2", "D+3"];
const SPEC_FAMILIES = ["常规", "特殊", "超特"];

type ViewMode = "single" | "compare";
type TabType = "heatmap" | "aggregation" | "difficulty";

interface RhythmCell {
  count: number;
  level: "low" | "medium" | "high" | "critical";
  contracts: ContractPriority[];
}

// 聚合度数据
interface AggregationItem {
  key: string;
  label: string;
  count: number;
  percentage: number;
  avgPriority: number;
}

// 难度分布数据
interface DifficultyItem {
  level: string;
  label: string;
  count: number;
  percentage: number;
  contracts: ContractPriority[];
}

export function CockpitRhythm() {
  const [strategies, setStrategies] = useState<string[]>([]);
  const [strategyWeights, setStrategyWeights] = useState<StrategyWeights[]>([]);
  const [loading, setLoading] = useState(false);

  // 视图模式
  const [viewMode, setViewMode] = useState<ViewMode>("single");
  const [activeTab, setActiveTab] = useState<TabType>("heatmap");

  // 单策略模式
  const [selectedStrategy, setSelectedStrategy] = useState("");
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [rhythmAnalysis, setRhythmAnalysis] = useState<RhythmFlowAnalysis | null>(null);

  // 对比模式
  const [strategy1, setStrategy1] = useState("");
  const [strategy2, setStrategy2] = useState("");
  const [contracts1, setContracts1] = useState<ContractPriority[]>([]);
  const [contracts2, setContracts2] = useState<ContractPriority[]>([]);

  // 选中状态
  const [selectedCell, setSelectedCell] = useState<{ family: string; day: string } | null>(null);

  useEffect(() => {
    Promise.all([api.getStrategies(), api.getStrategyWeightsList()]).then(
      ([strats, weights]) => {
        setStrategies(strats);
        setStrategyWeights(weights);
        if (strats.length > 0) {
          setSelectedStrategy(strats[0]);
          setStrategy1(strats[0]);
          setStrategy2(strats.length > 1 ? strats[1] : strats[0]);
        }
      }
    );
  }, []);

  useEffect(() => {
    if (viewMode === "single" && selectedStrategy) {
      loadSingleData();
    }
  }, [selectedStrategy, viewMode]);

  useEffect(() => {
    if (viewMode === "compare" && strategy1 && strategy2) {
      loadCompareData();
    }
  }, [strategy1, strategy2, viewMode]);

  const loadSingleData = async () => {
    setLoading(true);
    try {
      const [contractsData, rhythmData] = await Promise.all([
        api.computeAllPriorities(selectedStrategy),
        api.analyzeRhythmFlow(selectedStrategy),
      ]);
      setContracts(contractsData);
      setRhythmAnalysis(rhythmData);
    } catch (err) {
      console.error("加载数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  const loadCompareData = async () => {
    setLoading(true);
    try {
      const [data1, data2] = await Promise.all([
        api.computeAllPriorities(strategy1),
        api.computeAllPriorities(strategy2),
      ]);
      setContracts1(data1);
      setContracts2(data2);
    } catch (err) {
      console.error("加载对比数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 生成节奏热图数据
  const generateRhythmData = (contractList: ContractPriority[]) => {
    const data: Record<string, Record<string, RhythmCell>> = {};

    SPEC_FAMILIES.forEach((family) => {
      data[family] = {};
      RHYTHM_DAYS.forEach((day, dayIndex) => {
        const dayContracts = contractList.filter(
          (c) =>
            c.spec_family === family &&
            c.days_to_pdd >= dayIndex + 1 &&
            c.days_to_pdd <= dayIndex + 2
        );
        const count = dayContracts.length;
        const level: RhythmCell["level"] =
          count > 8 ? "critical" : count > 5 ? "high" : count > 2 ? "medium" : "low";

        data[family][day] = { count, level, contracts: dayContracts };
      });
    });

    return data;
  };

  // 单策略热图数据
  const rhythmData = useMemo(() => generateRhythmData(contracts), [contracts]);

  // 对比模式热图数据
  const rhythmData1 = useMemo(() => generateRhythmData(contracts1), [contracts1]);
  const rhythmData2 = useMemo(() => generateRhythmData(contracts2), [contracts2]);

  // 统计概览
  const summary = useMemo(() => {
    const data = viewMode === "single" ? rhythmData : rhythmData1;
    let totalD1 = 0, totalD2 = 0, totalD3 = 0;
    SPEC_FAMILIES.forEach((family) => {
      totalD1 += data[family]?.["D+1"]?.count || 0;
      totalD2 += data[family]?.["D+2"]?.count || 0;
      totalD3 += data[family]?.["D+3"]?.count || 0;
    });
    return { totalD1, totalD2, totalD3, total: totalD1 + totalD2 + totalD3 };
  }, [rhythmData, rhythmData1, viewMode]);

  // 聚合度分析（按规格族统计）
  const aggregationBySpecFamily = useMemo(() => {
    const contractList = viewMode === "single" ? contracts : contracts1;
    const total = contractList.length;
    if (total === 0) return [];

    const groups: Record<string, ContractPriority[]> = {};
    contractList.forEach((c) => {
      const key = c.spec_family || "未知";
      if (!groups[key]) groups[key] = [];
      groups[key].push(c);
    });

    return Object.entries(groups)
      .map(([key, list]) => ({
        key,
        label: key,
        count: list.length,
        percentage: (list.length / total) * 100,
        avgPriority: list.reduce((sum, c) => sum + c.priority, 0) / list.length,
      }))
      .sort((a, b) => b.count - a.count);
  }, [contracts, contracts1, viewMode]);

  // 聚合度分析（按钢种统计）
  const aggregationBySteelGrade = useMemo(() => {
    const contractList = viewMode === "single" ? contracts : contracts1;
    const total = contractList.length;
    if (total === 0) return [];

    const groups: Record<string, ContractPriority[]> = {};
    contractList.forEach((c) => {
      const key = c.steel_grade || "未知";
      if (!groups[key]) groups[key] = [];
      groups[key].push(c);
    });

    return Object.entries(groups)
      .map(([key, list]) => ({
        key,
        label: key,
        count: list.length,
        percentage: (list.length / total) * 100,
        avgPriority: list.reduce((sum, c) => sum + c.priority, 0) / list.length,
      }))
      .sort((a, b) => b.count - a.count)
      .slice(0, 10); // Top 10
  }, [contracts, contracts1, viewMode]);

  // 难度分布（基于 P-Score 分段）
  const difficultyDistribution = useMemo(() => {
    const contractList = viewMode === "single" ? contracts : contracts1;
    const total = contractList.length;
    if (total === 0) return [];

    const levels = [
      { level: "high", label: "高难度", min: 70, max: 100 },
      { level: "medium", label: "中难度", min: 40, max: 70 },
      { level: "low", label: "低难度", min: 0, max: 40 },
    ];

    return levels.map(({ level, label, min, max }) => {
      const list = contractList.filter((c) => c.p_score >= min && c.p_score < max);
      return {
        level,
        label,
        count: list.length,
        percentage: total > 0 ? (list.length / total) * 100 : 0,
        contracts: list,
      };
    });
  }, [contracts, contracts1, viewMode]);

  // 选中的合同列表
  const selectedContracts = useMemo(() => {
    if (!selectedCell) return [];
    const data = viewMode === "single" ? rhythmData : rhythmData1;
    return data[selectedCell.family]?.[selectedCell.day]?.contracts || [];
  }, [selectedCell, rhythmData, rhythmData1, viewMode]);

  // 获取权重信息
  const getWeightInfo = (strategyName: string) => {
    const w = strategyWeights.find((sw) => sw.strategy_name === strategyName);
    return w ? `(${w.ws.toFixed(1)}/${w.wp.toFixed(1)})` : "";
  };

  // 渲染热图
  const renderHeatmap = (
    data: Record<string, Record<string, RhythmCell>>,
    strategyName: string,
    isCompare = false
  ) => (
    <div className="heatmap-container">
      {isCompare && <div className="heatmap-title">{strategyName} {getWeightInfo(strategyName)}</div>}
      <div className="heatmap-grid">
        <div className="heatmap-header"></div>
        {RHYTHM_DAYS.map((day) => (
          <div key={day} className="heatmap-header">{day}</div>
        ))}

        {SPEC_FAMILIES.map((family) => (
          <>
            <div key={`label-${family}`} className="heatmap-row-label">{family}</div>
            {RHYTHM_DAYS.map((day) => {
              const cell = data[family]?.[day];
              const isSelected = selectedCell?.family === family && selectedCell?.day === day;
              return (
                <div
                  key={`${family}-${day}`}
                  className={`heatmap-cell ${cell?.level || "low"} ${isSelected ? "selected" : ""}`}
                  onClick={() => setSelectedCell({ family, day })}
                  title={`${family} ${day}: ${cell?.count || 0} 份合同`}
                >
                  {cell?.count || 0}
                </div>
              );
            })}
          </>
        ))}
      </div>
    </div>
  );

  // 渲染聚合度分布
  const renderAggregationBar = (item: AggregationItem, maxCount: number) => (
    <div key={item.key} className="aggregation-item">
      <div className="aggregation-label">{item.label}</div>
      <div className="aggregation-bar-container">
        <div
          className="aggregation-bar"
          style={{ width: `${(item.count / maxCount) * 100}%` }}
        />
        <span className="aggregation-count">{item.count}</span>
      </div>
      <div className="aggregation-percentage">{item.percentage.toFixed(1)}%</div>
      <div className="aggregation-priority">P={item.avgPriority.toFixed(1)}</div>
    </div>
  );

  // 渲染难度分布
  const renderDifficultyBar = (item: DifficultyItem) => (
    <div key={item.level} className={`difficulty-item ${item.level}`}>
      <div className="difficulty-label">{item.label}</div>
      <div className="difficulty-bar-container">
        <div
          className="difficulty-bar"
          style={{ width: `${item.percentage}%` }}
        />
      </div>
      <div className="difficulty-stats">
        <span className="difficulty-count">{item.count}</span>
        <span className="difficulty-percentage">{item.percentage.toFixed(1)}%</span>
      </div>
    </div>
  );

  return (
    <div className="cockpit-rhythm-page">
      <div className="page-header">
        <div>
          <h1 className="page-header__title">节拍与顺行分析</h1>
          <p className="page-header__subtitle">
            {rhythmAnalysis
              ? `${rhythmAnalysis.cycle_days}日节拍 · 整体匹配率 ${(rhythmAnalysis.overall_match_rate * 100).toFixed(0)}%`
              : "按规格族查看近3日合同分布与聚合度"}
          </p>
        </div>
        <div className="header-controls">
          {/* 视图模式切换 */}
          <div className="view-mode-toggle">
            <button
              className={viewMode === "single" ? "active" : ""}
              onClick={() => setViewMode("single")}
            >
              单策略
            </button>
            <button
              className={viewMode === "compare" ? "active" : ""}
              onClick={() => setViewMode("compare")}
            >
              对比
            </button>
          </div>

          {viewMode === "single" ? (
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
          ) : (
            <>
              <select
                value={strategy1}
                onChange={(e) => setStrategy1(e.target.value)}
                disabled={loading}
                className="strategy-select"
              >
                {strategies.map((s) => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
              <span className="vs-label">VS</span>
              <select
                value={strategy2}
                onChange={(e) => setStrategy2(e.target.value)}
                disabled={loading}
                className="strategy-select"
              >
                {strategies.map((s) => (
                  <option key={s} value={s}>{s}</option>
                ))}
              </select>
            </>
          )}

          <button
            className="btn-refresh"
            onClick={viewMode === "single" ? loadSingleData : loadCompareData}
            disabled={loading}
          >
            {loading ? "加载中..." : "刷新"}
          </button>
        </div>
      </div>

      {/* 统计概览 */}
      <div className="rhythm-summary">
        <div className="summary-card">
          <div className="summary-value">{summary.total}</div>
          <div className="summary-label">3日内合同总数</div>
        </div>
        <div className="summary-card d1">
          <div className="summary-value">{summary.totalD1}</div>
          <div className="summary-label">D+1 (明天)</div>
        </div>
        <div className="summary-card d2">
          <div className="summary-value">{summary.totalD2}</div>
          <div className="summary-label">D+2 (后天)</div>
        </div>
        <div className="summary-card d3">
          <div className="summary-value">{summary.totalD3}</div>
          <div className="summary-label">D+3 (大后天)</div>
        </div>
        {rhythmAnalysis && (
          <div className={`summary-card ${rhythmAnalysis.overall_status === "smooth" ? "good" : rhythmAnalysis.overall_status === "partial" ? "warning" : "danger"}`}>
            <div className="summary-value">
              {rhythmAnalysis.overall_status === "smooth" ? "顺行" : rhythmAnalysis.overall_status === "partial" ? "部分" : "拥堵"}
            </div>
            <div className="summary-label">节拍状态</div>
          </div>
        )}
      </div>

      {/* Tab 导航 */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === "heatmap" ? "active" : ""}`}
          onClick={() => setActiveTab("heatmap")}
        >
          节奏热图
        </button>
        <button
          className={`tab-btn ${activeTab === "aggregation" ? "active" : ""}`}
          onClick={() => setActiveTab("aggregation")}
        >
          聚合度分布
        </button>
        <button
          className={`tab-btn ${activeTab === "difficulty" ? "active" : ""}`}
          onClick={() => setActiveTab("difficulty")}
        >
          难度分布
        </button>
      </div>

      {/* 主内容区域 */}
      <div className="rhythm-content">
        {/* 节奏热图 Tab */}
        {activeTab === "heatmap" && (
          <>
            <div className="rhythm-heatmap-panel">
              <div className="panel-header">
                <h3>节奏热图</h3>
                <div className="legend">
                  <div className="legend-item"><span className="dot critical"></span>紧急 (&gt;8)</div>
                  <div className="legend-item"><span className="dot high"></span>高 (6-8)</div>
                  <div className="legend-item"><span className="dot medium"></span>中 (3-5)</div>
                  <div className="legend-item"><span className="dot low"></span>低 (&lt;3)</div>
                </div>
              </div>

              {viewMode === "single" ? (
                renderHeatmap(rhythmData, selectedStrategy)
              ) : (
                <div className="compare-heatmaps">
                  {renderHeatmap(rhythmData1, strategy1, true)}
                  {renderHeatmap(rhythmData2, strategy2, true)}
                </div>
              )}
            </div>

            {/* 详情面板 */}
            <div className="rhythm-detail-panel">
              <div className="panel-header">
                <h3>
                  {selectedCell
                    ? `${selectedCell.family} - ${selectedCell.day} 合同列表`
                    : "点击热图单元格查看详情"}
                </h3>
              </div>

              {!selectedCell ? (
                <div className="empty-state">
                  <div className="empty-state-icon">📅</div>
                  <div className="empty-state-text">点击左侧热图单元格查看该规格族的合同详情</div>
                </div>
              ) : selectedContracts.length === 0 ? (
                <div className="empty-state">
                  <div className="empty-state-icon">📭</div>
                  <div className="empty-state-text">该单元格无合同</div>
                </div>
              ) : (
                <div className="detail-table-container">
                  <table className="detail-table">
                    <thead>
                      <tr>
                        <th>合同编号</th>
                        <th>客户</th>
                        <th>钢种</th>
                        <th>到期天数</th>
                        <th>优先级</th>
                      </tr>
                    </thead>
                    <tbody>
                      {selectedContracts.map((c) => (
                        <tr key={c.contract_id}>
                          <td className="contract-id">{c.contract_id}</td>
                          <td>{c.customer_id}</td>
                          <td>{c.steel_grade}</td>
                          <td className={c.days_to_pdd <= 1 ? "urgent" : ""}>
                            {c.days_to_pdd}天
                          </td>
                          <td>{c.priority.toFixed(2)}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </div>
          </>
        )}

        {/* 聚合度分布 Tab */}
        {activeTab === "aggregation" && (
          <>
            <div className="aggregation-panel">
              <div className="panel-header">
                <h3>规格族聚合度</h3>
              </div>
              <div className="aggregation-content">
                {aggregationBySpecFamily.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-state-icon">📊</div>
                    <div className="empty-state-text">暂无数据</div>
                  </div>
                ) : (
                  aggregationBySpecFamily.map((item) =>
                    renderAggregationBar(item, aggregationBySpecFamily[0].count)
                  )
                )}
              </div>
            </div>

            <div className="aggregation-panel">
              <div className="panel-header">
                <h3>钢种聚合度 (Top 10)</h3>
              </div>
              <div className="aggregation-content">
                {aggregationBySteelGrade.length === 0 ? (
                  <div className="empty-state">
                    <div className="empty-state-icon">📊</div>
                    <div className="empty-state-text">暂无数据</div>
                  </div>
                ) : (
                  aggregationBySteelGrade.map((item) =>
                    renderAggregationBar(item, aggregationBySteelGrade[0].count)
                  )
                )}
              </div>
            </div>
          </>
        )}

        {/* 难度分布 Tab */}
        {activeTab === "difficulty" && (
          <>
            <div className="difficulty-panel">
              <div className="panel-header">
                <h3>工艺难度分布</h3>
                <span className="panel-subtitle">基于 P-Score 分段统计</span>
              </div>
              <div className="difficulty-content">
                {difficultyDistribution.map(renderDifficultyBar)}
              </div>

              <div className="difficulty-explanation">
                <div className="explanation-item">
                  <span className="explanation-dot high"></span>
                  <span>高难度 (P≥70): 特殊工艺、窄规格、高精度要求</span>
                </div>
                <div className="explanation-item">
                  <span className="explanation-dot medium"></span>
                  <span>中难度 (40≤P&lt;70): 常规工艺、标准规格</span>
                </div>
                <div className="explanation-item">
                  <span className="explanation-dot low"></span>
                  <span>低难度 (P&lt;40): 简单工艺、宽规格</span>
                </div>
              </div>
            </div>

            <div className="difficulty-detail-panel">
              <div className="panel-header">
                <h3>难度分布明细</h3>
              </div>
              <div className="difficulty-table-container">
                <table className="detail-table">
                  <thead>
                    <tr>
                      <th>难度等级</th>
                      <th>合同数</th>
                      <th>占比</th>
                      <th>主要规格族</th>
                    </tr>
                  </thead>
                  <tbody>
                    {difficultyDistribution.map((item) => {
                      // 统计该难度等级的规格族分布
                      const specFamilyCount: Record<string, number> = {};
                      item.contracts.forEach((c) => {
                        const sf = c.spec_family || "未知";
                        specFamilyCount[sf] = (specFamilyCount[sf] || 0) + 1;
                      });
                      const topSpecFamilies = Object.entries(specFamilyCount)
                        .sort((a, b) => b[1] - a[1])
                        .slice(0, 3)
                        .map(([name, count]) => `${name}(${count})`)
                        .join(", ");

                      return (
                        <tr key={item.level}>
                          <td>
                            <span className={`difficulty-badge ${item.level}`}>
                              {item.label}
                            </span>
                          </td>
                          <td>{item.count}</td>
                          <td>{item.percentage.toFixed(1)}%</td>
                          <td>{topSpecFamilies || "-"}</td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>
            </div>
          </>
        )}
      </div>

      <style>{`
        .cockpit-rhythm-page {
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

        .view-mode-toggle {
          display: flex;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          overflow: hidden;
          border: 1px solid var(--color-border);
        }

        .view-mode-toggle button {
          padding: var(--spacing-xs) var(--spacing-md);
          border: none;
          background: transparent;
          cursor: pointer;
          font-size: var(--font-size-sm);
        }

        .view-mode-toggle button.active {
          background: var(--color-primary);
          color: #fff;
        }

        .vs-label {
          font-weight: 600;
          color: var(--color-text-tertiary);
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
        .rhythm-summary {
          display: grid;
          grid-template-columns: repeat(5, 1fr);
          gap: var(--spacing-md);
        }

        .summary-card {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          text-align: center;
        }

        .summary-card.d1 { border-left: 4px solid #ff4d4f; }
        .summary-card.d2 { border-left: 4px solid #faad14; }
        .summary-card.d3 { border-left: 4px solid #52c41a; }
        .summary-card.good { border-left: 4px solid #52c41a; }
        .summary-card.warning { border-left: 4px solid #faad14; }
        .summary-card.danger { border-left: 4px solid #ff4d4f; }

        .summary-value {
          font-size: var(--font-size-xl);
          font-weight: 700;
        }

        .summary-label {
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

        /* 主内容 */
        .rhythm-content {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: var(--spacing-md);
          flex: 1;
          min-height: 0;
        }

        .rhythm-heatmap-panel, .rhythm-detail-panel,
        .aggregation-panel, .difficulty-panel, .difficulty-detail-panel {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          display: flex;
          flex-direction: column;
          overflow: hidden;
        }

        .panel-header {
          padding: var(--spacing-md);
          border-bottom: 1px solid var(--color-border-light);
          display: flex;
          justify-content: space-between;
          align-items: center;
        }

        .panel-header h3 {
          margin: 0;
          font-size: var(--font-size-base);
        }

        .panel-subtitle {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .legend {
          display: flex;
          gap: var(--spacing-md);
          font-size: var(--font-size-sm);
        }

        .legend-item {
          display: flex;
          align-items: center;
          gap: 4px;
        }

        .dot {
          width: 12px;
          height: 12px;
          border-radius: 2px;
        }

        .dot.critical { background: #ff4d4f; }
        .dot.high { background: #fa8c16; }
        .dot.medium { background: #fadb14; }
        .dot.low { background: #52c41a; }

        /* 对比热图 */
        .compare-heatmaps {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: var(--spacing-md);
          padding: var(--spacing-md);
        }

        .heatmap-container {
          border: 1px solid var(--color-border-light);
          border-radius: var(--border-radius-sm);
          padding: var(--spacing-sm);
        }

        .heatmap-title {
          text-align: center;
          font-weight: 600;
          margin-bottom: var(--spacing-sm);
          padding-bottom: var(--spacing-sm);
          border-bottom: 1px dashed var(--color-border-light);
        }

        .heatmap-grid {
          display: grid;
          grid-template-columns: 60px repeat(3, 1fr);
          gap: 4px;
          padding: var(--spacing-sm);
        }

        .heatmap-header {
          padding: var(--spacing-xs);
          text-align: center;
          font-weight: 600;
          font-size: var(--font-size-sm);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
        }

        .heatmap-row-label {
          padding: var(--spacing-xs);
          font-weight: 600;
          font-size: var(--font-size-sm);
          display: flex;
          align-items: center;
        }

        .heatmap-cell {
          padding: var(--spacing-sm);
          text-align: center;
          border-radius: var(--border-radius-sm);
          font-weight: 600;
          cursor: pointer;
          transition: all var(--transition-fast);
        }

        .heatmap-cell:hover {
          transform: scale(1.05);
        }

        .heatmap-cell.selected {
          box-shadow: 0 0 0 3px var(--color-primary);
        }

        .heatmap-cell.low { background: rgba(82, 196, 26, 0.2); color: #389e0d; }
        .heatmap-cell.medium { background: rgba(250, 219, 20, 0.3); color: #d48806; }
        .heatmap-cell.high { background: rgba(250, 140, 22, 0.3); color: #d46b08; }
        .heatmap-cell.critical { background: rgba(255, 77, 79, 0.3); color: #cf1322; }

        /* 详情表格 */
        .detail-table-container, .difficulty-table-container {
          flex: 1;
          overflow: auto;
        }

        .detail-table {
          width: 100%;
          border-collapse: collapse;
        }

        .detail-table th,
        .detail-table td {
          padding: var(--spacing-sm);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .detail-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
        }

        .contract-id {
          font-family: monospace;
          font-weight: 500;
        }

        .urgent {
          color: var(--color-error);
          font-weight: 600;
        }

        /* 聚合度分布 */
        .aggregation-content {
          padding: var(--spacing-md);
          flex: 1;
          overflow: auto;
        }

        .aggregation-item {
          display: grid;
          grid-template-columns: 80px 1fr 60px 80px;
          gap: var(--spacing-sm);
          align-items: center;
          margin-bottom: var(--spacing-sm);
        }

        .aggregation-label {
          font-weight: 500;
          font-size: var(--font-size-sm);
        }

        .aggregation-bar-container {
          position: relative;
          height: 24px;
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
        }

        .aggregation-bar {
          height: 100%;
          background: var(--color-primary);
          border-radius: var(--border-radius-sm);
          transition: width var(--transition-normal);
        }

        .aggregation-count {
          position: absolute;
          right: 8px;
          top: 50%;
          transform: translateY(-50%);
          font-size: var(--font-size-sm);
          font-weight: 600;
        }

        .aggregation-percentage {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
          text-align: right;
        }

        .aggregation-priority {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          font-family: monospace;
        }

        /* 难度分布 */
        .difficulty-content {
          padding: var(--spacing-md);
        }

        .difficulty-item {
          display: grid;
          grid-template-columns: 80px 1fr 120px;
          gap: var(--spacing-md);
          align-items: center;
          margin-bottom: var(--spacing-md);
        }

        .difficulty-label {
          font-weight: 600;
        }

        .difficulty-item.high .difficulty-label { color: #cf1322; }
        .difficulty-item.medium .difficulty-label { color: #d46b08; }
        .difficulty-item.low .difficulty-label { color: #389e0d; }

        .difficulty-bar-container {
          height: 32px;
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
        }

        .difficulty-bar {
          height: 100%;
          border-radius: var(--border-radius-sm);
          transition: width var(--transition-normal);
        }

        .difficulty-item.high .difficulty-bar { background: #ff4d4f; }
        .difficulty-item.medium .difficulty-bar { background: #faad14; }
        .difficulty-item.low .difficulty-bar { background: #52c41a; }

        .difficulty-stats {
          display: flex;
          gap: var(--spacing-sm);
          font-size: var(--font-size-sm);
        }

        .difficulty-count {
          font-weight: 700;
        }

        .difficulty-percentage {
          color: var(--color-text-secondary);
        }

        .difficulty-explanation {
          padding: var(--spacing-md);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
          margin: var(--spacing-md);
        }

        .explanation-item {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
          margin-bottom: var(--spacing-xs);
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .explanation-dot {
          width: 10px;
          height: 10px;
          border-radius: 2px;
        }

        .explanation-dot.high { background: #ff4d4f; }
        .explanation-dot.medium { background: #faad14; }
        .explanation-dot.low { background: #52c41a; }

        .difficulty-badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: var(--font-size-sm);
          font-weight: 600;
        }

        .difficulty-badge.high { background: rgba(255, 77, 79, 0.15); color: #cf1322; }
        .difficulty-badge.medium { background: rgba(250, 173, 20, 0.15); color: #d46b08; }
        .difficulty-badge.low { background: rgba(82, 196, 26, 0.15); color: #389e0d; }

        /* 空状态 */
        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          flex: 1;
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
