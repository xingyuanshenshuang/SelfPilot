use tauri::State;
use uuid::Uuid;

use crate::db::models::{CompleteTaskInput, CreateTaskInput, Task, TodayTask};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

/// 手动创建任务
#[tauri::command]
pub async fn create_task(input: CreateTaskInput, state: State<'_, DbPool>) -> AppResult<Task> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let plan_qty = input.plan_qty.unwrap_or(1.0);
    let unit = input.unit.unwrap_or_default();
    let path = format!("/{}/{}", input.goal_id, id);

    sqlx::query(
        "INSERT INTO tasks (id, goal_id, stage_id, parent_id, path, name, plan_date,
         plan_qty, actual_qty, unit, status, is_manual, source, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.goal_id)
    .bind(&input.stage_id)
    .bind(&input.goal_id)
    .bind(&path)
    .bind(&input.name)
    .bind(&input.plan_date)
    .bind(plan_qty)
    .bind(0.0)
    .bind(&unit)
    .bind("pending")
    .bind(1)
    .bind("manual")
    .bind(0)
    .bind(&now)
    .execute(&state.0)
    .await?;

    let task: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.0)
        .await?;

    Ok(task)
}

/// 完成任务（支持部分完成）
///
/// PRD §4.2 模块二：
/// - actual_qty >= plan_qty → done
/// - 0 < actual_qty < plan_qty → partial
/// - 禁止对已完成任务重复标记
#[tauri::command]
pub async fn complete_task(
    input: CompleteTaskInput,
    state: State<'_, DbPool>,
) -> AppResult<Task> {
    if input.actual_qty < 0.0 {
        return Err(AppError::Param("实际完成量不能为负".into()));
    }

    let task: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .bind(&input.task_id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("任务 {} 不存在", input.task_id)))?;

    // 禁止对已完成任务再次标记完成
    if task.status == "done" {
        return Err(AppError::Business("任务已完成，不能重复标记".into()));
    }

    // 校验不超过计划数量
    if input.actual_qty > task.plan_qty {
        return Err(AppError::Business(format!(
            "实际完成量({})不能超过计划数量({})",
            input.actual_qty, task.plan_qty
        )));
    }

    // 计算新状态
    let new_status = if input.actual_qty >= task.plan_qty {
        "done"
    } else if input.actual_qty > 0.0 {
        "partial"
    } else {
        "pending"
    };

    sqlx::query("UPDATE tasks SET actual_qty = ?, status = ? WHERE id = ?")
        .bind(input.actual_qty)
        .bind(new_status)
        .bind(&input.task_id)
        .execute(&state.0)
        .await?;

    let updated: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .bind(&input.task_id)
        .fetch_one(&state.0)
        .await?;

    Ok(updated)
}

/// 跳过任务
///
/// PRD §4.2 模块二：标记为"已跳过"，不计入完成，不影响后续计划
#[tauri::command]
pub async fn skip_task(task_id: String, state: State<'_, DbPool>) -> AppResult<Task> {
    let _task: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .bind(&task_id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("任务 {} 不存在", task_id)))?;

    sqlx::query("UPDATE tasks SET status = 'skipped' WHERE id = ?")
        .bind(&task_id)
        .execute(&state.0)
        .await?;

    let updated: Task = sqlx::query_as("SELECT * FROM tasks WHERE id = ?")
        .bind(&task_id)
        .fetch_one(&state.0)
        .await?;

    Ok(updated)
}

/// 列出今日待办任务（今日计划任务，不含跳过）
#[tauri::command]
pub async fn list_today_tasks(state: State<'_, DbPool>) -> AppResult<Vec<TodayTask>> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let tasks: Vec<TodayTask> = sqlx::query_as(
        "SELECT t.id, t.goal_id, g.name as goal_name, t.stage_id, t.name,
                t.plan_date, t.plan_qty, t.actual_qty, t.unit, t.status, t.source
         FROM tasks t
         JOIN goals g ON t.goal_id = g.id
         WHERE t.plan_date = ? AND t.status != 'skipped'
         ORDER BY t.sort_order",
    )
    .bind(&today)
    .fetch_all(&state.0)
    .await?;

    Ok(tasks)
}

/// 列出逾期未完成任务（截止日期早于今日且未完成）
#[tauri::command]
pub async fn list_overdue_tasks(state: State<'_, DbPool>) -> AppResult<Vec<TodayTask>> {
    let today = chrono::Local::now().format("%Y-%m-%d").to_string();

    let tasks: Vec<TodayTask> = sqlx::query_as(
        "SELECT t.id, t.goal_id, g.name as goal_name, t.stage_id, t.name,
                t.plan_date, t.plan_qty, t.actual_qty, t.unit, t.status, t.source
         FROM tasks t
         JOIN goals g ON t.goal_id = g.id
         WHERE t.plan_date < ? AND t.status IN ('pending', 'partial')
         ORDER BY t.plan_date",
    )
    .bind(&today)
    .fetch_all(&state.0)
    .await?;

    Ok(tasks)
}

/// 列出某目标下所有任务
#[tauri::command]
pub async fn list_tasks_by_goal(
    goal_id: String,
    state: State<'_, DbPool>,
) -> AppResult<Vec<Task>> {
    let tasks: Vec<Task> = sqlx::query_as(
        "SELECT * FROM tasks WHERE goal_id = ? ORDER BY plan_date, sort_order",
    )
    .bind(&goal_id)
    .fetch_all(&state.0)
    .await?;

    Ok(tasks)
}
