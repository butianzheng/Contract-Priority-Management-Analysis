import { useCallback, useEffect, useMemo, useState } from "react";
import { Link } from "react-router-dom";
import {
  api,
  FieldAlignmentChangeLog,
  FieldAlignmentRule,
  TargetFieldDef,
} from "../../api/tauri";
import "./Settings.css";

type MappingDataType = "contracts" | "customers" | "process_difficulty";
type DefaultCondition = "when_empty" | "when_missing" | "always";
type TransformType =
  | "mapping"
  | "regex"
  | "date_format"
  | "formula"
  | "condition"
  | "concat"
  | "split"
  | "script"
  | "trim"
  | "case"
  | "lookup_table";

interface RuleTransformItem {
  id: number;
  field: string;
  type: TransformType;
  config: Record<string, any>;
}

interface DefaultValueItem {
  id: number;
  field: string;
  value: string;
  condition: DefaultCondition;
}

const DATA_TYPES: Array<{ key: MappingDataType; label: string; icon: string }> = [
  { key: "contracts", label: "合同", icon: "📋" },
  { key: "customers", label: "客户", icon: "👥" },
  { key: "process_difficulty", label: "工艺难度", icon: "🔧" },
];

const TRANSFORM_TYPE_LABELS: Record<TransformType, string> = {
  mapping: "值映射",
  regex: "正则替换",
  date_format: "日期格式",
  formula: "数学公式",
  condition: "条件表达式",
  concat: "字段拼接",
  split: "字段拆分",
  script: "自定义表达式",
  trim: "去空格",
  case: "大小写",
  lookup_table: "查找表",
};

const TRANSFORM_DEFAULT_CONFIGS: Record<TransformType, Record<string, any>> = {
  mapping: { values: { VIP: "A", 重点: "A", 普通: "B" } },
  regex: { pattern: "\\s+", replacement: "" },
  date_format: { input_formats: ["YYYY/MM/DD", "DD-MM-YYYY"], output_format: "YYYY-MM-DD" },
  formula: { expression: "value / 1000" },
  condition: { rules: [{ condition: "value > 100", result: "高" }], default: "低" },
  concat: { template: "{field_a}-{field_b}", separator: "-" },
  split: { separator: "-", target_fields: ["field_a", "field_b"] },
  script: { expression: "IF(value > 100, \"高\", \"低\")" },
  trim: {},
  case: { mode: "lower" },
  lookup_table: {
    table_name: "customer_master",
    key_field: "customer_id",
    value_field: "customer_level",
  },
};

const ALLOWED_TRANSFORM_TYPES: TransformType[] = [
  "mapping",
  "regex",
  "date_format",
  "formula",
  "condition",
  "concat",
  "split",
  "script",
  "trim",
  "case",
  "lookup_table",
];

function isTransformType(value: string): value is TransformType {
  return ALLOWED_TRANSFORM_TYPES.includes(value as TransformType);
}

function emptyRule(dataType: MappingDataType): FieldAlignmentRule {
  return {
    rule_name: "",
    data_type: dataType,
    source_type: "manual",
    description: "",
    enabled: 1,
    priority: 1,
    field_mapping: "{}",
    value_transform: "{}",
    default_values: "{}",
    created_by: "admin",
  };
}

function safeParseObject(raw?: string | null): Record<string, any> {
  if (!raw || !raw.trim()) return {};
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return parsed as Record<string, any>;
    }
  } catch {
    return {};
  }
  return {};
}

function parseFieldMapping(raw?: string | null): Record<string, string[]> {
  const parsed = safeParseObject(raw);
  const out: Record<string, string[]> = {};
  Object.entries(parsed).forEach(([field, value]) => {
    if (Array.isArray(value)) {
      out[field] = value.map((v) => String(v)).filter(Boolean);
    } else if (typeof value === "string") {
      out[field] = value
        .split(",")
        .map((s) => s.trim())
        .filter(Boolean);
    }
  });
  return out;
}

function parseTransformStep(step: any): { type: TransformType; config: Record<string, any> } | null {
  if (!step || typeof step !== "object") return null;
  const type = String(step.type || "").toLowerCase();
  if (!isTransformType(type)) return null;

  const scope = step.config && typeof step.config === "object" ? step.config : step;

  return {
    type,
    config: { ...scope },
  };
}

