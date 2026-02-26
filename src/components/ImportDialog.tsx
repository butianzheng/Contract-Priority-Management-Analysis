import { useMemo, useState, useCallback } from "react";
import { open } from "@tauri-apps/api/dialog";
import {
  api,
  ImportPreview,
  ConflictRecord,
  FileFormat,
  ImportDataType,
  ConflictStrategy,
  TargetFieldDef,
  FieldMappingResult,
} from "../api/tauri";
import "./Dialog.css";

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  dataType: ImportDataType;
  onImportComplete: () => void;
}

type ImportStep =
  | "select"
  | "mapping"
  | "transform"
  | "preview"
  | "importing"
  | "complete";

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

interface TransformItem {
  id: number;
  field: string;
  type: TransformType;
  config: Record<string, any>;
}

const DATA_TYPE_LABELS: Record<ImportDataType, string> = {
  contracts: "合同数据",
  customers: "客户数据",
  process_difficulty: "工艺难度",
  strategy_weights: "策略权重",
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
  lookup_table: { table_name: "customer_master", key_field: "customer_id", value_field: "customer_level" },
};

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

function confidenceColor(value?: number): string {
  if (value === undefined) return "var(--color-text-tertiary)";
  if (value >= 0.9) return "var(--color-success)";
  if (value >= 0.6) return "var(--color-warning)";
  return "var(--color-error)";
}

