import { useState, useEffect } from "react";
import { api, BatchOperation, ContractPriority } from "../api/tauri";
import "./BatchOperationsDialog.css";

interface BatchOperationsDialogProps {
  selectedContracts: ContractPriority[];
  onClose: () => void;
  onSuccess: () => void;
}

type TabType = "adjust" | "restore" | "history";

export function BatchOperationsDialog({
  selectedContracts,
  onClose,
  onSuccess,
}: BatchOperationsDialogProps) {
  const [activeTab, setActiveTab] = useState<TabType>("adjust");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");

  // 批量调整状态
  const [adjustAlpha, setAdjustAlpha] = useState<string>("1.2");
  const [adjustReason, setAdjustReason] = useState<string>("");
  const [adjustUser, setAdjustUser] = useState<string>("admin");

  // 批量恢复状态
  const [restoreReason, setRestoreReason] = useState<string>("");
  const [restoreUser, setRestoreUser] = useState<string>("admin");

  // 批量历史状态
  const [history, setHistory] = useState<BatchOperation[]>([]);
  const [selectedBatch, setSelectedBatch] = useState<number | null>(null);
  const [batchContracts, setBatchContracts] = useState<string[]>([]);
  const [historyLimit, setHistoryLimit] = useState<number>(50);

  useEffect(() => {
    if (activeTab === "history") {
      loadHistory();
    }
  }, [activeTab, historyLimit]);

  const loadHistory = async () => {
    setLoading(true);
    setError("");
    try {
      const data = await api.getBatchOperations(historyLimit);
      setHistory(data);
    } catch (err) {
      setError(`加载历史失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const loadBatchContracts = async (batchId: number) => {
    try {
      const contracts = await api.getBatchOperationContracts(batchId);
      setBatchContracts(contracts);
      setSelectedBatch(batchId);
    } catch (err) {
      setError(`加载批量操作合同失败: ${err}`);
    }
  };

  const handleAdjust = async () => {
    const alpha = parseFloat(adjustAlpha);
    if (isNaN(alpha) || alpha < 0.5 || alpha > 2.0) {
      alert("Alpha 值必须在 0.5 ~ 2.0 之间");
      return;
    }

    if (!adjustReason.trim()) {
      alert("请填写调整原因");
      return;
    }

    if (!confirm(`确认批量调整 ${selectedContracts.length} 个合同的优先级？\nalpha 值: ${alpha}\n原因: ${adjustReason}`)) {
      return;
    }

    setLoading(true);
    setError("");

    try {
      const contractIds = selectedContracts.map((c) => c.contract_id);
      await api.batchAdjustAlpha(contractIds, alpha, adjustReason, adjustUser);
      alert("批量调整成功！");
      onSuccess();
      onClose();
    } catch (err) {
      setError(`批量调整失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleRestore = async () => {
    if (!restoreReason.trim()) {
      alert("请填写恢复原因");
      return;
    }

    if (!confirm(`确认恢复 ${selectedContracts.length} 个合同的优先级到原始值？\n原因: ${restoreReason}`)) {
      return;
    }

    setLoading(true);
    setError("");

    try {
      const contractIds = selectedContracts.map((c) => c.contract_id);
      await api.batchRestoreAlpha(contractIds, restoreReason, restoreUser);
      alert("批量恢复成功！");
      onSuccess();
      onClose();
    } catch (err) {
      setError(`批量恢复失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content batch-operations-dialog"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h2>批量操作</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {/* 选项卡 */}
          <div className="tabs">
            <button
              className={`tab ${activeTab === "adjust" ? "active" : ""}`}
              onClick={() => setActiveTab("adjust")}
            >
              批量调整 ({selectedContracts.length})
            </button>
            <button
              className={`tab ${activeTab === "restore" ? "active" : ""}`}
              onClick={() => setActiveTab("restore")}
            >
              批量恢复 ({selectedContracts.length})
            </button>
            <button
              className={`tab ${activeTab === "history" ? "active" : ""}`}
              onClick={() => setActiveTab("history")}
            >
              操作历史
            </button>
          </div>

          {error && <div className="error">{error}</div>}

          {/* 批量调整页面 */}
          {activeTab === "adjust" && (
            <div className="tab-content">
              <div className="info-box">
                <p>选中的合同：{selectedContracts.length} 个</p>
                <div className="selected-contracts">
                  {selectedContracts.slice(0, 5).map((c) => (
                    <span key={c.contract_id} className="contract-tag">
                      {c.contract_id}
                    </span>
                  ))}
                  {selectedContracts.length > 5 && (
                    <span className="contract-tag more">
                      +{selectedContracts.length - 5}
                    </span>
                  )}
                </div>
              </div>

              <div className="form-section">
                <div className="form-row">
                  <label>
                    Alpha 值 (0.5 ~ 2.0):
                    <input
                      type="number"
                      value={adjustAlpha}
                      onChange={(e) => setAdjustAlpha(e.target.value)}
                      min="0.5"
                      max="2.0"
                      step="0.1"
                      placeholder="例如: 1.2"
                    />
                  </label>
                  <div className="help-text">
                    1.0 为默认值，大于1提高优先级，小于1降低优先级
                  </div>
                </div>

                <div className="form-row">
                  <label>
                    调整原因 *:
                    <textarea
                      value={adjustReason}
                      onChange={(e) => setAdjustReason(e.target.value)}
                      placeholder="请说明批量调整的原因"
                      rows={3}
                    />
                  </label>
                </div>

                <div className="form-row">
                  <label>
                    操作人:
                    <input
                      type="text"
                      value={adjustUser}
                      onChange={(e) => setAdjustUser(e.target.value)}
                    />
                  </label>
                </div>

                <button
                  className="btn-primary btn-large"
                  onClick={handleAdjust}
                  disabled={loading}
                >
                  {loading ? "处理中..." : "确认批量调整"}
                </button>
              </div>
            </div>
          )}

          {/* 批量恢复页面 */}
          {activeTab === "restore" && (
            <div className="tab-content">
              <div className="info-box warning">
                <p>⚠️ 注意：此操作将恢复选中合同的优先级到原始计算值</p>
                <p>选中的合同：{selectedContracts.length} 个</p>
                <div className="selected-contracts">
                  {selectedContracts.slice(0, 5).map((c) => (
                    <span key={c.contract_id} className="contract-tag">
                      {c.contract_id}
                    </span>
                  ))}
                  {selectedContracts.length > 5 && (
                    <span className="contract-tag more">
                      +{selectedContracts.length - 5}
                    </span>
                  )}
                </div>
              </div>

              <div className="form-section">
                <div className="form-row">
                  <label>
                    恢复原因 *:
                    <textarea
                      value={restoreReason}
                      onChange={(e) => setRestoreReason(e.target.value)}
                      placeholder="请说明批量恢复的原因"
                      rows={3}
                    />
                  </label>
                </div>

                <div className="form-row">
                  <label>
                    操作人:
                    <input
                      type="text"
                      value={restoreUser}
                      onChange={(e) => setRestoreUser(e.target.value)}
                    />
                  </label>
                </div>

                <button
                  className="btn-danger btn-large"
                  onClick={handleRestore}
                  disabled={loading}
                >
                  {loading ? "处理中..." : "确认批量恢复"}
                </button>
              </div>
            </div>
          )}

          {/* 操作历史页面 */}
          {activeTab === "history" && (
            <div className="tab-content">
              <div className="controls-bar">
                <label>
                  显示条数：
                  <select
                    value={historyLimit}
                    onChange={(e) => setHistoryLimit(parseInt(e.target.value))}
                  >
                    <option value={20}>最近 20 条</option>
                    <option value={50}>最近 50 条</option>
                    <option value={100}>最近 100 条</option>
                  </select>
                </label>
              </div>

              {loading ? (
                <div className="loading">加载中...</div>
              ) : history.length === 0 ? (
                <div className="no-data">暂无批量操作历史</div>
              ) : (
                <div className="history-list">
                  <table>
                    <thead>
                      <tr>
                        <th>ID</th>
                        <th>操作类型</th>
                        <th>合同数量</th>
                        <th>原因</th>
                        <th>操作人</th>
                        <th>时间</th>
                        <th>操作</th>
                      </tr>
                    </thead>
                    <tbody>
                      {history.map((batch) => (
                        <tr key={batch.batch_id}>
                          <td>#{batch.batch_id}</td>
                          <td>
                            <span
                              className={`type-badge ${batch.operation_type}`}
                            >
                              {batch.operation_type === "adjust"
                                ? "调整"
                                : "恢复"}
                            </span>
                          </td>
                          <td>{batch.contract_count}</td>
                          <td className="reason-cell">{batch.reason}</td>
                          <td>{batch.user}</td>
                          <td className="time-cell">
                            {batch.created_at
                              ? new Date(batch.created_at).toLocaleString(
                                  "zh-CN"
                                )
                              : "-"}
                          </td>
                          <td>
                            <button
                              className="btn-small"
                              onClick={() =>
                                batch.batch_id &&
                                loadBatchContracts(batch.batch_id)
                              }
                            >
                              查看合同
                            </button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>

                  {/* 显示选中批次的合同列表 */}
                  {selectedBatch && batchContracts.length > 0 && (
                    <div className="batch-details">
                      <h4>批次 #{selectedBatch} 涉及的合同：</h4>
                      <div className="contracts-list">
                        {batchContracts.map((contractId) => (
                          <span key={contractId} className="contract-tag">
                            {contractId}
                          </span>
                        ))}
                      </div>
                      <button
                        className="btn-small"
                        onClick={() => {
                          setSelectedBatch(null);
                          setBatchContracts([]);
                        }}
                      >
                        关闭
                      </button>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button onClick={onClose}>关闭</button>
        </div>
      </div>
    </div>
  );
}
