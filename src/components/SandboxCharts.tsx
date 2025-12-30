import { useMemo, useState } from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ScatterChart,
  Scatter,
  ZAxis,
  Cell,
  PieChart,
  Pie,
  LineChart,
  Line,
  ReferenceLine,
} from "recharts";

// 模拟合同数据类型
interface SimulatedContract {
  contract_id: string;
  customer_id: string;
  spec_family: string;
  steel_grade: string;
  days_to_pdd: number;
  s_score: number;
  p_score: number;
  priority: number;
  originalRank: number;
  newPriority: number;
  newRank: number;
  rankChange: number;
  priorityChange: number;
}

// 通用颜色配置
const COLORS = {
  primary: "#1890ff",
  success: "#52c41a",
  warning: "#faad14",
  error: "#ff4d4f",
  up: "#52c41a",
  down: "#ff4d4f",
  same: "#8c8c8c",
  // 分类颜色
  category: [
    "#1890ff",
    "#52c41a",
    "#faad14",
    "#eb2f96",
    "#722ed1",
    "#13c2c2",
    "#fa8c16",
    "#a0d911",
    "#2f54eb",
    "#f5222d",
  ],
};

// ============================================
// 1. 排名变化分布图
// ============================================
interface RankChangeDistributionChartProps {
  data: SimulatedContract[];
}

