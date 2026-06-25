-- SelfPilot Sprint 4 迁移：鼓励语库 + 设置项
-- 对应 PRD §4.2 模块七、模块八

-- 鼓励语库
CREATE TABLE IF NOT EXISTS encouragements (
    id TEXT PRIMARY KEY,
    text TEXT NOT NULL,
    -- preset 预设 | custom 用户自定义
    category TEXT NOT NULL DEFAULT 'custom'
        CHECK(category IN ('preset','custom')),
    created_at TEXT NOT NULL
);

-- 设置项（key-value，用于主题等）
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- 初始化预设鼓励语
-- PRD §4.2 模块七：预设鼓励语库，用户可查看并添加自定义
INSERT OR IGNORE INTO encouragements (id, text, category, created_at) VALUES
    ('preset-01', '每一次坚持，都是未来的你在感谢现在的你。', 'preset', '2026-06-15T00:00:00'),
    ('preset-02', '今天的努力，是明天的底气。', 'preset', '2026-06-15T00:00:00'),
    ('preset-03', '小步快跑，终将抵达。', 'preset', '2026-06-15T00:00:00'),
    ('preset-04', '完成比完美更重要。', 'preset', '2026-06-15T00:00:00'),
    ('preset-05', '你比昨天的自己更进了一步。', 'preset', '2026-06-15T00:00:00'),
    ('preset-06', '自律给我自由。', 'preset', '2026-06-15T00:00:00'),
    ('preset-07', '日拱一卒，功不唐捐。', 'preset', '2026-06-15T00:00:00'),
    ('preset-08', '学习是最稳赚不赔的投资。', 'preset', '2026-06-15T00:00:00'),
    ('preset-09', '今日事今日毕，明日还有明日事。', 'preset', '2026-06-15T00:00:00'),
    ('preset-10', '保持节奏，稳步前行。', 'preset', '2026-06-15T00:00:00');

-- 默认设置：浅色主题
INSERT OR IGNORE INTO settings (key, value) VALUES ('theme', 'light');
