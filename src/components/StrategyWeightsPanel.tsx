import { useState, useEffect } from "react";
import { api, StrategyScoringWeights } from "../api/tauri";
import "./StrategyWeightsPanel.css";

interface StrategyWeightsPanelProps {
  onClose: () => void;
  onWeightsChanged?: () => void;
  embedded?: boolean;
}

export function StrategyWeightsPanel({
  onClose,
  onWeightsChanged,
  embedded = false,
}: StrategyWeightsPanelProps) {
  const [weights, setWeights] = useState<StrategyScoringWeights[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");
  const [editingStrategy, setEditingStrategy] = useState<string | null>(null);
  const [editW1, setEditW1] = useState<number>(0);
  const [editW2, setEditW2] = useState<number>(0);
  const [editW3, setEditW3] = useState<number>(0);

  useEffect(() => {
    loadWeights();
  }, []);

  const loadWeights = async () => {
    setLoading(true);
    setError("");
    try {
      const data = await api.getAllStrategyWeights();
      setWeights(data);
    } catch (err) {
      setError(`加载策略权重失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const startEdit = (weight: StrategyScoringWeights) => {
    setEditingStrategy(weight.strategy_name);
    setEditW1(weight.w1);
    setEditW2(weight.w2);
    setEditW3(weight.w3);
  };

  const cancelEdit = () => {
    setEditingStrategy(null);
  };

  const saveEdit = async (strategyName: string) => {
    // 验证权重和是否为 1
    const sum = editW1 + editW2 + editW3;
    if (Math.abs(sum - 1.0) > 0.001) {
      alert(`权重之和必须为 1.0，当前为 ${sum.toFixed(3)}`);
      return;
    }

    // 验证权重都是非负数
    if (editW1 < 0 || editW2 < 0 || editW3 < 0) {
      alert("权重不能为负数");
      return;
    }

    setLoading(true);
    setError("");

    try {
      await api.updateStrategyWeights(strategyName, editW1, editW2, editW3);
      await loadWeights();
      cancelEdit();
      onWeightsChanged?.();
    } catch (err) {
      setError(`更新策略权重失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const getWeightSum = () => {
    return editW1 + editW2 + editW3;
  };

  const isValidSum = () => {
    return Math.abs(getWeightSum() - 1.0) <= 0.001;
  };

  const content = (
    <>
      <div className="info-box">
        <h3>权重说明</h3>
        <ul>
          <li>
            <strong>w1 - 客户等级权重</strong>: 客户等级（A/B/C）在
            S-Score 中的权重
          </li>
          <li>
            <strong>w2 - 毛利权重</strong>: 合同毛利在 S-Score 中的权重
          </li>
          <li>
            <strong>w3 - 紧急度权重</strong>: 距离交期天数在 S-Score
            中的权重
          </li>
          <li>
            <strong>约束条件</strong>: w1 + w2 + w3 = 1.0，且所有权重 ≥ 0
          </li>
        </ul>
      </div>

      {error && <div className="error">{error}</div>}

      {loading && <div className="loading">加载中...</div>}

      <div className="weights-list">
        <table>
          <thead>
            <tr>
              <th>策略名称</th>
              <th>w1 (客户等级)</th>
              <th>w2 (毛利)</th>
              <th>w3 (紧急度)</th>
              <th>权重和</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {weights.map((weight) => {
              const isEditing = editingStrategy === weight.strategy_name;

              return (
                <tr key={weight.strategy_name}>
                  <td className="strategy-name">{weight.strategy_name}</td>
                  <td>
                    {isEditing ? (
                      <input
                        type="number"
                        value={editW1}
                        onChange={(e) => setEditW1(parseFloat(e.target.value) || 0)}
                        step="0.01"
                        min="0"
                        max="1"
                      />
                    ) : (
                      <span>{weight.w1.toFixed(2)}</span>
                    )}
                  </td>
                  <td>
                    {isEditing ? (
                      <input
                        type="number"
                        value={editW2}
                        onChange={(e) => setEditW2(parseFloat(e.target.value) || 0)}
                        step="0.01"
                        min="0"
                        max="1"
                      />
                    ) : (
                      <span>{weight.w2.toFixed(2)}</span>
                    )}
                  </td>
                  <td>
                    {isEditing ? (
                      <input
                        type="number"
                        value={editW3}
                        onChange={(e) => setEditW3(parseFloat(e.target.value) || 0)}
                        step="0.01"
                        min="0"
                        max="1"
                      />
                    ) : (
                      <span>{weight.w3.toFixed(2)}</span>
                    )}
                  </td>
                  <td>
                    {isEditing ? (
                      <span
                        className={
                          isValidSum() ? "sum-valid" : "sum-invalid"
                        }
                      >
                        {getWeightSum().toFixed(3)}
                        {isValidSum() ? " ✓" : " ✗"}
                      </span>
                    ) : (
                      <span className="sum-valid">
                        {(weight.w1 + weight.w2 + weight.w3).toFixed(3)} ✓
                      </span>
                    )}
                  </td>
                  <td className="actions">
                    {isEditing ? (
                      <>
                        <button
                          className="btn-small btn-primary"
                          onClick={() => saveEdit(weight.strategy_name)}
                          disabled={loading || !isValidSum()}
                        >
                          保存
                        </button>
                        <button
                          className="btn-small"
                          onClick={cancelEdit}
                          disabled={loading}
                        >
                          取消
                        </button>
                      </>
                    ) : (
                      <button
                        className="btn-small"
                        onClick={() => startEdit(weight)}
                        disabled={loading}
                      >
                        编辑
                      </button>
                    )}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </>
  );

  // 嵌入模式
  if (embedded) {
    return <div className="strategy-weights-panel-embedded">{content}</div>;
  }

  // 弹窗模式
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content strategy-weights-panel"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h2>策略评分权重管理</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {content}
        </div>

        <div className="modal-footer">
          <button onClick={onClose} disabled={loading}>
            关闭
          </button>
        </div>
      </div>
    </div>
  );
}