export function RankChangeDistributionChart({ data }: RankChangeDistributionChartProps) {
  // 计算排名变化分布
  const distributionData = useMemo(() => {
    // 分组：大幅下降(-10以下), 下降(-10到-1), 不变(0), 上升(1到10), 大幅上升(10以上)
    const groups = [
      { name: "大幅下降\n(<-10)", range: [-Infinity, -10], count: 0, fill: COLORS.error },
      { name: "下降\n(-10~-1)", range: [-10, -1], count: 0, fill: "#ff7875" },
      { name: "不变\n(0)", range: [0, 0], count: 0, fill: COLORS.same },
      { name: "上升\n(1~10)", range: [1, 10], count: 0, fill: "#95de64" },
      { name: "大幅上升\n(>10)", range: [10, Infinity], count: 0, fill: COLORS.success },
    ];

    data.forEach((c) => {
      if (c.rankChange < -10) groups[0].count++;
      else if (c.rankChange >= -10 && c.rankChange < 0) groups[1].count++;
      else if (c.rankChange === 0) groups[2].count++;
      else if (c.rankChange > 0 && c.rankChange <= 10) groups[3].count++;
      else if (c.rankChange > 10) groups[4].count++;
    });

    return groups;
  }, [data]);

  // 详细分布数据（每个排名变化值的数量）
  const detailedData = useMemo(() => {
    const changeMap = new Map<number, number>();
    data.forEach((c) => {
      const change = Math.max(-15, Math.min(15, c.rankChange)); // 限制范围便于展示
      changeMap.set(change, (changeMap.get(change) || 0) + 1);
    });

    return Array.from(changeMap.entries())
      .map(([change, count]) => ({
        change,
        count,
        fill: change > 0 ? COLORS.up : change < 0 ? COLORS.down : COLORS.same,
      }))
      .sort((a, b) => a.change - b.change);
  }, [data]);

  return (
    <div className="chart-container">
      <h4 className="chart-title">排名变化分布</h4>
      <div className="chart-row">
        {/* 分组柱状图 */}
        <div className="chart-half">
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={distributionData} margin={{ top: 20, right: 30, left: 20, bottom: 40 }}>
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
              <XAxis
                dataKey="name"
                tick={{ fontSize: 11 }}
                interval={0}
                angle={0}
                textAnchor="middle"
              />
              <YAxis tick={{ fontSize: 12 }} />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    return (
                      <div className="custom-tooltip">
                        <p>{data.name.replace("\n", " ")}</p>
                        <p style={{ color: data.fill }}>{data.count} 个合同</p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Bar dataKey="count" name="合同数量" radius={[4, 4, 0, 0]}>
                {distributionData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* 详细分布图 */}
        <div className="chart-half">
          <ResponsiveContainer width="100%" height={250}>
            <BarChart data={detailedData} margin={{ top: 20, right: 30, left: 20, bottom: 40 }}>
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
              <XAxis
                dataKey="change"
                tick={{ fontSize: 11 }}
                label={{ value: "排名变化", position: "bottom", offset: 0 }}
              />
              <YAxis tick={{ fontSize: 12 }} />
              <ReferenceLine x={0} stroke="#666" strokeDasharray="5 5" />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    const changeText =
                      data.change > 0 ? `上升 ${data.change}` : data.change < 0 ? `下降 ${Math.abs(data.change)}` : "不变";
                    return (
                      <div className="custom-tooltip">
                        <p>排名{changeText}</p>
                        <p style={{ color: data.fill }}>{data.count} 个合同</p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Bar dataKey="count" name="合同数量" radius={[2, 2, 0, 0]}>
                {detailedData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}

// ============================================
// 2. 优先级对比散点图
// ============================================
interface PriorityScatterChartProps {
  data: SimulatedContract[];
}

export function PriorityScatterChart({ data }: PriorityScatterChartProps) {
  // 计算范围
  const range = useMemo(() => {
    const allValues = data.flatMap((c) => [c.priority, c.newPriority]);
    const min = Math.floor(Math.min(...allValues));
    const max = Math.ceil(Math.max(...allValues));
    return { min, max };
  }, [data]);

  // 准备散点数据
  const scatterData = useMemo(() => {
    return data.map((c) => ({
      x: c.priority, // 原优先级
      y: c.newPriority, // 新优先级
      z: Math.abs(c.priorityChange) * 10 + 5, // 点大小与变化幅度相关
      name: c.contract_id,
      change: c.priorityChange,
      fill: c.priorityChange > 0.1 ? COLORS.up : c.priorityChange < -0.1 ? COLORS.down : COLORS.same,
    }));
  }, [data]);

  return (
    <div className="chart-container">
      <h4 className="chart-title">优先级对比散点图</h4>
      <p className="chart-subtitle">对角线以上 = 优先级上升，以下 = 下降</p>
      <ResponsiveContainer width="100%" height={350}>
        <ScatterChart margin={{ top: 20, right: 30, left: 20, bottom: 40 }}>
          <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
          <XAxis
            type="number"
            dataKey="x"
            name="原优先级"
            domain={[range.min, range.max]}
            tick={{ fontSize: 12 }}
            label={{ value: "原优先级", position: "bottom", offset: 0 }}
          />
          <YAxis
            type="number"
            dataKey="y"
            name="新优先级"
            domain={[range.min, range.max]}
            tick={{ fontSize: 12 }}
            label={{ value: "新优先级", angle: -90, position: "insideLeft" }}
          />
          <ZAxis type="number" dataKey="z" range={[30, 150]} />
          {/* 对角线 - 表示无变化 */}
          <ReferenceLine
            segment={[
              { x: range.min, y: range.min },
              { x: range.max, y: range.max },
            ]}
            stroke="#666"
            strokeDasharray="5 5"
            label={{ value: "无变化", position: "end" }}
          />
          <Tooltip
            content={({ active, payload }) => {
              if (active && payload && payload.length) {
                const point = payload[0].payload;
                return (
                  <div className="custom-tooltip">
                    <p>
                      <strong>{point.name}</strong>
                    </p>
                    <p>原优先级: {point.x.toFixed(2)}</p>
                    <p>新优先级: {point.y.toFixed(2)}</p>
                    <p style={{ color: point.fill }}>
                      变化: {point.change > 0 ? "+" : ""}
                      {point.change.toFixed(2)}
                    </p>
                  </div>
                );
              }
              return null;
            }}
          />
          <Legend />
          <Scatter name="合同" data={scatterData}>
            {scatterData.map((entry, index) => (
              <Cell key={`cell-${index}`} fill={entry.fill} fillOpacity={0.7} />
            ))}
          </Scatter>
        </ScatterChart>
      </ResponsiveContainer>
    </div>
  );
}

// ============================================
// 3. S-Score vs P-Score 分析图
// ============================================
interface SPScoreAnalysisChartProps {
  data: SimulatedContract[];
  ws: number; // S-Score 权重
  wp: number; // P-Score 权重
}

export function SPScoreAnalysisChart({ data, ws, wp }: SPScoreAnalysisChartProps) {
  // 准备数据
  const scatterData = useMemo(() => {
    return data.map((c) => ({
      sScore: c.s_score,
      pScore: c.p_score,
      priority: c.newPriority,
      name: c.contract_id,
      specFamily: c.spec_family,
      rankChange: c.rankChange,
      fill: c.rankChange > 0 ? COLORS.up : c.rankChange < 0 ? COLORS.down : COLORS.same,
    }));
  }, [data]);

  // 按规格族分组数据
  const groupedBySpecFamily = useMemo(() => {
    const groups = new Map<string, typeof scatterData>();
    scatterData.forEach((point) => {
      const group = groups.get(point.specFamily) || [];
      group.push(point);
      groups.set(point.specFamily, group);
    });
    return Array.from(groups.entries()).map(([name, points], index) => ({
      name,
      points,
      color: COLORS.category[index % COLORS.category.length],
    }));
  }, [scatterData]);

  return (
    <div className="chart-container">
      <h4 className="chart-title">S-Score vs P-Score 分析</h4>
      <p className="chart-subtitle">
        当前权重: S-Score × {ws.toFixed(2)} + P-Score × {wp.toFixed(2)}
      </p>
      <ResponsiveContainer width="100%" height={350}>
        <ScatterChart margin={{ top: 20, right: 30, left: 20, bottom: 40 }}>
          <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
          <XAxis
            type="number"
            dataKey="sScore"
            name="S-Score"
            tick={{ fontSize: 12 }}
            label={{ value: "S-Score (战略价值)", position: "bottom", offset: 0 }}
          />
          <YAxis
            type="number"
            dataKey="pScore"
            name="P-Score"
            tick={{ fontSize: 12 }}
            label={{ value: "P-Score (生产难度)", angle: -90, position: "insideLeft" }}
          />
          <ZAxis type="number" range={[50, 100]} />
          <Tooltip
            content={({ active, payload }) => {
              if (active && payload && payload.length) {
                const point = payload[0].payload;
                return (
                  <div className="custom-tooltip">
                    <p>
                      <strong>{point.name}</strong>
                    </p>
                    <p>规格族: {point.specFamily}</p>
                    <p>S-Score: {point.sScore.toFixed(2)}</p>
                    <p>P-Score: {point.pScore.toFixed(2)}</p>
                    <p>新优先级: {point.priority.toFixed(2)}</p>
                    <p style={{ color: point.fill }}>
                      排名变化: {point.rankChange > 0 ? "+" : ""}
                      {point.rankChange}
                    </p>
                  </div>
                );
              }
              return null;
            }}
          />
          <Legend />
          {groupedBySpecFamily.map((group) => (
            <Scatter key={group.name} name={group.name} data={group.points} fill={group.color}>
              {group.points.map((_, index) => (
                <Cell key={`cell-${index}`} fillOpacity={0.7} />
              ))}
            </Scatter>
          ))}
        </ScatterChart>
      </ResponsiveContainer>
    </div>
  );
}

// ============================================
// 4. 分类统计饼图
// ============================================
interface CategoryDistributionChartProps {
  data: SimulatedContract[];
}

export function CategoryDistributionChart({ data }: CategoryDistributionChartProps) {
  const [groupBy, setGroupBy] = useState<"spec_family" | "customer">("spec_family");

  // 按规格族统计
  const specFamilyStats = useMemo(() => {
    const stats = new Map<string, { up: number; down: number; same: number }>();

    data.forEach((c) => {
      const key = c.spec_family;
      const existing = stats.get(key) || { up: 0, down: 0, same: 0 };
      if (c.rankChange > 0) existing.up++;
      else if (c.rankChange < 0) existing.down++;
      else existing.same++;
      stats.set(key, existing);
    });

    return Array.from(stats.entries())
      .map(([name, counts]) => ({
        name,
        total: counts.up + counts.down + counts.same,
        up: counts.up,
        down: counts.down,
        same: counts.same,
      }))
      .sort((a, b) => b.total - a.total);
  }, [data]);

  // 按客户统计
  const customerStats = useMemo(() => {
    const stats = new Map<string, { up: number; down: number; same: number }>();

    data.forEach((c) => {
      const key = c.customer_id;
      const existing = stats.get(key) || { up: 0, down: 0, same: 0 };
      if (c.rankChange > 0) existing.up++;
      else if (c.rankChange < 0) existing.down++;
      else existing.same++;
      stats.set(key, existing);
    });

    return Array.from(stats.entries())
      .map(([name, counts]) => ({
        name,
        total: counts.up + counts.down + counts.same,
        up: counts.up,
        down: counts.down,
        same: counts.same,
      }))
      .sort((a, b) => b.total - a.total)
      .slice(0, 10); // 只取前10个客户
  }, [data]);

  const currentStats = groupBy === "spec_family" ? specFamilyStats : customerStats;

  // 饼图数据 - 总体排名变化分布
  const pieData = useMemo(() => {
    const up = data.filter((c) => c.rankChange > 0).length;
    const down = data.filter((c) => c.rankChange < 0).length;
    const same = data.filter((c) => c.rankChange === 0).length;
    return [
      { name: "排名上升", value: up, fill: COLORS.up },
      { name: "排名下降", value: down, fill: COLORS.down },
      { name: "排名不变", value: same, fill: COLORS.same },
    ];
  }, [data]);

  return (
    <div className="chart-container">
      <div className="chart-header">
        <h4 className="chart-title">分类统计分析</h4>
        <div className="chart-controls">
          <button
            className={`chart-btn ${groupBy === "spec_family" ? "active" : ""}`}
            onClick={() => setGroupBy("spec_family")}
          >
            按规格族
          </button>
          <button
            className={`chart-btn ${groupBy === "customer" ? "active" : ""}`}
            onClick={() => setGroupBy("customer")}
          >
            按客户
          </button>
        </div>
      </div>

      <div className="chart-row">
        {/* 饼图 - 总体分布 */}
        <div className="chart-third">
          <p className="chart-label">总体排名变化分布</p>
          <ResponsiveContainer width="100%" height={220}>
            <PieChart>
              <Pie
                data={pieData}
                cx="50%"
                cy="50%"
                innerRadius={40}
                outerRadius={70}
                paddingAngle={2}
                dataKey="value"
                label={({ name, percent }) => `${name} ${((percent || 0) * 100).toFixed(0)}%`}
                labelLine={{ stroke: "#666", strokeWidth: 1 }}
              >
                {pieData.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Pie>
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    return (
                      <div className="custom-tooltip">
                        <p>{data.name}</p>
                        <p style={{ color: data.fill }}>{data.value} 个合同</p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* 堆叠柱状图 - 按分类 */}
        <div className="chart-two-thirds">
          <p className="chart-label">
            {groupBy === "spec_family" ? "各规格族" : "各客户"}排名变化统计
          </p>
          <ResponsiveContainer width="100%" height={220}>
            <BarChart
              data={currentStats}
              margin={{ top: 10, right: 30, left: 20, bottom: 40 }}
              layout="vertical"
            >
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} horizontal={false} />
              <XAxis type="number" tick={{ fontSize: 11 }} />
              <YAxis
                type="category"
                dataKey="name"
                tick={{ fontSize: 11 }}
                width={80}
                interval={0}
              />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    return (
                      <div className="custom-tooltip">
                        <p>
                          <strong>{data.name}</strong>
                        </p>
                        <p>总计: {data.total} 个合同</p>
                        <p style={{ color: COLORS.up }}>上升: {data.up}</p>
                        <p style={{ color: COLORS.down }}>下降: {data.down}</p>
                        <p style={{ color: COLORS.same }}>不变: {data.same}</p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Legend />
              <Bar dataKey="up" stackId="a" fill={COLORS.up} name="排名上升" />
              <Bar dataKey="down" stackId="a" fill={COLORS.down} name="排名下降" />
              <Bar dataKey="same" stackId="a" fill={COLORS.same} name="排名不变" />
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}

// ============================================
// 5. Top N 排名变化对比图
// ============================================
interface TopRankChangeChartProps {
  data: SimulatedContract[];
  topN?: number;
}

export function TopRankChangeChart({ data, topN = 20 }: TopRankChangeChartProps) {
  // 获取原始 Top N
  const topData = useMemo(() => {
    // 按新排名排序，取前 N 个
    return data.slice(0, topN).map((c) => ({
      name: c.contract_id,
      originalRank: c.originalRank,
      newRank: c.newRank,
      rankChange: c.rankChange,
      customer: c.customer_id,
      specFamily: c.spec_family,
      fill: c.rankChange > 0 ? COLORS.up : c.rankChange < 0 ? COLORS.down : COLORS.same,
    }));
  }, [data, topN]);

  // 排名变化最大的合同
  const biggestChanges = useMemo(() => {
    return [...data]
      .sort((a, b) => Math.abs(b.rankChange) - Math.abs(a.rankChange))
      .slice(0, 10)
      .map((c) => ({
        name: c.contract_id,
        originalRank: c.originalRank,
        newRank: c.newRank,
        change: c.rankChange,
        fill: c.rankChange > 0 ? COLORS.up : COLORS.down,
      }));
  }, [data]);

  return (
    <div className="chart-container">
      <h4 className="chart-title">Top {topN} 排名变化对比</h4>

      <div className="chart-row">
        {/* 原排名 vs 新排名 */}
        <div className="chart-half">
          <p className="chart-label">Top {topN} 合同排名变化</p>
          <ResponsiveContainer width="100%" height={300}>
            <LineChart
              data={topData}
              margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
            >
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} />
              <XAxis
                dataKey="name"
                tick={{ fontSize: 10 }}
                tickLine={false}
                interval={0}
                height={60}
              />
              <YAxis
                tick={{ fontSize: 12 }}
                reversed
                domain={[1, "dataMax"]}
                label={{ value: "排名", angle: -90, position: "insideLeft" }}
              />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const point = payload[0].payload;
                    return (
                      <div className="custom-tooltip">
                        <p>
                          <strong>{point.name}</strong>
                        </p>
                        <p>客户: {point.customer}</p>
                        <p>规格族: {point.specFamily}</p>
                        <p>原排名: {point.originalRank}</p>
                        <p>新排名: {point.newRank}</p>
                        <p style={{ color: point.fill }}>
                          变化: {point.rankChange > 0 ? "+" : ""}
                          {point.rankChange}
                        </p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Legend />
              <Line
                type="monotone"
                dataKey="originalRank"
                stroke="#8884d8"
                strokeWidth={2}
                name="原排名"
                dot={{ fill: "#8884d8", r: 3 }}
              />
              <Line
                type="monotone"
                dataKey="newRank"
                stroke={COLORS.primary}
                strokeWidth={2}
                name="新排名"
                dot={{ fill: COLORS.primary, r: 3 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </div>

        {/* 变化最大的合同 */}
        <div className="chart-half">
          <p className="chart-label">排名变化最大的 10 个合同</p>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart
              data={biggestChanges}
              margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
              layout="vertical"
            >
              <CartesianGrid strokeDasharray="3 3" opacity={0.3} horizontal={false} />
              <XAxis type="number" tick={{ fontSize: 12 }} />
              <YAxis
                type="category"
                dataKey="name"
                tick={{ fontSize: 10 }}
                width={70}
              />
              <ReferenceLine x={0} stroke="#666" />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const point = payload[0].payload;
                    return (
                      <div className="custom-tooltip">
                        <p>
                          <strong>{point.name}</strong>
                        </p>
                        <p>原排名: {point.originalRank}</p>
                        <p>新排名: {point.newRank}</p>
                        <p style={{ color: point.fill }}>
                          变化: {point.change > 0 ? "+" : ""}
                          {point.change}
                        </p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              <Bar dataKey="change" name="排名变化" radius={[0, 4, 4, 0]}>
                {biggestChanges.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.fill} />
                ))}
              </Bar>
            </BarChart>
          </ResponsiveContainer>
        </div>
      </div>
    </div>
  );
}

// ============================================
// 6. 综合图表视图容器
// ============================================
type ChartType = "distribution" | "scatter" | "sp-analysis" | "category" | "top-rank";

interface SandboxChartsProps {
  data: SimulatedContract[];
  ws: number;
  wp: number;
}

export function SandboxCharts({ data, ws, wp }: SandboxChartsProps) {
  const [activeChart, setActiveChart] = useState<ChartType>("distribution");

  const chartOptions: { key: ChartType; label: string }[] = [
    { key: "distribution", label: "排名分布" },
    { key: "scatter", label: "优先级对比" },
    { key: "sp-analysis", label: "S/P分析" },
    { key: "category", label: "分类统计" },
    { key: "top-rank", label: "Top N变化" },
  ];

  return (
    <div className="sandbox-charts">
      {/* 图表选择器 */}
      <div className="chart-selector">
        {chartOptions.map((opt) => (
          <button
            key={opt.key}
            className={`chart-selector-btn ${activeChart === opt.key ? "active" : ""}`}
            onClick={() => setActiveChart(opt.key)}
          >
            {opt.label}
          </button>
        ))}
      </div>

      {/* 图表内容 */}
      <div className="chart-content">
        {activeChart === "distribution" && <RankChangeDistributionChart data={data} />}
        {activeChart === "scatter" && <PriorityScatterChart data={data} />}
        {activeChart === "sp-analysis" && <SPScoreAnalysisChart data={data} ws={ws} wp={wp} />}
        {activeChart === "category" && <CategoryDistributionChart data={data} />}
        {activeChart === "top-rank" && <TopRankChangeChart data={data} />}
      </div>

      <style>{`
        .sandbox-charts {
          display: flex;
          flex-direction: column;
          height: 100%;
        }

        .chart-selector {
          display: flex;
          gap: 8px;
          padding: 12px 16px;
          background: var(--color-bg-layout);
          border-bottom: 1px solid var(--color-border-light);
          flex-wrap: wrap;
        }

        .chart-selector-btn {
          padding: 6px 14px;
          border: 1px solid var(--color-border);
          border-radius: 16px;
          background: var(--color-bg-container);
          font-size: 13px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .chart-selector-btn:hover {
          border-color: var(--color-primary);
          color: var(--color-primary);
        }

        .chart-selector-btn.active {
          background: var(--color-primary);
          border-color: var(--color-primary);
          color: #fff;
        }

        .chart-content {
          flex: 1;
          overflow-y: auto;
          padding: 16px;
        }

        .chart-container {
          background: var(--color-bg-container);
          border-radius: 8px;
          padding: 16px;
        }

        .chart-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 16px;
        }

        .chart-title {
          margin: 0 0 8px 0;
          font-size: 15px;
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .chart-subtitle {
          margin: 0 0 16px 0;
          font-size: 12px;
          color: var(--color-text-tertiary);
        }

        .chart-label {
          margin: 0 0 8px 0;
          font-size: 13px;
          font-weight: 500;
          color: var(--color-text-secondary);
          text-align: center;
        }

        .chart-controls {
          display: flex;
          gap: 4px;
        }

        .chart-btn {
          padding: 4px 10px;
          border: 1px solid var(--color-border);
          border-radius: 4px;
          background: var(--color-bg-container);
          font-size: 12px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .chart-btn:hover {
          border-color: var(--color-primary);
          color: var(--color-primary);
        }

        .chart-btn.active {
          background: var(--color-primary);
          border-color: var(--color-primary);
          color: #fff;
        }

        .chart-row {
          display: flex;
          gap: 16px;
        }

        .chart-half {
          flex: 1;
          min-width: 0;
        }

        .chart-third {
          flex: 1;
          min-width: 0;
        }

        .chart-two-thirds {
          flex: 2;
          min-width: 0;
        }

        .custom-tooltip {
          background: rgba(255, 255, 255, 0.96);
          border: 1px solid var(--color-border);
          border-radius: 6px;
          padding: 10px 14px;
          font-size: 12px;
          box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
        }

        .custom-tooltip p {
          margin: 4px 0;
        }

        .custom-tooltip strong {
          font-weight: 600;
        }

        @media (max-width: 900px) {
          .chart-row {
            flex-direction: column;
          }
        }
      `}</style>
    </div>
  );
}
