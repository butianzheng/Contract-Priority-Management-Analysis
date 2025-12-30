import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api, ContractPriority } from "../api/tauri";

interface DashboardStats {
  totalContracts: number;
  highPriorityCount: number;
  urgentCount: number;
  alphaAdjustedCount: number;
}

export function Dashboard() {
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    loadDashboardData();
  }, []);

  const loadDashboardData = async () => {
    try {
      setLoading(true);
      const strategies = await api.getStrategies();
      if (strategies.length > 0) {
        const contracts = await api.computeAllPriorities(strategies[0]);
        calculateStats(contracts);
      }
    } catch (err) {
      setError(`加载数据失败: ${err}`);
    } finally {
      setLoading(false);
    }
  };

  const calculateStats = (contracts: ContractPriority[]) => {
    setStats({
      totalContracts: contracts.length,
      highPriorityCount: contracts.filter((c) => c.priority >= 80).length,
      urgentCount: contracts.filter((c) => c.days_to_pdd <= 3).length,
      alphaAdjustedCount: contracts.filter((c) => c.alpha !== undefined && c.alpha !== null).length,
    });
  };

  return (
    <div className="dashboard-page">
      <div className="page-header">
        <h1 className="page-header__title">工作台</h1>
        <p className="page-header__subtitle">欢迎使用 DPM 合同动态优先级管理系统</p>
      </div>

      {error && <div className="error-message">{error}</div>}

      {loading ? (
        <div className="loading">加载中...</div>
      ) : (
        <>
          {/* 统计卡片 */}
          <div className="stats-grid">
            <div className="stat-card">
              <div className="stat-card__icon">📋</div>
              <div className="stat-card__content">
                <div className="stat-card__value">{stats?.totalContracts || 0}</div>
                <div className="stat-card__label">合同总数</div>
              </div>
            </div>
            <div className="stat-card stat-card--primary">
              <div className="stat-card__icon">🔥</div>
              <div className="stat-card__content">
                <div className="stat-card__value">{stats?.highPriorityCount || 0}</div>
                <div className="stat-card__label">高优先级</div>
              </div>
            </div>
            <div className="stat-card stat-card--warning">
              <div className="stat-card__icon">⚠️</div>
              <div className="stat-card__content">
                <div className="stat-card__value">{stats?.urgentCount || 0}</div>
                <div className="stat-card__label">紧急交付 (3天内)</div>
              </div>
            </div>
            <div className="stat-card stat-card--info">
              <div className="stat-card__icon">✋</div>
              <div className="stat-card__content">
                <div className="stat-card__value">{stats?.alphaAdjustedCount || 0}</div>
                <div className="stat-card__label">人工调整</div>
              </div>
            </div>
          </div>

          {/* 快捷入口 */}
          <div className="quick-actions">
            <h2 className="section-title">快捷入口</h2>
            <div className="quick-actions__grid">
              <Link to="/contracts" className="quick-action-card">
                <span className="quick-action-card__icon">📋</span>
                <span className="quick-action-card__title">合同优先级中心</span>
                <span className="quick-action-card__desc">查看和管理合同优先级排序</span>
              </Link>
              <Link to="/sandbox" className="quick-action-card">
                <span className="quick-action-card__icon">🎯</span>
                <span className="quick-action-card__title">策略沙盘模拟器</span>
                <span className="quick-action-card__desc">调整参数模拟优先级变化</span>
              </Link>
              <Link to="/cockpit" className="quick-action-card">
                <span className="quick-action-card__icon">🖥️</span>
                <span className="quick-action-card__title">产销协同驾驶舱</span>
                <span className="quick-action-card__desc">会议展示与决策支持</span>
              </Link>
              <Link to="/settings/scoring" className="quick-action-card">
                <span className="quick-action-card__icon">⚙️</span>
                <span className="quick-action-card__title">评分配置</span>
                <span className="quick-action-card__desc">配置评分参数与权重</span>
              </Link>
            </div>
          </div>
        </>
      )}

      <style>{`
        .dashboard-page {
          max-width: 1200px;
        }

        .stats-grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
          gap: var(--spacing-lg);
          margin-bottom: var(--spacing-xl);
        }

        .stat-card {
          display: flex;
          align-items: center;
          gap: var(--spacing-md);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          box-shadow: var(--shadow-sm);
          transition: transform var(--transition-fast), box-shadow var(--transition-fast);
        }

        .stat-card:hover {
          transform: translateY(-2px);
          box-shadow: var(--shadow-md);
        }

        .stat-card--primary {
          border-left: 4px solid var(--color-primary);
        }

        .stat-card--warning {
          border-left: 4px solid var(--color-warning);
        }

        .stat-card--info {
          border-left: 4px solid var(--color-info);
        }

        .stat-card__icon {
          font-size: 32px;
          width: 56px;
          height: 56px;
          display: flex;
          align-items: center;
          justify-content: center;
          background: var(--color-bg-layout);
          border-radius: var(--border-radius-md);
        }

        .stat-card__value {
          font-size: 28px;
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .stat-card__label {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .section-title {
          font-size: var(--font-size-lg);
          font-weight: 600;
          color: var(--color-text-primary);
          margin: 0 0 var(--spacing-md) 0;
        }

        .quick-actions__grid {
          display: grid;
          grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
          gap: var(--spacing-md);
        }

        .quick-action-card {
          display: flex;
          flex-direction: column;
          gap: var(--spacing-xs);
          background: var(--color-bg-container);
          border-radius: var(--border-radius-lg);
          padding: var(--spacing-lg);
          box-shadow: var(--shadow-sm);
          text-decoration: none;
          transition: all var(--transition-fast);
          border: 1px solid transparent;
        }

        .quick-action-card:hover {
          border-color: var(--color-primary);
          transform: translateY(-2px);
          box-shadow: var(--shadow-md);
        }

        .quick-action-card__icon {
          font-size: 32px;
          margin-bottom: var(--spacing-xs);
        }

        .quick-action-card__title {
          font-size: var(--font-size-base);
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .quick-action-card__desc {
          font-size: var(--font-size-sm);
          color: var(--color-text-tertiary);
        }

        .loading {
          text-align: center;
          padding: var(--spacing-xl);
          color: var(--color-text-tertiary);
        }

        .error-message {
          background: #fff2f0;
          border: 1px solid #ffccc7;
          color: var(--color-error);
          padding: var(--spacing-md);
          border-radius: var(--border-radius-md);
          margin-bottom: var(--spacing-lg);
        }
      `}</style>
    </div>
  );
}
