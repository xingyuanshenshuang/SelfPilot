use tauri::State;

use crate::db::models::{
    Encouragement, ExportData, Goal, ImportInput, ImportResult, Setting, Stage, Task,
};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

/// 导出全部数据为 JSON 字符串
///
/// PRD §4.2 模块八 & 分阶段计划 Sprint 4：
/// 完整备份 goals + stages + tasks + encouragements + settings
#[tauri::command]
pub async fn export_data(state: State<'_, DbPool>) -> AppResult<String> {
    let goals: Vec<Goal> = sqlx::query_as("SELECT * FROM goals ORDER BY created_at")
        .fetch_all(&state.0)
        .await?;
    let stages: Vec<Stage> = sqlx::query_as("SELECT * FROM stages ORDER BY sort_order")
        .fetch_all(&state.0)
        .await?;
    let tasks: Vec<Task> =
        sqlx::query_as("SELECT * FROM tasks ORDER BY plan_date, sort_order")
            .fetch_all(&state.0)
            .await?;
    let encouragements: Vec<Encouragement> =
        sqlx::query_as("SELECT * FROM encouragements ORDER BY created_at")
            .fetch_all(&state.0)
            .await?;
    let settings: Vec<Setting> = sqlx::query_as("SELECT * FROM settings ORDER BY key")
        .fetch_all(&state.0)
        .await?;

    let data = ExportData {
        version: "1.0".to_string(),
        exported_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        goals,
        stages,
        tasks,
        encouragements,
        settings,
    };

    serde_json::to_string_pretty(&data)
        .map_err(|e| AppError::Internal(format!("序列化失败: {}", e)))
}