function parseValueTransforms(raw?: string | null): RuleTransformItem[] {
  const parsed = safeParseObject(raw);
  const items: RuleTransformItem[] = [];
  let id = 1;

  Object.entries(parsed).forEach(([field, conf]) => {
    const pushStep = (step: any) => {
      const parsedStep = parseTransformStep(step);
      if (!parsedStep) return;
      const { type, config } = parsedStep;
      const { type: _drop, config: _dropConfig, ...rest } = config;
      items.push({ id: id++, field, type, config: rest });
    };

    if (Array.isArray(conf)) {
      conf.forEach(pushStep);
      return;
    }

    if (conf && typeof conf === "object" && Array.isArray((conf as any).steps)) {
      (conf as any).steps.forEach(pushStep);
      return;
    }

    pushStep(conf);
  });

  return items;
}

function parseDefaultValues(raw?: string | null): DefaultValueItem[] {
  const parsed = safeParseObject(raw);
  const items: DefaultValueItem[] = [];
  let id = 1;

  Object.entries(parsed).forEach(([field, conf]) => {
    if (!conf || typeof conf !== "object") return;
    items.push({
      id: id++,
      field,
      value: String((conf as any).value || ""),
      condition: (["when_empty", "when_missing", "always"] as const).includes((conf as any).condition)
        ? (conf as any).condition
        : "when_empty",
    });
  });

  return items;
}

function serializeFieldMapping(mappingText: Record<string, string>): string {
  const out: Record<string, string[]> = {};
  Object.entries(mappingText).forEach(([field, raw]) => {
    const aliases = raw
      .split(",")
      .map((s) => s.trim())
      .filter(Boolean);
    if (aliases.length > 0) {
      out[field] = aliases;
    }
  });
  return JSON.stringify(out, null, 2);
}

function cleanTransformConfig(type: TransformType, config: Record<string, any>): Record<string, any> {
  switch (type) {
    case "mapping": {
      const values = config.values && typeof config.values === "object" ? config.values : {};
      const normalizedValues: Record<string, string> = {};
      Object.entries(values).forEach(([k, v]) => {
        const key = String(k || "").trim();
        if (key) normalizedValues[key] = String(v ?? "");
      });
      const out: Record<string, any> = { values: normalizedValues };
      if (config.fallback !== undefined && String(config.fallback).trim()) {
        out.fallback = String(config.fallback);
      }
      return out;
    }
    case "regex":
      return {
        pattern: String(config.pattern || ""),
        replacement: String(config.replacement || ""),
      };
    case "date_format":
      return {
        input_formats: Array.isArray(config.input_formats)
          ? config.input_formats.map((s: any) => String(s)).filter(Boolean)
          : [],
        output_format: String(config.output_format || "YYYY-MM-DD"),
      };
    case "formula":
    case "script":
      return { expression: String(config.expression || "") };
    case "condition":
      return {
        rules: Array.isArray(config.rules)
          ? config.rules
              .filter((r: any) => r && String(r.condition || "").trim())
              .map((r: any) => ({
                condition: String(r.condition || ""),
                result: String(r.result || ""),
              }))
          : [],
        default: config.default !== undefined ? String(config.default) : undefined,
      };
    case "concat":
      return {
        template: String(config.template || ""),
        separator: config.separator !== undefined ? String(config.separator) : undefined,
      };
    case "split":
      return {
        separator: String(config.separator || ","),
        target_fields: Array.isArray(config.target_fields)
          ? config.target_fields.map((s: any) => String(s)).filter(Boolean)
          : [],
      };
    case "trim":
      return {};
    case "case":
      return { mode: String(config.mode || "lower") };
    case "lookup_table":
      return {
        table_name: String(config.table_name || ""),
        key_field: String(config.key_field || ""),
        value_field: String(config.value_field || ""),
      };
    default:
      return {};
  }
}

function serializeValueTransforms(items: RuleTransformItem[]): string | null {
  const grouped: Record<string, any[]> = {};

  items.forEach((item) => {
    const field = item.field.trim();
    if (!field) return;
    if (!grouped[field]) grouped[field] = [];

    grouped[field].push({
      type: item.type,
      ...cleanTransformConfig(item.type, item.config || {}),
    });
  });

  const out: Record<string, any> = {};
  Object.entries(grouped).forEach(([field, steps]) => {
    if (steps.length === 1) out[field] = steps[0];
    else out[field] = { steps };
  });

  if (Object.keys(out).length === 0) return null;
  return JSON.stringify(out, null, 2);
}

