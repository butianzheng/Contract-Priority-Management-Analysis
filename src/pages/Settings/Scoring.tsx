import { useState } from "react";
import { ConfigPanel } from "../../components/ConfigPanel";

export function Scoring() {
  const [refreshKey, setRefreshKey] = useState(0);

  const handleConfigChanged = () => {
    setRefreshKey((prev) => prev + 1);
  };

  return (
    <div className="scoring-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <a href="/settings">设置</a>
          <span>/</span>
          <span>评分配置</span>
        </div>
        <h1 className="page-header__title">评分配置</h1>
        <p className="page-header__subtitle">配置 S-Score 和 P-Score 评分参数</p>
      </div>

      <div className="scoring-content">
        <ConfigPanel
          key={refreshKey}
          onClose={() => {}}
          onConfigChanged={handleConfigChanged}
          embedded={true}
        />
      </div>

      <style>{`
        .scoring-page {
          max-width: 1200px;
        }

        .scoring-content {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          box-shadow: var(--shadow-sm);
        }
      `}</style>
    </div>
  );
}
