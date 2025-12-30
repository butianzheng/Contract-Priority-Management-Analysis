import { Link } from "react-router-dom";

export function Settings() {
  return (
    <div className="settings-page">
      <div className="page-header">
        <h1 className="page-header__title">数据与模型配置</h1>
        <p className="page-header__subtitle">管理数据源、清洗规则和评分模型</p>
      </div>

      <div className="settings-grid">
        <Link to="/settings/data" className="settings-card">
          <span className="settings-card__icon">💾</span>
          <h3 className="settings-card__title">数据源管理</h3>
          <p className="settings-card__desc">
            管理合同、客户、工艺难度等基础数据
          </p>
          <ul className="settings-card__list">
            <li>合同主数据 (A1)</li>
            <li>客户主数据 (A2)</li>
            <li>工艺难度字典 (A3)</li>
            <li>规格族字典 (A4)</li>
          </ul>
        </Link>

        <Link to="/settings/transform" className="settings-card">
          <span className="settings-card__icon">🔧</span>
          <h3 className="settings-card__title">清洗规则管理</h3>
          <p className="settings-card__desc">
            配置数据清洗与标准化规则
          </p>
          <ul className="settings-card__list">
            <li>字段标准化规则 (B1)</li>
            <li>规格段提取规则 (B2)</li>
            <li>客户等级归一化 (B3)</li>
            <li>周期标签映射 (B4)</li>
          </ul>
        </Link>

        <Link to="/settings/spec-family" className="settings-card">
          <span className="settings-card__icon">📦</span>
          <h3 className="settings-card__title">规格族管理</h3>
          <p className="settings-card__desc">
            管理产品规格族分类和系数配置
          </p>
          <ul className="settings-card__list">
            <li>规格族主数据维护</li>
            <li>P-Score 系数配置</li>
            <li>钢种关联管理</li>
            <li>规格范围设置</li>
          </ul>
        </Link>

        <Link to="/settings/scoring" className="settings-card">
          <span className="settings-card__icon">⚙️</span>
          <h3 className="settings-card__title">评分配置</h3>
          <p className="settings-card__desc">
            配置 S-Score 和 P-Score 评分参数
          </p>
          <ul className="settings-card__list">
            <li>S-Score 参数 (C1)</li>
            <li>P-Score 参数 (C2)</li>
            <li>阈值与区间配置</li>
          </ul>
        </Link>

        <Link to="/settings/weights" className="settings-card">
          <span className="settings-card__icon">📊</span>
          <h3 className="settings-card__title">策略权重</h3>
          <p className="settings-card__desc">
            管理策略权重矩阵和 Alpha 系数
          </p>
          <ul className="settings-card__list">
            <li>WS/WP 权重矩阵 (C3)</li>
            <li>Alpha 治理策略 (C4)</li>
          </ul>
        </Link>

        <Link to="/settings/system" className="settings-card">
          <span className="settings-card__icon">🔒</span>
          <h3 className="settings-card__title">系统参数</h3>
          <p className="settings-card__desc">
            系统级配置与权限管理
          </p>
          <ul className="settings-card__list">
            <li>角色权限配置</li>
            <li>环境参数</li>
            <li>导入导出路径</li>
          </ul>
        </Link>
      </div>

      <style>{`
        .settings-page {
          max-width: 1200px;
        }

        .settings-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
          gap: var(--spacing-lg);
        }

        .settings-card {
          display: flex;
          flex-direction: column;
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          box-shadow: var(--shadow-sm);
          text-decoration: none;
          transition: all var(--transition-fast);
          border: 1px solid transparent;
        }

        .settings-card:hover {
          border-color: var(--color-primary);
          transform: translateY(-2px);
          box-shadow: var(--shadow-md);
        }

        .settings-card__icon {
          font-size: 32px;
          margin-bottom: var(--spacing-sm);
        }

        .settings-card__title {
          font-size: var(--font-size-lg);
          font-weight: 600;
          color: var(--color-text-primary);
          margin: 0 0 var(--spacing-xs) 0;
        }

        .settings-card__desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
          margin: 0 0 var(--spacing-md) 0;
        }

        .settings-card__list {
          margin: 0;
          padding-left: var(--spacing-md);
          font-size: var(--font-size-sm);
          color: var(--color-text-secondary);
        }

        .settings-card__list li {
          margin-bottom: var(--spacing-xs);
        }
      `}</style>
    </div>
  );
}
