use tauri::State;
use uuid::Uuid;

use crate::db::models::{CreateStageInput, DeleteStageInput, Stage, StageWithProgress, UpdateStageInput};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

/// 创建阶段
///
/// PRD §4.2 模块三：目标 → 阶段 → 任务 层级
#[tauri::command]
pub async fn create_stage(input: CreateStageInput, state: State<'_, DbPool>) -> AppResult<Stage> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    // 计算 sort_order：当前目标下阶段数量
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM stages WHERE goal_id = ?")
        .bind(&input.goal_id)
        .fetch_one(&state.0)
        .await?;

    // 构建 path：/goal_id/stage_id
    let path = format!("/{}/{}", input.goal_id, id);

    sqlx::query(
        "INSERT INTO stages (id, goal_id, name, parent_id, path, sort_order, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&input.goal_id)
    .bind(&input.name)
    .bind(&input.parent_id)
    .bind(&path)
    .bind(count)
    .bind(&now)
    .execute(&state.0)
    .await?;

    let stage: Stage = sqlx::query_as("SELECT * FROM stages WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.0)
        .await?;

    Ok(stage)
}

/// 列出某目标下所有阶段（带进度）
#[tauri::command]
pub async fn list_stages(goal_id: String, state: State<'_, DbPool>) -> AppResult<Vec<StageWithProgress>> {
    // 阶段进度按子任务加权汇总（排除已跳过任务的计划量，但保留已完成任务的完成量）
    let stages: Vec<StageWithProgress> = sqlx::query_as(
        "SELECT s.id, s.goal_id, s.name, s.parent_id, s.path, s.sort_order, s.created_at,
                COALESCE(SUM(CASE WHEN t.status != 'skipped' THEN t.plan_qty ELSE 0 END), 0) as total_plan,
                COALESCE(SUM(t.actual_qty), 0) as total_actual,
                CASE
                    WHEN COALESCE(SUM(CASE WHEN t.status != 'skipped' THEN t.plan_qty ELSE 0 END), 0) = 0 THEN 0
                    ELSE COALESCE(SUM(t.actual_qty), 0) * 1.0
                         / COALESCE(SUM(CASE WHEN t.status != 'skipped' THEN t.plan_qty ELSE 0 END), 0)
                END as percentage,
                COUNT(t.id) as task_count
         FROM stages s
         LEFT JOIN tasks t ON t.stage_id = s.id
         WHERE s.goal_id = ?
         GROUP BY s.id, s.goal_id, s.name, s.parent_id, s.path, s.sort_order, s.created_at
         ORDER BY s.sort_order",
    )
    .bind(&goal_id)
    .fetch_all(&state.0)
    .await?;

    Ok(stages)
}

/// 更新阶段（名称或排序）
#[tauri::command]
pub async fn update_stage(input: UpdateStageInput, state: State<'_, DbPool>) -> AppResult<Stage> {
    let mut updates: Vec<String> = Vec::new();
    if input.name.is_some() {
        updates.push("name = ?".to_string());
    }
    if input.sort_order.is_some() {
        updates.push("sort_order = ?".to_string());
    }

    if updates.is_empty() {
        return Err(AppError::Param("未提供任何更新字段".into()));
    }

    let sql = format!("UPDATE stages SET {} WHERE id = ?", updates.join(", "));

    // 动态绑定参数
    if let Some(name) = &input.name {
        if let Some(sort_order) = input.sort_order {
            sqlx::query(&sql)
                .bind(name)
                .bind(sort_order)
                .bind(&input.id)
                .execute(&state.0)
                .await?;
        } else {
            sqlx::query(&sql)
                .bind(name)
                .bind(&input.id)
                .execute(&state.0)
                .await?;
        }
    } else if let Some(sort_order) = input.sort_order {
        sqlx::query(&sql)
            .bind(sort_order)
            .bind(&input.id)
            .execute(&state.0)
            .await?;
    }

    let stage: Stage = sqlx::query_as("SELECT * FROM stages WHERE id = ?")
        .bind(&input.id)
        .fetch_one(&state.0)
        .await?;

    Ok(stage)
}

/// 删除阶段
///
/// PRD §4.2：删除时提示子任务转为独立任务或级联删除
/// - mode="detach": 子任务的 stage_id 置空，转为独立任务
/// - mode="cascade": 级联删除该阶段下所有任务
#[tauri::command]
pub async fn delete_stage(input: DeleteStageInput, state: State<'_, DbPool>) -> AppResult<()> {
    // 校验阶段存在
    let stage: Stage = sqlx::query_as("SELECT * FROM stages WHERE id = ?")
        .bind(&input.id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("阶段 {} 不存在", input.id)))?;

    match input.mode.as_str() {
        "detach" => {
            // 子任务转为独立任务：stage_id 置空，path 更新为 /goal_id/task_id
            sqlx::query(
                "UPDATE tasks SET stage_id = NULL,
                 path = '/' || goal_id || '/' || id
                 WHERE stage_id = ?",
            )
            .bind(&input.id)
            .execute(&state.0)
            .await?;
        }
        "cascade" => {
            // 级联删除子任务
            sqlx::query("DELETE FROM tasks WHERE stage_id = ?")
                .bind(&input.id)
                .execute(&state.0)
                .await?;
        }
        _ => {
            return Err(AppError::Param(format!(
                "未知的删除模式: {}，应为 'detach' 或 'cascade'",
                input.mode
            )));
        }
    }

    // 删除阶段本身
    let _ = stage; // 消除未使用警告
    sqlx::query("DELETE FROM stages WHERE id = ?")
        .bind(&input.id)
        .execute(&state.0)
        .await?;

    Ok(())
}
