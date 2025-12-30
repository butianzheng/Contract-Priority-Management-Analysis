import { useState, useEffect } from "react";
import {
  api,
  ConsensusPackage,
  MeetingType,
  CsvFileInfo,
} from "../api/tauri";
import { open } from "@tauri-apps/api/dialog";
import { desktopDir } from "@tauri-apps/api/path";

type TabType = "overview" | "conclusions" | "attachments" | "export";

export function CockpitConsensus() {
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState("");
  const [loading, setLoading] = useState(false);
  const [exporting, setExporting] = useState(false);

  // 会议配置
  const [meetingType, setMeetingType] = useState<MeetingType>("production_sales");
  const [meetingDate, setMeetingDate] = useState(
    new Date().toISOString().split("T")[0]
  );

  // 数据状态
  const [consensusPackage, setConsensusPackage] = useState<ConsensusPackage | null>(null);
  const [exportedFiles, setExportedFiles] = useState<CsvFileInfo[]>([]);

  // UI 状态
  const [activeTab, setActiveTab] = useState<TabType>("overview");
  const [editMode, setEditMode] = useState(false);
  const [customConclusions, setCustomConclusions] = useState("");
  const [exportMessage, setExportMessage] = useState<{ type: "success" | "error"; text: string } | null>(null);

  useEffect(() => {
    api.getStrategies().then((strats) => {
      setStrategies(strats);
      if (strats.length > 0) {
        setSelectedStrategy(strats[0]);
      }
    });
  }, []);

  const generatePackage = async () => {
    if (!selectedStrategy) return;
    setLoading(true);
    try {
      const pkg = await api.generateConsensusPackage(
        selectedStrategy,
        meetingType,
        meetingDate,
        "系统用户"
      );
      setConsensusPackage(pkg);
      // 生成默认结论
      setCustomConclusions(generateDefaultConclusions(pkg));
    } catch (err) {
      console.error("生成共识包失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 生成默认会议结论文本
  const generateDefaultConclusions = (pkg: ConsensusPackage): string => {
    const lines: string[] = [];
    const typeLabel = pkg.meeting_type === "production_sales" ? "产销会" : "经营会";

    lines.push(`【${typeLabel}共识】`);
    lines.push(`会议日期：${pkg.meeting_date}`);
    lines.push(`策略方案：${pkg.strategy_name}`);
    lines.push("");

    // KPI 概要
    lines.push("一、核心指标");
    const allKpis = [
      ...pkg.kpi_summary.leadership,
      ...pkg.kpi_summary.sales,
      ...pkg.kpi_summary.production,
      ...pkg.kpi_summary.finance,
    ];
    const keyKpis = allKpis.filter((k) => k.status === "danger" || k.status === "warning");
    if (keyKpis.length > 0) {
      keyKpis.forEach((kpi) => {
        lines.push(`  • ${kpi.kpi_name}：${kpi.display_value}（${kpi.status === "danger" ? "需关注" : "警告"}）`);
      });
    } else {
      lines.push("  • 各项指标正常");
    }
    lines.push("");

    // 风险摘要
    lines.push("二、风险合同");
    const highRisk = pkg.risk_contracts.filter((r) => r.risk_level === "high");
    lines.push(`  • 高风险合同：${highRisk.length} 份`);
    if (highRisk.length > 0) {
      highRisk.slice(0, 3).forEach((r) => {
        lines.push(`    - ${r.contract_id}：${r.risk_description}`);
      });
    }
    lines.push("");

    // 客户保障
    lines.push("三、客户保障");
    lines.push(`  • 风险客户：${pkg.customer_analysis.risk_count} 个`);
    pkg.customer_analysis.risk_customers.slice(0, 3).forEach((c) => {
      lines.push(`    - ${c.customer_id}（${c.customer_level}级）：${c.risk_description || "需关注"}`);
    });
    lines.push("");

    // 节拍状态
    lines.push("四、节拍状态");
    lines.push(`  • 整体匹配率：${(pkg.rhythm_analysis.overall_match_rate * 100).toFixed(0)}%`);
    lines.push(`  • 状态：${pkg.rhythm_analysis.overall_status === "smooth" ? "顺行" : pkg.rhythm_analysis.overall_status === "partial" ? "部分拥堵" : "拥堵"}`);
    lines.push("");

    // 行动建议
    if (pkg.recommendations.length > 0) {
      lines.push("五、行动建议");
      pkg.recommendations.slice(0, 5).forEach((r, i) => {
        lines.push(`  ${i + 1}. ${r.title}`);
        lines.push(`     ${r.description}`);
      });
    }

    return lines.join("\n");
  };

  // 导出为 CSV 文件
  const exportToCsv = async () => {
    if (!consensusPackage) return;

    try {
      const defaultPath = await desktopDir();
      const selectedDir = await open({
        directory: true,
        defaultPath,
        title: "选择导出目录",
      });

      if (!selectedDir || Array.isArray(selectedDir)) return;

      setExporting(true);
      const result = await api.exportConsensusCsv(
        selectedStrategy,
        meetingType,
        meetingDate,
        selectedDir,
        `consensus_${meetingDate}`,
        "系统用户"
      );

      if (result.success) {
        setExportedFiles(result.file_paths);
        setExportMessage({ type: "success", text: `成功导出 ${result.file_paths.length} 个文件` });
      } else {
        setExportMessage({ type: "error", text: result.message });
      }
    } catch (err) {
      console.error("导出失败:", err);
      setExportMessage({ type: "error", text: String(err) });
    } finally {
      setExporting(false);
    }
  };

  // 导出排名表
  const exportRankingCsv = async () => {
    if (!consensusPackage) return;

    try {
      const defaultPath = await desktopDir();
      const selectedPath = await open({
        directory: false,
        defaultPath,
        title: "保存排名表",
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });

      if (!selectedPath || Array.isArray(selectedPath)) return;

      setExporting(true);
      const result = await api.exportContractsRankingCsv(
        selectedStrategy,
        meetingType,
        meetingDate,
        selectedPath,
        "系统用户"
      );

      if (result.success) {
        setExportMessage({ type: "success", text: `排名表已导出：${result.total_rows} 条记录` });
      } else {
        setExportMessage({ type: "error", text: result.message });
      }
    } catch (err) {
      console.error("导出排名表失败:", err);
      setExportMessage({ type: "error", text: String(err) });
    } finally {
      setExporting(false);
    }
  };

  // 复制结论到剪贴板
  const copyConclusions = () => {
    navigator.clipboard.writeText(customConclusions).then(() => {
      setExportMessage({ type: "success", text: "已复制到剪贴板" });
      setTimeout(() => setExportMessage(null), 2000);
    });
  };

  // 统计数据
  const stats = consensusPackage
    ? {
        totalContracts: consensusPackage.top_contracts.length,
        riskCount: consensusPackage.risk_contracts.length,
        highRiskCount: consensusPackage.risk_contracts.filter((r) => r.risk_level === "high").length,
        riskCustomers: consensusPackage.customer_analysis.risk_count,
        recommendations: consensusPackage.recommendations.length,
      }
    : null;

  return (
    <div className="cockpit-consensus-page">
      <div className="page-header">
        <div>
          <h1 className="page-header__title">共识交付包</h1>
          <p className="page-header__subtitle">
            生成会议材料、导出附件清单、共识结论
          </p>
        </div>
        <div className="header-controls">
          <select
            value={meetingType}
            onChange={(e) => setMeetingType(e.target.value as MeetingType)}
            className="type-select"
          >
            <option value="production_sales">产销会</option>
            <option value="business">经营会</option>
          </select>
          <input
            type="date"
            value={meetingDate}
            onChange={(e) => setMeetingDate(e.target.value)}
            className="date-input"
          />
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
          <button
            className="btn-generate"
            onClick={generatePackage}
            disabled={loading || !selectedStrategy}
          >
            {loading ? "生成中..." : "生成共识包"}
          </button>
        </div>
      </div>

      {/* 统计概览 */}
      {stats && (
        <div className="stats-overview">
          <div className="stat-card">
            <div className="stat-value">{stats.totalContracts}</div>
            <div className="stat-label">合同总数</div>
          </div>
          <div className="stat-card warning">
            <div className="stat-value">{stats.riskCount}</div>
            <div className="stat-label">风险合同</div>
          </div>
          <div className="stat-card danger">
            <div className="stat-value">{stats.highRiskCount}</div>
            <div className="stat-label">高风险</div>
          </div>
          <div className="stat-card warning">
            <div className="stat-value">{stats.riskCustomers}</div>
            <div className="stat-label">风险客户</div>
          </div>
          <div className="stat-card">
            <div className="stat-value">{stats.recommendations}</div>
            <div className="stat-label">行动建议</div>
          </div>
        </div>
      )}

      {/* Tab 导航 */}
      <div className="tab-nav">
        <button
          className={`tab-btn ${activeTab === "overview" ? "active" : ""}`}
          onClick={() => setActiveTab("overview")}
        >
          概览
        </button>
        <button
          className={`tab-btn ${activeTab === "conclusions" ? "active" : ""}`}
          onClick={() => setActiveTab("conclusions")}
        >
          会议结论
        </button>
        <button
          className={`tab-btn ${activeTab === "attachments" ? "active" : ""}`}
          onClick={() => setActiveTab("attachments")}
        >
          附件清单
        </button>
        <button
          className={`tab-btn ${activeTab === "export" ? "active" : ""}`}
          onClick={() => setActiveTab("export")}
        >
          导出
        </button>
      </div>

      {/* Tab 内容 */}
      <div className="tab-content">
        {!consensusPackage ? (
          <div className="empty-state">
            <div className="empty-icon">📦</div>
            <div className="empty-text">请先生成共识包</div>
            <p className="empty-hint">选择会议类型、日期和策略后点击"生成共识包"</p>
          </div>
        ) : (
          <>
            {/* 概览 Tab */}
            {activeTab === "overview" && (
              <div className="overview-panel">
                <div className="overview-grid">
                  {/* KPI 摘要 */}
                  <div className="overview-section">
                    <h3>KPI 概要</h3>
                    <div className="kpi-grid">
                      {[
                        { label: "领导视角", kpis: consensusPackage.kpi_summary.leadership },
                        { label: "销售视角", kpis: consensusPackage.kpi_summary.sales },
                        { label: "生产视角", kpis: consensusPackage.kpi_summary.production },
                        { label: "财务视角", kpis: consensusPackage.kpi_summary.finance },
                      ].map(({ label, kpis }) => (
                        <div key={label} className="kpi-section">
                          <div className="kpi-section-title">{label}</div>
                          {kpis.map((kpi) => (
                            <div key={kpi.kpi_code} className={`kpi-item ${kpi.status}`}>
                              <span className="kpi-name">{kpi.kpi_name}</span>
                              <span className="kpi-value">{kpi.display_value}</span>
                            </div>
                          ))}
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* 行动建议 */}
                  <div className="overview-section">
                    <h3>行动建议 Top 5</h3>
                    <div className="recommendations-list">
                      {consensusPackage.recommendations.slice(0, 5).map((rec, idx) => (
                        <div key={idx} className={`recommendation-item ${rec.category}`}>
                          <div className="rec-header">
                            <span className="rec-priority">#{rec.priority}</span>
                            <span className="rec-category">{rec.category}</span>
                          </div>
                          <div className="rec-title">{rec.title}</div>
                          <div className="rec-description">{rec.description}</div>
                        </div>
                      ))}
                    </div>
                  </div>

                  {/* Top 10 合同 */}
                  <div className="overview-section full-width">
                    <h3>Top 10 合同排名</h3>
                    <table className="data-table">
                      <thead>
                        <tr>
                          <th>排名</th>
                          <th>合同ID</th>
                          <th>客户</th>
                          <th>等级</th>
                          <th>规格族</th>
                          <th>到期天数</th>
                          <th>优先级</th>
                          <th>风险</th>
                        </tr>
                      </thead>
                      <tbody>
                        {consensusPackage.top_contracts.slice(0, 10).map((c) => (
                          <tr key={c.contract_id} className={c.has_risk ? "risk-row" : ""}>
                            <td className="rank">#{c.rank}</td>
                            <td className="contract-id">{c.contract_id}</td>
                            <td>{c.customer_id}</td>
                            <td>
                              <span className={`level-badge ${c.customer_level}`}>
                                {c.customer_level}
                              </span>
                            </td>
                            <td>{c.spec_family}</td>
                            <td className={c.days_to_pdd <= 2 ? "urgent" : ""}>
                              T-{c.days_to_pdd}
                            </td>
                            <td>{c.priority.toFixed(2)}</td>
                            <td>
                              {c.has_risk && (
                                <span className="risk-badge">
                                  {c.risk_types.join(", ")}
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
            )}

            {/* 会议结论 Tab */}
            {activeTab === "conclusions" && (
              <div className="conclusions-panel">
                <div className="conclusions-header">
                  <h3>会议结论</h3>
                  <div className="conclusions-actions">
                    <button
                      className={`btn-edit ${editMode ? "active" : ""}`}
                      onClick={() => setEditMode(!editMode)}
                    >
                      {editMode ? "完成编辑" : "编辑"}
                    </button>
                    <button className="btn-copy" onClick={copyConclusions}>
                      复制
                    </button>
                  </div>
                </div>
                {editMode ? (
                  <textarea
                    className="conclusions-editor"
                    value={customConclusions}
                    onChange={(e) => setCustomConclusions(e.target.value)}
                    placeholder="编辑会议结论..."
                  />
                ) : (
                  <pre className="conclusions-content">{customConclusions}</pre>
                )}
              </div>
            )}

            {/* 附件清单 Tab */}
            {activeTab === "attachments" && (
              <div className="attachments-panel">
                <div className="attachments-header">
                  <h3>附件清单</h3>
                  <span className="attachments-count">
                    {exportedFiles.length > 0 ? `${exportedFiles.length} 个文件` : "未导出"}
                  </span>
                </div>

                <div className="attachment-categories">
                  <div className="attachment-category">
                    <div className="category-title">可生成附件</div>
                    <div className="attachment-list">
                      <div className="attachment-item">
                        <span className="attachment-icon">📊</span>
                        <div className="attachment-info">
                          <div className="attachment-name">合同排名表</div>
                          <div className="attachment-desc">
                            完整的合同优先级排名（{consensusPackage.top_contracts.length} 条）
                          </div>
                        </div>
                        <span className="attachment-format">CSV</span>
                      </div>
                      <div className="attachment-item">
                        <span className="attachment-icon">⚠️</span>
                        <div className="attachment-info">
                          <div className="attachment-name">风险合同清单</div>
                          <div className="attachment-desc">
                            {consensusPackage.risk_contracts.length} 份风险合同详情
                          </div>
                        </div>
                        <span className="attachment-format">CSV</span>
                      </div>
                      <div className="attachment-item">
                        <span className="attachment-icon">👥</span>
                        <div className="attachment-info">
                          <div className="attachment-name">客户保障分析</div>
                          <div className="attachment-desc">
                            {consensusPackage.customer_analysis.total_customers} 个客户保障情况
                          </div>
                        </div>
                        <span className="attachment-format">CSV</span>
                      </div>
                      <div className="attachment-item">
                        <span className="attachment-icon">📅</span>
                        <div className="attachment-info">
                          <div className="attachment-name">节拍分析</div>
                          <div className="attachment-desc">
                            {consensusPackage.rhythm_analysis.cycle_days}日周期节拍详情
                          </div>
                        </div>
                        <span className="attachment-format">CSV</span>
                      </div>
                      <div className="attachment-item">
                        <span className="attachment-icon">📈</span>
                        <div className="attachment-info">
                          <div className="attachment-name">KPI 指标汇总</div>
                          <div className="attachment-desc">四视角 KPI 完整数据</div>
                        </div>
                        <span className="attachment-format">CSV</span>
                      </div>
                    </div>
                  </div>

                  {exportedFiles.length > 0 && (
                    <div className="attachment-category">
                      <div className="category-title">已导出文件</div>
                      <div className="attachment-list">
                        {exportedFiles.map((file, idx) => (
                          <div key={idx} className="attachment-item exported">
                            <span className="attachment-icon">✅</span>
                            <div className="attachment-info">
                              <div className="attachment-name">{file.file_name}</div>
                              <div className="attachment-desc">
                                {file.row_count} 条记录 · {file.data_type}
                              </div>
                            </div>
                            <span className="attachment-path" title={file.file_path}>
                              {file.file_path.split("/").pop()}
                            </span>
                          </div>
                        ))}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            )}

            {/* 导出 Tab */}
            {activeTab === "export" && (
              <div className="export-panel">
                <div className="export-options">
                  <div className="export-option">
                    <div className="export-option-header">
                      <span className="export-icon">📦</span>
                      <div className="export-info">
                        <div className="export-title">完整共识包</div>
                        <div className="export-desc">
                          导出所有附件（合同排名、风险清单、客户分析、节拍分析、KPI 汇总）
                        </div>
                      </div>
                    </div>
                    <button
                      className="btn-export"
                      onClick={exportToCsv}
                      disabled={exporting}
                    >
                      {exporting ? "导出中..." : "导出 CSV 文件包"}
                    </button>
                  </div>

                  <div className="export-option">
                    <div className="export-option-header">
                      <span className="export-icon">📊</span>
                      <div className="export-info">
                        <div className="export-title">合同排名表</div>
                        <div className="export-desc">
                          仅导出合同优先级排名（单个 CSV 文件）
                        </div>
                      </div>
                    </div>
                    <button
                      className="btn-export secondary"
                      onClick={exportRankingCsv}
                      disabled={exporting}
                    >
                      导出排名表
                    </button>
                  </div>

                  <div className="export-option">
                    <div className="export-option-header">
                      <span className="export-icon">📝</span>
                      <div className="export-info">
                        <div className="export-title">会议结论</div>
                        <div className="export-desc">复制纯文本格式的会议结论到剪贴板</div>
                      </div>
                    </div>
                    <button className="btn-export secondary" onClick={copyConclusions}>
                      复制到剪贴板
                    </button>
                  </div>
                </div>

                {exportMessage && (
                  <div className={`export-message ${exportMessage.type}`}>
                    {exportMessage.type === "success" ? "✓" : "✗"} {exportMessage.text}
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>

      <style>{`
        .cockpit-consensus-page {
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

        .type-select,
        .strategy-select,
        .date-input {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        .btn-generate {
          padding: var(--spacing-xs) var(--spacing-lg);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
          font-weight: 500;
        }

        .btn-generate:disabled {
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

        /* 空状态 */
        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          height: 100%;
          color: var(--color-text-tertiary);
        }

        .empty-icon {
          font-size: 64px;
          margin-bottom: var(--spacing-md);
        }

        .empty-text {
          font-size: var(--font-size-lg);
          font-weight: 500;
          margin-bottom: var(--spacing-sm);
        }

        .empty-hint {
          font-size: var(--font-size-sm);
        }

        /* 概览面板 */
        .overview-panel {
          padding: var(--spacing-md);
          height: 100%;
          overflow: auto;
        }

        .overview-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: var(--spacing-md);
        }

        .overview-section {
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-md);
          padding: var(--spacing-md);
        }

        .overview-section.full-width {
          grid-column: 1 / -1;
        }

        .overview-section h3 {
          margin: 0 0 var(--spacing-md) 0;
          font-size: var(--font-size-base);
        }

        /* KPI 网格 */
        .kpi-grid {
          display: grid;
          grid-template-columns: repeat(2, 1fr);
          gap: var(--spacing-sm);
        }

        .kpi-section {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          padding: var(--spacing-sm);
        }

        .kpi-section-title {
          font-weight: 600;
          font-size: var(--font-size-sm);
          margin-bottom: var(--spacing-xs);
          color: var(--color-text-secondary);
        }

        .kpi-item {
          display: flex;
          justify-content: space-between;
          padding: 4px 0;
          font-size: var(--font-size-sm);
        }

        .kpi-item.danger { color: #cf1322; }
        .kpi-item.warning { color: #d46b08; }
        .kpi-item.good { color: #389e0d; }

        .kpi-value {
          font-weight: 600;
        }

        /* 行动建议 */
        .recommendations-list {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-sm);
        }

        .recommendation-item {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          padding: var(--spacing-sm);
          border-left: 3px solid var(--color-border);
        }

        .recommendation-item.risk { border-left-color: #ff4d4f; }
        .recommendation-item.kpi { border-left-color: #1890ff; }
        .recommendation-item.rhythm { border-left-color: #52c41a; }
        .recommendation-item.customer { border-left-color: #faad14; }

        .rec-header {
          display: flex;
          gap: var(--spacing-sm);
          margin-bottom: var(--spacing-xs);
        }

        .rec-priority {
          font-weight: 700;
          color: var(--color-primary);
        }

        .rec-category {
          font-size: var(--font-size-xs);
          background: var(--color-bg-layout);
          padding: 2px 6px;
          border-radius: 4px;
        }

        .rec-title {
          font-weight: 600;
          margin-bottom: 4px;
        }

        .rec-description {
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        /* 数据表格 */
        .data-table {
          width: 100%;
          border-collapse: collapse;
        }

        .data-table th,
        .data-table td {
          padding: var(--spacing-sm);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .data-table th {
          background: var(--color-bg-container);
          font-weight: 600;
        }

        .data-table .rank {
          font-weight: 700;
          color: var(--color-primary);
        }

        .data-table .contract-id {
          font-family: monospace;
        }

        .data-table .urgent {
          color: #cf1322;
          font-weight: 600;
        }

        .data-table .risk-row {
          background: rgba(255, 77, 79, 0.05);
        }

        .level-badge {
          display: inline-block;
          padding: 2px 8px;
          border-radius: 4px;
          font-size: var(--font-size-sm);
        }

        .level-badge.A { background: #fff0f0; color: #cf1322; }
        .level-badge.B { background: #fff7e6; color: #d46b08; }
        .level-badge.C { background: #f6ffed; color: #389e0d; }

        .risk-badge {
          font-size: var(--font-size-xs);
          background: #fff0f0;
          color: #cf1322;
          padding: 2px 6px;
          border-radius: 4px;
        }

        /* 会议结论 */
        .conclusions-panel {
          padding: var(--spacing-md);
          height: 100%;
          display: flex;
          flex-direction: column;
        }

        .conclusions-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: var(--spacing-md);
        }

        .conclusions-header h3 {
          margin: 0;
        }

        .conclusions-actions {
          display: flex;
          gap: var(--spacing-sm);
        }

        .btn-edit,
        .btn-copy {
          padding: var(--spacing-xs) var(--spacing-md);
          border: 1px solid var(--color-border);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .btn-edit.active {
          background: var(--color-primary);
          color: #fff;
          border-color: var(--color-primary);
        }

        .conclusions-editor {
          flex: 1;
          padding: var(--spacing-md);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-md);
          font-family: monospace;
          font-size: var(--font-size-sm);
          line-height: 1.6;
          resize: none;
        }

        .conclusions-content {
          flex: 1;
          background: var(--color-bg-layout);
          padding: var(--spacing-lg);
          border-radius: var(--border-radius-md);
          font-family: monospace;
          font-size: var(--font-size-sm);
          line-height: 1.6;
          white-space: pre-wrap;
          overflow: auto;
        }

        /* 附件清单 */
        .attachments-panel {
          padding: var(--spacing-md);
          height: 100%;
          overflow: auto;
        }

        .attachments-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: var(--spacing-md);
        }

        .attachments-header h3 {
          margin: 0;
        }

        .attachments-count {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .attachment-categories {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-lg);
        }

        .category-title {
          font-weight: 600;
          margin-bottom: var(--spacing-sm);
          color: var(--color-text-secondary);
        }

        .attachment-list {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-sm);
        }

        .attachment-item {
          display: flex;
          align-items: center;
          gap: var(--spacing-md);
          padding: var(--spacing-md);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-md);
        }

        .attachment-item.exported {
          background: rgba(82, 196, 26, 0.1);
        }

        .attachment-icon {
          font-size: 24px;
        }

        .attachment-info {
          flex: 1;
        }

        .attachment-name {
          font-weight: 500;
        }

        .attachment-desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .attachment-format {
          font-size: var(--font-size-xs);
          background: var(--color-bg-container);
          padding: 4px 8px;
          border-radius: 4px;
          color: var(--color-text-secondary);
        }

        .attachment-path {
          font-size: var(--font-size-xs);
          color: var(--color-text-tertiary);
          max-width: 200px;
          overflow: hidden;
          text-overflow: ellipsis;
        }

        /* 导出面板 */
        .export-panel {
          padding: var(--spacing-lg);
          display: flex;
          flex-direction: column;
          gap: var(--spacing-lg);
        }

        .export-options {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-md);
        }

        .export-option {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: var(--spacing-lg);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-lg);
        }

        .export-option-header {
          display: flex;
          align-items: center;
          gap: var(--spacing-md);
        }

        .export-icon {
          font-size: 32px;
        }

        .export-title {
          font-weight: 600;
          font-size: var(--font-size-base);
        }

        .export-desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .btn-export {
          padding: var(--spacing-sm) var(--spacing-xl);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
          font-weight: 500;
        }

        .btn-export.secondary {
          background: var(--color-bg-container);
          color: var(--color-text);
          border: 1px solid var(--color-border);
        }

        .btn-export:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }

        .export-message {
          padding: var(--spacing-md);
          border-radius: var(--border-radius-md);
          text-align: center;
        }

        .export-message.success {
          background: rgba(82, 196, 26, 0.1);
          color: #389e0d;
        }

        .export-message.error {
          background: rgba(255, 77, 79, 0.1);
          color: #cf1322;
        }
      `}</style>
    </div>
  );
}
