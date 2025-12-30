import React, { useState, useEffect, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { save } from "@tauri-apps/api/dialog";
import { documentDir } from "@tauri-apps/api/path";
import {
  api,
  ContractPriority,
  KpiSummary,
  KpiValue,
  RiskIdentificationResult,
  CustomerProtectionAnalysis,
  RhythmFlowAnalysis,
  MeetingType,
  StrategyWeights
} from "../api/tauri";
import "./Cockpit.css";

interface Issue {
  id: string;
  type: "critical" | "warning" | "info";
  title: string;
  description: string;
  relatedContracts: string[];
  time: string;
}

interface Decision {
  id: string;
  priority: "high" | "medium" | "low";
  action: string;
  impact: string;
}

// 节拍天数
const RHYTHM_DAYS = ["D+1", "D+2", "D+3"];
const SPEC_FAMILIES = ["常规", "特殊", "超特"];

export function Cockpit() {
  const navigate = useNavigate();
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<string>("");
  const [strategyWeights, setStrategyWeights] = useState<StrategyWeights | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Phase 16: 新增 KPI 和分析状态
  const [kpiSummary, setKpiSummary] = useState<KpiSummary | null>(null);
  const [riskResult, setRiskResult] = useState<RiskIdentificationResult | null>(null);
  const [customerAnalysis, setCustomerAnalysis] = useState<CustomerProtectionAnalysis | null>(null);
  const [rhythmAnalysis, setRhythmAnalysis] = useState<RhythmFlowAnalysis | null>(null);
  const [kpiLoading, setKpiLoading] = useState(false);
  const [exporting, setExporting] = useState(false);

  // 加载策略列表
  useEffect(() => {
    api.getStrategies().then((strats) => {
      setStrategies(strats);
      if (strats.length > 0) {
        setSelectedStrategy(strats[0]);
      }
    }).catch((err) => {
      console.error("加载策略失败:", err);
      setError("加载策略失败: " + err);
    });
  }, []);

  // 策略变化时加载数据
  useEffect(() => {
    if (selectedStrategy) {
      loadAllData();
      // 加载策略权重
      api.getStrategyWeightsList().then((weights) => {
        const current = weights.find((w) => w.strategy_name === selectedStrategy);
        setStrategyWeights(current || null);
      });
    }
  }, [selectedStrategy]);

  // 加载所有数据（合同优先级 + KPI + 分析）
  const loadAllData = async () => {
    setLoading(true);
    setError(null);

    try {
      // 并行加载合同优先级和 KPI 数据
      const [contractsData] = await Promise.all([
        api.computeAllPriorities(selectedStrategy).catch(err => {
          console.error("计算优先级失败:", err);
          return [] as ContractPriority[];
        }),
        loadKpiData()
      ]);

      setContracts(contractsData);
    } catch (err) {
      console.error("加载数据失败:", err);
      setError("加载数据失败: " + String(err));
    } finally {
      setLoading(false);
    }
  };

  // 加载 KPI 和分析数据
  const loadKpiData = async () => {
    setKpiLoading(true);
    const errors: string[] = [];
    try {
      // 并行加载所有 KPI 相关数据
      const [kpi, risk, customer, rhythm] = await Promise.all([
        api.calculateMeetingKpis(selectedStrategy).catch(err => {
          console.error("计算 KPI 失败:", err);
          errors.push("KPI计算: " + String(err));
          return null;
        }),
        api.identifyRiskContracts(selectedStrategy).catch(err => {
          console.error("识别风险失败:", err);
          errors.push("风险识别: " + String(err));
          return null;
        }),
        api.analyzeCustomerProtection(selectedStrategy).catch(err => {
          console.error("客户分析失败:", err);
          errors.push("客户分析: " + String(err));
          return null;
        }),
        api.analyzeRhythmFlow(selectedStrategy).catch(err => {
          console.error("节拍分析失败:", err);
          errors.push("节拍分析: " + String(err));
          return null;
        })
      ]);

      if (errors.length > 0) {
        setError("KPI 加载部分失败: " + errors.join("; "));
      }

      setKpiSummary(kpi);
      setRiskResult(risk);
      setCustomerAnalysis(customer);
      setRhythmAnalysis(rhythm);
    } catch (err) {
      console.error("加载 KPI 数据失败:", err);
      setError("加载 KPI 数据失败: " + String(err));
    } finally {
      setKpiLoading(false);
    }
  };

  // 导出共识包为 CSV
  const handleExport = async () => {
    if (!selectedStrategy) return;

    setExporting(true);
    try {
      // 获取文档目录
      const docDir = await documentDir();
      const today = new Date().toISOString().split('T')[0];

      // 让用户选择保存目录
      const filePath = await save({
        defaultPath: `${docDir}/consensus_${today}.csv`,
        filters: [{
          name: 'CSV',
          extensions: ['csv']
        }]
      });

      if (filePath) {
        // 从完整路径中提取目录
        const lastSlash = filePath.lastIndexOf('/');
        const outputDir = filePath.substring(0, lastSlash) || docDir;
        const fileName = filePath.substring(lastSlash + 1);
        const filePrefix = fileName.replace('.csv', '');

        // 调用导出 API
        const result = await api.exportConsensusCsv(
          selectedStrategy,
          'production_sales' as MeetingType,
          today,
          outputDir,
          filePrefix,
          'user'
        );

        if (result.success) {
          alert(`导出成功！\n${result.message}\n\n文件保存至：\n${result.file_paths.map(f => f.file_path).join('\n')}`);
        } else {
          alert('导出失败：' + result.message);
        }
      }
    } catch (err) {
      console.error("导出失败:", err);
      alert("导出失败: " + String(err));
    } finally {
      setExporting(false);
    }
  };

  // 统计数据
  const stats = useMemo(() => {
    const total = contracts.length;
    const urgent = contracts.filter((c) => c.days_to_pdd <= 3).length;
    const alphaAdjusted = contracts.filter((c) => c.alpha).length;
    const avgPriority =
      total > 0
        ? contracts.reduce((sum, c) => sum + c.priority, 0) / total
        : 0;
    const highPriority = contracts.filter((c) => c.priority >= 80).length;

    return { total, urgent, alphaAdjusted, avgPriority, highPriority };
  }, [contracts]);

  // 优先级分布
  const distribution = useMemo(() => {
    const total = contracts.length;
    if (total === 0) return { urgent: 0, high: 0, normal: 0, low: 0 };

    const urgent = contracts.filter((c) => c.days_to_pdd <= 3).length;
    const high = contracts.filter(
      (c) => c.days_to_pdd > 3 && c.priority >= 70
    ).length;
    const normal = contracts.filter(
      (c) => c.priority >= 40 && c.priority < 70
    ).length;
    const low = contracts.filter((c) => c.priority < 40).length;

    return {
      urgent: (urgent / total) * 100,
      high: (high / total) * 100,
      normal: (normal / total) * 100,
      low: (low / total) * 100,
    };
  }, [contracts]);

  // 生成节奏热图数据
  const rhythmData = useMemo(() => {
    const data: Record<string, Record<string, { count: number; level: string }>> = {};

    SPEC_FAMILIES.forEach((family) => {
      data[family] = {};
      RHYTHM_DAYS.forEach((day) => {
        const dayIndex = RHYTHM_DAYS.indexOf(day);
        const familyContracts = contracts.filter(
          (c) =>
            c.spec_family === family &&
            c.days_to_pdd >= dayIndex + 1 &&
            c.days_to_pdd <= dayIndex + 2
        );
        const count = familyContracts.length;
        const level = count > 5 ? "high" : count > 2 ? "medium" : "low";
        data[family][day] = { count, level };
      });
    });

    return data;
  }, [contracts]);

  // 基于风险识别结果生成问题清单
  const issues: Issue[] = useMemo(() => {
    const result: Issue[] = [];

    // 如果有风险识别结果，使用真实数据
    if (riskResult && riskResult.high_risk_count > 0) {
      result.push({
        id: "risk-high",
        type: "critical",
        title: `${riskResult.high_risk_count} 份高风险合同`,
        description: "需要立即处理",
        relatedContracts: riskResult.risk_contracts
          .filter(r => r.risk_level === "high")
          .map(r => r.contract_id),
        time: "刚刚",
      });
    }

    if (riskResult && riskResult.medium_risk_count > 0) {
      result.push({
        id: "risk-medium",
        type: "warning",
        title: `${riskResult.medium_risk_count} 份中风险合同`,
        description: "需要关注",
        relatedContracts: riskResult.risk_contracts
          .filter(r => r.risk_level === "medium")
          .map(r => r.contract_id),
        time: "刚刚",
      });
    }

    // 检查客户保障风险
    if (customerAnalysis && customerAnalysis.risk_count > 0) {
      result.push({
        id: "customer-risk",
        type: "warning",
        title: `${customerAnalysis.risk_count} 位客户保障不足`,
        description: "重点客户排名靠后",
        relatedContracts: customerAnalysis.risk_customers
          .slice(0, 5)
          .map(c => c.customer_id),
        time: "刚刚",
      });
    }

    // 检查节拍拥堵
    if (rhythmAnalysis?.congestion_days && rhythmAnalysis.congestion_days.length > 0) {
      result.push({
        id: "rhythm-congestion",
        type: "warning",
        title: `节拍拥堵: D+${rhythmAnalysis.congestion_days.join(", D+")}`,
        description: "生产排期可能冲突",
        relatedContracts: [],
        time: "刚刚",
      });
    }

    // 回退到合同数据的检测逻辑
    if (result.length === 0) {
      const urgentContracts = contracts.filter((c) => c.days_to_pdd <= 2);
      if (urgentContracts.length > 0) {
        result.push({
          id: "issue-1",
          type: "critical",
          title: `${urgentContracts.length} 份合同即将超期`,
          description: "2天内到期，需要优先处理",
          relatedContracts: urgentContracts.map((c) => c.contract_id),
          time: "10分钟前",
        });
      }

      const alphaContracts = contracts.filter((c) => c.alpha && c.alpha !== 1.0);
      if (alphaContracts.length > 0) {
        result.push({
          id: "issue-3",
          type: "info",
          title: `${alphaContracts.length} 份合同有人工调整`,
          description: "请确认调整是否仍然有效",
          relatedContracts: alphaContracts.map((c) => c.contract_id),
          time: "1小时前",
        });
      }
    }

    return result;
  }, [contracts, riskResult, customerAnalysis, rhythmAnalysis]);

  // 决策建议
  const decisions: Decision[] = useMemo(() => {
    const result: Decision[] = [];

    if (riskResult && riskResult.high_risk_count > 0) {
      result.push({
        id: "dec-risk",
        priority: "high",
        action: "处理高风险合同",
        impact: `避免 ${riskResult.high_risk_count} 份合同延期`,
      });
    }

    if (customerAnalysis && customerAnalysis.risk_count > 0) {
      result.push({
        id: "dec-customer",
        priority: "high",
        action: "提升重点客户保障",
        impact: `${customerAnalysis.risk_count} 位客户需要优先排产`,
      });
    }

    if (rhythmAnalysis && rhythmAnalysis.overall_status !== "smooth") {
      result.push({
        id: "dec-rhythm",
        priority: "medium",
        action: "优化节拍匹配",
        impact: `当前匹配率 ${(rhythmAnalysis.overall_match_rate * 100).toFixed(0)}%`,
      });
    }

    if (stats.alphaAdjusted > 0) {
      result.push({
        id: "dec-2",
        priority: "medium",
        action: "审核人工调整合同",
        impact: `涉及 ${stats.alphaAdjusted} 份合同`,
      });
    }

    if (result.length === 0) {
      result.push({
        id: "dec-default",
        priority: "low",
        action: "持续优化排产效率",
        impact: "提升产线利用率",
      });
    }

    return result;
  }, [stats, riskResult, customerAnalysis, rhythmAnalysis]);

  // 获取 KPI 状态颜色
  const getKpiStatusColor = (status: string) => {
    switch (status) {
      case "good": return "var(--color-success)";
      case "warning": return "var(--color-warning)";
      case "danger": return "var(--color-error)";
      default: return "var(--color-text-secondary)";
    }
  };

  // 渲染 KPI 卡片
  const renderKpiCard = (kpi: KpiValue) => (
    <div key={kpi.kpi_code} className="kpi-card">
      <div className="kpi-card-header">
        <span className="kpi-name">{kpi.kpi_name}</span>
        <span
          className="kpi-status"
          style={{
            background: getKpiStatusColor(kpi.status),
            padding: "2px 6px",
            borderRadius: "4px",
            fontSize: "10px",
            color: "#fff"
          }}
        >
          {kpi.status === "good" ? "良好" : kpi.status === "warning" ? "警告" : "异常"}
        </span>
      </div>
      <div className="kpi-value" style={{ color: getKpiStatusColor(kpi.status) }}>
        {kpi.display_value}
      </div>
      {kpi.change !== undefined && (
        <div className="kpi-change" style={{
          color: kpi.change_direction === "up" ? "var(--color-success)" :
                 kpi.change_direction === "down" ? "var(--color-error)" :
                 "var(--color-text-tertiary)"
        }}>
          {kpi.change_direction === "up" ? "↑" : kpi.change_direction === "down" ? "↓" : "→"}
          {Math.abs(kpi.change).toFixed(1)}%
        </div>
      )}
    </div>
  );

  return (
    <div className="cockpit-page">
      <div className="page-header" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
        <div>
          <h1 className="page-header__title">产销协同会议驾驶舱</h1>
          <p className="page-header__subtitle">会议展示与决策支持 {kpiLoading && "(KPI 计算中...)"}</p>
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
            onClick={loadAllData}
            disabled={loading}
            style={{
              padding: "var(--spacing-xs) var(--spacing-md)",
              borderRadius: "var(--border-radius-sm)",
              border: "1px solid var(--color-primary)",
              background: "var(--color-primary)",
              color: "#fff",
              cursor: loading ? "not-allowed" : "pointer",
              opacity: loading ? 0.6 : 1,
            }}
          >
            {loading ? "刷新中..." : "刷新数据"}
          </button>
          <button
            onClick={handleExport}
            disabled={exporting || loading || !contracts.length}
            style={{
              padding: "var(--spacing-xs) var(--spacing-md)",
              borderRadius: "var(--border-radius-sm)",
              border: "1px solid var(--color-success)",
              background: "var(--color-success)",
              color: "#fff",
              cursor: (exporting || loading || !contracts.length) ? "not-allowed" : "pointer",
              opacity: (exporting || loading || !contracts.length) ? 0.6 : 1,
            }}
          >
            {exporting ? "导出中..." : "导出 CSV"}
          </button>
        </div>
      </div>

      {/* 错误提示 */}
      {error && (
        <div style={{
          padding: "var(--spacing-md)",
          background: "var(--color-error-bg)",
          color: "var(--color-error)",
          borderRadius: "var(--border-radius-sm)",
          marginBottom: "var(--spacing-md)"
        }}>
          {error}
        </div>
      )}

      {/* 参数版本信息栏 */}
      <div className="param-version-bar" style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "var(--spacing-sm) var(--spacing-md)",
        background: "var(--color-bg-container)",
        borderRadius: "var(--border-radius-md)",
        marginBottom: "var(--spacing-md)",
        fontSize: "var(--font-size-sm)"
      }}>
        <div style={{ display: "flex", gap: "var(--spacing-lg)", alignItems: "center" }}>
          <span style={{ color: "var(--color-text-tertiary)" }}>当前参数版本：</span>
          <span style={{ fontWeight: 600 }}>策略 {selectedStrategy}</span>
          {strategyWeights && (
            <>
              <span style={{
                background: "var(--color-primary-light)",
                padding: "2px 8px",
                borderRadius: "4px",
                fontFamily: "monospace"
              }}>
                Ws={strategyWeights.ws.toFixed(2)}
              </span>
              <span style={{
                background: "var(--color-success-light)",
                padding: "2px 8px",
                borderRadius: "4px",
                fontFamily: "monospace"
              }}>
                Wp={strategyWeights.wp.toFixed(2)}
              </span>
            </>
          )}
          <span style={{ color: "var(--color-text-tertiary)" }}>
            数据时间：{new Date().toLocaleString()}
          </span>
        </div>
        <button
          onClick={() => navigate("/settings/weights")}
          style={{
            padding: "4px 12px",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--border-radius-sm)",
            background: "transparent",
            cursor: "pointer",
            fontSize: "var(--font-size-sm)"
          }}
        >
          调整参数
        </button>
      </div>

      {/* 快速导航卡片 */}
      <div className="quick-nav" style={{
        display: "grid",
        gridTemplateColumns: "repeat(4, 1fr)",
        gap: "var(--spacing-md)",
        marginBottom: "var(--spacing-lg)"
      }}>
        <div
          className="nav-card"
          onClick={() => navigate("/cockpit/rhythm")}
          style={{
            padding: "var(--spacing-md)",
            background: "var(--color-bg-container)",
            borderRadius: "var(--border-radius-md)",
            cursor: "pointer",
            transition: "all var(--transition-fast)",
            borderLeft: "4px solid #1890ff"
          }}
        >
          <div style={{ fontSize: "24px", marginBottom: "var(--spacing-xs)" }}>📅</div>
          <div style={{ fontWeight: 600, marginBottom: "4px" }}>节奏视图</div>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
            {rhythmAnalysis
              ? `匹配率 ${(rhythmAnalysis.overall_match_rate * 100).toFixed(0)}%`
              : "节拍分布分析"}
          </div>
        </div>
        <div
          className="nav-card"
          onClick={() => navigate("/cockpit/customer")}
          style={{
            padding: "var(--spacing-md)",
            background: "var(--color-bg-container)",
            borderRadius: "var(--border-radius-md)",
            cursor: "pointer",
            transition: "all var(--transition-fast)",
            borderLeft: "4px solid #faad14"
          }}
        >
          <div style={{ fontSize: "24px", marginBottom: "var(--spacing-xs)" }}>👥</div>
          <div style={{ fontWeight: 600, marginBottom: "4px" }}>客户保障</div>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
            {customerAnalysis
              ? `${customerAnalysis.risk_count} 位风险客户`
              : "重点客户分析"}
          </div>
        </div>
        <div
          className="nav-card"
          onClick={() => navigate("/cockpit/issues")}
          style={{
            padding: "var(--spacing-md)",
            background: "var(--color-bg-container)",
            borderRadius: "var(--border-radius-md)",
            cursor: "pointer",
            transition: "all var(--transition-fast)",
            borderLeft: "4px solid #ff4d4f"
          }}
        >
          <div style={{ fontSize: "24px", marginBottom: "var(--spacing-xs)" }}>⚠️</div>
          <div style={{ fontWeight: 600, marginBottom: "4px" }}>问题清单</div>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
            {riskResult
              ? `${riskResult.high_risk_count} 高风险`
              : "风险合同追踪"}
          </div>
        </div>
        <div
          className="nav-card"
          onClick={() => navigate("/cockpit/consensus")}
          style={{
            padding: "var(--spacing-md)",
            background: "var(--color-bg-container)",
            borderRadius: "var(--border-radius-md)",
            cursor: "pointer",
            transition: "all var(--transition-fast)",
            borderLeft: "4px solid #52c41a"
          }}
        >
          <div style={{ fontSize: "24px", marginBottom: "var(--spacing-xs)" }}>📦</div>
          <div style={{ fontWeight: 600, marginBottom: "4px" }}>共识交付</div>
          <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
            生成会议材料
          </div>
        </div>
      </div>

      {/* 四视角 KPI 概览 */}
      {kpiSummary && (
        <div className="kpi-overview" style={{ marginBottom: "var(--spacing-lg)" }}>
          <h3 style={{ marginBottom: "var(--spacing-sm)" }}>四视角 KPI 概览</h3>
          <div style={{
            display: "grid",
            gridTemplateColumns: "repeat(4, 1fr)",
            gap: "var(--spacing-md)"
          }}>
            {/* 领导视角 */}
            <div className="kpi-section">
              <h4 style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginBottom: "var(--spacing-xs)" }}>
                领导视角
              </h4>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-xs)" }}>
                {kpiSummary.leadership.slice(0, 2).map(renderKpiCard)}
              </div>
            </div>

            {/* 销售视角 */}
            <div className="kpi-section">
              <h4 style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginBottom: "var(--spacing-xs)" }}>
                销售视角
              </h4>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-xs)" }}>
                {kpiSummary.sales.slice(0, 2).map(renderKpiCard)}
              </div>
            </div>

            {/* 生产视角 */}
            <div className="kpi-section">
              <h4 style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginBottom: "var(--spacing-xs)" }}>
                生产视角
              </h4>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-xs)" }}>
                {kpiSummary.production.slice(0, 2).map(renderKpiCard)}
              </div>
            </div>

            {/* 财务视角 */}
            <div className="kpi-section">
              <h4 style={{ fontSize: "12px", color: "var(--color-text-secondary)", marginBottom: "var(--spacing-xs)" }}>
                财务视角
              </h4>
              <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-xs)" }}>
                {kpiSummary.finance.slice(0, 2).map(renderKpiCard)}
              </div>
            </div>
          </div>
        </div>
      )}

      {/* 顶部统计概览 */}
      <div className="cockpit-overview">
        <div className="overview-card highlight">
          <div className="overview-card-label">合同总数</div>
          <div className="overview-card-value">{stats.total}</div>
          <div className="overview-card-trend up">当前策略: {selectedStrategy}</div>
        </div>
        <div className="overview-card">
          <div className="overview-card-label">紧急合同</div>
          <div className="overview-card-value" style={{ color: "var(--color-error)" }}>
            {stats.urgent}
          </div>
          <div className="overview-card-trend down">3天内到期</div>
        </div>
        <div className="overview-card">
          <div className="overview-card-label">风险合同</div>
          <div className="overview-card-value" style={{ color: "var(--color-warning)" }}>
            {riskResult ? riskResult.total_count : "-"}
          </div>
          <div className="overview-card-trend">
            {riskResult ? `高:${riskResult.high_risk_count} 中:${riskResult.medium_risk_count}` : "计算中..."}
          </div>
        </div>
        <div className="overview-card">
          <div className="overview-card-label">节拍匹配率</div>
          <div className="overview-card-value">
            {rhythmAnalysis ? `${(rhythmAnalysis.overall_match_rate * 100).toFixed(0)}%` : "-"}
          </div>
          <div className="overview-card-trend">
            {rhythmAnalysis?.overall_status === "smooth" ? "顺行" :
             rhythmAnalysis?.overall_status === "partial" ? "部分匹配" : "待优化"}
          </div>
        </div>
        <div className="overview-card">
          <div className="overview-card-label">客户保障</div>
          <div className="overview-card-value" style={{ color: customerAnalysis && customerAnalysis.risk_count > 0 ? "var(--color-warning)" : "var(--color-success)" }}>
            {customerAnalysis ? `${customerAnalysis.well_protected_count}/${customerAnalysis.total_customers}` : "-"}
          </div>
          <div className="overview-card-trend">
            {customerAnalysis ? `${customerAnalysis.risk_count} 位风险客户` : "分析中..."}
          </div>
        </div>
      </div>

      {/* 主内容区域 */}
      <div className="cockpit-main">
        {/* 左侧 */}
        <div className="cockpit-left">
          {/* 节奏热图 */}
          <div className="rhythm-heatmap">
            <div className="rhythm-heatmap-header">
              <h3>
                {rhythmAnalysis ? `${rhythmAnalysis.cycle_days}日节奏热图` : "3日节奏热图"}
              </h3>
              <div className="rhythm-legend">
                <div className="legend-item">
                  <div className="legend-color high"></div>
                  <span>高负载 (&gt;5)</span>
                </div>
                <div className="legend-item">
                  <div className="legend-color medium"></div>
                  <span>中负载 (3-5)</span>
                </div>
                <div className="legend-item">
                  <div className="legend-color low"></div>
                  <span>低负载 (&lt;3)</span>
                </div>
              </div>
            </div>
            <div className="rhythm-grid">
              {/* 表头 */}
              <div className="rhythm-header"></div>
              {RHYTHM_DAYS.map((day) => (
                <div key={day} className="rhythm-header">{day}</div>
              ))}

              {/* 数据行 */}
              {SPEC_FAMILIES.map((family) => (
                <React.Fragment key={family}>
                  <div className="rhythm-row-label">{family}</div>
                  {RHYTHM_DAYS.map((day) => {
                    const cell = rhythmData[family]?.[day] || { count: 0, level: "low" };
                    return (
                      <div
                        key={`${family}-${day}`}
                        className={`rhythm-cell ${cell.level}`}
                        title={`${family} ${day}: ${cell.count} 份合同`}
                      >
                        {cell.count}
                      </div>
                    );
                  })}
                </React.Fragment>
              ))}
            </div>
          </div>

          {/* 优先级分布 */}
          <div className="priority-distribution">
            <h3>优先级分布</h3>
            <div className="distribution-bars">
              <div className="distribution-bar">
                <span className="bar-label">紧急 (≤3天)</span>
                <div className="bar-track">
                  <div
                    className="bar-fill urgent"
                    style={{ width: `${distribution.urgent}%` }}
                  ></div>
                </div>
                <span className="bar-value">{distribution.urgent.toFixed(1)}%</span>
              </div>
              <div className="distribution-bar">
                <span className="bar-label">高优先级</span>
                <div className="bar-track">
                  <div
                    className="bar-fill high"
                    style={{ width: `${distribution.high}%` }}
                  ></div>
                </div>
                <span className="bar-value">{distribution.high.toFixed(1)}%</span>
              </div>
              <div className="distribution-bar">
                <span className="bar-label">正常</span>
                <div className="bar-track">
                  <div
                    className="bar-fill normal"
                    style={{ width: `${distribution.normal}%` }}
                  ></div>
                </div>
                <span className="bar-value">{distribution.normal.toFixed(1)}%</span>
              </div>
              <div className="distribution-bar">
                <span className="bar-label">低优先级</span>
                <div className="bar-track">
                  <div
                    className="bar-fill low"
                    style={{ width: `${distribution.low}%` }}
                  ></div>
                </div>
                <span className="bar-value">{distribution.low.toFixed(1)}%</span>
              </div>
            </div>
          </div>
        </div>

        {/* 右侧 */}
        <div className="cockpit-right">
          {/* 问题清单 */}
          <div className="issues-panel">
            <div className="issues-header">
              <h3>问题清单</h3>
              {issues.length > 0 && (
                <span className="issue-count">{issues.length}</span>
              )}
            </div>
            <div className="issues-list">
              {issues.length === 0 ? (
                <div style={{ textAlign: "center", padding: "var(--spacing-lg)", color: "var(--color-text-tertiary)" }}>
                  暂无问题
                </div>
              ) : (
                issues.map((issue) => (
                  <div key={issue.id} className={`issue-item ${issue.type}`}>
                    <div className="issue-title">{issue.title}</div>
                    <div className="issue-meta">
                      <span>{issue.description}</span>
                      <span>{issue.time}</span>
                    </div>
                  </div>
                ))
              )}
            </div>
          </div>

          {/* 决策建议 */}
          <div className="decisions-panel">
            <div className="decisions-header">
              <h3>决策建议</h3>
            </div>
            <div className="decisions-list">
              {decisions.map((decision) => (
                <div key={decision.id} className="decision-item">
                  <div className="decision-item-header">
                    <span className={`decision-priority ${decision.priority}`}>
                      {decision.priority === "high" ? "高" : decision.priority === "medium" ? "中" : "低"}
                    </span>
                  </div>
                  <div className="decision-action">{decision.action}</div>
                  <div className="decision-impact">{decision.impact}</div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
