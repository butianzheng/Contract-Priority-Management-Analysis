import { useState, useEffect, useMemo } from "react";
import { Link } from "react-router-dom";
import { api, Contract, ImportDataType, Customer, ProcessDifficulty } from "../../api/tauri";
import { ImportDialog } from "../../components/ImportDialog";
import { ExportDialog } from "../../components/ExportDialog";
import "./Settings.css";

type DataSourceTab = "contracts" | "customers" | "difficulty" | "specFamily";

interface DataSourceInfo {
  id: DataSourceTab;
  name: string;
  icon: string;
  description: string;
}

const DATA_SOURCES: DataSourceInfo[] = [
  { id: "contracts", name: "合同主数据", icon: "📋", description: "合同基础信息" },
  { id: "customers", name: "客户主数据", icon: "👥", description: "客户分级信息" },
  { id: "difficulty", name: "工艺难度字典", icon: "🔧", description: "生产难度配置" },
  { id: "specFamily", name: "规格族字典", icon: "📦", description: "规格族映射" },
];

// 规格族数据（暂无后端API）
interface SpecFamilyData {
  id: number;
  spec_family: string;
  description: string;
  steel_grades: string[];
}

// 映射 Tab 到 ImportDataType
const TAB_TO_IMPORT_TYPE: Partial<Record<DataSourceTab, ImportDataType>> = {
  contracts: "contracts",
  customers: "customers",
  difficulty: "process_difficulty",
};

