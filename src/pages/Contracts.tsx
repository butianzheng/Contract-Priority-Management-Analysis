import { useState, useEffect } from "react";
// import { FixedSizeList } from "react-window";  // 暂时禁用虚拟列表
import { api, ContractPriority } from "../api/tauri";
import { ContractDetailSidebar } from "../components/ContractDetailSidebar";
import { FilterPanel } from "../components/FilterPanel";
import { BatchOperationsDialog } from "../components/BatchOperationsDialog";

export function Contracts() {
  const [contracts, setContracts] = useState<ContractPriority[]>([]);
  const [filteredContracts, setFilteredContracts] = useState<ContractPriority[]>([]);
  const [strategies, setStrategies] = useState<string[]>([]);
  const [selectedStrategy, setSelectedStrategy] = useState<string>("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");
  const [isFiltered, setIsFiltered] = useState(false);

  // 侧栏状态
  const [selectedContract, setSelectedContract] = useState<ContractPriority | null>(null);
  const [showSidebar, setShowSidebar] = useState(true);

  // 筛选器状态
  const [showFilterPanel, setShowFilterPanel] = useState(false);

  // 批量操作状态
  const [selectedContractIds, setSelectedContractIds] = useState<Set<string>>(new Set());
  const [showBatchOperations, setShowBatchOperations] = useState(false);

  // 加载策略列表
  useEffect(() => {
    api
      .getStrategies()
      .then((data) => {
        setStrategies(data);
        if (data.length > 0) {
          setSelectedStrategy(data[0]);
        }
      })
      .catch((err) => setError(`加载策略失败: ${err}`));
  }, []);

  // 当策略变化时，重新计算优先级
  useEffect(() => {
    if (selectedStrategy) {
      loadPriorities();
    }
  }, [selectedStrategy]);

  const loadPriorities = async () => {
    if (!selectedStrategy) return;

    setLoading(true);
    setError("");

    try {
      const data = await api.computeAllPriorities(selectedStrategy);
      setContracts(data);
      setFilteredContracts(data);
      setIsFiltered(false);

      // 如果当前选中的合同已更新，也更新侧栏
      if (selectedContract) {
        const updated = data.find(c => c.contract_id === selectedContract.contract_id);
        if (updated) {
          setSelectedContract(updated);
        }
      }
    } catch (err) {
      setError(`计算优先级失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const handleFilter = (filtered: ContractPriority[]) => {
    setFilteredContracts(filtered);
    setIsFiltered(true);
    setShowFilterPanel(false);
  };

  const clearFilter = () => {
    setFilteredContracts(contracts);
    setIsFiltered(false);
  };

  const displayContracts = isFiltered ? filteredContracts : contracts;

  // 批量操作相关函数
  const toggleSelectContract = (contractId: string) => {
    setSelectedContractIds((prev) => {
      const newSet = new Set(prev);
      if (newSet.has(contractId)) {
        newSet.delete(contractId);
      } else {
        newSet.add(contractId);
      }
      return newSet;
    });
  };

  const selectAllContracts = () => {
    setSelectedContractIds(new Set(displayContracts.map((c) => c.contract_id)));
  };

  const clearSelection = () => {
    setSelectedContractIds(new Set());
  };

  const getSelectedContracts = (): ContractPriority[] => {
    return displayContracts.filter((c) => selectedContractIds.has(c.contract_id));
  };

  const handleBatchOperationSuccess = () => {
    clearSelection();
    loadPriorities();
  };

  const handleRowClick = (contract: ContractPriority) => {
    setSelectedContract(contract);
    setShowSidebar(true);
  };

  const handleCloseSidebar = () => {
    setSelectedContract(null);
  };

  return (
    <div className="contracts-page">
      <div className="contracts-main">
        <div className="page-header">
          <h1 className="page-header__title">合同优先级中心</h1>
          <p className="page-header__subtitle">管理和查看合同优先级排序</p>
        </div>

        {/* 工具栏 */}
        <div className="contracts-toolbar">
          <div className="toolbar-left">
            <label className="strategy-select">
              <span>策略：</span>
              <select
                value={selectedStrategy}
                onChange={(e) => setSelectedStrategy(e.target.value)}
                disabled={loading}
              >
                {strategies.map((s) => (
                  <option key={s} value={s}>
                    {s}
                  </option>
                ))}
              </select>
            </label>
            <button className="btn btn--primary" onClick={loadPriorities} disabled={loading}>
              {loading ? "计算中..." : "刷新"}
            </button>
            <button className="btn" onClick={() => setShowFilterPanel(true)} disabled={loading}>
              🔍 筛选
            </button>
            {isFiltered && (
              <button className="btn btn--danger" onClick={clearFilter} disabled={loading}>
                ✕ 清除筛选 ({filteredContracts.length}/{contracts.length})
              </button>
            )}
          </div>

          <div className="toolbar-right">
            {selectedContractIds.size > 0 ? (
              <>
                <button
                  className="btn btn--success"
                  onClick={() => setShowBatchOperations(true)}
                  disabled={loading}
                >
                  ✓ 批量操作 ({selectedContractIds.size})
                </button>
                <button className="btn" onClick={clearSelection} disabled={loading}>
                  清除选择
                </button>
              </>
            ) : (
              displayContracts.length > 0 && (
                <button className="btn" onClick={selectAllContracts} disabled={loading}>
                  全选
                </button>
              )
            )}
            <button
              className={`btn ${showSidebar ? 'btn--active' : ''}`}
              onClick={() => setShowSidebar(!showSidebar)}
              title={showSidebar ? "隐藏侧栏" : "显示侧栏"}
            >
              {showSidebar ? "◀" : "▶"} 详情
            </button>
          </div>
        </div>

        {error && <div className="error-message">{error}</div>}

        {/* 合同表格 */}
        <div className="contracts-table-container">
          <table className="contracts-table">
            <thead>
              <tr>
                <th>
                  <input
                    type="checkbox"
                    checked={selectedContractIds.size > 0 && selectedContractIds.size === displayContracts.length}
                    onChange={(e) => {
                      if (e.target.checked) {
                        selectAllContracts();
                      } else {
                        clearSelection();
                      }
                    }}
                  />
                </th>
                <th>排名</th>
                <th>合同编号</th>
                <th>客户</th>
                <th>钢种</th>
                <th>厚度</th>
                <th>宽度</th>
                <th>规格族</th>
                <th>交期</th>
                <th>剩余天数</th>
                <th>S分数</th>
                <th>P分数</th>
                <th>优先级</th>
              </tr>
            </thead>
            <tbody>
              {displayContracts.length === 0 ? (
                <tr>
                  <td colSpan={13} style={{ textAlign: "center" }}>
                    {loading ? "加载中..." : "暂无数据"}
                  </td>
                </tr>
              ) : (
                displayContracts.map((contract, index) => (
                  <tr
                    key={contract.contract_id}
                    className={`
                      ${selectedContractIds.has(contract.contract_id) ? "selected" : ""}
                      ${selectedContract?.contract_id === contract.contract_id ? "active" : ""}
                    `}
                    onClick={() => handleRowClick(contract)}
                  >
                    <td onClick={(e) => e.stopPropagation()}>
                      <input
                        type="checkbox"
                        checked={selectedContractIds.has(contract.contract_id)}
                        onChange={() => toggleSelectContract(contract.contract_id)}
                      />
                    </td>
                    <td>{index + 1}</td>
                    <td>{contract.contract_id}</td>
                    <td>{contract.customer_id}</td>
                    <td>{contract.steel_grade}</td>
                    <td>{contract.thickness.toFixed(1)}</td>
                    <td>{contract.width.toFixed(0)}</td>
                    <td>{contract.spec_family}</td>
                    <td>{contract.pdd}</td>
                    <td
                      className={
                        contract.days_to_pdd <= 3 ? "urgent" : contract.days_to_pdd <= 7 ? "warning" : ""
                      }
                    >
                      {contract.days_to_pdd}天
                    </td>
                    <td>{contract.s_score.toFixed(1)}</td>
                    <td>{contract.p_score.toFixed(1)}</td>
                    <td className="priority">
                      {contract.priority.toFixed(2)}
                      {contract.alpha && <span className="alpha-indicator">★</span>}
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>

        {/* 页脚统计 */}
        <div className="contracts-footer">
          {isFiltered && `筛选后: ${displayContracts.length} 条 / `}
          共 {contracts.length} 条合同 | 当前策略: {selectedStrategy}
        </div>
      </div>

      {/* 右侧详情侧栏 */}
      {showSidebar && (
        <ContractDetailSidebar
          contract={selectedContract}
          strategy={selectedStrategy}
          onClose={handleCloseSidebar}
          onAlphaChanged={loadPriorities}
        />
      )}

      {/* 筛选面板弹窗 */}
      {showFilterPanel && (
        <FilterPanel
          contracts={contracts}
          onFilter={handleFilter}
          onClose={() => setShowFilterPanel(false)}
        />
      )}

      {/* 批量操作弹窗 */}
      {showBatchOperations && (
        <BatchOperationsDialog
          selectedContracts={getSelectedContracts()}
          onClose={() => setShowBatchOperations(false)}
          onSuccess={handleBatchOperationSuccess}
        />
      )}

      <style>{`
        .contracts-page {
          display: flex;
          height: calc(100vh - var(--header-height) - var(--spacing-lg) * 2);
          gap: 0;
        }

        .contracts-main {
          flex: 1;
          display: flex;
          flex-direction: column;
          min-width: 0;
          padding-right: var(--spacing-md);
        }

        .contracts-toolbar {
          display: flex;
          justify-content: space-between;
          align-items: center;
          flex-wrap: wrap;
          gap: var(--spacing-md);
          margin-bottom: var(--spacing-md);
          padding: var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          box-shadow: var(--shadow-sm);
        }

        .toolbar-left, .toolbar-right {
          display: flex;
          align-items: center;
          gap: var(--spacing-sm);
        }

        .strategy-select {
          display: flex;
          align-items: center;
          gap: var(--spacing-xs);
        }

        .strategy-select select {
          padding: var(--spacing-xs) var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          background: var(--color-bg-container);
          font-size: var(--font-size-base);
        }

        .btn {
          padding: var(--spacing-xs) var(--spacing-md);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          background: var(--color-bg-container);
          cursor: pointer;
          font-size: var(--font-size-base);
          transition: all var(--transition-fast);
        }

        .btn:hover:not(:disabled) {
          border-color: var(--color-primary);
          color: var(--color-primary);
        }

        .btn:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .btn--primary {
          background: var(--color-primary);
          border-color: var(--color-primary);
          color: #fff;
        }

        .btn--primary:hover:not(:disabled) {
          background: var(--color-primary-hover);
          border-color: var(--color-primary-hover);
          color: #fff;
        }

        .btn--success {
          background: var(--color-success);
          border-color: var(--color-success);
          color: #fff;
        }

        .btn--danger {
          background: var(--color-error);
          border-color: var(--color-error);
          color: #fff;
        }

        .btn--active {
          background: var(--color-primary-light);
          border-color: var(--color-primary);
          color: var(--color-primary);
        }

        .contracts-table-container {
          flex: 1;
          overflow: auto;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          box-shadow: var(--shadow-sm);
        }

        .contracts-table {
          width: 100%;
          border-collapse: collapse;
          font-size: var(--font-size-sm);
        }

        .contracts-table th,
        .contracts-table td {
          padding: var(--spacing-sm) var(--spacing-md);
          text-align: left;
          border-bottom: 1px solid var(--color-border-light);
        }

        .contracts-table th {
          background: var(--color-bg-layout);
          font-weight: 600;
          position: sticky;
          top: 0;
          z-index: 1;
        }

        .contracts-table tbody tr {
          cursor: pointer;
          transition: background var(--transition-fast);
        }

        .contracts-table tbody tr:hover {
          background: var(--color-primary-light);
        }

        .contracts-table tbody tr.selected {
          background: #e6f7ff;
        }

        .contracts-table tbody tr.active {
          background: #bae7ff;
        }

        .contracts-table .urgent {
          color: var(--color-error);
          font-weight: 600;
        }

        .contracts-table .warning {
          color: var(--color-warning);
          font-weight: 500;
        }

        .contracts-table .priority {
          font-weight: 600;
          color: var(--color-primary);
        }

        .alpha-indicator {
          color: var(--color-warning);
          margin-left: 4px;
        }

        .contracts-footer {
          padding: var(--spacing-md);
          text-align: center;
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          margin-top: var(--spacing-md);
          box-shadow: var(--shadow-sm);
        }

        .error-message {
          background: #fff2f0;
          border: 1px solid #ffccc7;
          color: var(--color-error);
          padding: var(--spacing-md);
          border-radius: var(--border-radius-md);
          margin-bottom: var(--spacing-md);
        }
      `}</style>
    </div>
  );
}
