use tauri::State;

use crate::db::models::ProgressInfo;
use crate::db::DbPool;
use crate::error::AppResult;
use crate::services::progress_service;

/// 获取单个目标的进度
#[tauri::command]
pub async fn get_goal_progress(
    goal_id: String,
    state: State<'_, DbPool>,
) -> AppResult<ProgressInfo> {
    progress_service::calc_goal_progress(&state.0, &goal_id).await
}

/// 获取所有目标的进度
#[tauri::command]
pub async fn get_all_goals_progress(state: State<'_, DbPool>) -> AppResult<Vec<ProgressInfo>> {
    progress_service::calc_all_goals_progress(&state.0).await
}