export function ImportDialog({
  isOpen,
  onClose,
  dataType,
  onImportComplete,
}: ImportDialogProps) {
  const [step, setStep] = useState<ImportStep>("select");
  const [filePath, setFilePath] = useState<string | null>(null);
  const [fileFormat, setFileFormat] = useState<FileFormat>("csv");
  const [preview, setPreview] = useState<ImportPreview | null>(null);
  const [conflictStrategy, setConflictStrategy] = useState<ConflictStrategy>("skip");
  const [conflictDecisions, setConflictDecisions] = useState<ConflictRecord[]>([]);
  const [importResult, setImportResult] = useState<{ success: boolean; message: string } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [sourceHeaders, setSourceHeaders] = useState<string[]>([]);
  const [targetFields, setTargetFields] = useState<TargetFieldDef[]>([]);
  const [autoMappingResult, setAutoMappingResult] = useState<FieldMappingResult | null>(null);
  const [fieldMapping, setFieldMapping] = useState<Record<string, string>>({});

  const [transformItems, setTransformItems] = useState<TransformItem[]>([]);
  const [nextTransformId, setNextTransformId] = useState(1);
  const [activeValueTransforms, setActiveValueTransforms] = useState<Record<string, any> | undefined>(undefined);

  const [expressionInput, setExpressionInput] = useState("IF(value > 100, \"高\", \"低\")");
  const [expressionSample, setExpressionSample] = useState('{"value": "120"}');
  const [expressionOutput, setExpressionOutput] = useState<string | null>(null);

  const mappedRequiredMissing = useMemo(() => {
    return targetFields.filter((f) => f.required && !fieldMapping[f.name]).map((f) => f.name);
  }, [targetFields, fieldMapping]);

  const normalizedMapping = useMemo(() => {
    const entries = Object.entries(fieldMapping).filter(([, source]) => source && source.trim().length > 0);
    return Object.fromEntries(entries);
  }, [fieldMapping]);

  const unusedSourceHeaders = useMemo(() => {
    const used = new Set(Object.values(normalizedMapping));
    return sourceHeaders.filter((h) => !used.has(h));
  }, [sourceHeaders, normalizedMapping]);

  const conflictActionStats = useMemo(() => {
    let skip = 0;
    let overwrite = 0;
    for (const item of conflictDecisions) {
      const action = item.action ?? conflictStrategy;
      if (action === "overwrite") overwrite += 1;
      else skip += 1;
    }
    return { skip, overwrite };
  }, [conflictDecisions, conflictStrategy]);

  const handleSelectFile = useCallback(async () => {
    try {
      const selected = await open({
        filters: [
          { name: "支持的格式", extensions: ["csv", "xlsx", "json"] },
          { name: "CSV 文件", extensions: ["csv"] },
          { name: "Excel 文件", extensions: ["xlsx"] },
          { name: "JSON 文件", extensions: ["json"] },
        ],
        title: `选择要导入的${DATA_TYPE_LABELS[dataType]}文件`,
      });

      if (selected && typeof selected === "string") {
        setFilePath(selected);
        const ext = selected.split(".").pop()?.toLowerCase();
        if (ext === "csv") setFileFormat("csv");
        else if (ext === "xlsx" || ext === "xls") setFileFormat("excel");
        else if (ext === "json") setFileFormat("json");
      }
    } catch (err) {
      setError(`选择文件失败: ${err}`);
    }
  }, [dataType]);

  const handleLoadMappingContext = useCallback(async () => {
    if (!filePath) return;

    try {
      setLoading(true);
      setError(null);
      const [headers, targets, detected] = await Promise.all([
        api.parseFileHeaders(filePath, fileFormat),
        api.getTargetFields(dataType),
        api.autoDetectMapping(filePath, fileFormat, dataType),
      ]);

      setSourceHeaders(headers);
      setTargetFields(targets);
      setAutoMappingResult(detected);
      setFieldMapping(detected.mappings || {});
      setStep("mapping");
    } catch (err) {
      setError(`加载字段映射失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [filePath, fileFormat, dataType]);

  const handleAutoMatch = useCallback(async () => {
    if (!filePath) return;

    try {
      setLoading(true);
      setError(null);
      const detected = await api.autoDetectMapping(filePath, fileFormat, dataType);
      setAutoMappingResult(detected);
      setFieldMapping(detected.mappings || {});
    } catch (err) {
      setError(`自动匹配失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [filePath, fileFormat, dataType]);

  const handleResetMapping = useCallback(() => {
    setFieldMapping({});
  }, []);

  const buildValueTransforms = useCallback((): Record<string, any> | undefined => {
    if (transformItems.length === 0) {
      return undefined;
    }

    const result: Record<string, any> = {};
    for (const item of transformItems) {
      if (!item.field.trim()) {
        continue;
      }

      result[item.field] = {
        type: item.type,
        ...(item.config || {}),
      };
    }

    return Object.keys(result).length > 0 ? result : undefined;
  }, [transformItems]);

  const updateTransformItem = useCallback((id: number, patch: Partial<TransformItem>) => {
    setTransformItems((prev) => prev.map((it) => (it.id === id ? { ...it, ...patch } : it)));
  }, []);

  const updateTransformConfig = useCallback(
    (id: number, patch: Record<string, any>) => {
      setTransformItems((prev) =>
        prev.map((it) => (
          it.id === id
            ? { ...it, config: { ...(it.config || {}), ...patch } }
            : it
        ))
      );
    },
    []
  );

  const handlePreview = useCallback(
    async (withTransforms: boolean) => {
      if (!filePath) return;

      try {
        setLoading(true);
        setError(null);

        const transforms = withTransforms ? buildValueTransforms() : undefined;
        setActiveValueTransforms(transforms);

        const result = await api.previewImport(
          filePath,
          dataType,
          fileFormat,
          normalizedMapping,
          transforms
        );

        setPreview(result);
        if (result.conflicts.length > 0) {
          setConflictDecisions(result.conflicts.map((c) => ({ ...c, action: undefined })));
        } else {
          setConflictDecisions([]);
        }
        setStep("preview");
      } catch (err) {
        setError(`预览失败: ${err}`);
      } finally {
        setLoading(false);
      }
    },
    [buildValueTransforms, dataType, fileFormat, filePath, normalizedMapping]
  );

  const handleImport = useCallback(async () => {
    if (!filePath) return;

    try {
      setStep("importing");
      setLoading(true);

      const result = await api.executeImport(
        filePath,
        dataType,
        fileFormat,
        conflictStrategy,
        conflictDecisions.length > 0 ? conflictDecisions : undefined,
        normalizedMapping,
        activeValueTransforms
      );

      setImportResult({ success: result.success, message: result.message });
      setStep("complete");
      if (result.success) {
        onImportComplete();
      }
    } catch (err) {
      setImportResult({ success: false, message: `导入失败: ${err}` });
      setStep("complete");
    } finally {
      setLoading(false);
    }
  }, [
    filePath,
    dataType,
    fileFormat,
    conflictStrategy,
    conflictDecisions,
    normalizedMapping,
    activeValueTransforms,
    onImportComplete,
  ]);

  const handleAddTransform = useCallback(() => {
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
  }, [nextTransformId, targetFields]);

  const handleTestExpression = useCallback(async () => {
    try {
      setError(null);
      const sample = JSON.parse(expressionSample);
      const normalizedSample: Record<string, string> = Object.fromEntries(
        Object.entries(sample).map(([k, v]) => [k, String(v)])
      );
      const result = await api.testExpression(expressionInput, normalizedSample);
      setExpressionOutput(result);
    } catch (err) {
      setError(`表达式测试失败: ${err}`);
    }
  }, [expressionInput, expressionSample]);

  const handleConflictActionChange = useCallback(
    (rowNumber: number, primaryKey: string, action: ConflictStrategy) => {
      setConflictDecisions((prev) =>
        prev.map((item) => (
          item.row_number === rowNumber && item.primary_key === primaryKey
            ? { ...item, action }
            : item
        ))
      );
    },
    []
  );

  const handleApplyAllConflicts = useCallback((action: ConflictStrategy) => {
    setConflictDecisions((prev) => prev.map((item) => ({ ...item, action })));
    setConflictStrategy(action);
  }, []);

  const handleReset = () => {
    setStep("select");
    setFilePath(null);
    setPreview(null);
    setConflictDecisions([]);
    setImportResult(null);
    setError(null);
    setSourceHeaders([]);
    setTargetFields([]);
    setAutoMappingResult(null);
    setFieldMapping({});
    setTransformItems([]);
    setNextTransformId(1);
    setActiveValueTransforms(undefined);
    setExpressionOutput(null);
  };

  const handleClose = () => {
    handleReset();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="modal-content" style={{ width: "920px", maxHeight: "88vh", overflow: "hidden", display: "flex", flexDirection: "column" }}>
        <div className="modal-header">
          <h2>导入{DATA_TYPE_LABELS[dataType]}</h2>
          <button className="close-btn" onClick={handleClose}>×</button>
        </div>

        <div className="modal-body" style={{ overflow: "auto" }}>
          <div style={{ display: "flex", gap: "var(--spacing-sm)", marginBottom: "var(--spacing-lg)" }}>
            {[
              ["select", "选择文件"],
              ["mapping", "字段映射"],
              ["transform", "值转换"],
              ["preview", "预览验证"],
              ["complete", "完成"],
            ].map(([s, label], i) => {
              const active = step === s || (step === "importing" && s === "complete");
              return (
                <div
                  key={s}
                  style={{
                    flex: 1,
                    padding: "var(--spacing-sm)",
                    textAlign: "center",
                    background: active ? "var(--color-primary-light)" : "var(--color-bg-layout)",
                    borderRadius: "var(--border-radius-sm)",
                    fontSize: "var(--font-size-sm)",
                    color: active ? "var(--color-primary)" : "var(--color-text-tertiary)",
                  }}
                >
                  {i + 1}. {label}
                </div>
              );
            })}
          </div>

          {error && (
            <div
              style={{
                padding: "var(--spacing-sm)",
                background: "rgba(255,77,79,0.1)",
                color: "var(--color-error)",
                borderRadius: "var(--border-radius-sm)",
                marginBottom: "var(--spacing-md)",
              }}
            >
              {error}
            </div>
          )}

          {step === "select" && (
            <div>
              <div
                onClick={handleSelectFile}
                style={{
                  padding: "var(--spacing-xl)",
                  border: "2px dashed var(--color-border)",
                  borderRadius: "var(--border-radius-md)",
                  textAlign: "center",
                  cursor: "pointer",
                }}
              >
                <div style={{ fontSize: "32px", marginBottom: "var(--spacing-sm)" }}>📂</div>
                <div>点击选择文件</div>
                <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)", marginTop: "var(--spacing-xs)" }}>
                  支持 CSV、Excel(.xlsx)、JSON
                </div>
              </div>

              {filePath && (
                <div
                  style={{
                    marginTop: "var(--spacing-md)",
                    padding: "var(--spacing-sm)",
                    background: "var(--color-bg-layout)",
                    borderRadius: "var(--border-radius-sm)",
                  }}
                >
                  <strong>已选择:</strong> {filePath.split("/").pop()}
                </div>
              )}

              <div style={{ marginTop: "var(--spacing-md)" }}>
                <label style={{ display: "block", marginBottom: "var(--spacing-xs)", fontWeight: 500 }}>文件格式</label>
                <select
                  value={fileFormat}
                  onChange={(e) => setFileFormat(e.target.value as FileFormat)}
                  className="form-control"
                  style={{ width: "220px" }}
                >
                  <option value="csv">CSV</option>
                  <option value="excel">Excel (.xlsx)</option>
                  <option value="json">JSON</option>
                </select>
              </div>
            </div>
          )}

          {step === "mapping" && (
            <div>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "var(--spacing-sm)" }}>
                <h4 style={{ margin: 0 }}>字段映射</h4>
                <div style={{ display: "flex", gap: "var(--spacing-sm)" }}>
                  <button className="btn-secondary" onClick={handleAutoMatch} disabled={loading}>自动匹配</button>
                  <button className="btn-secondary" onClick={handleResetMapping} disabled={loading}>重置</button>
                </div>
              </div>

              <div style={{ border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)", overflow: "hidden" }}>
                <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                  <thead>
                    <tr style={{ background: "var(--color-bg-layout)" }}>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>目标字段</th>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>源文件列</th>
                      <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>置信度</th>
                    </tr>
                  </thead>
                  <tbody>
                    {targetFields.map((field) => {
                      const confidence = autoMappingResult?.confidence?.[field.name];
                      const selectedSource = fieldMapping[field.name] || "";
                      return (
                        <tr key={field.name}>
                          <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                            {field.display_name}
                            {field.required && <span style={{ color: "var(--color-error)", marginLeft: 4 }}>*</span>}
                          </td>
                          <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                            <select
                              className="form-control"
                              value={selectedSource}
                              onChange={(e) => {
                                const value = e.target.value;
                                setFieldMapping((prev) => ({ ...prev, [field.name]: value }));
                              }}
                            >
                              <option value="">(未映射)</option>
                              {sourceHeaders.map((header) => (
                                <option key={header} value={header}>{header}</option>
                              ))}
                            </select>
                          </td>
                          <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: confidenceColor(confidence) }}>
                            {confidence === undefined ? "-" : `${Math.round(confidence * 100)}%`}
                          </td>
                        </tr>
                      );
                    })}
                  </tbody>
                </table>
              </div>

              {mappedRequiredMissing.length > 0 && (
                <div style={{ marginTop: "var(--spacing-sm)", color: "var(--color-error)", fontSize: "var(--font-size-sm)" }}>
                  必填字段未映射: {mappedRequiredMissing.join(", ")}
                </div>
              )}

              <div style={{ marginTop: "var(--spacing-sm)", fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
                未使用源列: {unusedSourceHeaders.length > 0 ? unusedSourceHeaders.join(", ") : "无"}
              </div>
            </div>
          )}

          {step === "transform" && (
            <div>
              <h4 style={{ marginTop: 0, marginBottom: "var(--spacing-sm)" }}>值转换规则（可选）</h4>
              <p style={{ marginTop: 0, color: "var(--color-text-tertiary)", fontSize: "var(--font-size-sm)" }}>
                通过表单配置转换规则，系统会按字段逐条执行。
              </p>

              {transformItems.map((item) => (
                <div
                  key={item.id}
                  style={{
                    border: "1px solid var(--color-border-light)",
                    borderRadius: "var(--border-radius-sm)",
                    padding: "var(--spacing-sm)",
                    marginBottom: "var(--spacing-sm)",
                  }}
                >
                  <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr auto", gap: "var(--spacing-sm)", marginBottom: "var(--spacing-sm)" }}>
                    <select
                      className="form-control"
                      value={item.field}
                      onChange={(e) => {
                        const value = e.target.value;
                        setTransformItems((prev) => prev.map((it) => (it.id === item.id ? { ...it, field: value } : it)));
                      }}
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
                        updateTransformItem(item.id, {
                          type: nextType,
                          config: { ...TRANSFORM_DEFAULT_CONFIGS[nextType] },
                        });
                      }}
                    >
                      {Object.entries(TRANSFORM_TYPE_LABELS).map(([k, label]) => (
                        <option key={k} value={k}>{label}</option>
                      ))}
                    </select>

                    <button
                      className="btn-secondary"
                      onClick={() => setTransformItems((prev) => prev.filter((it) => it.id !== item.id))}
                    >
                      删除
                    </button>
                  </div>

                  {item.type === "mapping" && (
                    <div style={{ display: "grid", gridTemplateColumns: "1fr auto", gap: "var(--spacing-sm)" }}>
                      <textarea
                        className="form-control"
                        style={{ minHeight: "72px", fontFamily: "monospace" }}
                        value={Object.entries(item.config?.values || {})
                          .map(([k, v]) => `${k} => ${String(v)}`)
                          .join("\n")}
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
                      <input
                        className="form-control"
                        placeholder="pattern"
                        value={item.config?.pattern || ""}
                        onChange={(e) => updateTransformConfig(item.id, { pattern: e.target.value })}
                      />
                      <input
                        className="form-control"
                        placeholder="replacement"
                        value={item.config?.replacement || ""}
                        onChange={(e) => updateTransformConfig(item.id, { replacement: e.target.value })}
                      />
                    </div>
                  )}

                  {item.type === "date_format" && (
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "var(--spacing-sm)" }}>
                      <input
                        className="form-control"
                        placeholder="输入格式，逗号分隔"
                        value={Array.isArray(item.config?.input_formats) ? item.config.input_formats.join(", ") : ""}
                        onChange={(e) => updateTransformConfig(item.id, {
                          input_formats: e.target.value.split(",").map((s) => s.trim()).filter(Boolean),
                        })}
                      />
                      <input
                        className="form-control"
                        placeholder="输出格式"
                        value={item.config?.output_format || ""}
                        onChange={(e) => updateTransformConfig(item.id, { output_format: e.target.value })}
                      />
                    </div>
                  )}

                  {item.type === "formula" && (
                    <input
                      className="form-control"
                      placeholder="例如: value / 1000"
                      value={item.config?.expression || ""}
                      onChange={(e) => updateTransformConfig(item.id, { expression: e.target.value })}
                    />
                  )}

                  {item.type === "condition" && (
                    <div style={{ display: "grid", gap: "var(--spacing-sm)" }}>
                      <textarea
                        className="form-control"
                        style={{ minHeight: "68px", fontFamily: "monospace" }}
                        placeholder={"value > 100 => 高\nvalue > 50 => 中"}
                        value={(item.config?.rules || [])
                          .map((r: any) => `${r.condition || ""} => ${r.result || ""}`)
                          .join("\n")}
                        onChange={(e) => {
                          const rules = e.target.value
                            .split("\n")
                            .map((line) => line.split("=>").map((part) => part.trim()))
                            .filter(([condition]) => !!condition)
                            .map(([condition, result]) => ({ condition, result: result || "" }));
                          updateTransformConfig(item.id, { rules });
                        }}
                      />
                      <input
                        className="form-control"
                        placeholder="default(可选)"
                        value={item.config?.default || ""}
                        onChange={(e) => updateTransformConfig(item.id, { default: e.target.value })}
                      />
                    </div>
                  )}

                  {item.type === "concat" && (
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 220px", gap: "var(--spacing-sm)" }}>
                      <input
                        className="form-control"
                        placeholder="{field_a}-{field_b}"
                        value={item.config?.template || ""}
                        onChange={(e) => updateTransformConfig(item.id, { template: e.target.value })}
                      />
                      <input
                        className="form-control"
                        placeholder="separator(可选)"
                        value={item.config?.separator || ""}
                        onChange={(e) => updateTransformConfig(item.id, { separator: e.target.value })}
                      />
                    </div>
                  )}

                  {item.type === "split" && (
                    <div style={{ display: "grid", gridTemplateColumns: "220px 1fr", gap: "var(--spacing-sm)" }}>
                      <input
                        className="form-control"
                        placeholder="separator"
                        value={item.config?.separator || ""}
                        onChange={(e) => updateTransformConfig(item.id, { separator: e.target.value })}
                      />
                      <input
                        className="form-control"
                        placeholder="target fields，逗号分隔"
                        value={Array.isArray(item.config?.target_fields) ? item.config.target_fields.join(", ") : ""}
                        onChange={(e) => updateTransformConfig(item.id, {
                          target_fields: e.target.value.split(",").map((s) => s.trim()).filter(Boolean),
                        })}
                      />
                    </div>
                  )}

                  {item.type === "script" && (
                    <input
                      className="form-control"
                      placeholder='例如: IF(value > 100, "高", "低")'
                      value={item.config?.expression || ""}
                      onChange={(e) => updateTransformConfig(item.id, { expression: e.target.value })}
                    />
                  )}

                  {item.type === "trim" && (
                    <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
                      无额外配置，执行字段值去前后空格。
                    </div>
                  )}

                  {item.type === "case" && (
                    <select
                      className="form-control"
                      style={{ width: "220px" }}
                      value={item.config?.mode || "lower"}
                      onChange={(e) => updateTransformConfig(item.id, { mode: e.target.value })}
                    >
                      <option value="upper">UPPER</option>
                      <option value="lower">lower</option>
                      <option value="title">Title</option>
                    </select>
                  )}

                  {item.type === "lookup_table" && (
                    <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr 1fr", gap: "var(--spacing-sm)" }}>
                      <input
                        className="form-control"
                        placeholder="table_name"
                        value={item.config?.table_name || ""}
                        onChange={(e) => updateTransformConfig(item.id, { table_name: e.target.value })}
                      />
                      <input
                        className="form-control"
                        placeholder="key_field"
                        value={item.config?.key_field || ""}
                        onChange={(e) => updateTransformConfig(item.id, { key_field: e.target.value })}
                      />
                      <input
                        className="form-control"
                        placeholder="value_field"
                        value={item.config?.value_field || ""}
                        onChange={(e) => updateTransformConfig(item.id, { value_field: e.target.value })}
                      />
                    </div>
                  )}
                </div>
              ))}

              <button className="btn-secondary" onClick={handleAddTransform}>+ 添加转换规则</button>

              <div style={{ marginTop: "var(--spacing-lg)", borderTop: "1px solid var(--color-border-light)", paddingTop: "var(--spacing-md)" }}>
                <h4 style={{ marginTop: 0 }}>表达式测试</h4>
                <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr auto", gap: "var(--spacing-sm)", alignItems: "end" }}>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "var(--font-size-sm)" }}>表达式</label>
                    <input className="form-control" value={expressionInput} onChange={(e) => setExpressionInput(e.target.value)} />
                  </div>
                  <div>
                    <label style={{ display: "block", marginBottom: 4, fontSize: "var(--font-size-sm)" }}>样本行(JSON)</label>
                    <input className="form-control" value={expressionSample} onChange={(e) => setExpressionSample(e.target.value)} />
                  </div>
                  <button className="btn-secondary" onClick={handleTestExpression}>测试</button>
                </div>

                {expressionOutput !== null && (
                  <div style={{ marginTop: "var(--spacing-sm)", color: "var(--color-success)", fontSize: "var(--font-size-sm)" }}>
                    输出: {expressionOutput}
                  </div>
                )}
              </div>
            </div>
          )}

          {step === "preview" && preview && (
            <div>
              <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "var(--spacing-md)", marginBottom: "var(--spacing-lg)" }}>
                <div style={{ padding: "var(--spacing-md)", background: "var(--color-bg-layout)", borderRadius: "var(--border-radius-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600 }}>{preview.total_rows}</div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>总行数</div>
                </div>
                <div style={{ padding: "var(--spacing-md)", background: "rgba(82,196,26,0.1)", borderRadius: "var(--border-radius-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600, color: "var(--color-success)" }}>{preview.valid_rows}</div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>有效行</div>
                </div>
                <div style={{ padding: "var(--spacing-md)", background: "rgba(255,77,79,0.1)", borderRadius: "var(--border-radius-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600, color: "var(--color-error)" }}>{preview.error_rows}</div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>错误行</div>
                </div>
                <div style={{ padding: "var(--spacing-md)", background: "rgba(250,173,20,0.1)", borderRadius: "var(--border-radius-sm)", textAlign: "center" }}>
                  <div style={{ fontSize: "var(--font-size-xl)", fontWeight: 600, color: "var(--color-warning)" }}>{preview.conflicts.length}</div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>冲突数</div>
                </div>
              </div>

              {preview.validation_errors.length > 0 && (
                <div style={{ marginBottom: "var(--spacing-lg)" }}>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>验证错误</h4>
                  <div style={{ maxHeight: "160px", overflow: "auto", border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)" }}>
                    <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                      <thead>
                        <tr style={{ background: "var(--color-bg-layout)" }}>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>行号</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>字段</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>错误信息</th>
                        </tr>
                      </thead>
                      <tbody>
                        {preview.validation_errors.slice(0, 12).map((err, idx) => (
                          <tr key={idx}>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{err.row_number}</td>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{err.field}</td>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)", color: "var(--color-error)" }}>{err.message}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {preview.conflicts.length > 0 && (
                <div style={{ marginBottom: "var(--spacing-lg)" }}>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>发现 {preview.conflicts.length} 条冲突记录</h4>
                  <div style={{ display: "flex", gap: "var(--spacing-md)", flexWrap: "wrap", alignItems: "center" }}>
                    <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)" }}>
                      <input type="radio" name="conflict" value="skip" checked={conflictStrategy === "skip"} onChange={() => setConflictStrategy("skip")} />
                      全局默认: 跳过
                    </label>
                    <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)" }}>
                      <input type="radio" name="conflict" value="overwrite" checked={conflictStrategy === "overwrite"} onChange={() => setConflictStrategy("overwrite")} />
                      全局默认: 覆盖
                    </label>
                    <button className="btn-secondary" onClick={() => handleApplyAllConflicts("skip")}>全部设为跳过</button>
                    <button className="btn-secondary" onClick={() => handleApplyAllConflicts("overwrite")}>全部设为覆盖</button>
                  </div>

                  <div style={{ marginTop: "var(--spacing-xs)", fontSize: "var(--font-size-sm)", color: "var(--color-text-secondary)" }}>
                    当前决策统计: 跳过 {conflictActionStats.skip} 条，覆盖 {conflictActionStats.overwrite} 条
                  </div>

                  <div style={{ marginTop: "var(--spacing-sm)", maxHeight: "220px", overflow: "auto", border: "1px solid var(--color-border-light)", borderRadius: "var(--border-radius-sm)" }}>
                    <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                      <thead>
                        <tr style={{ background: "var(--color-bg-layout)" }}>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>行号</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>主键</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>决策</th>
                        </tr>
                      </thead>
                      <tbody>
                        {conflictDecisions.slice(0, 30).map((conflict) => (
                          <tr key={`${conflict.row_number}-${conflict.primary_key}`}>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{conflict.row_number}</td>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>{conflict.primary_key}</td>
                            <td style={{ padding: "var(--spacing-xs) var(--spacing-sm)" }}>
                              <select
                                className="form-control"
                                value={conflict.action ?? conflictStrategy}
                                onChange={(e) =>
                                  handleConflictActionChange(
                                    conflict.row_number,
                                    conflict.primary_key,
                                    e.target.value as ConflictStrategy
                                  )
                                }
                              >
                                <option value="skip">跳过</option>
                                <option value="overwrite">覆盖</option>
                              </select>
                            </td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>

                  {conflictDecisions.length > 30 && (
                    <div style={{ marginTop: "var(--spacing-xs)", fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
                      仅显示前 30 条冲突，导入时会应用全部冲突决策。
                    </div>
                  )}
                </div>
              )}

              {preview.sample_data.length > 0 && (
                <div>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>数据预览（前5行）</h4>
                  <pre style={{ background: "var(--color-bg-layout)", padding: "var(--spacing-sm)", borderRadius: "var(--border-radius-sm)", fontSize: "var(--font-size-sm)", maxHeight: "180px", overflow: "auto" }}>
                    {JSON.stringify(preview.sample_data, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}

          {(step === "importing" || step === "complete") && (
            <div style={{ textAlign: "center", padding: "var(--spacing-xl)" }}>
              {step === "importing" && (
                <div>
                  <div className="loading__spinner" style={{ margin: "0 auto var(--spacing-md)" }} />
                  <div>正在导入...</div>
                </div>
              )}

              {step === "complete" && importResult && (
                <div>
                  <div style={{ fontSize: "48px", marginBottom: "var(--spacing-md)" }}>
                    {importResult.success ? "✓" : "✗"}
                  </div>
                  <div style={{ fontSize: "var(--font-size-lg)", color: importResult.success ? "var(--color-success)" : "var(--color-error)" }}>
                    {importResult.message}
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        <div className="modal-footer">
          {step === "select" && (
            <>
              <button className="btn-secondary" onClick={handleClose}>取消</button>
              <button className="btn-primary" disabled={!filePath || loading} onClick={handleLoadMappingContext}>
                {loading ? "加载中..." : "下一步"}
              </button>
            </>
          )}

          {step === "mapping" && (
            <>
              <button className="btn-secondary" onClick={() => setStep("select")}>上一步</button>
              <button className="btn-primary" disabled={mappedRequiredMissing.length > 0 || loading} onClick={() => setStep("transform")}>下一步</button>
            </>
          )}

          {step === "transform" && (
            <>
              <button className="btn-secondary" onClick={() => setStep("mapping")}>上一步</button>
              <button className="btn-secondary" disabled={loading} onClick={() => handlePreview(false)}>跳过此步骤</button>
              <button className="btn-primary" disabled={loading} onClick={() => handlePreview(true)}>
                {loading ? "加载中..." : "预览验证"}
              </button>
            </>
          )}

          {step === "preview" && (
            <>
              <button className="btn-secondary" onClick={() => setStep("transform")}>上一步</button>
              <button className="btn-primary" disabled={preview?.error_rows === preview?.total_rows || loading} onClick={handleImport}>开始导入</button>
            </>
          )}

          {step === "complete" && (
            <>
              <button className="btn-secondary" onClick={handleReset}>重新导入</button>
              <button className="btn-primary" onClick={handleClose}>完成</button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
