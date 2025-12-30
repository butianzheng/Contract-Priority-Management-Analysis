import { useState, useEffect } from "react";
import { api, ContractPriority, InterventionLog, PriorityExplain, SScoreComponent, PScoreComponent } from "../api/tauri";
import "./ContractDetailSidebar.css";

interface ContractDetailSidebarProps {
  contract: ContractPriority | null;
  strategy: string;
  onClose: () => void;
  onAlphaChanged?: () => void;
}

type TabType = "detail" | "explain" | "history";

export function ContractDetailSidebar({
  contract,
  strategy,
  onClose,
  onAlphaChanged,
}: ContractDetailSidebarProps) {
  const [activeTab, setActiveTab] = useState<TabType>("detail");
  const [history, setHistory] = useState<InterventionLog[]>([]);
  const [historyLoading, setHistoryLoading] = useState(false);
  const [historyError, setHistoryError] = useState("");

  // Explain 状态
  const [explainData, setExplainData] = useState<PriorityExplain | null>(null);
  const [explainLoading, setExplainLoading] = useState(false);
  const [explainError, setExplainError] = useState("");

  // Alpha 调整状态
  const [alphaValue, setAlphaValue] = useState<string>("1.0");
  const [reason, setReason] = useState<string>("");
  const [userName, setUserName] = useState<string>("admin");
  const [adjusting, setAdjusting] = useState(false);

  // 加载历史记录
  useEffect(() => {
    if (contract && activeTab === "history") {
      loadHistory();
    }
  }, [contract?.contract_id, activeTab]);

  // 加载评分拆分
  useEffect(() => {
    if (contract && strategy && activeTab === "explain") {
      loadExplain();
    }
  }, [contract?.contract_id, strategy, activeTab]);

  // 当合同变化时，重置 alpha 值和 explain
  useEffect(() => {
    if (contract) {
      setAlphaValue(contract.alpha?.toFixed(2) || "1.0");
      setReason("");
      setExplainData(null);
    }
  }, [contract?.contract_id]);

  const loadHistory = async () => {
    if (!contract) return;

    setHistoryLoading(true);
    setHistoryError("");

    try {
      const data = await api.getInterventionHistory(contract.contract_id);
      setHistory(data);
    } catch (err) {
      setHistoryError(`加载历史失败: ${err}`);
    } finally {
      setHistoryLoading(false);
    }
  };

  const loadExplain = async () => {
    if (!contract || !strategy) return;

    setExplainLoading(true);
    setExplainError("");

    try {
      const data = await api.explainPriority(contract.contract_id, strategy);
      setExplainData(data);
    } catch (err) {
      setExplainError(`加载评分详情失败: ${err}`);
    } finally {
      setExplainLoading(false);
    }
  };

  const handleAlphaAdjust = async () => {
    if (!contract) return;

    const alpha = parseFloat(alphaValue);
    if (isNaN(alpha) || alpha < 0.5 || alpha > 2.0) {
      alert("Alpha值必须在 0.5 ~ 2.0 之间");
      return;
    }

    if (!reason.trim()) {
      alert("请填写调整原因");
      return;
    }

    setAdjusting(true);

    try {
      await api.setAlpha(
        contract.contract_id,
        alpha,
        reason,
        userName
      );
      setReason("");
      onAlphaChanged?.();
      // 刷新历史
      if (activeTab === "history") {
        loadHistory();
      }
    } catch (err) {
      alert(`调整失败: ${err}`);
    } finally {
      setAdjusting(false);
    }
  };

  const handleResetAlpha = async () => {
    if (!contract) return;

    if (!confirm("确定要重置 Alpha 值为 1.0 吗？")) {
      return;
    }

    setAdjusting(true);

    try {
      await api.setAlpha(contract.contract_id, 1.0, "重置 Alpha 值", userName);
      setAlphaValue("1.0");
      onAlphaChanged?.();
      if (activeTab === "history") {
        loadHistory();
      }
    } catch (err) {
      alert(`重置失败: ${err}`);
    } finally {
      setAdjusting(false);
    }
  };

  // 渲染详情 Tab
  const renderDetailTab = () => {
    if (!contract) return null;

    return (
      <>
        {/* 评分卡片 */}
        <div className="score-cards">
          <div className="score-card">
            <div className="score-card-label">S-Score</div>
            <div className="score-card-value">{contract.s_score.toFixed(1)}</div>
          </div>
          <div className="score-card">
            <div className="score-card-label">P-Score</div>
            <div className="score-card-value">{contract.p_score.toFixed(1)}</div>
          </div>
          <div className="score-card priority">
            <div className="score-card-label">优先级</div>
            <div className="score-card-value">
              {contract.priority.toFixed(2)}
              {contract.alpha && <span className="alpha-badge">Alpha</span>}
            </div>
          </div>
        </div>

        {/* 基本信息 */}
        <div className="detail-section">
          <h4>基本信息</h4>
          <div className="detail-grid">
            <div className="detail-item">
              <label>合同编号</label>
              <span>{contract.contract_id}</span>
            </div>
            <div className="detail-item">
              <label>客户编号</label>
              <span>{contract.customer_id}</span>
            </div>
            <div className="detail-item">
              <label>钢种</label>
              <span>{contract.steel_grade}</span>
            </div>
            <div className="detail-item">
              <label>规格族</label>
              <span>{contract.spec_family}</span>
            </div>
          </div>
        </div>

        {/* 规格参数 */}
        <div className="detail-section">
          <h4>规格参数</h4>
          <div className="detail-grid">
            <div className="detail-item">
              <label>厚度</label>
              <span>{contract.thickness.toFixed(1)} mm</span>
            </div>
            <div className="detail-item">
              <label>宽度</label>
              <span>{contract.width.toFixed(0)} mm</span>
            </div>
            <div className="detail-item">
              <label>毛利</label>
              <span>{contract.margin.toFixed(2)}</span>
            </div>
          </div>
        </div>

        {/* 交期信息 */}
        <div className="detail-section">
          <h4>交期信息</h4>
          <div className="detail-grid">
            <div className="detail-item">
              <label>计划交货日期</label>
              <span>{contract.pdd}</span>
            </div>
            <div className="detail-item">
              <label>剩余天数</label>
              <span
                className={
                  contract.days_to_pdd <= 3
                    ? "urgent"
                    : contract.days_to_pdd <= 7
                    ? "warning"
                    : ""
                }
              >
                {contract.days_to_pdd} 天
              </span>
            </div>
          </div>
        </div>

        {/* Alpha 调整 */}
        <div className="alpha-adjust-section">
          <h4>优先级调整</h4>
          <div className="alpha-form-row">
            <label>Alpha 系数 (0.5 ~ 2.0)</label>
            <input
              type="number"
              value={alphaValue}
              onChange={(e) => setAlphaValue(e.target.value)}
              min="0.5"
              max="2.0"
              step="0.1"
              disabled={adjusting}
            />
          </div>
          <div className="alpha-form-row">
            <label>操作人</label>
            <input
              type="text"
              value={userName}
              onChange={(e) => setUserName(e.target.value)}
              disabled={adjusting}
            />
          </div>
          <div className="alpha-form-row">
            <label>调整原因</label>
            <textarea
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="请输入调整原因..."
              disabled={adjusting}
            />
          </div>
          <div className="alpha-buttons">
            <button
              className="btn-primary"
              onClick={handleAlphaAdjust}
              disabled={adjusting || !reason.trim()}
            >
              {adjusting ? "处理中..." : "应用调整"}
            </button>
            {contract.alpha && (
              <button
                className="btn-danger"
                onClick={handleResetAlpha}
                disabled={adjusting}
              >
                重置
              </button>
            )}
          </div>
        </div>
      </>
    );
  };

  // 渲染历史 Tab
  const renderHistoryTab = () => {
    if (historyLoading) {
      return <div className="sidebar-loading">加载中...</div>;
    }

    if (historyError) {
      return <div className="error">{historyError}</div>;
    }

    if (history.length === 0) {
      return <div className="history-empty">暂无干预记录</div>;
    }

    return (
      <div className="history-list">
        {history.map((log) => (
          <div key={log.id} className="history-item">
            <div className="history-item-header">
              <span className="history-alpha">
                Alpha: {log.alpha_value.toFixed(2)}
              </span>
              <span className="history-time">
                {log.timestamp
                  ? new Date(log.timestamp).toLocaleString("zh-CN")
                  : "-"}
              </span>
            </div>
            <div className="history-reason">{log.reason}</div>
            <div className="history-user">操作人: {log.user}</div>
          </div>
        ))}
      </div>
    );
  };

  // 渲染评分拆分 Tab
  const renderExplainTab = () => {
    if (explainLoading) {
      return <div className="sidebar-loading">加载评分详情...</div>;
    }

    if (explainError) {
      return <div className="error">{explainError}</div>;
    }

    if (!explainData) {
      return <div className="sidebar-loading">准备加载...</div>;
    }

    const renderScoreComponent = (comp: SScoreComponent | PScoreComponent) => (
      <div className="score-component">
        <div className="score-component-header">
          <span className="comp-name">{comp.name}</span>
          <span className="comp-score">{comp.score.toFixed(1)}</span>
        </div>
        <div className="score-component-detail">
          <div className="comp-row">
            <span className="comp-label">输入值:</span>
            <span className="comp-value">{comp.input_value}</span>
          </div>
          <div className="comp-row">
            <span className="comp-label">权重:</span>
            <span className="comp-value">{(comp.weight * 100).toFixed(0)}%</span>
          </div>
          <div className="comp-row">
            <span className="comp-label">贡献:</span>
            <span className="comp-value contribution">{comp.contribution.toFixed(2)}</span>
          </div>
        </div>
        <div className="score-component-rule">
          {comp.rule_description}
        </div>
      </div>
    );

    return (
      <div className="explain-content">
        {/* 验证状态 */}
        <div className={`verification-badge ${explainData.all_verifications_passed ? 'passed' : 'failed'}`}>
          {explainData.all_verifications_passed ? '✓ 计算验证通过' : '⚠ 计算验证异常'}
        </div>

        {/* S-Score 拆分 */}
        <div className="explain-section">
          <div className="explain-section-header">
            <h4>S-Score 战略价值</h4>
            <span className="section-total">{explainData.s_score.toFixed(2)}</span>
          </div>
          <div className="explain-section-content">
            {renderScoreComponent(explainData.s_score_explain.s1_customer_level)}
            {renderScoreComponent(explainData.s_score_explain.s2_margin)}
            {renderScoreComponent(explainData.s_score_explain.s3_urgency)}
          </div>
        </div>

        {/* P-Score 拆分 */}
        <div className="explain-section">
          <div className="explain-section-header">
            <h4>P-Score 生产难度</h4>
            <span className="section-total">{explainData.p_score.toFixed(2)}</span>
          </div>
          <div className="explain-section-content">
            {renderScoreComponent(explainData.p_score_explain.p1_difficulty)}
            {renderScoreComponent(explainData.p_score_explain.p2_aggregation)}
            {renderScoreComponent(explainData.p_score_explain.p3_rhythm)}
          </div>
        </div>

        {/* 综合计算 */}
        <div className="explain-section final">
          <div className="explain-section-header">
            <h4>综合优先级</h4>
            <span className="section-total final-priority">{explainData.final_priority.toFixed(2)}</span>
          </div>
          <div className="explain-section-content">
            <div className="final-calc">
              <div className="calc-row">
                <span>S-Score × ws:</span>
                <span>{explainData.s_score.toFixed(2)} × {explainData.ws.toFixed(2)} = {(explainData.s_score * explainData.ws).toFixed(2)}</span>
              </div>
              <div className="calc-row">
                <span>P-Score × wp:</span>
                <span>{explainData.p_score.toFixed(2)} × {explainData.wp.toFixed(2)} = {(explainData.p_score * explainData.wp).toFixed(2)}</span>
              </div>
              <div className="calc-row base">
                <span>基础优先级:</span>
                <span>{explainData.base_priority.toFixed(2)}</span>
              </div>
              {explainData.alpha && (
                <div className="calc-row alpha">
                  <span>× Alpha ({explainData.alpha.toFixed(2)}):</span>
                  <span>{explainData.final_priority.toFixed(2)}</span>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* 公式汇总 */}
        <div className="formula-summary">
          <h5>计算公式</h5>
          <pre>{explainData.formula_summary}</pre>
        </div>
      </div>
    );
  };

  return (
    <div className="contract-detail-sidebar">
      <div className="sidebar-header">
        <h3>{contract ? `合同 ${contract.contract_id}` : "合同详情"}</h3>
        <button className="sidebar-close-btn" onClick={onClose}>
          ×
        </button>
      </div>

      {!contract ? (
        <div className="sidebar-content">
          <div className="sidebar-empty">
            <div className="sidebar-empty-icon">📋</div>
            <div className="sidebar-empty-text">
              点击表格行查看合同详情
            </div>
          </div>
        </div>
      ) : (
        <>
          <div className="sidebar-tabs">
            <button
              className={`sidebar-tab ${activeTab === "detail" ? "active" : ""}`}
              onClick={() => setActiveTab("detail")}
            >
              详情 & 调整
            </button>
            <button
              className={`sidebar-tab ${activeTab === "explain" ? "active" : ""}`}
              onClick={() => setActiveTab("explain")}
            >
              评分拆分
            </button>
            <button
              className={`sidebar-tab ${activeTab === "history" ? "active" : ""}`}
              onClick={() => setActiveTab("history")}
            >
              干预历史
            </button>
          </div>

          <div className="sidebar-content">
            {activeTab === "detail" && renderDetailTab()}
            {activeTab === "explain" && renderExplainTab()}
            {activeTab === "history" && renderHistoryTab()}
          </div>
        </>
      )}
    </div>
  );
}
