import { NavLink, useLocation } from "react-router-dom";

interface NavItem {
  path: string;
  label: string;
  icon: string;
}

const navItems: NavItem[] = [
  { path: "/dashboard", label: "首页", icon: "📊" },
  { path: "/contracts", label: "合同中心", icon: "📋" },
  { path: "/sandbox", label: "策略沙盘", icon: "🎯" },
  { path: "/cockpit", label: "驾驶舱", icon: "🖥️" },
  { path: "/aps", label: "APS视图", icon: "⚡" },
];

export function TopNav() {
  const location = useLocation();

  const isActive = (path: string) => {
    if (path === "/dashboard") {
      return location.pathname === "/" || location.pathname === "/dashboard";
    }
    return location.pathname.startsWith(path);
  };

  return (
    <header className="top-nav">
      <NavLink to="/" className="top-nav__logo">
        <span className="top-nav__logo-icon">D</span>
        <span>DPM</span>
      </NavLink>

      <nav className="top-nav__menu">
        {navItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            className={`top-nav__item ${isActive(item.path) ? "top-nav__item--active" : ""}`}
          >
            <span>{item.icon}</span>
            <span>{item.label}</span>
          </NavLink>
        ))}
      </nav>

      <div className="top-nav__actions">
        <NavLink
          to="/settings"
          className={`top-nav__item ${location.pathname.startsWith("/settings") ? "top-nav__item--active" : ""}`}
        >
          <span>⚙️</span>
          <span>设置</span>
        </NavLink>
      </div>
    </header>
  );
}
