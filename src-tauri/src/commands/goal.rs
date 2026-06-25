use tauri::State;
use uuid::Uuid;

use crate::db::models::{CreateGoalInput, Goal, Task};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};
use crate::services::split_service;

/// 创建目标
#[tauri::command]
pub async fn create_goal(input: CreateGoalInput, state: State<'_, DbPool>) -> AppResult<Goal> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let total_qty = input.total_qty.unwrap_or(0.0);
    let unit = input.unit.unwrap_or_default();

    sqlx::query(
        "INSERT INTO goals (id, name, deadline, total_qty, unit, created_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.name)
    .bind(&input.deadline)
    .bind(total_qty)
    .bind(&unit)
    .bind(&now)
    .execute(&state.0)
    .await?;

    let goal: Goal = sqlx::query_as("SELECT * FROM goals WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.0)
        .await?;

    Ok(goal)
}

/// 列出所有目标
#[tauri::command]
pub async fn list_goals(state: State<'_, DbPool>) -> AppResult<Vec<Goal>> {
    let goals: Vec<Goal> = sqlx::query_as("SELECT * FROM goals ORDER BY created_at")
        .fetch_all(&state.0)
        .await?;
    Ok(goals)
}

/// 获取单个目标
#[tauri::command]
pub async fn get_goal(id: String, state: State<'_, DbPool>) -> AppResult<Goal> {
    let goal: Goal = sqlx::query_as("SELECT * FROM goals WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("目标 {} 不存在", id)))?;
    Ok(goal)
}

/// 删除目标（级联删除任务）
#[tauri::command]
pub async fn delete_goal(id: String, state: State<'_, DbPool>) -> AppResult<()> {
    sqlx::query("DELETE FROM goals WHERE id = ?")
        .bind(&id)
        .execute(&state.0)
        .await?;
    Ok(())
}

/// 自动拆解目标为每日任务
///
/// PRD §4.2 模块二：按剩余天数平均分配，余数分到前几天
#[tauri::command]
pub async fn auto_split(goal_id: String, state: State<'_, DbPool>) -> AppResult<Vec<Task>> {
    let goal: Goal = sqlx::query_as("SELECT * FROM goals WHERE id = ?")
        .bind(&goal_id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("目标 {} 不存在", goal_id)))?;

    // 检查是否已有自动拆解任务，避免重复
    let existing: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE goal_id = ? AND source = 'auto'")
            .bind(&goal_id)
            .fetch_one(&state.0)
            .await?;

    if existing > 0 {
        return Err(AppError::Business(
            "该目标已有自动拆解任务，请先删除旧任务再重新拆解".into(),
        ));
    }

    // 执行拆解算法
    let today = chrono::Local::now().date_naive();
    let tasks = split_service::split_goal_into_tasks(&goal, today)?;

    // 批量插入任务
    for task in &tasks {
        sqlx::query(
            "INSERT INTO tasks (id, goal_id, stage_id, parent_id, path, name, plan_date,
             plan_qty, actual_qty, unit, status, is_manual, source, sort_order, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&task.id)
        .bind(&task.goal_id)
        .bind(&task.stage_id)
        .bind(&task.parent_id)
        .bind(&task.path)
        .bind(&task.name)
        .bind(&task.plan_date)
        .bind(task.plan_qty)
        .bind(task.actual_qty)
        .bind(&task.unit)
        .bind(&task.status)
        .bind(task.is_manual)
        .bind(&task.source)
        .bind(task.sort_order)
        .bind(&task.created_at)
        .execute(&state.0)
        .await?;
    }

    Ok(tasks)
}
