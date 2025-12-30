import { HashRouter, Routes, Route, Navigate } from "react-router-dom";
import { AppLayout } from "./layouts/AppLayout";
import { Dashboard } from "./pages/Dashboard";
import { Contracts } from "./pages/Contracts";
import { ContractHistory } from "./pages/ContractHistory";
import { Sandbox } from "./pages/Sandbox";
import { SandboxCompare } from "./pages/SandboxCompare";
import { StrategyTemplates } from "./pages/StrategyTemplates";
import { Cockpit } from "./pages/Cockpit";
import { CockpitRhythm } from "./pages/CockpitRhythm";
import { CockpitCustomer } from "./pages/CockpitCustomer";
import { CockpitIssues } from "./pages/CockpitIssues";
import { CockpitConsensus } from "./pages/CockpitConsensus";
import { APS } from "./pages/APS";
import { APSLogs } from "./pages/APSLogs";
import { Settings } from "./pages/Settings";
import { DataSource } from "./pages/Settings/DataSource";
import { Transform } from "./pages/Settings/Transform";
import { Scoring } from "./pages/Settings/Scoring";
import { Weights } from "./pages/Settings/Weights";
import { System } from "./pages/Settings/System";
import { SpecFamilyPage } from "./pages/Settings/SpecFamily";

import "./styles/variables.css";
import "./styles/common.css";
import "./styles/layout.css";
import "./styles.css";

function App() {
  return (
    <HashRouter>
      <Routes>
        <Route path="/" element={<AppLayout />}>
          {/* 首页重定向到 Dashboard */}
          <Route index element={<Navigate to="/dashboard" replace />} />

          {/* 主要页面 */}
          <Route path="dashboard" element={<Dashboard />} />

          {/* 合同管理 */}
          <Route path="contracts" element={<Contracts />} />
          <Route path="contracts/history" element={<ContractHistory />} />

          {/* 策略沙盘 */}
          <Route path="sandbox" element={<Sandbox />} />
          <Route path="sandbox/compare" element={<SandboxCompare />} />
          <Route path="sandbox/templates" element={<StrategyTemplates />} />

          {/* 会议驾驶舱 */}
          <Route path="cockpit" element={<Cockpit />} />
          <Route path="cockpit/rhythm" element={<CockpitRhythm />} />
          <Route path="cockpit/customer" element={<CockpitCustomer />} />
          <Route path="cockpit/issues" element={<CockpitIssues />} />
          <Route path="cockpit/consensus" element={<CockpitConsensus />} />

          {/* APS管理 */}
          <Route path="aps" element={<APS />} />
          <Route path="aps/logs" element={<APSLogs />} />

          {/* 设置页面 */}
          <Route path="settings">
            <Route index element={<Settings />} />
            <Route path="data" element={<DataSource />} />
            <Route path="transform" element={<Transform />} />
            <Route path="spec-family" element={<SpecFamilyPage />} />
            <Route path="scoring" element={<Scoring />} />
            <Route path="weights" element={<Weights />} />
            <Route path="system" element={<System />} />
          </Route>

          {/* 404 重定向到首页 */}
          <Route path="*" element={<Navigate to="/dashboard" replace />} />
        </Route>
      </Routes>
    </HashRouter>
  );
}

export default App;
