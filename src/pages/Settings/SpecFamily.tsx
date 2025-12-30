import { useState, useEffect, useCallback } from "react";
import { Link } from "react-router-dom";
import { api, SpecFamily, SpecFamilyChangeLog } from "../../api/tauri";
import "./Settings.css";

interface EditingFamily {
  family_id?: number;
  family_name: string;
  family_code: string;
  description: string;
  factor: number;
  steel_grades: string;
  thickness_min: string;
  thickness_max: string;
  width_min: string;
  width_max: string;
  sort_order: number;
}

const defaultEditingFamily: EditingFamily = {
  family_name: "",
  family_code: "",
  description: "",
  factor: 1.0,
  steel_grades: "[]",
  thickness_min: "",
  thickness_max: "",
  width_min: "",
  width_max: "",
  sort_order: 0,
};

export function SpecFamilyPage() {
  const [families, setFamilies] = useState<SpecFamily[]>([]);
  const [selectedFamily, setSelectedFamily] = useState<SpecFamily | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 编辑模式
  const [isEditing, setIsEditing] = useState(false);
  const [editingFamily, setEditingFamily] = useState<EditingFamily | null>(null);

  // 添加对话框
  const [showAddDialog, setShowAddDialog] = useState(false);

  // 变更历史
  const [showHistory, setShowHistory] = useState(false);
  const [changeHistory, setChangeHistory] = useState<SpecFamilyChangeLog[]>([]);

  // 统计
  const enabledCount = families.filter((f) => f.enabled === 1).length;

  // 加载规格族列表
  const loadFamilies = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getSpecFamilies();
      setFamilies(data);
    } catch (err) {
      setError(`加载规格族失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadFamilies();
  }, [loadFamilies]);

  // 解析钢种列表
  const parseSteelGrades = (json?: string): string[] => {
    if (!json) return [];
    try {
      return JSON.parse(json);
    } catch {
      return [];
    }
  };

  // 切换启用状态
  const handleToggleEnabled = async (family: SpecFamily) => {
    if (!family.family_id) return;
    try {
      await api.toggleSpecFamily(family.family_id, family.enabled === 0, "admin");
      await loadFamilies();
      if (selectedFamily?.family_id === family.family_id) {
        setSelectedFamily({ ...family, enabled: family.enabled === 0 ? 1 : 0 });
      }
    } catch (err) {
      setError(`切换状态失败: ${err}`);
    }
  };

  // 选择规格族
  const handleSelectFamily = (family: SpecFamily) => {
    setSelectedFamily(family);
    setIsEditing(false);
    setEditingFamily(null);
  };

  // 进入编辑模式
  const handleEdit = () => {
    if (!selectedFamily) return;
    setIsEditing(true);
    setEditingFamily({
      family_id: selectedFamily.family_id,
      family_name: selectedFamily.family_name,
      family_code: selectedFamily.family_code,
      description: selectedFamily.description || "",
      factor: selectedFamily.factor,
      steel_grades: selectedFamily.steel_grades || "[]",
      thickness_min: selectedFamily.thickness_min?.toString() || "",
      thickness_max: selectedFamily.thickness_max?.toString() || "",
      width_min: selectedFamily.width_min?.toString() || "",
      width_max: selectedFamily.width_max?.toString() || "",
      sort_order: selectedFamily.sort_order,
    });
  };

  // 取消编辑
  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditingFamily(null);
  };

  // 保存规格族
  const handleSave = async () => {
    if (!editingFamily) return;

    try {
      setSaving(true);
      setError(null);

      // 验证必填字段
      if (!editingFamily.family_name.trim()) {
        setError("规格族名称不能为空");
        return;
      }
      if (!editingFamily.family_code.trim()) {
        setError("规格族代码不能为空");
        return;
      }

      // 验证钢种 JSON 格式
      try {
        JSON.parse(editingFamily.steel_grades);
      } catch {
        setError("钢种列表 JSON 格式无效");
        return;
      }

      const thicknessMin = editingFamily.thickness_min ? parseFloat(editingFamily.thickness_min) : null;
      const thicknessMax = editingFamily.thickness_max ? parseFloat(editingFamily.thickness_max) : null;
      const widthMin = editingFamily.width_min ? parseFloat(editingFamily.width_min) : null;
      const widthMax = editingFamily.width_max ? parseFloat(editingFamily.width_max) : null;

      if (editingFamily.family_id) {
        // 更新现有规格族
        await api.updateSpecFamily(
          editingFamily.family_id,
          editingFamily.family_name,
          editingFamily.family_code,
          editingFamily.description || null,
          editingFamily.factor,
          editingFamily.steel_grades,
          thicknessMin,
          thicknessMax,
          widthMin,
          widthMax,
          editingFamily.sort_order,
          "admin"
        );
      } else {
        // 创建新规格族
        await api.createSpecFamily(
          editingFamily.family_name,
          editingFamily.family_code,
          editingFamily.description || null,
          editingFamily.factor,
          editingFamily.steel_grades,
          thicknessMin,
          thicknessMax,
          widthMin,
          widthMax,
          editingFamily.sort_order,
          "admin"
        );
      }

      await loadFamilies();
      setIsEditing(false);
      setEditingFamily(null);
      setShowAddDialog(false);

      // 更新选中项
      if (editingFamily.family_id) {
        const updated = families.find((f) => f.family_id === editingFamily.family_id);
        if (updated) setSelectedFamily(updated);
      }
    } catch (err) {
      setError(`保存失败: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  // 删除规格族
  const handleDelete = async () => {
    if (!selectedFamily?.family_id) return;
    if (!confirm(`确定要删除规格族 "${selectedFamily.family_name}" 吗？`)) return;

    try {
      await api.deleteSpecFamily(selectedFamily.family_id, "admin");
      await loadFamilies();
      setSelectedFamily(null);
    } catch (err) {
      setError(`删除失败: ${err}`);
    }
  };

  // 查看变更历史
  const handleShowHistory = async () => {
    try {
      const history = await api.getSpecFamilyHistory(selectedFamily?.family_id, 20);
      setChangeHistory(history);
      setShowHistory(true);
    } catch (err) {
      setError(`加载历史失败: ${err}`);
    }
  };

  // 添加新规格族
  const handleAddFamily = () => {
    setEditingFamily({ ...defaultEditingFamily });
    setShowAddDialog(true);
  };

  // 格式化时间
  const formatTime = (time?: string) => {
    if (!time) return "-";
    return time.replace("T", " ").substring(0, 19);
  };

  // 获取变更类型标签
  const getChangeTypeLabel = (type: string) => {
    const labels: Record<string, string> = {
      create: "创建",
      update: "更新",
      delete: "删除",
      enable: "启用",
      disable: "禁用",
    };
    return labels[type] || type;
  };

  if (loading && families.length === 0) {
    return (
      <div className="spec-family-page">
        <div className="settings-loading">加载中...</div>
      </div>
    );
  }

  return (
    <div className="spec-family-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <Link to="/settings">设置</Link>
          <span>/</span>
          <span>规格族管理</span>
        </div>
        <h1 className="page-header__title">规格族管理</h1>
        <p className="page-header__subtitle">管理产品规格族分类和 P-Score 系数配置</p>
      </div>

      {error && (
        <div className="settings-error" style={{ marginBottom: "var(--spacing-md)" }}>
          {error}
          <button onClick={() => setError(null)} style={{ marginLeft: "var(--spacing-md)" }}>
            关闭
          </button>
        </div>
      )}

      <div className="settings-content">
        {/* 统计信息 */}
        <div className="stats-row">
          <div className="stat-item">
            <div className="stat-item-label">总规格族</div>
            <div className="stat-item-value">{families.length}</div>
          </div>
          <div className="stat-item">
            <div className="stat-item-label">已启用</div>
            <div className="stat-item-value primary">{enabledCount}</div>
          </div>
          <div className="stat-item">
            <div className="stat-item-label">已禁用</div>
            <div className="stat-item-value">{families.length - enabledCount}</div>
          </div>
          <div className="stat-item">
            <div className="stat-item-label">平均系数</div>
            <div className="stat-item-value">
              {families.length > 0
                ? (families.reduce((sum, f) => sum + f.factor, 0) / families.length).toFixed(2)
                : "-"}
            </div>
          </div>
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-lg)" }}>
          {/* 规格族列表 */}
          <div className="settings-section">
            <div className="settings-section-header">
              <h3>规格族列表</h3>
              <button className="settings-btn settings-btn--primary" onClick={handleAddFamily}>
                + 添加规格族
              </button>
            </div>
            <div className="settings-section-body">
              <div className="rule-list">
                {families.map((family) => (
                  <div
                    key={family.family_id}
                    className={`rule-item ${family.enabled === 0 ? "disabled" : ""} ${
                      selectedFamily?.family_id === family.family_id ? "selected" : ""
                    }`}
                    onClick={() => handleSelectFamily(family)}
                    style={{ cursor: "pointer" }}
                  >
                    <div className="rule-item-content">
                      <div className="rule-item-name">
                        <span className="family-code">{family.family_code}</span>
                        {family.family_name}
                        {selectedFamily?.family_id === family.family_id && (
                          <span style={{ marginLeft: "8px", color: "var(--color-primary)" }}>●</span>
                        )}
                      </div>
                      <div className="rule-item-desc">
                        系数: {family.factor} | {parseSteelGrades(family.steel_grades).length} 钢种
                      </div>
                    </div>
                    <div className="rule-item-actions">
                      <div
                        className={`toggle-switch ${family.enabled === 1 ? "active" : ""}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleToggleEnabled(family);
                        }}
                      />
                    </div>
                  </div>
                ))}
                {families.length === 0 && (
                  <div className="settings-empty">
                    <div className="settings-empty-icon">📋</div>
                    <div className="settings-empty-text">暂无规格族</div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* 规格族详情 */}
          <div className="settings-section">
            <div className="settings-section-header">
              <h3>规格族详情</h3>
              {selectedFamily && !isEditing && (
                <div className="btn-group">
                  <button className="settings-btn" onClick={handleShowHistory}>
                    历史
                  </button>
                  <button className="settings-btn" onClick={handleEdit}>
                    编辑
                  </button>
                  <button className="settings-btn settings-btn--danger" onClick={handleDelete}>
                    删除
                  </button>
                </div>
              )}
              {isEditing && (
                <div className="btn-group">
                  <button className="settings-btn" onClick={handleCancelEdit}>
                    取消
                  </button>
                  <button
                    className="settings-btn settings-btn--primary"
                    onClick={handleSave}
                    disabled={saving}
                  >
                    {saving ? "保存中..." : "保存"}
                  </button>
                </div>
              )}
            </div>
            <div className="settings-section-body">
              {selectedFamily && !isEditing ? (
                <div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">规格族名称</label>
                      <input
                        type="text"
                        className="form-input"
                        value={selectedFamily.family_name}
                        readOnly
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">代码</label>
                      <input
                        type="text"
                        className="form-input"
                        value={selectedFamily.family_code}
                        readOnly
                      />
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">描述</label>
                    <textarea
                      className="form-textarea"
                      value={selectedFamily.description || ""}
                      readOnly
                    />
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">P-Score 系数</label>
                      <input
                        type="number"
                        className="form-input"
                        value={selectedFamily.factor}
                        readOnly
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">排序</label>
                      <input
                        type="number"
                        className="form-input"
                        value={selectedFamily.sort_order}
                        readOnly
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">状态</label>
                      <input
                        type="text"
                        className="form-input"
                        value={selectedFamily.enabled === 1 ? "启用" : "禁用"}
                        readOnly
                      />
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">厚度范围 (mm)</label>
                      <input
                        type="text"
                        className="form-input"
                        value={
                          selectedFamily.thickness_min || selectedFamily.thickness_max
                            ? `${selectedFamily.thickness_min || "-"} ~ ${selectedFamily.thickness_max || "-"}`
                            : "不限"
                        }
                        readOnly
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">宽度范围 (mm)</label>
                      <input
                        type="text"
                        className="form-input"
                        value={
                          selectedFamily.width_min || selectedFamily.width_max
                            ? `${selectedFamily.width_min || "-"} ~ ${selectedFamily.width_max || "-"}`
                            : "不限"
                        }
                        readOnly
                      />
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">关联钢种</label>
                    <div className="steel-grades-list">
                      {parseSteelGrades(selectedFamily.steel_grades).map((grade, index) => (
                        <span key={index} className="tag">
                          {grade}
                        </span>
                      ))}
                      {parseSteelGrades(selectedFamily.steel_grades).length === 0 && (
                        <span className="text-muted">无关联钢种</span>
                      )}
                    </div>
                  </div>
                  <div className="form-hint">
                    创建者: {selectedFamily.created_by} | 创建时间:{" "}
                    {formatTime(selectedFamily.created_at)} | 更新时间:{" "}
                    {formatTime(selectedFamily.updated_at)}
                  </div>
                </div>
              ) : isEditing && editingFamily ? (
                <div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">规格族名称 *</label>
                      <input
                        type="text"
                        className="form-input"
                        value={editingFamily.family_name}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, family_name: e.target.value })
                        }
                        placeholder="如：双相钢"
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">代码 *</label>
                      <input
                        type="text"
                        className="form-input"
                        value={editingFamily.family_code}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, family_code: e.target.value.toUpperCase() })
                        }
                        placeholder="如：DP"
                        disabled={!!editingFamily.family_id}
                      />
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">描述</label>
                    <textarea
                      className="form-textarea"
                      value={editingFamily.description}
                      onChange={(e) =>
                        setEditingFamily({ ...editingFamily, description: e.target.value })
                      }
                      placeholder="规格族描述"
                    />
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">P-Score 系数 *</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.factor}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, factor: parseFloat(e.target.value) || 1.0 })
                        }
                        step="0.1"
                        min="0.1"
                        max="5.0"
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">排序</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.sort_order}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, sort_order: parseInt(e.target.value) || 0 })
                        }
                        min="0"
                      />
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">厚度最小 (mm)</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.thickness_min}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, thickness_min: e.target.value })
                        }
                        step="0.1"
                        placeholder="可选"
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">厚度最大 (mm)</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.thickness_max}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, thickness_max: e.target.value })
                        }
                        step="0.1"
                        placeholder="可选"
                      />
                    </div>
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">宽度最小 (mm)</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.width_min}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, width_min: e.target.value })
                        }
                        step="1"
                        placeholder="可选"
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">宽度最大 (mm)</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingFamily.width_max}
                        onChange={(e) =>
                          setEditingFamily({ ...editingFamily, width_max: e.target.value })
                        }
                        step="1"
                        placeholder="可选"
                      />
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">关联钢种 (JSON数组)</label>
                    <textarea
                      className="form-textarea"
                      value={editingFamily.steel_grades}
                      onChange={(e) =>
                        setEditingFamily({ ...editingFamily, steel_grades: e.target.value })
                      }
                      style={{ fontFamily: "monospace", minHeight: "80px" }}
                      placeholder='["DP590", "DP780", "DP980"]'
                    />
                  </div>
                </div>
              ) : (
                <div className="settings-empty">
                  <div className="settings-empty-icon">👈</div>
                  <div className="settings-empty-text">选择左侧规格族查看详情</div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* 系数说明 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>P-Score 系数说明</h3>
          </div>
          <div className="settings-section-body">
            <div className="factor-guide">
              <div className="factor-item">
                <span className="factor-range">0.8 - 1.0</span>
                <span className="factor-label">常规难度</span>
                <span className="factor-desc">标准工艺流程，常规规格</span>
              </div>
              <div className="factor-item">
                <span className="factor-range">1.0 - 1.3</span>
                <span className="factor-label">中等难度</span>
                <span className="factor-desc">需要特殊工艺或设备调整</span>
              </div>
              <div className="factor-item">
                <span className="factor-range">1.3 - 1.5</span>
                <span className="factor-label">较高难度</span>
                <span className="factor-desc">高强度钢、特殊合金钢</span>
              </div>
              <div className="factor-item">
                <span className="factor-range">1.5+</span>
                <span className="factor-label">高难度</span>
                <span className="factor-desc">超高强度、热成形等特殊工艺</span>
              </div>
            </div>
            <div className="form-hint" style={{ marginTop: "var(--spacing-md)" }}>
              P-Score = 工艺难度基础分 × 规格族系数。系数越高表示生产难度越大，优先级计算时会相应调整。
            </div>
          </div>
        </div>
      </div>

      {/* 添加规格族对话框 */}
      {showAddDialog && editingFamily && (
        <div className="dialog-overlay" onClick={() => setShowAddDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()} style={{ maxWidth: "600px" }}>
            <div className="dialog-header">
              <h3>添加新规格族</h3>
              <button className="dialog-close" onClick={() => setShowAddDialog(false)}>
                ×
              </button>
            </div>
            <div className="dialog-body">
              <div className="form-row">
                <div className="form-group">
                  <label className="form-label">规格族名称 *</label>
                  <input
                    type="text"
                    className="form-input"
                    value={editingFamily.family_name}
                    onChange={(e) =>
                      setEditingFamily({ ...editingFamily, family_name: e.target.value })
                    }
                    placeholder="如：双相钢"
                  />
                </div>
                <div className="form-group">
                  <label className="form-label">代码 *</label>
                  <input
                    type="text"
                    className="form-input"
                    value={editingFamily.family_code}
                    onChange={(e) =>
                      setEditingFamily({ ...editingFamily, family_code: e.target.value.toUpperCase() })
                    }
                    placeholder="如：DP"
                  />
                </div>
              </div>
              <div className="form-group">
                <label className="form-label">描述</label>
                <textarea
                  className="form-textarea"
                  value={editingFamily.description}
                  onChange={(e) =>
                    setEditingFamily({ ...editingFamily, description: e.target.value })
                  }
                  placeholder="规格族描述"
                />
              </div>
              <div className="form-row">
                <div className="form-group">
                  <label className="form-label">P-Score 系数 *</label>
                  <input
                    type="number"
                    className="form-input"
                    value={editingFamily.factor}
                    onChange={(e) =>
                      setEditingFamily({ ...editingFamily, factor: parseFloat(e.target.value) || 1.0 })
                    }
                    step="0.1"
                    min="0.1"
                    max="5.0"
                  />
                </div>
                <div className="form-group">
                  <label className="form-label">排序</label>
                  <input
                    type="number"
                    className="form-input"
                    value={editingFamily.sort_order}
                    onChange={(e) =>
                      setEditingFamily({ ...editingFamily, sort_order: parseInt(e.target.value) || 0 })
                    }
                    min="0"
                  />
                </div>
              </div>
              <div className="form-group">
                <label className="form-label">关联钢种 (JSON数组)</label>
                <textarea
                  className="form-textarea"
                  value={editingFamily.steel_grades}
                  onChange={(e) =>
                    setEditingFamily({ ...editingFamily, steel_grades: e.target.value })
                  }
                  style={{ fontFamily: "monospace", minHeight: "80px" }}
                  placeholder='["DP590", "DP780", "DP980"]'
                />
              </div>
            </div>
            <div className="dialog-footer">
              <button className="settings-btn" onClick={() => setShowAddDialog(false)}>
                取消
              </button>
              <button
                className="settings-btn settings-btn--primary"
                onClick={handleSave}
                disabled={saving || !editingFamily.family_name || !editingFamily.family_code}
              >
                {saving ? "保存中..." : "保存"}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 变更历史对话框 */}
      {showHistory && (
        <div className="dialog-overlay" onClick={() => setShowHistory(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()} style={{ maxWidth: "700px" }}>
            <div className="dialog-header">
              <h3>变更历史 {selectedFamily ? `- ${selectedFamily.family_name}` : ""}</h3>
              <button className="dialog-close" onClick={() => setShowHistory(false)}>
                ×
              </button>
            </div>
            <div className="dialog-body">
              {changeHistory.length > 0 ? (
                <table className="settings-table">
                  <thead>
                    <tr>
                      <th>时间</th>
                      <th>规格族</th>
                      <th>操作</th>
                      <th>操作人</th>
                      <th>原因</th>
                    </tr>
                  </thead>
                  <tbody>
                    {changeHistory.map((log) => (
                      <tr key={log.change_id}>
                        <td>{formatTime(log.changed_at)}</td>
                        <td>{log.family_name || `#${log.family_id}`}</td>
                        <td>
                          <span
                            className={`tag ${
                              log.change_type === "delete"
                                ? "danger"
                                : log.change_type === "create"
                                ? "success"
                                : "warning"
                            }`}
                          >
                            {getChangeTypeLabel(log.change_type)}
                          </span>
                        </td>
                        <td>{log.changed_by}</td>
                        <td>{log.change_reason || "-"}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              ) : (
                <div className="settings-empty">
                  <div className="settings-empty-text">暂无变更历史</div>
                </div>
              )}
            </div>
            <div className="dialog-footer">
              <button className="settings-btn" onClick={() => setShowHistory(false)}>
                关闭
              </button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .spec-family-page {
          max-width: 1200px;
        }

        .family-code {
          display: inline-block;
          background: var(--color-primary-light);
          color: var(--color-primary);
          padding: 2px 6px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 600;
          margin-right: 8px;
        }

        .rule-item.selected {
          background: var(--color-primary-light);
          border-color: var(--color-primary);
        }

        .steel-grades-list {
          display: flex;
          flex-wrap: wrap;
          gap: 6px;
          padding: 8px;
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-sm);
          min-height: 36px;
        }

        .steel-grades-list .tag {
          background: var(--color-bg);
          border: 1px solid var(--color-border);
        }

        .text-muted {
          color: var(--color-text-tertiary);
          font-size: 14px;
        }

        .factor-guide {
          display: grid;
          grid-template-columns: repeat(4, 1fr);
          gap: var(--spacing-md);
        }

        .factor-item {
          display: flex;
          flex-direction: column;
          align-items: center;
          padding: var(--spacing-md);
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-md);
          text-align: center;
        }

        .factor-range {
          font-size: 18px;
          font-weight: 600;
          color: var(--color-primary);
        }

        .factor-label {
          font-size: 14px;
          font-weight: 500;
          margin: 4px 0;
        }

        .factor-desc {
          font-size: 12px;
          color: var(--color-text-secondary);
        }

        .settings-error {
          padding: var(--spacing-sm) var(--spacing-md);
          background: var(--color-error-bg);
          color: var(--color-error);
          border-radius: var(--border-radius-sm);
          display: flex;
          align-items: center;
          justify-content: space-between;
        }

        .settings-table {
          width: 100%;
          border-collapse: collapse;
        }

        .settings-table th,
        .settings-table td {
          padding: var(--spacing-sm);
          text-align: left;
          border-bottom: 1px solid var(--color-border);
        }

        .settings-table th {
          font-weight: 600;
          background: var(--color-bg-layout);
        }

        .tag.danger {
          background: var(--color-error-bg);
          color: var(--color-error);
        }

        .tag.warning {
          background: var(--color-warning-bg);
          color: var(--color-warning);
        }

        .settings-btn--danger {
          background: var(--color-error);
          color: white;
        }

        .settings-btn--danger:hover {
          background: #ff7875;
        }

        /* 弹窗遮罩层 */
        .dialog-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.45);
          backdrop-filter: blur(2px);
          display: flex;
          align-items: center;
          justify-content: center;
          z-index: 1000;
          padding: var(--spacing-lg);
          animation: dialogFadeIn 0.2s ease-out;
        }

        @keyframes dialogFadeIn {
          from { opacity: 0; }
          to { opacity: 1; }
        }

        @keyframes dialogSlideIn {
          from {
            transform: translateY(-20px);
            opacity: 0;
          }
          to {
            transform: translateY(0);
            opacity: 1;
          }
        }

        /* 弹窗容器 */
        .dialog {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          box-shadow: var(--shadow-md);
          width: 90%;
          max-height: 90vh;
          overflow: hidden;
          display: flex;
          flex-direction: column;
          animation: dialogSlideIn 0.3s ease-out;
        }

        /* 弹窗头部 */
        .dialog-header {
          padding: var(--spacing-md) var(--spacing-lg);
          border-bottom: 1px solid var(--color-border-light);
          background: var(--color-bg-layout);
          display: flex;
          align-items: center;
          justify-content: space-between;
        }

        .dialog-header h3 {
          margin: 0;
          font-size: var(--font-size-lg);
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .dialog-close {
          width: 32px;
          height: 32px;
          background: transparent;
          border: none;
          font-size: 20px;
          cursor: pointer;
          color: var(--color-text-tertiary);
          border-radius: var(--border-radius-sm);
          display: flex;
          align-items: center;
          justify-content: center;
          transition: all var(--transition-fast);
        }

        .dialog-close:hover {
          background: var(--color-bg-container);
          color: var(--color-text-primary);
        }

        /* 弹窗主体 */
        .dialog-body {
          padding: var(--spacing-lg);
          overflow-y: auto;
          flex: 1;
          color: var(--color-text-secondary);
        }

        /* 弹窗内的表单样式 */
        .dialog-body .form-group {
          margin-bottom: var(--spacing-md);
        }

        .dialog-body .form-label {
          display: block;
          margin-bottom: var(--spacing-xs);
          font-weight: 500;
          color: var(--color-text-primary);
          font-size: var(--font-size-sm);
        }

        .dialog-body .form-input,
        .dialog-body .form-select,
        .dialog-body .form-textarea {
          width: 100%;
          padding: var(--spacing-sm);
          border: 1px solid var(--color-border);
          border-radius: var(--border-radius-sm);
          background: var(--color-bg-container);
          color: var(--color-text-primary);
          font-size: var(--font-size-base);
          font-family: inherit;
          transition: all var(--transition-fast);
          box-sizing: border-box;
        }

        .dialog-body .form-input:focus,
        .dialog-body .form-select:focus,
        .dialog-body .form-textarea:focus {
          outline: none;
          border-color: var(--color-primary);
          box-shadow: 0 0 0 2px var(--color-primary-light);
        }

        .dialog-body .form-input::placeholder,
        .dialog-body .form-textarea::placeholder {
          color: var(--color-text-disabled);
        }

        .dialog-body .form-textarea {
          resize: vertical;
          min-height: 80px;
        }

        /* 弹窗底部 */
        .dialog-footer {
          padding: var(--spacing-md) var(--spacing-lg);
          border-top: 1px solid var(--color-border-light);
          background: var(--color-bg-layout);
          display: flex;
          justify-content: flex-end;
          gap: var(--spacing-sm);
        }
      `}</style>
    </div>
  );
}
