import { NavLink, useLocation, useNavigate } from "react-router-dom";

export interface MenuItem {
  path: string;
  label: string;
  icon: string;
}

export interface MenuSection {
  title?: string;
  items: MenuItem[];
}

// 各模块的侧边菜单配置
const menuConfigs: Record<string, MenuSection[]> = {
  // 首页 - 无侧边菜单
  dashboard: [],

  // 合同中心
  contracts: [
    {
      title: "合同管理",
      items: [
        { path: "/contracts", label: "优先级列表", icon: "📋" },
        { path: "/contracts/history", label: "历史记录", icon: "📜" },
      ],
    },
  ],

  // 策略沙盘
  sandbox: [
    {
      title: "沙盘工具",
      items: [
        { path: "/sandbox", label: "参数调节", icon: "🎚️" },
        { path: "/sandbox/compare", label: "结果对比", icon: "📊" },
        { path: "/sandbox/templates", label: "策略模板", icon: "📁" },
      ],
    },
  ],

  // 驾驶舱
  cockpit: [
    {
      title: "会议视图",
      items: [
        { path: "/cockpit", label: "主屏概览", icon: "🖥️" },
        { path: "/cockpit/rhythm", label: "节奏视图", icon: "📅" },
        { path: "/cockpit/customer", label: "客户保障", icon: "👥" },
        { path: "/cockpit/issues", label: "问题清单", icon: "⚠️" },
        { path: "/cockpit/consensus", label: "共识交付", icon: "📦" },
      ],
    },
  ],

  // APS视图
  aps: [
    {
      title: "APS管理",
      items: [
        { path: "/aps", label: "接口监控", icon: "📡" },
        { path: "/aps/logs", label: "调用日志", icon: "📝" },
      ],
    },
  ],

  // 设置
  settings: [
    {
      title: "数据源管理",
      items: [
        { path: "/settings", label: "概览", icon: "🏠" },
        { path: "/settings/data", label: "数据源", icon: "💾" },
        { path: "/settings/transform", label: "清洗规则", icon: "🔧" },
        { path: "/settings/spec-family", label: "规格族", icon: "📦" },
      ],
    },
    {
      title: "模型配置",
      items: [
        { path: "/settings/scoring", label: "评分配置", icon: "⚙️" },
        { path: "/settings/weights", label: "策略权重", icon: "📊" },
      ],
    },
    {
      title: "系统",
      items: [
        { path: "/settings/system", label: "系统参数", icon: "🔒" },
      ],
    },
  ],
};

// 根据当前路径获取模块名
function getModuleFromPath(pathname: string): string {
  if (pathname === "/" || pathname === "/dashboard") return "dashboard";
  const segment = pathname.split("/")[1];
  return segment || "dashboard";
}

interface SideMenuProps {
  collapsed?: boolean;
}

export function SideMenu({ collapsed = false }: SideMenuProps) {
  const location = useLocation();
  const navigate = useNavigate();
  const module = getModuleFromPath(location.pathname);
  const sections = menuConfigs[module] || [];

  // 如果没有侧边菜单配置，不渲染
  if (sections.length === 0) {
    return null;
  }

  const handleItemClick = (path: string, e: React.MouseEvent) => {
    e.preventDefault();
    console.log("SideMenu: Navigating to", path);
    navigate(path);
  };

  return (
    <aside className={`side-menu ${collapsed ? "side-menu--collapsed" : ""}`}>
      {sections.map((section, sectionIndex) => (
        <div key={sectionIndex} className="side-menu__section">
          {section.title && (
            <div className="side-menu__section-title">{section.title}</div>
          )}
          {section.items.map((item) => (
            <NavLink
              key={item.path}
              to={item.path}
              end={item.path === "/contracts" || item.path === "/sandbox" || item.path === "/cockpit" || item.path === "/aps" || item.path === "/settings"}
              className={({ isActive }) =>
                `side-menu__item ${isActive ? "side-menu__item--active" : ""}`
              }
              onClick={(e) => handleItemClick(item.path, e)}
            >
              <span className="side-menu__item-icon">{item.icon}</span>
              {!collapsed && <span>{item.label}</span>}
            </NavLink>
          ))}
        </div>
      ))}
    </aside>
  );
}
