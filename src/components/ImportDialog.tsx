import { useState, useCallback } from "react";
import { open } from "@tauri-apps/api/dialog";
import {
  api,
  ImportPreview,
  ConflictRecord,
  FileFormat,
  ImportDataType,
  ConflictStrategy,
} from "../api/tauri";
import "./Dialog.css";

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  dataType: ImportDataType;
  onImportComplete: () => void;
}

type ImportStep = "select" | "preview" | "conflict" | "importing" | "complete";

const DATA_TYPE_LABELS: Record<ImportDataType, string> = {
  contracts: "合同数据",
  customers: "客户数据",
  process_difficulty: "工艺难度",
  strategy_weights: "策略权重",
};

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
  const [conflictStrategy, setConflictStrategy] =
    useState<ConflictStrategy>("skip");
  const [conflictDecisions, setConflictDecisions] = useState<ConflictRecord[]>(
    []
  );
  const [importResult, setImportResult] = useState<{
    success: boolean;
    message: string;
  } | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // 选择文件
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
        // 自动检测格式
        const ext = selected.split(".").pop()?.toLowerCase();
        if (ext === "csv") setFileFormat("csv");
        else if (ext === "xlsx" || ext === "xls") setFileFormat("excel");
        else if (ext === "json") setFileFormat("json");
      }
    } catch (err) {
      setError(`选择文件失败: ${err}`);
    }
  }, [dataType]);

  // 预览数据
  const handlePreview = useCallback(async () => {
    if (!filePath) return;

    try {
      setLoading(true);
      setError(null);
      const result = await api.previewImport(filePath, dataType, fileFormat);
      setPreview(result);

      if (result.conflicts.length > 0) {
        setConflictDecisions(
          result.conflicts.map((c) => ({ ...c, action: "skip" as const }))
        );
      }
      setStep("preview");
    } catch (err) {
      setError(`预览失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, [filePath, dataType, fileFormat]);

  // 执行导入
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
        conflictStrategy === "skip" || conflictStrategy === "overwrite"
          ? undefined
          : conflictDecisions
      );

      setImportResult({
        success: result.success,
        message: result.message,
      });
      setStep("complete");

      if (result.success) {
        onImportComplete();
      }
    } catch (err) {
      setImportResult({
        success: false,
        message: `导入失败: ${err}`,
      });
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
    onImportComplete,
  ]);

  // 重置状态
  const handleReset = () => {
    setStep("select");
    setFilePath(null);
    setPreview(null);
    setConflictDecisions([]);
    setImportResult(null);
    setError(null);
  };

  const handleClose = () => {
    handleReset();
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="modal-content" style={{ width: "700px" }}>
        <div className="modal-header">
          <h2>导入{DATA_TYPE_LABELS[dataType]}</h2>
          <button className="close-btn" onClick={handleClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {/* 步骤指示器 */}
          <div
            style={{
              display: "flex",
              gap: "var(--spacing-md)",
              marginBottom: "var(--spacing-lg)",
            }}
          >
            {["select", "preview", "complete"].map((s, i) => (
              <div
                key={s}
                style={{
                  flex: 1,
                  padding: "var(--spacing-sm)",
                  textAlign: "center",
                  background:
                    step === s || (step === "importing" && s === "complete")
                      ? "var(--color-primary-light)"
                      : "var(--color-bg-layout)",
                  borderRadius: "var(--border-radius-sm)",
                  fontSize: "var(--font-size-sm)",
                  color:
                    step === s
                      ? "var(--color-primary)"
                      : "var(--color-text-tertiary)",
                }}
              >
                {i + 1}.{" "}
                {s === "select"
                  ? "选择文件"
                  : s === "preview"
                  ? "预览验证"
                  : "完成"}
              </div>
            ))}
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

          {/* 步骤1: 选择文件 */}
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
                  transition: "all var(--transition-fast)",
                }}
                onMouseOver={(e) => {
                  e.currentTarget.style.borderColor = "var(--color-primary)";
                  e.currentTarget.style.background =
                    "var(--color-primary-light)";
                }}
                onMouseOut={(e) => {
                  e.currentTarget.style.borderColor = "var(--color-border)";
                  e.currentTarget.style.background = "transparent";
                }}
              >
                <div style={{ fontSize: "32px", marginBottom: "var(--spacing-sm)" }}>
                  📂
                </div>
                <div>点击选择文件</div>
                <div
                  style={{
                    fontSize: "var(--font-size-sm)",
                    color: "var(--color-text-tertiary)",
                    marginTop: "var(--spacing-xs)",
                  }}
                >
                  支持 CSV、Excel(.xlsx)、JSON 格式
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
                <label
                  style={{
                    display: "block",
                    marginBottom: "var(--spacing-xs)",
                    fontWeight: 500,
                  }}
                >
                  文件格式
                </label>
                <select
                  value={fileFormat}
                  onChange={(e) => setFileFormat(e.target.value as FileFormat)}
                  className="form-control"
                  style={{ width: "200px" }}
                >
                  <option value="csv">CSV</option>
                  <option value="excel">Excel (.xlsx)</option>
                  <option value="json">JSON</option>
                </select>
              </div>
            </div>
          )}

          {/* 步骤2: 预览验证 */}
          {step === "preview" && preview && (
            <div>
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(4, 1fr)",
                  gap: "var(--spacing-md)",
                  marginBottom: "var(--spacing-lg)",
                }}
              >
                <div
                  style={{
                    padding: "var(--spacing-md)",
                    background: "var(--color-bg-layout)",
                    borderRadius: "var(--border-radius-sm)",
                    textAlign: "center",
                  }}
                >
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                    }}
                  >
                    {preview.total_rows}
                  </div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
                    总行数
                  </div>
                </div>
                <div
                  style={{
                    padding: "var(--spacing-md)",
                    background: "rgba(82,196,26,0.1)",
                    borderRadius: "var(--border-radius-sm)",
                    textAlign: "center",
                  }}
                >
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                      color: "var(--color-success)",
                    }}
                  >
                    {preview.valid_rows}
                  </div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
                    有效行
                  </div>
                </div>
                <div
                  style={{
                    padding: "var(--spacing-md)",
                    background: "rgba(255,77,79,0.1)",
                    borderRadius: "var(--border-radius-sm)",
                    textAlign: "center",
                  }}
                >
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                      color: "var(--color-error)",
                    }}
                  >
                    {preview.error_rows}
                  </div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
                    错误行
                  </div>
                </div>
                <div
                  style={{
                    padding: "var(--spacing-md)",
                    background: "rgba(250,173,20,0.1)",
                    borderRadius: "var(--border-radius-sm)",
                    textAlign: "center",
                  }}
                >
                  <div
                    style={{
                      fontSize: "var(--font-size-xl)",
                      fontWeight: 600,
                      color: "var(--color-warning)",
                    }}
                  >
                    {preview.conflicts.length}
                  </div>
                  <div style={{ fontSize: "var(--font-size-sm)", color: "var(--color-text-tertiary)" }}>
                    冲突数
                  </div>
                </div>
              </div>

              {/* 验证错误列表 */}
              {preview.validation_errors.length > 0 && (
                <div style={{ marginBottom: "var(--spacing-lg)" }}>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>验证错误</h4>
                  <div
                    style={{
                      maxHeight: "150px",
                      overflow: "auto",
                      border: "1px solid var(--color-border-light)",
                      borderRadius: "var(--border-radius-sm)",
                    }}
                  >
                    <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "var(--font-size-sm)" }}>
                      <thead>
                        <tr style={{ background: "var(--color-bg-layout)" }}>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>行号</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>字段</th>
                          <th style={{ padding: "var(--spacing-xs) var(--spacing-sm)", textAlign: "left" }}>错误信息</th>
                        </tr>
                      </thead>
                      <tbody>
                        {preview.validation_errors.slice(0, 10).map((err, idx) => (
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

              {/* 冲突处理策略 */}
              {preview.conflicts.length > 0 && (
                <div style={{ marginBottom: "var(--spacing-lg)" }}>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>
                    发现 {preview.conflicts.length} 条冲突记录
                  </h4>
                  <div style={{ display: "flex", gap: "var(--spacing-md)" }}>
                    <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)" }}>
                      <input
                        type="radio"
                        name="conflict"
                        value="skip"
                        checked={conflictStrategy === "skip"}
                        onChange={() => setConflictStrategy("skip")}
                      />
                      跳过冲突记录
                    </label>
                    <label style={{ display: "flex", alignItems: "center", gap: "var(--spacing-xs)" }}>
                      <input
                        type="radio"
                        name="conflict"
                        value="overwrite"
                        checked={conflictStrategy === "overwrite"}
                        onChange={() => setConflictStrategy("overwrite")}
                      />
                      覆盖现有记录
                    </label>
                  </div>
                </div>
              )}

              {/* 数据预览 */}
              {preview.sample_data.length > 0 && (
                <div>
                  <h4 style={{ marginBottom: "var(--spacing-sm)" }}>数据预览（前5行）</h4>
                  <pre
                    style={{
                      background: "var(--color-bg-layout)",
                      padding: "var(--spacing-sm)",
                      borderRadius: "var(--border-radius-sm)",
                      fontSize: "var(--font-size-sm)",
                      maxHeight: "150px",
                      overflow: "auto",
                    }}
                  >
                    {JSON.stringify(preview.sample_data, null, 2)}
                  </pre>
                </div>
              )}
            </div>
          )}

          {/* 步骤3: 导入进度/完成 */}
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
                  <div
                    style={{
                      fontSize: "48px",
                      marginBottom: "var(--spacing-md)",
                    }}
                  >
                    {importResult.success ? "✓" : "✗"}
                  </div>
                  <div
                    style={{
                      fontSize: "var(--font-size-lg)",
                      color: importResult.success
                        ? "var(--color-success)"
                        : "var(--color-error)",
                    }}
                  >
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
              <button className="btn-secondary" onClick={handleClose}>
                取消
              </button>
              <button
                className="btn-primary"
                disabled={!filePath || loading}
                onClick={handlePreview}
              >
                {loading ? "加载中..." : "下一步"}
              </button>
            </>
          )}

          {step === "preview" && (
            <>
              <button className="btn-secondary" onClick={() => setStep("select")}>
                上一步
              </button>
              <button
                className="btn-primary"
                disabled={preview?.error_rows === preview?.total_rows || loading}
                onClick={handleImport}
              >
                开始导入
              </button>
            </>
          )}

          {step === "complete" && (
            <>
              <button className="btn-secondary" onClick={handleReset}>
                重新导入
              </button>
              <button className="btn-primary" onClick={handleClose}>
                完成
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
