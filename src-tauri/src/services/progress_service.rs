use sqlx::SqlitePool;

use crate::db::models::ProgressInfo;
use crate::error::{AppError, AppResult};

/// 计算单个目标的进度
///
/// PRD §4.2 模块二：
/// - 目标进度 = sum(actual_qty) / sum(plan_qty)（基于子任务加权）
/// - 已跳过任务不计入
pub async fn calc_goal_progress(pool: &SqlitePool, goal_id: &str) -> AppResult<ProgressInfo> {
    let goal_name: String = sqlx::query_scalar("SELECT name FROM goals WHERE id = ?")
        .bind(goal_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("目标 {} 不存在", goal_id)))?;

    // 查询目标下所有非跳过任务的计划量与实际完成量
    let rows: Vec<(f64, f64)> = sqlx::query_as(
        "SELECT plan_qty, actual_qty FROM tasks
         WHERE goal_id = ? AND status != 'skipped'",
    )
    .bind(goal_id)
    .fetch_all(pool)
    .await?;

    let (total_plan, total_actual) = rows
        .iter()
        .fold((0.0_f64, 0.0_f64), |(p, a), (pq, aq)| (p + pq, a + aq));

    let percentage = if total_plan > 0.0 {
        (total_actual / total_plan).min(1.0)
    } else {
        0.0
    };

    Ok(ProgressInfo {
        id: goal_id.to_string(),
        name: goal_name,
        total_plan,
        total_actual,
        percentage,
    })
}

/// 计算所有目标的进度
pub async fn calc_all_goals_progress(pool: &SqlitePool) -> AppResult<Vec<ProgressInfo>> {
    let goal_ids: Vec<String> =
        sqlx::query_scalar("SELECT id FROM goals ORDER BY created_at")
            .fetch_all(pool)
            .await?;

    let mut result = Vec::with_capacity(goal_ids.len());
    for id in goal_ids {
        result.push(calc_goal_progress(pool, &id).await?);
    }
    Ok(result)
}
