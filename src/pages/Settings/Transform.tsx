import { useState, useEffect, useCallback } from "react";
import { Link } from "react-router-dom";
import { api, TransformRule, TransformRuleChangeLog, RuleTestResult, RuleCategory } from "../../api/tauri";
import "./Settings.css";

const RULE_CATEGORIES = [
  { id: "standardization" as RuleCategory, name: "字段标准化", icon: "📝" },
  { id: "extraction" as RuleCategory, name: "规格段提取", icon: "📐" },
  { id: "normalization" as RuleCategory, name: "等级归一化", icon: "⭐" },
  { id: "mapping" as RuleCategory, name: "标签映射", icon: "🏷️" },
];

interface EditingRule {
  rule_id?: number;
  rule_name: string;
  category: RuleCategory;
  description: string;
  priority: number;
  config_json: string;
}

export function Transform() {
  const [activeCategory, setActiveCategory] = useState<RuleCategory>("standardization");
  const [rules, setRules] = useState<TransformRule[]>([]);
  const [selectedRule, setSelectedRule] = useState<TransformRule | null>(null);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [testing, setTesting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 编辑模式
  const [isEditing, setIsEditing] = useState(false);
  const [editingRule, setEditingRule] = useState<EditingRule | null>(null);

  // 添加规则对话框
  const [showAddDialog, setShowAddDialog] = useState(false);

  // 测试结果
  const [testResult, setTestResult] = useState<RuleTestResult | null>(null);

  // 变更历史
  const [showHistory, setShowHistory] = useState(false);
  const [changeHistory, setChangeHistory] = useState<TransformRuleChangeLog[]>([]);

  // 加载规则列表
  const loadRules = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getTransformRules();
      setRules(data);
    } catch (err) {
      setError(`加载规则失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadRules();
  }, [loadRules]);

  // 筛选当前分类的规则
  const filteredRules = rules.filter((r) => r.category === activeCategory);

  // 获取分类统计
  const getCategoryStats = (category: RuleCategory) => {
    const categoryRules = rules.filter((r) => r.category === category);
    const enabledCount = categoryRules.filter((r) => r.enabled === 1).length;
    return { total: categoryRules.length, enabled: enabledCount };
  };

  // 切换规则启用状态
  const handleToggleEnabled = async (rule: TransformRule) => {
    if (!rule.rule_id) return;
    try {
      await api.toggleTransformRule(rule.rule_id, rule.enabled === 0, "admin");
      await loadRules();
      // 如果当前选中的是这个规则，更新选中状态
      if (selectedRule?.rule_id === rule.rule_id) {
        const updated = rules.find(r => r.rule_id === rule.rule_id);
        if (updated) {
          setSelectedRule({ ...updated, enabled: rule.enabled === 0 ? 1 : 0 });
        }
      }
    } catch (err) {
      setError(`切换状态失败: ${err}`);
    }
  };

  // 选择规则
  const handleSelectRule = (rule: TransformRule) => {
    setSelectedRule(rule);
    setIsEditing(false);
    setEditingRule(null);
    setTestResult(null);
  };

  // 进入编辑模式
  const handleEdit = () => {
    if (!selectedRule) return;
    setIsEditing(true);
    setEditingRule({
      rule_id: selectedRule.rule_id,
      rule_name: selectedRule.rule_name,
      category: selectedRule.category,
      description: selectedRule.description || "",
      priority: selectedRule.priority,
      config_json: selectedRule.config_json,
    });
  };

  // 取消编辑
  const handleCancelEdit = () => {
    setIsEditing(false);
    setEditingRule(null);
  };

  // 保存规则
  const handleSave = async () => {
    if (!editingRule) return;

    try {
      setSaving(true);
      setError(null);

      // 验证 JSON 格式
      try {
        JSON.parse(editingRule.config_json);
      } catch {
        setError("配置 JSON 格式无效");
        return;
      }

      if (editingRule.rule_id) {
        // 更新现有规则
        await api.updateTransformRule(
          editingRule.rule_id,
          editingRule.rule_name,
          editingRule.description || null,
          editingRule.priority,
          editingRule.config_json,
          "admin"
        );
      } else {
        // 创建新规则
        await api.createTransformRule(
          editingRule.rule_name,
          editingRule.category,
          editingRule.description || null,
          editingRule.priority,
          editingRule.config_json,
          "admin"
        );
      }

      await loadRules();
      setIsEditing(false);
      setEditingRule(null);
      setShowAddDialog(false);

      // 更新选中的规则
      if (editingRule.rule_id) {
        const updated = rules.find(r => r.rule_id === editingRule.rule_id);
        if (updated) setSelectedRule(updated);
      }
    } catch (err) {
      setError(`保存失败: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  // 删除规则
  const handleDelete = async () => {
    if (!selectedRule?.rule_id) return;
    if (!confirm(`确定要删除规则 "${selectedRule.rule_name}" 吗？`)) return;

    try {
      await api.deleteTransformRule(selectedRule.rule_id, "admin");
      await loadRules();
      setSelectedRule(null);
    } catch (err) {
      setError(`删除失败: ${err}`);
    }
  };

  // 测试规则
  const handleTest = async () => {
    if (!selectedRule?.rule_id) return;

    try {
      setTesting(true);
      setError(null);
      const result = await api.testTransformRule(selectedRule.rule_id, 5);
      setTestResult(result);
    } catch (err) {
      setError(`测试失败: ${err}`);
    } finally {
      setTesting(false);
    }
  };

  // 执行规则
  const handleExecute = async () => {
    if (!selectedRule?.rule_id) return;
    if (!confirm(`确定要执行规则 "${selectedRule.rule_name}" 吗？`)) return;

    try {
      setError(null);
      const result = await api.executeTransformRule(selectedRule.rule_id, "admin");
      alert(`执行完成！\n处理记录: ${result.records_processed}\n修改记录: ${result.records_modified}\n状态: ${result.status}\n${result.error_message || ""}`);
    } catch (err) {
      setError(`执行失败: ${err}`);
    }
  };

  // 查看变更历史
  const handleShowHistory = async () => {
    try {
      const history = await api.getTransformRuleHistory(selectedRule?.rule_id, 20);
      setChangeHistory(history);
      setShowHistory(true);
    } catch (err) {
      setError(`加载历史失败: ${err}`);
    }
  };

  // 添加新规则
  const handleAddRule = () => {
    setEditingRule({
      rule_name: "",
      category: activeCategory,
      description: "",
      priority: 1,
      config_json: "{}",
    });
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

  if (loading && rules.length === 0) {
    return (
      <div className="transform-page">
        <div className="settings-loading">加载中...</div>
      </div>
    );
  }

  return (
    <div className="transform-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <Link to="/settings">设置</Link>
          <span>/</span>
          <span>清洗规则管理</span>
        </div>
        <h1 className="page-header__title">清洗规则管理</h1>
        <p className="page-header__subtitle">配置数据清洗与标准化规则</p>
      </div>

      {error && (
        <div className="settings-error" style={{ marginBottom: "var(--spacing-md)" }}>
          {error}
          <button onClick={() => setError(null)} style={{ marginLeft: "var(--spacing-md)" }}>关闭</button>
        </div>
      )}

      <div className="settings-content">
        {/* 规则分类统计 */}
        <div className="stats-row">
          {RULE_CATEGORIES.map((cat) => {
            const stats = getCategoryStats(cat.id);
            return (
              <div
                key={cat.id}
                className="stat-item"
                style={{ cursor: "pointer" }}
                onClick={() => setActiveCategory(cat.id)}
              >
                <div className="stat-item-label">{cat.icon} {cat.name}</div>
                <div className={`stat-item-value ${activeCategory === cat.id ? "primary" : ""}`}>
                  {stats.enabled}/{stats.total}
                </div>
              </div>
            );
          })}
        </div>

        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-lg)" }}>
          {/* 规则列表 */}
          <div className="settings-section">
            <div className="settings-section-header">
              <h3>
                {RULE_CATEGORIES.find((c) => c.id === activeCategory)?.icon}{" "}
                {RULE_CATEGORIES.find((c) => c.id === activeCategory)?.name}规则
              </h3>
              <button className="settings-btn settings-btn--primary" onClick={handleAddRule}>
                + 添加规则
              </button>
            </div>
            <div className="settings-section-body">
              <div className="rule-list">
                {filteredRules.map((rule) => (
                  <div
                    key={rule.rule_id}
                    className={`rule-item ${rule.enabled === 0 ? "disabled" : ""} ${selectedRule?.rule_id === rule.rule_id ? "selected" : ""}`}
                    onClick={() => handleSelectRule(rule)}
                    style={{ cursor: "pointer" }}
                  >
                    <div className="rule-item-content">
                      <div className="rule-item-name">
                        {rule.rule_name}
                        {selectedRule?.rule_id === rule.rule_id && (
                          <span style={{ marginLeft: "8px", color: "var(--color-primary)" }}>●</span>
                        )}
                      </div>
                      <div className="rule-item-desc">{rule.description || "无描述"}</div>
                    </div>
                    <div className="rule-item-actions">
                      <div
                        className={`toggle-switch ${rule.enabled === 1 ? "active" : ""}`}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleToggleEnabled(rule);
                        }}
                      />
                    </div>
                  </div>
                ))}
                {filteredRules.length === 0 && (
                  <div className="settings-empty">
                    <div className="settings-empty-icon">📋</div>
                    <div className="settings-empty-text">暂无规则</div>
                  </div>
                )}
              </div>
            </div>
          </div>

          {/* 规则详情 */}
          <div className="settings-section">
            <div className="settings-section-header">
              <h3>规则详情</h3>
              {selectedRule && !isEditing && (
                <div className="btn-group">
                  <button className="settings-btn" onClick={handleShowHistory}>历史</button>
                  <button className="settings-btn" onClick={handleTest} disabled={testing}>
                    {testing ? "测试中..." : "测试"}
                  </button>
                  <button className="settings-btn" onClick={handleExecute}>执行</button>
                  <button className="settings-btn" onClick={handleEdit}>编辑</button>
                  <button className="settings-btn settings-btn--danger" onClick={handleDelete}>删除</button>
                </div>
              )}
              {isEditing && (
                <div className="btn-group">
                  <button className="settings-btn" onClick={handleCancelEdit}>取消</button>
                  <button className="settings-btn settings-btn--primary" onClick={handleSave} disabled={saving}>
                    {saving ? "保存中..." : "保存"}
                  </button>
                </div>
              )}
            </div>
            <div className="settings-section-body">
              {selectedRule && !isEditing ? (
                <div>
                  <div className="form-group">
                    <label className="form-label">规则名称</label>
                    <input
                      type="text"
                      className="form-input"
                      value={selectedRule.rule_name}
                      readOnly
                    />
                  </div>
                  <div className="form-group">
                    <label className="form-label">描述</label>
                    <textarea
                      className="form-textarea"
                      value={selectedRule.description || ""}
                      readOnly
                    />
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">优先级</label>
                      <input
                        type="number"
                        className="form-input"
                        value={selectedRule.priority}
                        readOnly
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">状态</label>
                      <input
                        type="text"
                        className="form-input"
                        value={selectedRule.enabled === 1 ? "启用" : "禁用"}
                        readOnly
                      />
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">配置 (JSON)</label>
                    <textarea
                      className="form-textarea"
                      value={JSON.stringify(JSON.parse(selectedRule.config_json), null, 2)}
                      style={{ fontFamily: "monospace", minHeight: "150px" }}
                      readOnly
                    />
                  </div>
                  <div className="form-hint">
                    创建者: {selectedRule.created_by} | 创建时间: {formatTime(selectedRule.created_at)} | 更新时间: {formatTime(selectedRule.updated_at)}
                  </div>

                  {/* 测试结果 */}
                  {testResult && (
                    <div style={{ marginTop: "var(--spacing-lg)" }}>
                      <h4 style={{ marginBottom: "var(--spacing-sm)" }}>测试结果</h4>
                      <div style={{ padding: "var(--spacing-sm)", background: "var(--color-bg-layout)", borderRadius: "var(--border-radius-sm)" }}>
                        <div>匹配记录数: <strong>{testResult.records_matched}</strong></div>
                        <div style={{ marginTop: "var(--spacing-xs)" }}>
                          <span>输入样本:</span>
                          <pre style={{ fontSize: "12px", overflow: "auto", maxHeight: "100px" }}>
                            {JSON.stringify(testResult.input_sample, null, 2)}
                          </pre>
                        </div>
                        <div style={{ marginTop: "var(--spacing-xs)" }}>
                          <span>输出预览:</span>
                          <pre style={{ fontSize: "12px", overflow: "auto", maxHeight: "100px" }}>
                            {JSON.stringify(testResult.output_sample, null, 2)}
                          </pre>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ) : isEditing && editingRule ? (
                <div>
                  <div className="form-group">
                    <label className="form-label">规则名称 *</label>
                    <input
                      type="text"
                      className="form-input"
                      value={editingRule.rule_name}
                      onChange={(e) => setEditingRule({ ...editingRule, rule_name: e.target.value })}
                      placeholder="输入规则名称"
                    />
                  </div>
                  <div className="form-group">
                    <label className="form-label">描述</label>
                    <textarea
                      className="form-textarea"
                      value={editingRule.description}
                      onChange={(e) => setEditingRule({ ...editingRule, description: e.target.value })}
                      placeholder="输入规则描述"
                    />
                  </div>
                  <div className="form-row">
                    <div className="form-group">
                      <label className="form-label">优先级</label>
                      <input
                        type="number"
                        className="form-input"
                        value={editingRule.priority}
                        onChange={(e) => setEditingRule({ ...editingRule, priority: parseInt(e.target.value) || 1 })}
                        min={1}
                      />
                    </div>
                    <div className="form-group">
                      <label className="form-label">分类</label>
                      <select
                        className="form-input"
                        value={editingRule.category}
                        onChange={(e) => setEditingRule({ ...editingRule, category: e.target.value as RuleCategory })}
                        disabled={!!editingRule.rule_id}
                      >
                        {RULE_CATEGORIES.map((cat) => (
                          <option key={cat.id} value={cat.id}>{cat.icon} {cat.name}</option>
                        ))}
                      </select>
                    </div>
                  </div>
                  <div className="form-group">
                    <label className="form-label">配置 (JSON) *</label>
                    <textarea
                      className="form-textarea"
                      value={editingRule.config_json}
                      onChange={(e) => setEditingRule({ ...editingRule, config_json: e.target.value })}
                      style={{ fontFamily: "monospace", minHeight: "200px" }}
                      placeholder='{"type": "规则类型", ...}'
                    />
                  </div>
                </div>
              ) : (
                <div className="settings-empty">
                  <div className="settings-empty-icon">👈</div>
                  <div className="settings-empty-text">选择左侧规则查看详情</div>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* 规则执行顺序 */}
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>规则执行流程</h3>
          </div>
          <div className="settings-section-body">
            <div style={{ display: "flex", alignItems: "center", gap: "var(--spacing-md)", flexWrap: "wrap" }}>
              {RULE_CATEGORIES.map((cat, index) => (
                <div key={cat.id} style={{ display: "flex", alignItems: "center", gap: "var(--spacing-sm)" }}>
                  <div
                    style={{
                      padding: "var(--spacing-sm) var(--spacing-md)",
                      background: "var(--color-bg-layout)",
                      borderRadius: "var(--border-radius-md)",
                      display: "flex",
                      alignItems: "center",
                      gap: "var(--spacing-xs)",
                    }}
                  >
                    <span>{cat.icon}</span>
                    <span style={{ fontSize: "var(--font-size-sm)" }}>{cat.name}</span>
                    <span className="tag success">{getCategoryStats(cat.id).enabled}</span>
                  </div>
                  {index < RULE_CATEGORIES.length - 1 && (
                    <span style={{ color: "var(--color-text-tertiary)" }}>→</span>
                  )}
                </div>
              ))}
            </div>
            <div className="form-hint" style={{ marginTop: "var(--spacing-md)" }}>
              规则按照上述顺序依次执行，每个分类内部按优先级排序
            </div>
          </div>
        </div>
      </div>

      {/* 添加规则对话框 */}
      {showAddDialog && editingRule && (
        <div className="dialog-overlay" onClick={() => setShowAddDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()} style={{ maxWidth: "600px" }}>
            <div className="dialog-header">
              <h3>添加新规则</h3>
              <button className="dialog-close" onClick={() => setShowAddDialog(false)}>×</button>
            </div>
            <div className="dialog-body">
              <div className="form-group">
                <label className="form-label">规则名称 *</label>
                <input
                  type="text"
                  className="form-input"
                  value={editingRule.rule_name}
                  onChange={(e) => setEditingRule({ ...editingRule, rule_name: e.target.value })}
                  placeholder="输入规则名称"
                />
              </div>
              <div className="form-group">
                <label className="form-label">分类</label>
                <select
                  className="form-input"
                  value={editingRule.category}
                  onChange={(e) => setEditingRule({ ...editingRule, category: e.target.value as RuleCategory })}
                >
                  {RULE_CATEGORIES.map((cat) => (
                    <option key={cat.id} value={cat.id}>{cat.icon} {cat.name}</option>
                  ))}
                </select>
              </div>
              <div className="form-group">
                <label className="form-label">描述</label>
                <textarea
                  className="form-textarea"
                  value={editingRule.description}
                  onChange={(e) => setEditingRule({ ...editingRule, description: e.target.value })}
                  placeholder="输入规则描述"
                />
              </div>
              <div className="form-group">
                <label className="form-label">优先级</label>
                <input
                  type="number"
                  className="form-input"
                  value={editingRule.priority}
                  onChange={(e) => setEditingRule({ ...editingRule, priority: parseInt(e.target.value) || 1 })}
                  min={1}
                />
              </div>
              <div className="form-group">
                <label className="form-label">配置 (JSON) *</label>
                <textarea
                  className="form-textarea"
                  value={editingRule.config_json}
                  onChange={(e) => setEditingRule({ ...editingRule, config_json: e.target.value })}
                  style={{ fontFamily: "monospace", minHeight: "150px" }}
                  placeholder='{"type": "规则类型", ...}'
                />
              </div>
            </div>
            <div className="dialog-footer">
              <button className="settings-btn" onClick={() => setShowAddDialog(false)}>取消</button>
              <button
                className="settings-btn settings-btn--primary"
                onClick={handleSave}
                disabled={saving || !editingRule.rule_name || !editingRule.config_json}
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
              <h3>变更历史 {selectedRule ? `- ${selectedRule.rule_name}` : ""}</h3>
              <button className="dialog-close" onClick={() => setShowHistory(false)}>×</button>
            </div>
            <div className="dialog-body">
              {changeHistory.length > 0 ? (
                <table className="settings-table">
                  <thead>
                    <tr>
                      <th>时间</th>
                      <th>规则</th>
                      <th>操作</th>
                      <th>操作人</th>
                      <th>原因</th>
                    </tr>
                  </thead>
                  <tbody>
                    {changeHistory.map((log) => (
                      <tr key={log.change_id}>
                        <td>{formatTime(log.changed_at)}</td>
                        <td>{log.rule_name || `#${log.rule_id}`}</td>
                        <td>
                          <span className={`tag ${log.change_type === "delete" ? "danger" : log.change_type === "create" ? "success" : "warning"}`}>
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
              <button className="settings-btn" onClick={() => setShowHistory(false)}>关闭</button>
            </div>
          </div>
        </div>
      )}

      <style>{`
        .transform-page {
          max-width: 1200px;
        }

        .rule-item.selected {
          background: var(--color-primary-light);
          border-color: var(--color-primary);
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
