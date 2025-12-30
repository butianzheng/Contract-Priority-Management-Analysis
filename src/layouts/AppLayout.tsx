import { Outlet, useLocation } from "react-router-dom";
import { TopNav } from "../components/common/TopNav";
import { SideMenu } from "../components/common/SideMenu";

// 不显示侧边栏的路由
const noSiderRoutes = ["/", "/dashboard"];

export function AppLayout() {
  const location = useLocation();
  const showSider = !noSiderRoutes.includes(location.pathname);

  return (
    <div className="app-layout">
      <TopNav />
      <div className="app-body">
        {showSider && <SideMenu />}
        <main className={`main-content ${!showSider ? "main-content--no-sider" : ""}`}>
          <Outlet />
        </main>
      </div>
    </div>
  );
}
