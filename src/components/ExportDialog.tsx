import { useState } from "react";
import { save } from "@tauri-apps/api/dialog";
import {
  api,
  FileFormat,
  ImportDataType,
  ExportOptions,
} from "../api/tauri";
import "./Dialog.css";

interface ExportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  dataType: ImportDataType;
  strategies?: string[];
}

const DATA_TYPE_LABELS: Record<ImportDataType, string> = {
  contracts: "合同数据",
  customers: "客户数据",
  process_difficulty: "工艺难度",
  strategy_weights: "策略权重",
};

const FORMAT_LABELS: Record<FileFormat, { label: string; icon: string; ext: string }> = {
  excel: { label: "Excel (.xlsx)", icon: "📊", ext: "xlsx" },
  csv: { label: "CSV", icon: "📄", ext: "csv" },
  json: { label: "JSON", icon: "{ }", ext: "json" },
};

export function ExportDialog({
  isOpen,
  onClose,
  dataType,
  strategies = [],
}: ExportDialogProps) {
  const [format, setFormat] = useState<FileFormat>("excel");
  const [includeComputed, setIncludeComputed] = useState(true);
  const [selectedStrategy, setSelectedStrategy] = useState(
    strategies[0] || "均衡"
  );
  const [exporting, setExporting] = useState(false);
  const [result, setResult] = useState<{
    success: boolean;
    message: string;
    path?: string;
  } | null>(null);

  const handleExport = async () => {
    try {
      setExporting(true);
      setResult(null);

      // 生成默认文件名
      const timestamp = new Date()
        .toISOString()
        .replace(/[-:]/g, "")
        .slice(0, 15);
      const defaultName = `dpm_${dataType}_${timestamp}.${FORMAT_LABELS[format].ext}`;

      // 打开保存对话框
      const filePath = await save({
        filters: [
          {
            name: `${FORMAT_LABELS[format].label} 文件`,
            extensions: [FORMAT_LABELS[format].ext],
          },
        ],
        defaultPath: defaultName,
        title: "选择导出位置",
      });

      if (!filePath) {
        setExporting(false);
        return;
      }

      // 执行导出
      const options: ExportOptions = {
        format,
        data_type: dataType,
        include_computed: includeComputed,
        strategy: includeComputed ? selectedStrategy : undefined,
      };

      const exportResult = await api.exportData(filePath, options);

      setResult({
        success: exportResult.success,
        message: exportResult.message,
        path: exportResult.file_path,
      });
    } catch (err) {
      setResult({
        success: false,
        message: `导出失败: ${err}`,
      });
    } finally {
      setExporting(false);
    }
  };

  const handleClose = () => {
    setResult(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay">
      <div className="modal-content" style={{ width: "500px" }}>
        <div className="modal-header">
          <h2>导出{DATA_TYPE_LABELS[dataType]}</h2>
          <button className="close-btn" onClick={handleClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          {/* 格式选择 */}
          <div style={{ marginBottom: "var(--spacing-lg)" }}>
            <label
              style={{
                display: "block",
                marginBottom: "var(--spacing-sm)",
                fontWeight: 500,
              }}
            >
              导出格式
            </label>
            <div style={{ display: "flex", gap: "var(--spacing-sm)" }}>
              {(["excel", "csv", "json"] as FileFormat[]).map((f) => (
                <label
                  key={f}
                  style={{
                    flex: 1,
                    padding: "var(--spacing-md)",
                    border: `2px solid ${
                      format === f
                        ? "var(--color-primary)"
                        : "var(--color-border)"
                    }`,
                    borderRadius: "var(--border-radius-md)",
                    cursor: "pointer",
                    textAlign: "center",
                    background:
                      format === f
                        ? "var(--color-primary-light)"
                        : "transparent",
                    transition: "all var(--transition-fast)",
                  }}
                >
                  <input
                    type="radio"
                    name="format"
                    value={f}
                    checked={format === f}
                    onChange={() => setFormat(f)}
                    style={{ display: "none" }}
                  />
                  <div style={{ fontSize: "24px", marginBottom: "4px" }}>
                    {FORMAT_LABELS[f].icon}
                  </div>
                  <div style={{ fontSize: "var(--font-size-sm)" }}>
                    {FORMAT_LABELS[f].label}
                  </div>
                </label>
              ))}
            </div>
          </div>

          {/* 包含计算字段选项（仅合同数据） */}
          {dataType === "contracts" && (
            <>
              <div style={{ marginBottom: "var(--spacing-md)" }}>
                <label
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "var(--spacing-sm)",
                    cursor: "pointer",
                  }}
                >
                  <input
                    type="checkbox"
                    checked={includeComputed}
                    onChange={(e) => setIncludeComputed(e.target.checked)}
                  />
                  <span>包含优先级计算结果</span>
                </label>
                <div
                  style={{
                    marginLeft: "24px",
                    fontSize: "var(--font-size-sm)",
                    color: "var(--color-text-tertiary)",
                  }}
                >
                  勾选后将包含 S分数、P分数、综合优先级等计算字段
                </div>
              </div>

              {/* 策略选择 */}
              {includeComputed && strategies.length > 0 && (
                <div style={{ marginBottom: "var(--spacing-lg)" }}>
                  <label
                    style={{
                      display: "block",
                      marginBottom: "var(--spacing-xs)",
                      fontWeight: 500,
                    }}
                  >
                    计算策略
                  </label>
                  <select
                    value={selectedStrategy}
                    onChange={(e) => setSelectedStrategy(e.target.value)}
                    className="form-control"
                    style={{ width: "200px" }}
                  >
                    {strategies.map((s) => (
                      <option key={s} value={s}>
                        {s}
                      </option>
                    ))}
                  </select>
                </div>
              )}
            </>
          )}

          {/* 导出结果 */}
          {result && (
            <div
              style={{
                padding: "var(--spacing-md)",
                background: result.success
                  ? "rgba(82,196,26,0.1)"
                  : "rgba(255,77,79,0.1)",
                borderRadius: "var(--border-radius-sm)",
                textAlign: "center",
              }}
            >
              <div
                style={{
                  fontSize: "32px",
                  marginBottom: "var(--spacing-xs)",
                }}
              >
                {result.success ? "✓" : "✗"}
              </div>
              <div
                style={{
                  color: result.success
                    ? "var(--color-success)"
                    : "var(--color-error)",
                }}
              >
                {result.message}
              </div>
              {result.path && (
                <div
                  style={{
                    marginTop: "var(--spacing-xs)",
                    fontSize: "var(--font-size-sm)",
                    color: "var(--color-text-tertiary)",
                    wordBreak: "break-all",
                  }}
                >
                  {result.path}
                </div>
              )}
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button className="btn-secondary" onClick={handleClose}>
            {result ? "关闭" : "取消"}
          </button>
          {!result && (
            <button
              className="btn-primary"
              disabled={exporting}
              onClick={handleExport}
            >
              {exporting ? "导出中..." : "导出"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
