import { useState, useEffect } from "react";
import { api, ScoringConfigItem } from "../api/tauri";
import "./ConfigPanel.css";

interface ConfigPanelProps {
  onClose: () => void;
  onConfigChanged?: () => void;
  embedded?: boolean;
}

export function ConfigPanel({ onClose, onConfigChanged, embedded = false }: ConfigPanelProps) {
  const [configs, setConfigs] = useState<ScoringConfigItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string>("");
  const [editingKey, setEditingKey] = useState<string | null>(null);
  const [editValue, setEditValue] = useState<string>("");
  const [userName, setUserName] = useState<string>("admin");
  const [changeReason, setChangeReason] = useState<string>("");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  useEffect(() => {
    loadConfigs();
  }, []);

  const loadConfigs = async () => {
    setLoading(true);
    setError("");
    try {
      const data = await api.getScoringConfigs();
      setConfigs(data);
    } catch (err) {
      setError(`加载配置失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const startEdit = (config: ScoringConfigItem) => {
    setEditingKey(config.config_key);
    setEditValue(config.config_value);
    setChangeReason("");
  };

  const cancelEdit = () => {
    setEditingKey(null);
    setEditValue("");
    setChangeReason("");
  };

  const saveEdit = async (configKey: string) => {
    if (!editValue.trim()) {
      alert("配置值不能为空");
      return;
    }

    if (!changeReason.trim()) {
      alert("请填写变更原因");
      return;
    }

    setLoading(true);
    setError("");

    try {
      await api.updateConfig(configKey, editValue, userName, changeReason);
      await loadConfigs();
      cancelEdit();
      onConfigChanged?.();
    } catch (err) {
      setError(`更新配置失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const filteredConfigs =
    selectedCategory === "all"
      ? configs
      : configs.filter((c) => c.category === selectedCategory);

  // 按类别分组
  const categories = Array.from(new Set(configs.map((c) => c.category)));

  const content = (
    <>
      {/* 用户信息 */}
      <div className="user-info">
        <label>
          操作人：
          <input
            type="text"
            value={userName}
            onChange={(e) => setUserName(e.target.value)}
            placeholder="请输入用户名"
          />
        </label>
      </div>

      {/* 分类筛选 */}
      <div className="category-filter">
        <label>分类筛选：</label>
        <button
          className={selectedCategory === "all" ? "active" : ""}
          onClick={() => setSelectedCategory("all")}
        >
          全部 ({configs.length})
        </button>
        {categories.map((cat) => (
          <button
            key={cat}
            className={selectedCategory === cat ? "active" : ""}
            onClick={() => setSelectedCategory(cat)}
          >
            {cat} ({configs.filter((c) => c.category === cat).length})
          </button>
        ))}
      </div>

      {error && <div className="error">{error}</div>}

      {loading && <div className="loading">加载中...</div>}

      {/* 配置列表 */}
      <div className="config-list">
        <table>
          <thead>
            <tr>
              <th>配置项</th>
              <th>当前值</th>
              <th>类型</th>
              <th>分类</th>
              <th>说明</th>
              <th>约束</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            {filteredConfigs.map((config) => {
              const isEditing = editingKey === config.config_key;

              return (
                <tr key={config.config_key}>
                  <td className="config-key">{config.config_key}</td>
                  <td className="config-value">
                    {isEditing ? (
                      <input
                        type={config.value_type === "number" ? "number" : "text"}
                        value={editValue}
                        onChange={(e) => setEditValue(e.target.value)}
                        min={config.min_value}
                        max={config.max_value}
                        step={config.value_type === "number" ? "0.1" : undefined}
                      />
                    ) : (
                      <span>{config.config_value}</span>
                    )}
                  </td>
                  <td>{config.value_type}</td>
                  <td>
                    <span className={`badge badge-${config.category}`}>
                      {config.category}
                    </span>
                  </td>
                  <td className="description">{config.description || "-"}</td>
                  <td className="constraint">
                    {config.min_value !== undefined &&
                    config.max_value !== undefined
                      ? `${config.min_value} ~ ${config.max_value}`
                      : config.default_value
                      ? `默认: ${config.default_value}`
                      : "-"}
                  </td>
                  <td className="actions">
                    {isEditing ? (
                      <>
                        <input
                          type="text"
                          placeholder="变更原因（必填）"
                          value={changeReason}
                          onChange={(e) => setChangeReason(e.target.value)}
                          className="reason-input"
                        />
                        <button
                          className="btn-small btn-primary"
                          onClick={() => saveEdit(config.config_key)}
                          disabled={loading}
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
                        onClick={() => startEdit(config)}
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

  // 嵌入模式：不显示 modal
  if (embedded) {
    return <div className="config-panel-embedded">{content}</div>;
  }

  // 弹窗模式
  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content config-panel" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>评分配置管理</h2>
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
