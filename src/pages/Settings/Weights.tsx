import { useState } from "react";
import { StrategyWeightsPanel } from "../../components/StrategyWeightsPanel";

export function Weights() {
  const [refreshKey, setRefreshKey] = useState(0);

  const handleWeightsChanged = () => {
    setRefreshKey((prev) => prev + 1);
  };

  return (
    <div className="weights-page">
      <div className="page-header">
        <div className="page-header__breadcrumb">
          <a href="/settings">设置</a>
          <span>/</span>
          <span>策略权重</span>
        </div>
        <h1 className="page-header__title">策略权重</h1>
        <p className="page-header__subtitle">管理策略权重矩阵和 Alpha 系数</p>
      </div>

      <div className="weights-content">
        <StrategyWeightsPanel
          key={refreshKey}
          onClose={() => {}}
          onWeightsChanged={handleWeightsChanged}
          embedded={true}
        />
      </div>

      <style>{`
        .weights-page {
          max-width: 1200px;
        }

        .weights-content {
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          box-shadow: var(--shadow-sm);
        }
      `}</style>
    </div>
  );
}