function serializeDefaultValues(items: DefaultValueItem[]): string | null {
  const out: Record<string, any> = {};
  items.forEach((item) => {
    const field = item.field.trim();
    if (!field) return;
    out[field] = {
      value: item.value,
      condition: item.condition,
    };
  });
  if (Object.keys(out).length === 0) return null;
  return JSON.stringify(out, null, 2);
}

function countMappingFields(raw?: string | null): number | null {
  if (!raw) return null;
  try {
    const parsed = JSON.parse(raw);
    if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
      return Object.keys(parsed as Record<string, unknown>).length;
    }
  } catch {
    return null;
  }
  return null;
}

function summarizeLog(log: FieldAlignmentChangeLog): string {
  const oldCount = countMappingFields(log.old_value);
  const newCount = countMappingFields(log.new_value);

  if (log.change_type === "create") {
    return newCount !== null ? `新增映射字段 ${newCount} 个` : "创建规则";
  }
  if (log.change_type === "delete") {
    return oldCount !== null ? `删除映射字段 ${oldCount} 个` : "删除规则";
  }
  if (log.change_type === "update") {
    if (oldCount !== null && newCount !== null) {
      const delta = newCount - oldCount;
      if (delta > 0) return `映射字段 +${delta}（${oldCount} → ${newCount}）`;
      if (delta < 0) return `映射字段 ${delta}（${oldCount} → ${newCount}）`;
      return `映射字段数量不变（${newCount}）`;
    }
    return "更新规则";
  }
  return log.change_type;
}

