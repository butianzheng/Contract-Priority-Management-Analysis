-- 性能压测模拟数据（10,000 条合同）
-- 说明：仅在新建数据库时加载，用于前后端性能与交互压力测试

-- 使用 100x100 组合生成 10,000 条合约，避免递归深度限制
WITH RECURSIVE
  a(i) AS (VALUES(0) UNION ALL SELECT i + 1 FROM a WHERE i < 99),
  b(i) AS (VALUES(0) UNION ALL SELECT i + 1 FROM b WHERE i < 99)
INSERT INTO contract_master (
    contract_id,
    customer_id,
    steel_grade,
    thickness,
    width,
    spec_family,
    pdd,
    days_to_pdd,
    margin
)
SELECT
    printf('PERF%05d', (a.i * 100 + b.i) + 1) AS contract_id,
    (
        SELECT customer_id
        FROM customer_master
        ORDER BY customer_id
        LIMIT 1 OFFSET ((a.i * 100 + b.i) % (SELECT COUNT(*) FROM customer_master))
    ) AS customer_id,
    CASE ((a.i * 100 + b.i) % 4)
        WHEN 0 THEN 'Q235'
        WHEN 1 THEN 'Q345'
        WHEN 2 THEN '304'
        ELSE 'Q420'
    END AS steel_grade,
    6.0 + ((a.i * 100 + b.i) % 40) * 0.5 AS thickness,
    900 + ((a.i * 100 + b.i) % 800) AS width,
    CASE ((a.i * 100 + b.i) % 3)
        WHEN 0 THEN '常规'
        WHEN 1 THEN '特殊'
        ELSE '超特'
    END AS spec_family,
    date('now', printf('+%d day', ((a.i * 100 + b.i) % 60))) AS pdd,
    ((a.i * 100 + b.i) % 60) AS days_to_pdd,
    200.0 + ((a.i * 100 + b.i) % 900) AS margin
FROM a
CROSS JOIN b;
