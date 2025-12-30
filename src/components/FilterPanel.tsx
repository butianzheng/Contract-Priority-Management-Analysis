import { useState, useEffect } from "react";
import { api, FilterPreset, FilterCriteria, ContractPriority } from "../api/tauri";
import "./FilterPanel.css";

interface FilterPanelProps {
  contracts: ContractPriority[];
  onFilter: (filtered: ContractPriority[]) => void;
  onClose: () => void;
}

export function FilterPanel({ contracts, onFilter, onClose }: FilterPanelProps) {
  const [presets, setPresets] = useState<FilterPreset[]>([]);
  const [selectedPresetId, setSelectedPresetId] = useState<number | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");

  // 筛选条件状态
  const [contractId, setContractId] = useState<string>("");
  const [customerIds, setCustomerIds] = useState<string>("");
  const [steelGrades, setSteelGrades] = useState<string>("");
  const [specFamilies, setSpecFamilies] = useState<string>("");
  const [thicknessMin, setThicknessMin] = useState<string>("");
  const [thicknessMax, setThicknessMax] = useState<string>("");
  const [widthMin, setWidthMin] = useState<string>("");
  const [widthMax, setWidthMax] = useState<string>("");
  const [daysToPddMin, setDaysToPddMin] = useState<string>("");
  const [daysToPddMax, setDaysToPddMax] = useState<string>("");
  const [sScoreMin, setSScoreMin] = useState<string>("");
  const [sScoreMax, setSScoreMax] = useState<string>("");
  const [pScoreMin, setPScoreMin] = useState<string>("");
  const [pScoreMax, setPScoreMax] = useState<string>("");
  const [priorityMin, setPriorityMin] = useState<string>("");
  const [priorityMax, setPriorityMax] = useState<string>("");
  const [hasAlpha, setHasAlpha] = useState<string>("all"); // all, yes, no

  // 保存预设相关
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [presetName, setPresetName] = useState<string>("");
  const [presetDescription, setPresetDescription] = useState<string>("");
  const [userName, setUserName] = useState<string>("admin");

  useEffect(() => {
    loadPresets();
  }, []);

  const loadPresets = async () => {
    try {
      const data = await api.getFilterPresets();
      setPresets(data);

      // 自动选中默认预设
      const defaultPreset = data.find((p) => p.is_default === 1);
      if (defaultPreset && defaultPreset.preset_id) {
        setSelectedPresetId(defaultPreset.preset_id);
      }
    } catch (err) {
      setError(`加载筛选器预设失败: ${err}`);
    }
  };

  const getCurrentFilterCriteria = (): FilterCriteria => {
    return {
      contract_id: contractId || undefined,
      customer_ids: customerIds ? customerIds.split(",").map((s) => s.trim()) : undefined,
      steel_grades: steelGrades ? steelGrades.split(",").map((s) => s.trim()) : undefined,
      spec_families: specFamilies ? specFamilies.split(",").map((s) => s.trim()) : undefined,
      thickness_min: thicknessMin ? parseFloat(thicknessMin) : undefined,
      thickness_max: thicknessMax ? parseFloat(thicknessMax) : undefined,
      width_min: widthMin ? parseFloat(widthMin) : undefined,
      width_max: widthMax ? parseFloat(widthMax) : undefined,
      days_to_pdd_min: daysToPddMin ? parseInt(daysToPddMin) : undefined,
      days_to_pdd_max: daysToPddMax ? parseInt(daysToPddMax) : undefined,
      s_score_min: sScoreMin ? parseFloat(sScoreMin) : undefined,
      s_score_max: sScoreMax ? parseFloat(sScoreMax) : undefined,
      p_score_min: pScoreMin ? parseFloat(pScoreMin) : undefined,
      p_score_max: pScoreMax ? parseFloat(pScoreMax) : undefined,
      priority_min: priorityMin ? parseFloat(priorityMin) : undefined,
      priority_max: priorityMax ? parseFloat(priorityMax) : undefined,
      has_alpha: hasAlpha === "all" ? undefined : hasAlpha === "yes",
    };
  };

  const applyFilter = () => {
    const criteria = getCurrentFilterCriteria();

    let filtered = contracts.filter((contract) => {
      // 合同编号
      if (criteria.contract_id && !contract.contract_id.includes(criteria.contract_id)) {
        return false;
      }

      // 客户筛选
      if (criteria.customer_ids && !criteria.customer_ids.includes(contract.customer_id)) {
        return false;
      }

      // 钢种筛选
      if (criteria.steel_grades && !criteria.steel_grades.includes(contract.steel_grade)) {
        return false;
      }

      // 规格族筛选
      if (criteria.spec_families && !criteria.spec_families.includes(contract.spec_family)) {
        return false;
      }

      // 厚度范围
      if (criteria.thickness_min !== undefined && contract.thickness < criteria.thickness_min) {
        return false;
      }
      if (criteria.thickness_max !== undefined && contract.thickness > criteria.thickness_max) {
        return false;
      }

      // 宽度范围
      if (criteria.width_min !== undefined && contract.width < criteria.width_min) {
        return false;
      }
      if (criteria.width_max !== undefined && contract.width > criteria.width_max) {
        return false;
      }

      // 交期剩余天数
      if (criteria.days_to_pdd_min !== undefined && contract.days_to_pdd < criteria.days_to_pdd_min) {
        return false;
      }
      if (criteria.days_to_pdd_max !== undefined && contract.days_to_pdd > criteria.days_to_pdd_max) {
        return false;
      }

      // S分数范围
      if (criteria.s_score_min !== undefined && contract.s_score < criteria.s_score_min) {
        return false;
      }
      if (criteria.s_score_max !== undefined && contract.s_score > criteria.s_score_max) {
        return false;
      }

      // P分数范围
      if (criteria.p_score_min !== undefined && contract.p_score < criteria.p_score_min) {
        return false;
      }
      if (criteria.p_score_max !== undefined && contract.p_score > criteria.p_score_max) {
        return false;
      }

      // 优先级范围
      if (criteria.priority_min !== undefined && contract.priority < criteria.priority_min) {
        return false;
      }
      if (criteria.priority_max !== undefined && contract.priority > criteria.priority_max) {
        return false;
      }

      // 是否有人工调整
      if (criteria.has_alpha !== undefined) {
        const hasAlpha = contract.alpha !== null && contract.alpha !== undefined;
        if (criteria.has_alpha !== hasAlpha) {
          return false;
        }
      }

      return true;
    });

    onFilter(filtered);
  };

  const loadPreset = (preset: FilterPreset) => {
    try {
      const criteria: FilterCriteria = JSON.parse(preset.filter_json);

      setContractId(criteria.contract_id || "");
      setCustomerIds(criteria.customer_ids?.join(", ") || "");
      setSteelGrades(criteria.steel_grades?.join(", ") || "");
      setSpecFamilies(criteria.spec_families?.join(", ") || "");
      setThicknessMin(criteria.thickness_min?.toString() || "");
      setThicknessMax(criteria.thickness_max?.toString() || "");
      setWidthMin(criteria.width_min?.toString() || "");
      setWidthMax(criteria.width_max?.toString() || "");
      setDaysToPddMin(criteria.days_to_pdd_min?.toString() || "");
      setDaysToPddMax(criteria.days_to_pdd_max?.toString() || "");
      setSScoreMin(criteria.s_score_min?.toString() || "");
      setSScoreMax(criteria.s_score_max?.toString() || "");
      setPScoreMin(criteria.p_score_min?.toString() || "");
      setPScoreMax(criteria.p_score_max?.toString() || "");
      setPriorityMin(criteria.priority_min?.toString() || "");
      setPriorityMax(criteria.priority_max?.toString() || "");
      setHasAlpha(
        criteria.has_alpha === undefined
          ? "all"
          : criteria.has_alpha
          ? "yes"
          : "no"
      );

      setSelectedPresetId(preset.preset_id || null);
    } catch (err) {
      setError(`加载预设失败: ${err}`);
    }
  };

  const savePreset = async () => {
    if (!presetName.trim()) {
      alert("请输入预设名称");
      return;
    }

    setLoading(true);
    setError("");

    try {
      const criteria = getCurrentFilterCriteria();
      const filterJson = JSON.stringify(criteria);

      await api.saveFilterPreset(
        presetName,
        filterJson,
        presetDescription,
        userName
      );

      await loadPresets();
      setShowSaveDialog(false);
      setPresetName("");
      setPresetDescription("");
      alert("保存成功！");
    } catch (err) {
      setError(`保存预设失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const deletePreset = async (presetId: number) => {
    if (!confirm("确认要删除此预设吗？")) {
      return;
    }

    setLoading(true);
    setError("");

    try {
      await api.deleteFilterPreset(presetId);
      await loadPresets();
      if (selectedPresetId === presetId) {
        setSelectedPresetId(null);
      }
    } catch (err) {
      setError(`删除预设失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const setDefaultPreset = async (presetId: number) => {
    setLoading(true);
    setError("");

    try {
      await api.setDefaultFilterPreset(presetId);
      await loadPresets();
    } catch (err) {
      setError(`设置默认预设失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const resetFilter = () => {
    setContractId("");
    setCustomerIds("");
    setSteelGrades("");
    setSpecFamilies("");
    setThicknessMin("");
    setThicknessMax("");
    setWidthMin("");
    setWidthMax("");
    setDaysToPddMin("");
    setDaysToPddMax("");
    setSScoreMin("");
    setSScoreMax("");
    setPScoreMin("");
    setPScoreMax("");
    setPriorityMin("");
    setPriorityMax("");
    setHasAlpha("all");
    setSelectedPresetId(null);
  };

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div
        className="modal-content filter-panel"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="modal-header">
          <h2>筛选条件</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {error && <div className="error">{error}</div>}

          {/* 预设选择 */}
          <div className="preset-section">
            <h3>快速预设</h3>
            <div className="preset-list">
              {presets.map((preset) => (
                <div
                  key={preset.preset_id}
                  className={`preset-item ${
                    selectedPresetId === preset.preset_id ? "active" : ""
                  }`}
                >
                  <div className="preset-info" onClick={() => loadPreset(preset)}>
                    <span className="preset-name">
                      {preset.preset_name}
                      {preset.is_default === 1 && <span className="default-badge">默认</span>}
                    </span>
                    <span className="preset-description">{preset.description}</span>
                  </div>
                  <div className="preset-actions">
                    {preset.is_default === 0 && (
                      <>
                        <button
                          className="btn-small"
                          onClick={() => preset.preset_id && setDefaultPreset(preset.preset_id)}
                          disabled={loading}
                          title="设为默认"
                        >
                          ★
                        </button>
                        <button
                          className="btn-small btn-danger"
                          onClick={() => preset.preset_id && deletePreset(preset.preset_id)}
                          disabled={loading}
                          title="删除"
                        >
                          删除
                        </button>
                      </>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* 筛选条件表单 */}
          <div className="filter-form">
            <h3>自定义筛选</h3>

            <div className="form-row">
              <label>
                合同编号:
                <input
                  type="text"
                  value={contractId}
                  onChange={(e) => setContractId(e.target.value)}
                  placeholder="模糊匹配"
                />
              </label>
            </div>

            <div className="form-row">
              <label>
                客户ID (逗号分隔):
                <input
                  type="text"
                  value={customerIds}
                  onChange={(e) => setCustomerIds(e.target.value)}
                  placeholder="例如: A001, A002"
                />
              </label>
            </div>

            <div className="form-row">
              <label>
                钢种 (逗号分隔):
                <input
                  type="text"
                  value={steelGrades}
                  onChange={(e) => setSteelGrades(e.target.value)}
                  placeholder="例如: 304, 316L"
                />
              </label>
            </div>

            <div className="form-row">
              <label>
                规格族 (逗号分隔):
                <input
                  type="text"
                  value={specFamilies}
                  onChange={(e) => setSpecFamilies(e.target.value)}
                  placeholder="例如: SpecA, SpecB"
                />
              </label>
            </div>

            <div className="form-row">
              <label>
                厚度 (mm):
                <div className="range-input">
                  <input
                    type="number"
                    value={thicknessMin}
                    onChange={(e) => setThicknessMin(e.target.value)}
                    placeholder="最小"
                    step="0.1"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={thicknessMax}
                    onChange={(e) => setThicknessMax(e.target.value)}
                    placeholder="最大"
                    step="0.1"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                宽度 (mm):
                <div className="range-input">
                  <input
                    type="number"
                    value={widthMin}
                    onChange={(e) => setWidthMin(e.target.value)}
                    placeholder="最小"
                    step="1"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={widthMax}
                    onChange={(e) => setWidthMax(e.target.value)}
                    placeholder="最大"
                    step="1"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                剩余天数:
                <div className="range-input">
                  <input
                    type="number"
                    value={daysToPddMin}
                    onChange={(e) => setDaysToPddMin(e.target.value)}
                    placeholder="最小"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={daysToPddMax}
                    onChange={(e) => setDaysToPddMax(e.target.value)}
                    placeholder="最大"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                S分数:
                <div className="range-input">
                  <input
                    type="number"
                    value={sScoreMin}
                    onChange={(e) => setSScoreMin(e.target.value)}
                    placeholder="最小"
                    step="0.1"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={sScoreMax}
                    onChange={(e) => setSScoreMax(e.target.value)}
                    placeholder="最大"
                    step="0.1"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                P分数:
                <div className="range-input">
                  <input
                    type="number"
                    value={pScoreMin}
                    onChange={(e) => setPScoreMin(e.target.value)}
                    placeholder="最小"
                    step="0.1"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={pScoreMax}
                    onChange={(e) => setPScoreMax(e.target.value)}
                    placeholder="最大"
                    step="0.1"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                优先级:
                <div className="range-input">
                  <input
                    type="number"
                    value={priorityMin}
                    onChange={(e) => setPriorityMin(e.target.value)}
                    placeholder="最小"
                    step="0.1"
                  />
                  <span>~</span>
                  <input
                    type="number"
                    value={priorityMax}
                    onChange={(e) => setPriorityMax(e.target.value)}
                    placeholder="最大"
                    step="0.1"
                  />
                </div>
              </label>
            </div>

            <div className="form-row">
              <label>
                人工调整:
                <select value={hasAlpha} onChange={(e) => setHasAlpha(e.target.value)}>
                  <option value="all">全部</option>
                  <option value="yes">仅显示已调整</option>
                  <option value="no">仅显示未调整</option>
                </select>
              </label>
            </div>
          </div>

          {/* 保存预设对话框 */}
          {showSaveDialog && (
            <div className="save-dialog">
              <h3>保存为预设</h3>
              <div className="form-row">
                <label>
                  预设名称 *:
                  <input
                    type="text"
                    value={presetName}
                    onChange={(e) => setPresetName(e.target.value)}
                    placeholder="例如: 紧急订单"
                  />
                </label>
              </div>
              <div className="form-row">
                <label>
                  描述:
                  <input
                    type="text"
                    value={presetDescription}
                    onChange={(e) => setPresetDescription(e.target.value)}
                    placeholder="简要描述筛选条件"
                  />
                </label>
              </div>
              <div className="form-row">
                <label>
                  创建人:
                  <input
                    type="text"
                    value={userName}
                    onChange={(e) => setUserName(e.target.value)}
                  />
                </label>
              </div>
              <div className="dialog-buttons">
                <button onClick={savePreset} disabled={loading}>
                  保存
                </button>
                <button onClick={() => setShowSaveDialog(false)}>取消</button>
              </div>
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button className="btn-primary" onClick={applyFilter}>
            应用筛选
          </button>
          <button onClick={resetFilter}>重置</button>
          <button onClick={() => setShowSaveDialog(true)}>另存为预设</button>
          <button onClick={onClose}>关闭</button>
        </div>
      </div>
    </div>
  );
}
