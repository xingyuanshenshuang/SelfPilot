use tauri::State;

use crate::db::models::{SetSettingInput, Setting};
use crate::db::DbPool;
use crate::error::AppResult;

/// 获取所有设置项
#[tauri::command]
pub async fn get_all_settings(state: State<'_, DbPool>) -> AppResult<Vec<Setting>> {
    let list: Vec<Setting> = sqlx::query_as("SELECT * FROM settings ORDER BY key")
        .fetch_all(&state.0)
        .await?;
    Ok(list)
}

/// 获取单个设置项
#[tauri::command]
pub async fn get_setting(
    key: String,
    state: State<'_, DbPool>,
) -> AppResult<Option<String>> {
    let value: Option<String> =
        sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind(&key)
            .fetch_optional(&state.0)
            .await?;
    Ok(value)
}

/// 设置某个值（upsert）
#[tauri::command]
pub async fn set_setting(input: SetSettingInput, state: State<'_, DbPool>) -> AppResult<()> {
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(&input.key)
    .bind(&input.value)
    .execute(&state.0)
    .await?;
    Ok(())
}