export function DataSource() {
  const [activeTab, setActiveTab] = useState<DataSourceTab>("contracts");
  const [contracts, setContracts] = useState<Contract[]>([]);
  const [customers, setCustomers] = useState<Customer[]>([]);
  const [difficulties, setDifficulties] = useState<ProcessDifficulty[]>([]);
  const [loading, setLoading] = useState(false);

  // 导入/导出对话框状态
  const [showImportDialog, setShowImportDialog] = useState(false);
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [importDataType, setImportDataType] = useState<ImportDataType>("contracts");

  // 模拟规格族数据（暂无后端API）
  const [specFamilies] = useState<SpecFamilyData[]>([
    { id: 1, spec_family: "DP", description: "双相钢系列", steel_grades: ["DP590", "DP780", "DP980"] },
    { id: 2, spec_family: "QP", description: "淬火配分钢", steel_grades: ["QP980", "QP1180"] },
    { id: 3, spec_family: "MS", description: "马氏体钢", steel_grades: ["MS1180", "MS1500"] },
    { id: 4, spec_family: "HSLA", description: "高强低合金钢", steel_grades: ["HSLA340", "HSLA420"] },
    { id: 5, spec_family: "IF", description: "无间隙原子钢", steel_grades: ["IF340", "IF260"] },
  ]);

  // 加载数据
  useEffect(() => {
    if (activeTab === "contracts") {
      loadContracts();
    } else if (activeTab === "customers") {
      loadCustomers();
    } else if (activeTab === "difficulty") {
      loadDifficulties();
    }
  }, [activeTab]);

  const loadContracts = async () => {
    setLoading(true);
    try {
      const data = await api.getContracts();
      setContracts(data);
    } catch (err) {
      console.error("加载合同数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  const loadCustomers = async () => {
    setLoading(true);
    try {
      const data = await api.getCustomers();
      setCustomers(data);
    } catch (err) {
      console.error("加载客户数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  const loadDifficulties = async () => {
    setLoading(true);
    try {
      const data = await api.getProcessDifficulty();
      setDifficulties(data);
    } catch (err) {
      console.error("加载工艺难度数据失败:", err);
    } finally {
      setLoading(false);
    }
  };

  // 刷新当前数据
  const handleRefresh = () => {
    if (activeTab === "contracts") loadContracts();
    else if (activeTab === "customers") loadCustomers();
    else if (activeTab === "difficulty") loadDifficulties();
  };

  // 打开导入对话框
  const handleOpenImport = () => {
    const dataType = TAB_TO_IMPORT_TYPE[activeTab];
    if (dataType) {
      setImportDataType(dataType);
      setShowImportDialog(true);
    }
  };

  // 打开导出对话框
  const handleOpenExport = () => {
    const dataType = TAB_TO_IMPORT_TYPE[activeTab];
    if (dataType) {
      setImportDataType(dataType);
      setShowExportDialog(true);
    }
  };

  // 导入完成回调
  const handleImportComplete = () => {
    handleRefresh();
  };

  // 统计数据
  const stats = useMemo(() => {
    return {
      contracts: contracts.length,
      customers: customers.length,
      difficulties: difficulties.length,
      specFamilies: specFamilies.length,
    };
  }, [contracts, customers, difficulties, specFamilies]);

  const renderContractsTable = () => (
    <div className="data-table-container" style={{ maxHeight: "500px", overflow: "auto" }}>
      <table className="data-table">
        <thead>
          <tr>
            <th>合同编号</th>
            <th>客户编号</th>
            <th>钢种</th>
            <th>厚度</th>
            <th>宽度</th>
            <th>规格族</th>
            <th>交期</th>
            <th>剩余天数</th>
          </tr>
        </thead>
        <tbody>
          {contracts.length === 0 ? (
            <tr>
              <td colSpan={8} style={{ textAlign: "center" }}>
                {loading ? "加载中..." : "暂无数据"}
              </td>
            </tr>
          ) : (
            contracts.map((c) => (
              <tr key={c.contract_id}>
                <td>{c.contract_id}</td>
                <td>{c.customer_id}</td>
                <td>{c.steel_grade}</td>
                <td>{c.thickness.toFixed(1)} mm</td>
                <td>{c.width.toFixed(0)} mm</td>
                <td><span className="tag primary">{c.spec_family}</span></td>
                <td>{c.pdd}</td>
                <td className={c.days_to_pdd <= 3 ? "error" : c.days_to_pdd <= 7 ? "warning" : ""}>
                  {c.days_to_pdd} 天
                </td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );

  const renderCustomersTable = () => (
    <div className="data-table-container" style={{ maxHeight: "500px", overflow: "auto" }}>
      <table className="data-table">
        <thead>
          <tr>
            <th>客户编号</th>
            <th>客户名称</th>
            <th>客户等级</th>
            <th>信用等级</th>
            <th>客户分组</th>
          </tr>
        </thead>
        <tbody>
          {customers.length === 0 ? (
            <tr>
              <td colSpan={5} style={{ textAlign: "center" }}>
                {loading ? "加载中..." : "暂无数据"}
              </td>
            </tr>
          ) : (
            customers.map((c) => (
              <tr key={c.customer_id}>
                <td>{c.customer_id}</td>
                <td>{c.customer_name || "-"}</td>
                <td>
                  <span className={`tag ${c.customer_level === "A" ? "success" : c.customer_level === "B" ? "primary" : "default"}`}>
                    {c.customer_level}
                  </span>
                </td>
                <td>{c.credit_level || "-"}</td>
                <td>{c.customer_group || "-"}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );

  const renderDifficultyTable = () => (
    <div className="data-table-container" style={{ maxHeight: "500px", overflow: "auto" }}>
      <table className="data-table">
        <thead>
          <tr>
            <th>ID</th>
            <th>钢种</th>
            <th>厚度范围</th>
            <th>宽度范围</th>
            <th>难度等级</th>
            <th>难度分数</th>
          </tr>
        </thead>
        <tbody>
          {difficulties.length === 0 ? (
            <tr>
              <td colSpan={6} style={{ textAlign: "center" }}>
                {loading ? "加载中..." : "暂无数据"}
              </td>
            </tr>
          ) : (
            difficulties.map((d) => (
              <tr key={d.id}>
                <td>{d.id}</td>
                <td>{d.steel_grade}</td>
                <td>{d.thickness_min} - {d.thickness_max} mm</td>
                <td>{d.width_min} - {d.width_max} mm</td>
                <td>
                  <span className={`tag ${
                    d.difficulty_level === "极高" ? "error" :
                    d.difficulty_level === "高" ? "warning" :
                    d.difficulty_level === "中" ? "primary" : "success"
                  }`}>
                    {d.difficulty_level}
                  </span>
                </td>
                <td>{d.difficulty_score.toFixed(2)}</td>
              </tr>
            ))
          )}
        </tbody>
      </table>
    </div>
  );

  const renderSpecFamilyTable = () => (
    <div className="data-table-container">
      <table className="data-table">
        <thead>
          <tr>
            <th>规格族</th>
            <th>描述</th>
            <th>包含钢种</th>
          </tr>
        </thead>
        <tbody>
          {specFamilies.map((s) => (
            <tr key={s.id}>
              <td><span className="tag primary">{s.spec_family}</span></td>
              <td>{s.description}</td>
              <td>
                <div style={{ display: "flex", gap: "4px", flexWrap: "wrap" }}>
                  {s.steel_grades.map((g) => (
                    <span key={g} className="tag default">{g}</span>
                  ))}
                </div>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );

  return (
    <div className="data-source-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <Link to="/settings">设置</Link>
          <span>/</span>
          <span>数据源管理</span>
        </div>
        <h1 className="page-header__title">数据源管理</h1>
        <p className="page-header__subtitle">管理合同、客户、工艺难度等基础数据</p>
      </div>

      <div className="settings-content">
        {/* 统计概览 */}
        <div className="stats-row">
          {DATA_SOURCES.map((ds) => (
            <div
              key={ds.id}
              className="stat-item"
              style={{ cursor: "pointer" }}
              onClick={() => setActiveTab(ds.id)}
            >
              <div className="stat-item-label">{ds.icon} {ds.name}</div>
              <div className={`stat-item-value ${activeTab === ds.id ? "primary" : ""}`}>
                {ds.id === "contracts" ? stats.contracts :
                 ds.id === "customers" ? stats.customers :
                 ds.id === "difficulty" ? stats.difficulties :
                 stats.specFamilies}
              </div>
            </div>
          ))}
        </div>

        {/* 数据表格区域 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>
              {DATA_SOURCES.find(ds => ds.id === activeTab)?.icon}{" "}
              {DATA_SOURCES.find(ds => ds.id === activeTab)?.name}
            </h3>
            <div className="btn-group">
              <button
                className="settings-btn"
                onClick={handleRefresh}
                disabled={loading}
              >
                {loading ? "刷新中..." : "刷新"}
              </button>
              {TAB_TO_IMPORT_TYPE[activeTab] && (
                <button
                  className="settings-btn settings-btn--primary"
                  onClick={handleOpenImport}
                >
                  导入数据
                </button>
              )}
            </div>
          </div>
          <div className="settings-section-body">
            {activeTab === "contracts" && renderContractsTable()}
            {activeTab === "customers" && renderCustomersTable()}
            {activeTab === "difficulty" && renderDifficultyTable()}
            {activeTab === "specFamily" && renderSpecFamilyTable()}
          </div>
        </div>

        {/* 导入导出区域 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>数据导入导出</h3>
          </div>
          <div className="settings-section-body">
            <div className="import-export-section">
              <div
                className="import-area"
                onClick={handleOpenImport}
                style={{ cursor: TAB_TO_IMPORT_TYPE[activeTab] ? "pointer" : "not-allowed", opacity: TAB_TO_IMPORT_TYPE[activeTab] ? 1 : 0.5 }}
              >
                <div className="import-area-icon">📥</div>
                <div className="import-area-text">
                  点击导入{DATA_SOURCES.find(ds => ds.id === activeTab)?.name || "数据"}<br />
                  <span style={{ fontSize: "11px", color: "var(--color-text-tertiary)" }}>
                    支持 CSV、Excel、JSON 格式
                  </span>
                </div>
              </div>
              <div
                className="export-area"
                onClick={handleOpenExport}
                style={{ cursor: TAB_TO_IMPORT_TYPE[activeTab] ? "pointer" : "not-allowed", opacity: TAB_TO_IMPORT_TYPE[activeTab] ? 1 : 0.5 }}
              >
                <div className="export-area-icon">📤</div>
                <div className="export-area-text">
                  导出{DATA_SOURCES.find(ds => ds.id === activeTab)?.name || "当前数据"}<br />
                  <span style={{ fontSize: "11px", color: "var(--color-text-tertiary)" }}>
                    支持 CSV、Excel、JSON 格式
                  </span>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* 导入对话框 */}
      <ImportDialog
        isOpen={showImportDialog}
        onClose={() => setShowImportDialog(false)}
        dataType={importDataType}
        onImportComplete={handleImportComplete}
      />

      {/* 导出对话框 */}
      <ExportDialog
        isOpen={showExportDialog}
        onClose={() => setShowExportDialog(false)}
        dataType={importDataType}
      />

      <style>{`
        .data-source-page {
          max-width: 1200px;
        }

        .error {
          color: var(--color-error);
          font-weight: 500;
        }

        .warning {
          color: var(--color-warning);
          font-weight: 500;
        }
      `}</style>
    </div>
  );
}
