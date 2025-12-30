import { useState, useEffect } from "react";
import { api, StrategyWeights } from "../api/tauri";

interface TemplatePreset {
  name: string;
  description: string;
  ws: number;
  wp: number;
}

const PRESET_TEMPLATES: TemplatePreset[] = [
  { name: "均衡策略", description: "S-Score 和 P-Score 权重相等", ws: 0.5, wp: 0.5 },
  { name: "客户优先", description: "侧重客户战略价值", ws: 0.7, wp: 0.3 },
  { name: "生产优先", description: "侧重生产难度优化", ws: 0.3, wp: 0.7 },
  { name: "极端S策略", description: "完全依据客户战略价值", ws: 1.0, wp: 0.0 },
  { name: "极端P策略", description: "完全依据生产难度", ws: 0.0, wp: 1.0 },
];

export function StrategyTemplates() {
  const [strategies, setStrategies] = useState<StrategyWeights[]>([]);
  const [loading, setLoading] = useState(false);
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [editingStrategy, setEditingStrategy] = useState<StrategyWeights | null>(null);
  const [newStrategyName, setNewStrategyName] = useState("");
  const [newWs, setNewWs] = useState(0.5);
  const [newWp, setNewWp] = useState(0.5);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    loadStrategies();
  }, []);

  const loadStrategies = async () => {
    setLoading(true);
    try {
      const data = await api.getStrategyWeightsList();
      setStrategies(data);
    } catch (err) {
      console.error("Failed to load strategies:", err);
    } finally {
      setLoading(false);
    }
  };

  const applyTemplate = (template: TemplatePreset) => {
    setEditingStrategy(null);
    setNewStrategyName(template.name);
    setNewWs(template.ws);
    setNewWp(template.wp);
    setShowCreateModal(true);
  };

  const handleEdit = (strategy: StrategyWeights) => {
    setEditingStrategy(strategy);
    setNewStrategyName(strategy.strategy_name);
    setNewWs(strategy.ws);
    setNewWp(strategy.wp);
    setShowCreateModal(true);
  };

  const handleOpenCreate = () => {
    setEditingStrategy(null);
    setNewStrategyName("");
    setNewWs(0.5);
    setNewWp(0.5);
    setShowCreateModal(true);
  };

  const handleCloseModal = () => {
    setShowCreateModal(false);
    setEditingStrategy(null);
    setNewStrategyName("");
    setNewWs(0.5);
    setNewWp(0.5);
  };

  const handleCreate = async () => {
    if (!newStrategyName.trim()) {
      alert("请输入策略名称");
      return;
    }

    setSaving(true);
    try {
      await api.upsertStrategyWeight({
        strategy_name: newStrategyName,
        ws: newWs,
        wp: newWp,
      });
      handleCloseModal();
      loadStrategies();
    } catch (err) {
      alert(`保存失败: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDelete = async (strategyName: string) => {
    if (!confirm(`确认删除策略 "${strategyName}"？`)) return;

    try {
      await api.deleteStrategyWeight(strategyName);
      loadStrategies();
    } catch (err) {
      alert(`删除失败: ${err}`);
    }
  };

  const weightSum = newWs + newWp;
  const isWeightValid = Math.abs(weightSum - 1.0) < 0.01;
  const isEditing = editingStrategy !== null;

  return (
    <div className="strategy-templates-page">
      <div className="page-header">
        <h1 className="page-header__title">策略模板</h1>
        <p className="page-header__subtitle">管理和创建策略权重模板</p>
      </div>

      {/* Template Presets */}
      <div className="templates-section">
        <h3>预设模板</h3>
        <div className="preset-grid">
          {PRESET_TEMPLATES.map((template) => (
            <div key={template.name} className="preset-card" onClick={() => applyTemplate(template)}>
              <div className="preset-name">{template.name}</div>
              <div className="preset-desc">{template.description}</div>
              <div className="preset-weights">
                <span>ws: {template.ws}</span>
                <span>wp: {template.wp}</span>
              </div>
              <button className="btn-apply">应用模板</button>
            </div>
          ))}
        </div>
      </div>

      {/* Current Strategies */}
      <div className="strategies-section">
        <div className="section-header">
          <h3>当前策略</h3>
          <button className="btn-create" onClick={handleOpenCreate}>
            + 创建新策略
          </button>
        </div>

        {loading ? (
          <div className="loading-state">加载中...</div>
        ) : strategies.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">📁</div>
            <div className="empty-state-text">暂无策略，点击上方模板快速创建</div>
          </div>
        ) : (
          <div className="strategies-table-container">
            <table className="strategies-table">
              <thead>
                <tr>
                  <th>策略名称</th>
                  <th>S-Score 权重 (ws)</th>
                  <th>P-Score 权重 (wp)</th>
                  <th>权重和</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {strategies.map((s) => (
                  <tr key={s.strategy_name}>
                    <td className="strategy-name">{s.strategy_name}</td>
                    <td>
                      <div className="weight-bar">
                        <div className="weight-fill ws" style={{ width: `${s.ws * 100}%` }}></div>
                        <span>{s.ws.toFixed(2)}</span>
                      </div>
                    </td>
                    <td>
                      <div className="weight-bar">
                        <div className="weight-fill wp" style={{ width: `${s.wp * 100}%` }}></div>
                        <span>{s.wp.toFixed(2)}</span>
                      </div>
                    </td>
                    <td className={Math.abs(s.ws + s.wp - 1) < 0.01 ? "valid" : "invalid"}>
                      {(s.ws + s.wp).toFixed(2)}
                    </td>
                    <td className="action-buttons">
                      <button className="btn-edit" onClick={() => handleEdit(s)}>
                        编辑
                      </button>
                      <button className="btn-delete" onClick={() => handleDelete(s.strategy_name)}>
                        删除
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Create/Edit Modal */}
      {showCreateModal && (
        <div className="modal-overlay" onClick={handleCloseModal}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <div className="modal-header">
              <h2>{isEditing ? "编辑策略" : "创建策略"}</h2>
              <button className="close-btn" onClick={handleCloseModal}>×</button>
            </div>
            <div className="modal-body">
              <div className="form-group">
                <label>策略名称</label>
                <input
                  type="text"
                  value={newStrategyName}
                  onChange={(e) => setNewStrategyName(e.target.value)}
                  placeholder="输入策略名称"
                  disabled={isEditing}
                />
              </div>
              <div className="form-group">
                <label>S-Score 权重 (ws): {newWs.toFixed(2)}</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={newWs}
                  onChange={(e) => setNewWs(parseFloat(e.target.value))}
                />
              </div>
              <div className="form-group">
                <label>P-Score 权重 (wp): {newWp.toFixed(2)}</label>
                <input
                  type="range"
                  min="0"
                  max="1"
                  step="0.05"
                  value={newWp}
                  onChange={(e) => setNewWp(parseFloat(e.target.value))}
                />
              </div>
              <div className={`weight-sum ${isWeightValid ? "valid" : "invalid"}`}>
                权重和: {weightSum.toFixed(2)} {isWeightValid ? "✓" : "✗ (应为1.0)"}
              </div>
            </div>
            <div className="modal-footer">
              <button className="btn-secondary" onClick={handleCloseModal}>
                取消
              </button>
              <button className="btn-primary" onClick={handleCreate} disabled={saving}>
                {saving ? "保存中..." : "保存"}
              </button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .strategy-templates-page {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-lg);
        }

        .templates-section, .strategies-section {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
        }

        .templates-section h3, .strategies-section h3 {
          margin: 0 0 var(--spacing-md) 0;
          font-size: var(--font-size-lg);
        }

        .preset-grid {
          display: grid;
          grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
          gap: var(--spacing-md);
        }

        .preset-card {
          background: var(--color-bg-layout);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-md);
          padding: var(--spacing-md);
          cursor: pointer;
          transition: all var(--transition-fast);
        }

        .preset-card:hover {
          border-color: var(--color-primary);
          transform: translateY(-2px);
        }

        .preset-name {
          font-weight: 600;
          margin-bottom: var(--spacing-xs);
        }

        .preset-desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          margin-bottom: var(--spacing-sm);
        }

        .preset-weights {
          display: flex;
          gap: var(--spacing-md);
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
          margin-bottom: var(--spacing-sm);
        }

        .btn-apply {
          width: 100%;
          padding: var(--spacing-xs);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .section-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: var(--spacing-md);
        }

        .section-header h3 {
          margin: 0;
        }

        .btn-create {
          padding: var(--spacing-xs) var(--spacing-md);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .strategies-table-container {
          overflow-x: auto;
        }

        .strategies-table {
          width: 100%;
          border-collapse: collapse;
        }

        .strategies-table th,
        .strategies-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .strategies-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
        }

        .strategy-name {
          font-weight: 600;
        }

        .weight-bar {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
        }

        .weight-fill {
          height: 8px;
          border-radius: 4px;
        }

        .weight-fill.ws {
          background: var(--color-primary);
        }

        .weight-fill.wp {
          background: var(--color-success);
        }

        .valid {
          color: var(--color-success);
        }

        .invalid {
          color: var(--color-error);
        }

        .btn-delete {
          padding: var(--spacing-xs) var(--spacing-sm);
          background: transparent;
          border: 1px solid var(--color-error);
          color: var(--color-error);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .btn-delete:hover {
          background: var(--color-error);
          color: #fff;
        }

        .btn-edit {
          padding: var(--spacing-xs) var(--spacing-sm);
          background: transparent;
          border: 1px solid var(--color-primary);
          color: var(--color-primary);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .btn-edit:hover {
          background: var(--color-primary);
          color: #fff;
        }

        .action-buttons {
          display: flex;
          gap: var(--spacing-xs);
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
          width: 400px;
          max-width: 90vw;
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
        }

        .form-group {
          margin-bottom: var(--spacing-md);
        }

        .form-group label {
          display: block;
          margin-bottom: var(--spacing-xs);
          font-weight: 500;
        }

        .form-group input[type="text"] {
          width: 100%;
          padding: var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
        }

        .form-group input[type="text"]:disabled {
          background: var(--color-bg-layout);
          color: var(--color-text-tertiary);
          cursor: not-allowed;
        }

        .form-group input[type="range"] {
          width: 100%;
        }

        .weight-sum {
          padding: var(--spacing-sm);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
          text-align: center;
        }

        .modal-footer {
          display: flex;
          justify-content: flex-end;
          gap: var(--spacing-sm);
          padding: var(--spacing-md) var(--spacing-lg);
          border-top: 1px solid var(--color-border-light);
        }

        .btn-secondary {
          padding: var(--spacing-sm) var(--spacing-md);
          background: var(--color-bg-container);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .btn-primary {
          padding: var(--spacing-sm) var(--spacing-md);
          background: var(--color-primary);
          color: #fff;
          border: none;
          border-radius: var(--border-radius-sm);
          cursor: pointer;
        }

        .loading-state, .empty-state {
          text-align: center;
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