/// 导入数据
///
/// PRD §4.2 模块八 & 分阶段计划 Sprint 4：
/// conflict_mode:
///   - "skip": ID 冲突时跳过
///   - "overwrite": ID 冲突时覆盖
///   - "rename": ID 冲突时生成新 ID
#[tauri::command]
pub async fn import_data(
    input: ImportInput,
    state: State<'_, DbPool>,
) -> AppResult<ImportResult> {
    let data: ExportData = serde_json::from_str(&input.data)
        .map_err(|e| AppError::Param(format!("JSON 解析失败: {}", e)))?;

    let mode = input.conflict_mode.as_str();
    if !["skip", "overwrite", "rename"].contains(&mode) {
        return Err(AppError::Param(format!(
            "未知冲突模式: {}，应为 skip/overwrite/rename",
            mode
        )));
    }

    let mut result = ImportResult {
        goals_imported: 0,
        goals_skipped: 0,
        stages_imported: 0,
        stages_skipped: 0,
        tasks_imported: 0,
        tasks_skipped: 0,
        encouragements_imported: 0,
        settings_imported: 0,
    };

    // 用于 rename 模式下的 ID 映射（外键关联）
    use std::collections::HashMap;
    let mut goal_id_map: HashMap<String, String> = HashMap::new();
    let mut stage_id_map: HashMap<String, String> = HashMap::new();

    // 导入 goals
    for g in data.goals {
        let exists: bool = sqlx::query_scalar::<_, i64>("SELECT 1 FROM goals WHERE id = ?")
            .bind(&g.id)
            .fetch_optional(&state.0)
            .await?
            .is_some();

        let (id, action) = match (exists, mode) {
            (false, _) => (g.id.clone(), "import"),
            (true, "skip") => (g.id.clone(), "skip"),
            (true, "overwrite") => (g.id.clone(), "overwrite"),
            (true, "rename") => {
                let new_id = uuid::Uuid::new_v4().to_string();
                goal_id_map.insert(g.id.clone(), new_id.clone());
                (new_id, "rename")
            }
            _ => (g.id.clone(), "skip"),
        };

        match action {
            "skip" => {
                result.goals_skipped += 1;
                if mode == "rename" {
                    goal_id_map.insert(g.id.clone(), g.id.clone());
                }
            }
            "overwrite" => {
                sqlx::query(
                    "INSERT INTO goals (id, name, deadline, total_qty, unit, created_at)
                     VALUES (?, ?, ?, ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET
                     name=excluded.name, deadline=excluded.deadline,
                     total_qty=excluded.total_qty, unit=excluded.unit",
                )
                .bind(&id)
                .bind(&g.name)
                .bind(&g.deadline)
                .bind(g.total_qty)
                .bind(&g.unit)
                .bind(&g.created_at)
                .execute(&state.0)
                .await?;
                result.goals_imported += 1;
            }
            _ => {
                sqlx::query(
                    "INSERT INTO goals (id, name, deadline, total_qty, unit, created_at)
                     VALUES (?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(&g.name)
                .bind(&g.deadline)
                .bind(g.total_qty)
                .bind(&g.unit)
                .bind(&g.created_at)
                .execute(&state.0)
                .await?;
                result.goals_imported += 1;
                if mode == "rename" && id != g.id {
                    goal_id_map.insert(g.id.clone(), id.clone());
                }
            }
        }
    }

    // 导入 stages
    for s in data.stages {
        // 关联的 goal_id 可能被 rename
        let mapped_goal_id = if mode == "rename" {
            goal_id_map.get(&s.goal_id).cloned().unwrap_or(s.goal_id.clone())
        } else {
            s.goal_id.clone()
        };

        let exists: bool = sqlx::query_scalar::<_, i64>("SELECT 1 FROM stages WHERE id = ?")
            .bind(&s.id)
            .fetch_optional(&state.0)
            .await?
            .is_some();

        let (id, action) = match (exists, mode) {
            (false, _) => (s.id.clone(), "import"),
            (true, "skip") => (s.id.clone(), "skip"),
            (true, "overwrite") => (s.id.clone(), "overwrite"),
            (true, "rename") => {
                let new_id = uuid::Uuid::new_v4().to_string();
                stage_id_map.insert(s.id.clone(), new_id.clone());
                (new_id, "rename")
            }
            _ => (s.id.clone(), "skip"),
        };

        match action {
            "skip" => {
                result.stages_skipped += 1;
                if mode == "rename" {
                    stage_id_map.insert(s.id.clone(), s.id.clone());
                }
            }
            "overwrite" => {
                sqlx::query(
                    "INSERT INTO stages (id, goal_id, name, parent_id, path, sort_order, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET
                     name=excluded.name, sort_order=excluded.sort_order",
                )
                .bind(&id)
                .bind(&mapped_goal_id)
                .bind(&s.name)
                .bind(&s.parent_id)
                .bind(&s.path)
                .bind(s.sort_order)
                .bind(&s.created_at)
                .execute(&state.0)
                .await?;
                result.stages_imported += 1;
            }
            _ => {
                sqlx::query(
                    "INSERT INTO stages (id, goal_id, name, parent_id, path, sort_order, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(&mapped_goal_id)
                .bind(&s.name)
                .bind(&s.parent_id)
                .bind(&s.path)
                .bind(s.sort_order)
                .bind(&s.created_at)
                .execute(&state.0)
                .await?;
                result.stages_imported += 1;
                if mode == "rename" && id != s.id {
                    stage_id_map.insert(s.id.clone(), id.clone());
                }
            }
        }
    }

    // 导入 tasks
    for t in data.tasks {
        let mapped_goal_id = if mode == "rename" {
            goal_id_map.get(&t.goal_id).cloned().unwrap_or(t.goal_id.clone())
        } else {
            t.goal_id.clone()
        };
        let mapped_stage_id = if mode == "rename" {
            t.stage_id
                .as_ref()
                .and_then(|sid| stage_id_map.get(sid).cloned())
                .or(t.stage_id.clone())
        } else {
            t.stage_id.clone()
        };

        let exists: bool = sqlx::query_scalar::<_, i64>("SELECT 1 FROM tasks WHERE id = ?")
            .bind(&t.id)
            .fetch_optional(&state.0)
            .await?
            .is_some();

        let (id, action) = match (exists, mode) {
            (false, _) => (t.id.clone(), "import"),
            (true, "skip") => (t.id.clone(), "skip"),
            (true, "overwrite") => (t.id.clone(), "overwrite"),
            (true, "rename") => (uuid::Uuid::new_v4().to_string(), "rename"),
            _ => (t.id.clone(), "skip"),
        };

        match action {
            "skip" => {
                result.tasks_skipped += 1;
            }
            "overwrite" => {
                sqlx::query(
                    "INSERT INTO tasks (id, goal_id, stage_id, parent_id, path, name, plan_date,
                     plan_qty, actual_qty, unit, status, is_manual, source, sort_order, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(id) DO UPDATE SET
                     name=excluded.name, plan_date=excluded.plan_date, plan_qty=excluded.plan_qty,
                     actual_qty=excluded.actual_qty, status=excluded.status",
                )
                .bind(&id)
                .bind(&mapped_goal_id)
                .bind(&mapped_stage_id)
                .bind(&t.parent_id)
                .bind(&t.path)
                .bind(&t.name)
                .bind(&t.plan_date)
                .bind(t.plan_qty)
                .bind(t.actual_qty)
                .bind(&t.unit)
                .bind(&t.status)
                .bind(t.is_manual)
                .bind(&t.source)
                .bind(t.sort_order)
                .bind(&t.created_at)
                .execute(&state.0)
                .await?;
                result.tasks_imported += 1;
            }
            _ => {
                sqlx::query(
                    "INSERT INTO tasks (id, goal_id, stage_id, parent_id, path, name, plan_date,
                     plan_qty, actual_qty, unit, status, is_manual, source, sort_order, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                )
                .bind(&id)
                .bind(&mapped_goal_id)
                .bind(&mapped_stage_id)
                .bind(&t.parent_id)
                .bind(&t.path)
                .bind(&t.name)
                .bind(&t.plan_date)
                .bind(t.plan_qty)
                .bind(t.actual_qty)
                .bind(&t.unit)
                .bind(&t.status)
                .bind(t.is_manual)
                .bind(&t.source)
                .bind(t.sort_order)
                .bind(&t.created_at)
                .execute(&state.0)
                .await?;
                result.tasks_imported += 1;
            }
        }
    }

    // 导入 encouragements（自定义鼓励语，预设不导入）
    for e in data.encouragements {
        if e.category == "preset" {
            // 预设鼓励语跳过（已由迁移初始化）
            continue;
        }
        let exists: bool = sqlx::query_scalar::<_, i64>("SELECT 1 FROM encouragements WHERE id = ?")
            .bind(&e.id)
            .fetch_optional(&state.0)
            .await?
            .is_some();

        let id = if exists && mode == "rename" {
            uuid::Uuid::new_v4().to_string()
        } else if exists && mode == "skip" {
            continue;
        } else {
            e.id.clone()
        };

        sqlx::query(
            "INSERT INTO encouragements (id, text, category, created_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET text=excluded.text",
        )
        .bind(&id)
        .bind(&e.text)
        .bind(&e.category)
        .bind(&e.created_at)
        .execute(&state.0)
        .await?;
        result.encouragements_imported += 1;
    }

    // 导入 settings（upsert）
    for s in data.settings {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?, ?)
             ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        )
        .bind(&s.key)
        .bind(&s.value)
        .execute(&state.0)
        .await?;
        result.settings_imported += 1;
    }

    Ok(result)
}
