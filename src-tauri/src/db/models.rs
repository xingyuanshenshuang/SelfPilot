use serde::{Deserialize, Serialize};

/// 目标（一级节点）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub deadline: Option<String>,
    pub total_qty: f64,
    pub unit: String,
    pub created_at: String,
}

/// 创建目标的输入参数
#[derive(Debug, Clone, Deserialize)]
pub struct CreateGoalInput {
    pub name: String,
    pub deadline: Option<String>,
    pub total_qty: Option<f64>,
    pub unit: Option<String>,
}

/// 阶段（二级节点，可选）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Stage {
    pub id: String,
    pub goal_id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub path: String,
    pub sort_order: i64,
    pub created_at: String,
}

/// 任务（三级节点，实际执行单元）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: String,
    pub goal_id: String,
    pub stage_id: Option<String>,
    pub parent_id: Option<String>,
    pub path: String,
    pub name: String,
    pub plan_date: Option<String>,
    pub plan_qty: f64,
    pub actual_qty: f64,
    pub unit: String,
    pub status: String,
    pub is_manual: i64,
    pub source: String,
    pub sort_order: i64,
    pub created_at: String,
}

/// 创建任务的输入参数
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTaskInput {
    pub goal_id: String,
    pub stage_id: Option<String>,
    pub name: String,
    pub plan_date: Option<String>,
    pub plan_qty: Option<f64>,
    pub unit: Option<String>,
}

/// 完成任务的输入参数（支持部分完成）
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteTaskInput {
    pub task_id: String,
    pub actual_qty: f64,
}

/// 进度信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    pub id: String,
    pub name: String,
    pub total_plan: f64,
    pub total_actual: f64,
    /// 完成百分比 0.0 ~ 1.0
    pub percentage: f64,
}

/// 今日待办任务（带目标名称）
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TodayTask {
    pub id: String,
    pub goal_id: String,
    pub goal_name: String,
    pub stage_id: Option<String>,
    pub name: String,
    pub plan_date: Option<String>,
    pub plan_qty: f64,
    pub actual_qty: f64,
    pub unit: String,
    pub status: String,
    pub source: String,
}

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Pending,
    Partial,
    Done,
    Skipped,
}

impl TaskStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskStatus::Pending => "pending",
            TaskStatus::Partial => "partial",
            TaskStatus::Done => "done",
            TaskStatus::Skipped => "skipped",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(TaskStatus::Pending),
            "partial" => Some(TaskStatus::Partial),
            "done" => Some(TaskStatus::Done),
            "skipped" => Some(TaskStatus::Skipped),
            _ => None,
        }
    }
}