export function FieldMappingPage() {
  const [activeType, setActiveType] = useState<MappingDataType>("contracts");
  const [rules, setRules] = useState<FieldAlignmentRule[]>([]);
  const [selectedRuleId, setSelectedRuleId] = useState<number | null>(null);
  const [editingRule, setEditingRule] = useState<FieldAlignmentRule>(emptyRule("contracts"));
  const [targetFields, setTargetFields] = useState<TargetFieldDef[]>([]);

  const [mappingText, setMappingText] = useState<Record<string, string>>({});
  const [transformItems, setTransformItems] = useState<RuleTransformItem[]>([]);
  const [defaultItems, setDefaultItems] = useState<DefaultValueItem[]>([]);
  const [nextTransformId, setNextTransformId] = useState(1);
  const [nextDefaultId, setNextDefaultId] = useState(1);

  const [changeLogs, setChangeLogs] = useState<FieldAlignmentChangeLog[]>([]);
  const [loading, setLoading] = useState(false);
  const [logsLoading, setLogsLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const selectedRule = useMemo(
    () => rules.find((r) => r.rule_id === selectedRuleId) || null,
    [rules, selectedRuleId]
  );

  const targetFieldMap = useMemo(() => {
    const map: Record<string, TargetFieldDef> = {};
    targetFields.forEach((field) => {
      map[field.name] = field;
    });
    return map;
  }, [targetFields]);

  const mappingFieldKeys = useMemo(() => {
    const keys = new Set<string>(targetFields.map((f) => f.name));
    Object.keys(mappingText).forEach((k) => keys.add(k));
    return Array.from(keys);
  }, [targetFields, mappingText]);

  const generatedMappingJson = useMemo(() => serializeFieldMapping(mappingText), [mappingText]);
  const generatedTransformsJson = useMemo(() => serializeValueTransforms(transformItems) || "{}", [transformItems]);
  const generatedDefaultsJson = useMemo(() => serializeDefaultValues(defaultItems) || "{}", [defaultItems]);

  const syncEditorFromRule = useCallback(
    (rule: FieldAlignmentRule) => {
      const parsedMapping = parseFieldMapping(rule.field_mapping);
      const text: Record<string, string> = {};
      Object.entries(parsedMapping).forEach(([field, aliases]) => {
        text[field] = aliases.join(", ");
      });
      setMappingText(text);

      const parsedTransforms = parseValueTransforms(rule.value_transform);
      setTransformItems(parsedTransforms);
      setNextTransformId((parsedTransforms.reduce((max, item) => Math.max(max, item.id), 0) || 0) + 1);

      const parsedDefaults = parseDefaultValues(rule.default_values);
      setDefaultItems(parsedDefaults);
      setNextDefaultId((parsedDefaults.reduce((max, item) => Math.max(max, item.id), 0) || 0) + 1);
    },
    []
  );

  const loadTargetFields = useCallback(async () => {
    try {
      const fields = await api.getTargetFields(activeType);
      setTargetFields(fields);
    } catch (err) {
      setError(`加载目标字段失败: ${err}`);
      setTargetFields([]);
    }
  }, [activeType]);

  const loadChangeLogs = useCallback(async (ruleId?: number | null) => {
    if (!ruleId) {
      setChangeLogs([]);
      return;
    }

    try {
      setLogsLoading(true);
      const logs = await api.getFieldAlignmentChangeLogs(ruleId, 30);
      setChangeLogs(logs);
    } catch (err) {
      setError(`加载变更历史失败: ${err}`);
    } finally {
      setLogsLoading(false);
    }
  }, []);

  const loadRules = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const data = await api.getFieldAlignmentRules(activeType, true);
      setRules(data);

      if (data.length > 0) {
        const selected = data.find((r) => r.rule_id === selectedRuleId) || data[0];
        setSelectedRuleId(selected.rule_id || null);
        setEditingRule({ ...selected });
        syncEditorFromRule(selected);
      } else {
        setSelectedRuleId(null);
        const rule = emptyRule(activeType);
        setEditingRule(rule);
        syncEditorFromRule(rule);
      }
    } catch (err) {
      setError(`加载字段映射规则失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [activeType, selectedRuleId, syncEditorFromRule]);

  useEffect(() => {
    loadTargetFields();
  }, [loadTargetFields]);

  useEffect(() => {
    loadRules();
  }, [loadRules]);

  useEffect(() => {
    if (selectedRuleId) {
      loadChangeLogs(selectedRuleId);
    } else {
      setChangeLogs([]);
    }
  }, [loadChangeLogs, selectedRuleId]);

  const handleSelectRule = (rule: FieldAlignmentRule) => {
    setSelectedRuleId(rule.rule_id || null);
    setEditingRule({ ...rule });
    syncEditorFromRule(rule);
  };

  const handleCreateRule = () => {
    setSelectedRuleId(null);
    const rule = emptyRule(activeType);
    setEditingRule(rule);
    syncEditorFromRule(rule);
  };

  const handleSaveRule = async () => {
    try {
      setSaving(true);
      setError(null);

      if (!editingRule.rule_name.trim()) {
        throw new Error("规则名称不能为空");
      }

      const toSave: FieldAlignmentRule = {
        ...editingRule,
        data_type: activeType,
        created_by: editingRule.created_by || "admin",
        field_mapping: generatedMappingJson,
        value_transform: generatedTransformsJson,
        default_values: generatedDefaultsJson,
      };

      JSON.parse(toSave.field_mapping || "{}");
      if (toSave.value_transform?.trim()) {
        JSON.parse(toSave.value_transform);
      }
      if (toSave.default_values?.trim()) {
        JSON.parse(toSave.default_values);
      }

      const ruleId = await api.saveFieldAlignmentRule(toSave, "admin");
      setSelectedRuleId(ruleId);
      await loadRules();
      await loadChangeLogs(ruleId);
    } catch (err) {
      setError(`保存失败: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDeleteRule = async () => {
    if (!selectedRule?.rule_id) return;
    if (!confirm(`确定删除规则“${selectedRule.rule_name}”吗？`)) return;

    try {
      setError(null);
      await api.deleteFieldAlignmentRule(selectedRule.rule_id, "admin");
      await loadRules();
      setChangeLogs([]);
    } catch (err) {
      setError(`删除失败: ${err}`);
    }
  };

  const updateTransformItem = useCallback((id: number, patch: Partial<RuleTransformItem>) => {
    setTransformItems((prev) => prev.map((item) => (item.id === id ? { ...item, ...patch } : item)));
  }, []);

  const updateTransformConfig = useCallback((id: number, patch: Record<string, any>) => {
    setTransformItems((prev) =>
      prev.map((item) => (
        item.id === id
          ? { ...item, config: { ...(item.config || {}), ...patch } }
          : item
      ))
    );
  }, []);

  const addTransform = () => {
    const defaultField = targetFields[0]?.name || "";
    setTransformItems((prev) => [
      ...prev,
      {
        id: nextTransformId,
        field: defaultField,
        type: "mapping",
        config: { ...TRANSFORM_DEFAULT_CONFIGS.mapping },
      },
    ]);
    setNextTransformId((v) => v + 1);
  };

  const addDefaultValue = () => {
    const defaultField = targetFields[0]?.name || "";
    setDefaultItems((prev) => [
      ...prev,
      {
        id: nextDefaultId,
        field: defaultField,
        value: "",
        condition: "when_empty",
      },
    ]);
    setNextDefaultId((v) => v + 1);
  };

  return (
    <div className="transform-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <Link to="/settings">设置</Link>
          <span>/</span>
          <span>字段映射管理</span>
        </div>
        <h1 className="page-header__title">字段映射管理</h1>
        <p className="page-header__subtitle">维护导入字段对齐规则、值转换与默认值</p>
      </div>

      {error && (
        <div className="settings-error" style={{ marginBottom: "var(--spacing-md)" }}>
          {error}
          <button onClick={() => setError(null)} style={{ marginLeft: "var(--spacing-md)" }}>
            关闭
          </button>
        </div>
      )}

      <div className="settings-content" style={{ display: "grid", gridTemplateColumns: "280px 1fr", gap: "var(--spacing-lg)" }}>
        <div className="settings-section">
          <div className="settings-section-header">
            <h3>规则列表</h3>
            <button className="settings-btn settings-btn--primary" onClick={handleCreateRule}>
              新建
            </button>
          </div>

          <div style={{ display: "flex", gap: "var(--spacing-xs)", marginBottom: "var(--spacing-sm)" }}>
            {DATA_TYPES.map((type) => (
              <button
                key={type.key}
                className="settings-btn"
                style={{
                  background: activeType === type.key ? "var(--color-primary-light)" : undefined,
                  borderColor: activeType === type.key ? "var(--color-primary)" : undefined,
                }}
                onClick={() => setActiveType(type.key)}
              >
                {type.icon} {type.label}
              </button>
            ))}
          </div>

          {loading ? (
            <div className="settings-loading">加载中...</div>
          ) : rules.length === 0 ? (
            <div style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>暂无规则</div>
          ) : (
            <div style={{ display: "flex", flexDirection: "column", gap: "var(--spacing-xs)" }}>
              {rules.map((rule) => (
                <button
                  key={rule.rule_id}
                  className="settings-btn"
                  style={{
                    textAlign: "left",
                    justifyContent: "space-between",
                    display: "flex",
                    background: selectedRuleId === rule.rule_id ? "var(--color-primary-light)" : undefined,
                    borderColor: selectedRuleId === rule.rule_id ? "var(--color-primary)" : undefined,
                  }}
                  onClick={() => handleSelectRule(rule)}
                >
                  <span>{rule.rule_name}</span>
                  <span style={{ color: rule.enabled === 1 ? "var(--color-success)" : "var(--color-text-tertiary)" }}>
                    {rule.enabled === 1 ? "启用" : "禁用"}
                  </span>
                </button>
              ))}
            </div>
          )}
        </div>

        <div className="settings-section">
          <div className="settings-section-header">
            <h3>{selectedRule ? `编辑规则: ${selectedRule.rule_name}` : "新建规则"}</h3>
            <div className="btn-group">
              <button className="settings-btn" onClick={loadRules} disabled={loading}>刷新</button>
              {selectedRule?.rule_id && (
                <button className="settings-btn" onClick={handleDeleteRule}>删除</button>
              )}
              <button className="settings-btn settings-btn--primary" onClick={handleSaveRule} disabled={saving}>
                {saving ? "保存中..." : "保存"}
              </button>
            </div>
          </div>

          <div className="settings-form-row">
            <label>规则名称</label>
            <input
              className="form-control"
              value={editingRule.rule_name}
              onChange={(e) => setEditingRule((prev) => ({ ...prev, rule_name: e.target.value }))}
            />
          </div>

          <div className="settings-form-row" style={{ marginTop: "var(--spacing-sm)" }}>
            <label>数据来源类型</label>
            <input
              className="form-control"
              value={editingRule.source_type || ""}
              onChange={(e) => setEditingRule((prev) => ({ ...prev, source_type: e.target.value }))}
              placeholder="例如：ERP / excel_template"
            />
          </div>

          <div className="settings-form-row" style={{ marginTop: "var(--spacing-sm)" }}>
            <label>描述</label>
            <input
              className="form-control"
              value={editingRule.description || ""}
              onChange={(e) => setEditingRule((prev) => ({ ...prev, description: e.target.value }))}
            />
          </div>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-sm)", marginTop: "var(--spacing-sm)" }}>
            <div className="settings-form-row">
              <label>优先级</label>
              <input
                className="form-control"
                type="number"
                value={editingRule.priority}
                onChange={(e) => setEditingRule((prev) => ({ ...prev, priority: Number(e.target.value || 1) }))}
              />
            </div>
            <div className="settings-form-row">
              <label>状态</label>
              <select
                className="form-control"
                value={editingRule.enabled}
                onChange={(e) => setEditingRule((prev) => ({ ...prev, enabled: Number(e.target.value) }))}
              >
                <option value={1}>启用</option>
                <option value={0}>禁用</option>
              </select>
            </div>
          </div>

          <div style={{ marginTop: "var(--spacing-lg)" }}>
            <h4 style={{ marginTop: 0, marginBottom: "var(--spacing-sm)" }}>字段别名映射</h4>
            <div style={{ border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)", overflow: "hidden" }}>
              <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                <thead>
                  <tr style={{ background: "var(--color-bg-layout)" }}>
                    <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>目标字段</th>
                    <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>别名列表（逗号分隔）</th>
                  </tr>
                </thead>
                <tbody>
                  {mappingFieldKeys.map((field) => {
                    const meta = targetFieldMap[field];
                    return (
                      <tr key={field}>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                          {meta ? (
                            <>
                              {meta.display_name}
                              <span style={{ color: "var(--color-text-tertiary)", marginLeft: 6 }}>({meta.name})</span>
                              {meta.required && <span style={{ color: "var(--color-error)", marginLeft: 4 }}>*</span>}
                            </>
                          ) : (
                            field
                          )}
                        </td>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                          <input
                            className="form-control"
                            value={mappingText[field] || ""}
                            onChange={(e) => setMappingText((prev) => ({ ...prev, [field]: e.target.value }))}
                            placeholder="例如：合同号, 合同编号, Contract_ID"
                          />
                        </td>
                      </tr>
                    );
                  })}
                </tbody>
              </table>
            </div>
          </div>

          <div style={{ marginTop: "var(--spacing-lg)" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--spacing-sm)" }}>
              <h4 style={{ margin: 0 }}>值转换规则</h4>
              <button className="settings-btn" onClick={addTransform}>+ 添加规则</button>
            </div>

            {transformItems.length === 0 ? (
              <div style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>暂无转换规则</div>
            ) : (
              <div style={{ display: "grid", gap: "var(--spacing-sm)" }}>
                {transformItems.map((item) => (
                  <div key={item.id} style={{ border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)", padding: "var(--spacing-sm)" }}>
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr auto", gap: "var(--spacing-sm)", marginBottom: "var(--spacing-sm)" }}>
                      <select
                        className="form-control"
                        value={item.field}
                        onChange={(e) => updateTransformItem(item.id, { field: e.target.value })}
                      >
                        {targetFields.map((field) => (
                          <option key={field.name} value={field.name}>{field.display_name} ({field.name})</option>
                        ))}
                      </select>
                      <select
                        className="form-control"
                        value={item.type}
                        onChange={(e) => {
                          const nextType = e.target.value as TransformType;
                          updateTransformItem(item.id, { type: nextType, config: { ...TRANSFORM_DEFAULT_CONFIGS[nextType] } });
                        }}
                      >
                        {Object.entries(TRANSFORM_TYPE_LABELS).map(([k, label]) => (
                          <option key={k} value={k}>{label}</option>
                        ))}
                      </select>
                      <button className="settings-btn" onClick={() => setTransformItems((prev) => prev.filter((it) => it.id !== item.id))}>
                        删除
                      </button>
                    </div>

                    {item.type === "mapping" && (
                      <div style={{ display: "grid", gridTemplateColumns: "1fr auto", gap: "var(--spacing-sm)" }}>
                        <textarea
                          className="form-control"
                          style={{ minHeight: "72px", fontFamily: "monospace" }}
                          value={Object.entries(item.config?.values || {}).map(([k, v]) => `${k} => ${String(v)}`).join("\n")}
                          onChange={(e) => {
                            const values: Record<string, string> = {};
                            e.target.value.split("\n").forEach((line) => {
                              const [left, right] = line.split("=>").map((part) => part?.trim());
                              if (left) values[left] = right || "";
                            });
                            updateTransformConfig(item.id, { values });
                          }}
                          placeholder={"VIP => A\n重点 => A\n普通 => B"}
                        />
                        <input
                          className="form-control"
                          placeholder="fallback(可选)"
                          value={item.config?.fallback || ""}
                          onChange={(e) => updateTransformConfig(item.id, { fallback: e.target.value })}
                        />
                      </div>
                    )}

                    {item.type === "regex" && (
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-sm)" }}>
                        <input className="form-control" placeholder="pattern" value={item.config?.pattern || ""} onChange={(e) => updateTransformConfig(item.id, { pattern: e.target.value })} />
                        <input className="form-control" placeholder="replacement" value={item.config?.replacement || ""} onChange={(e) => updateTransformConfig(item.id, { replacement: e.target.value })} />
                      </div>
                    )}

                    {item.type === "date_format" && (
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-sm)" }}>
                        <input
                          className="form-control"
                          placeholder="输入格式，逗号分隔"
                          value={Array.isArray(item.config?.input_formats) ? item.config.input_formats.join(", ") : ""}
                          onChange={(e) => updateTransformConfig(item.id, { input_formats: e.target.value.split(",").map((s) => s.trim()).filter(Boolean) })}
                        />
                        <input className="form-control" placeholder="输出格式" value={item.config?.output_format || ""} onChange={(e) => updateTransformConfig(item.id, { output_format: e.target.value })} />
                      </div>
                    )}

                    {item.type === "formula" && (
                      <input className="form-control" placeholder="例如: value / 1000" value={item.config?.expression || ""} onChange={(e) => updateTransformConfig(item.id, { expression: e.target.value })} />
                    )}

                    {item.type === "condition" && (
                      <div style={{ display: "grid", gap: "var(--spacing-sm)" }}>
                        <textarea
                          className="form-control"
                          style={{ minHeight: "68px", fontFamily: "monospace" }}
                          placeholder={"value > 100 => 高\nvalue > 50 => 中"}
                          value={(item.config?.rules || []).map((r: any) => `${r.condition || ""} => ${r.result || ""}`).join("\n")}
                          onChange={(e) => {
                            const rules = e.target.value
                              .split("\n")
                              .map((line) => line.split("=>").map((part) => part.trim()))
                              .filter(([condition]) => !!condition)
                              .map(([condition, result]) => ({ condition, result: result || "" }));
                            updateTransformConfig(item.id, { rules });
                          }}
                        />
                        <input className="form-control" placeholder="default(可选)" value={item.config?.default || ""} onChange={(e) => updateTransformConfig(item.id, { default: e.target.value })} />
                      </div>
                    )}

                    {item.type === "concat" && (
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 220px", gap: "var(--spacing-sm)" }}>
                        <input className="form-control" placeholder="{field_a}-{field_b}" value={item.config?.template || ""} onChange={(e) => updateTransformConfig(item.id, { template: e.target.value })} />
                        <input className="form-control" placeholder="separator(可选)" value={item.config?.separator || ""} onChange={(e) => updateTransformConfig(item.id, { separator: e.target.value })} />
                      </div>
                    )}

                    {item.type === "split" && (
                      <div style={{ display: "grid", gridTemplateColumns: "220px 1fr", gap: "var(--spacing-sm)" }}>
                        <input className="form-control" placeholder="separator" value={item.config?.separator || ""} onChange={(e) => updateTransformConfig(item.id, { separator: e.target.value })} />
                        <input
                          className="form-control"
                          placeholder="target fields，逗号分隔"
                          value={Array.isArray(item.config?.target_fields) ? item.config.target_fields.join(", ") : ""}
                          onChange={(e) => updateTransformConfig(item.id, { target_fields: e.target.value.split(",").map((s) => s.trim()).filter(Boolean) })}
                        />
                      </div>
                    )}

                    {item.type === "script" && (
                      <input className="form-control" placeholder='例如: IF(value > 100, "高", "低")' value={item.config?.expression || ""} onChange={(e) => updateTransformConfig(item.id, { expression: e.target.value })} />
                    )}

                    {item.type === "trim" && (
                      <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
                        无额外配置，执行字段值去前后空格。
                      </div>
                    )}

                    {item.type === "case" && (
                      <select className="form-control" style={{ width: "220px" }} value={item.config?.mode || "lower"} onChange={(e) => updateTransformConfig(item.id, { mode: e.target.value })}>
                        <option value="upper">UPPER</option>
                        <option value="lower">lower</option>
                        <option value="title">Title</option>
                      </select>
                    )}

                    {item.type === "lookup_table" && (
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "var(--spacing-sm)" }}>
                        <input className="form-control" placeholder="table_name" value={item.config?.table_name || ""} onChange={(e) => updateTransformConfig(item.id, { table_name: e.target.value })} />
                        <input className="form-control" placeholder="key_field" value={item.config?.key_field || ""} onChange={(e) => updateTransformConfig(item.id, { key_field: e.target.value })} />
                        <input className="form-control" placeholder="value_field" value={item.config?.value_field || ""} onChange={(e) => updateTransformConfig(item.id, { value_field: e.target.value })} />
                      </div>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>

          <div style={{ marginTop: "var(--spacing-lg)" }}>
            <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "var(--spacing-sm)" }}>
              <h4 style={{ margin: 0 }}>默认值配置</h4>
              <button className="settings-btn" onClick={addDefaultValue}>+ 添加默认值</button>
            </div>

            {defaultItems.length === 0 ? (
              <div style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>暂无默认值配置</div>
            ) : (
              <div style={{ display: "grid", gap: "var(--spacing-xs)" }}>
                {defaultItems.map((item) => (
                  <div key={item.id} style={{ display: "grid", gridTemplateColumns: "1fr 1fr 180px auto", gap: "var(--spacing-sm)" }}>
                    <select
                      className="form-control"
                      value={item.field}
                      onChange={(e) => setDefaultItems((prev) => prev.map((it) => (it.id === item.id ? { ...it, field: e.target.value } : it)))}
                    >
                      {targetFields.map((field) => (
                        <option key={field.name} value={field.name}>{field.display_name} ({field.name})</option>
                      ))}
                    </select>
                    <input
                      className="form-control"
                      value={item.value}
                      onChange={(e) => setDefaultItems((prev) => prev.map((it) => (it.id === item.id ? { ...it, value: e.target.value } : it)))}
                      placeholder="默认值"
                    />
                    <select
                      className="form-control"
                      value={item.condition}
                      onChange={(e) => setDefaultItems((prev) => prev.map((it) => (it.id === item.id ? { ...it, condition: e.target.value as DefaultCondition } : it)))}
                    >
                      <option value="when_empty">当值为空</option>
                      <option value="when_missing">当字段缺失</option>
                      <option value="always">始终覆盖</option>
                    </select>
                    <button className="settings-btn" onClick={() => setDefaultItems((prev) => prev.filter((it) => it.id !== item.id))}>删除</button>
                  </div>
                ))}
              </div>
            )}
          </div>

          <div style={{ marginTop: "var(--spacing-md)", color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
            <details>
              <summary style={{ cursor: "pointer", color: "var(--color-primary)" }}>查看生成 JSON</summary>
              <pre style={{ marginTop: 8, padding: 8, background: "var(--color-bg-layout)", borderRadius: 4, whiteSpace: "pre-wrap" }}>
{`field_mapping:\n${generatedMappingJson}\n\nvalue_transform:\n${generatedTransformsJson}\n\ndefault_values:\n${generatedDefaultsJson}`}
              </pre>
            </details>
          </div>

          <div style={{ marginTop: "var(--spacing-lg)", borderTop: "1px solid var(--color-border-light)", paddingTop: "var(--spacing-md)" }}>
            <h4 style={{ marginTop: 0, marginBottom: "var(--spacing-sm)" }}>变更历史</h4>
            {logsLoading ? (
              <div className="settings-loading">加载中...</div>
            ) : changeLogs.length === 0 ? (
              <div style={{ color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
                暂无历史记录
              </div>
            ) : (
              <div style={{ maxHeight: "220px", overflow: "auto", border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)" }}>
                <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                  <thead>
                    <tr style={{ background: "var(--color-bg-layout)" }}>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>时间</th>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>类型</th>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>操作人</th>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>详情</th>
                    </tr>
                  </thead>
                  <tbody>
                    {changeLogs.map((log) => (
                      <tr key={log.log_id}>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{log.changed_at || "-"}</td>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{log.change_type}</td>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{log.changed_by}</td>
                        <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                          <div>{summarizeLog(log)}</div>
                          {(log.old_value || log.new_value) && (
                            <details style={{ marginTop: 4 }}>
                              <summary style={{ cursor: "pointer", color: "var(--color-primary)" }}>查看 JSON</summary>
                              <pre style={{ marginTop: 6, padding: 8, background: "var(--color-bg-layout)", borderRadius: 4, whiteSpace: "pre-wrap" }}>
                                {`old:\n${log.old_value || "-"}\n\nnew:\n${log.new_value || "-"}`}
                              </pre>
                            </details>
                          )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
