-- SelfPilot 初始数据库结构
-- 对应 PRD §6 数据模型：UUID + parent_id + path

-- 目标表（一级节点）
CREATE TABLE IF NOT EXISTS goals (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    deadline TEXT,
    total_qty REAL NOT NULL DEFAULT 0,
    unit TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- 阶段表（二级节点，可选，用户手动添加用于分组）
CREATE TABLE IF NOT EXISTS stages (
    id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    path TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

-- 任务表（三级节点，实际执行单元）
CREATE TABLE IF NOT EXISTS tasks (
    id TEXT PRIMARY KEY,
    goal_id TEXT NOT NULL,
    stage_id TEXT,
    parent_id TEXT,
    path TEXT NOT NULL,
    name TEXT NOT NULL,
    plan_date TEXT,
    plan_qty REAL NOT NULL DEFAULT 1,
    actual_qty REAL NOT NULL DEFAULT 0,
    unit TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK(status IN ('pending','partial','done','skipped')),
    is_manual INTEGER NOT NULL DEFAULT 0,
    source TEXT NOT NULL DEFAULT 'manual'
        CHECK(source IN ('auto','manual')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (goal_id) REFERENCES goals(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_tasks_plan_date ON tasks(plan_date);
CREATE INDEX IF NOT EXISTS idx_tasks_goal_id ON tasks(goal_id);
CREATE INDEX IF NOT EXISTS idx_tasks_stage_id ON tasks(stage_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_stages_goal_id ON stages(goal_id);
