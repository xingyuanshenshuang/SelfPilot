use tauri::State;
use uuid::Uuid;

use crate::db::models::{AddEncouragementInput, Encouragement, StreakInfo};
use crate::db::DbPool;
use crate::error::{AppError, AppResult};

/// 列出所有鼓励语
#[tauri::command]
pub async fn list_encouragements(state: State<'_, DbPool>) -> AppResult<Vec<Encouragement>> {
    let list: Vec<Encouragement> =
        sqlx::query_as("SELECT * FROM encouragements ORDER BY created_at")
            .fetch_all(&state.0)
            .await?;
    Ok(list)
}

/// 添加自定义鼓励语
#[tauri::command]
pub async fn add_encouragement(
    input: AddEncouragementInput,
    state: State<'_, DbPool>,
) -> AppResult<Encouragement> {
    if input.text.trim().is_empty() {
        return Err(AppError::Param("鼓励语内容不能为空".into()));
    }

    // 等级校验，默认 normal
    let level = match input.level.as_deref().unwrap_or("normal") {
        "normal" | "advanced" | "highlight" | "celebration" => {
            input.level.as_deref().unwrap_or("normal")
        }
        _ => {
            return Err(AppError::Param(
                "等级无效，应为 normal/advanced/highlight/celebration".into(),
            ));
        }
    };

    let id = Uuid::new_v4().to_string();
    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();

    sqlx::query(
        "INSERT INTO encouragements (id, text, category, level, created_at) VALUES (?, ?, 'custom', ?, ?)",
    )
    .bind(&id)
    .bind(&input.text)
    .bind(level)
    .bind(&now)
    .execute(&state.0)
    .await?;

    let item: Encouragement = sqlx::query_as("SELECT * FROM encouragements WHERE id = ?")
        .bind(&id)
        .fetch_one(&state.0)
        .await?;

    Ok(item)
}

/// 删除鼓励语（预设鼓励语不允许删除）
#[tauri::command]
pub async fn delete_encouragement(id: String, state: State<'_, DbPool>) -> AppResult<()> {
    let item: Encouragement = sqlx::query_as("SELECT * FROM encouragements WHERE id = ?")
        .bind(&id)
        .fetch_optional(&state.0)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("鼓励语 {} 不存在", id)))?;

    if item.category == "preset" {
        return Err(AppError::Business("预设鼓励语不允许删除".into()));
    }

    sqlx::query("DELETE FROM encouragements WHERE id = ?")
        .bind(&id)
        .execute(&state.0)
        .await?;

    Ok(())
}

/// 随机抽取一句鼓励语
#[tauri::command]
pub async fn random_encouragement(state: State<'_, DbPool>) -> AppResult<Option<Encouragement>> {
    // SQLite 的 ORDER BY RANDOM() LIMIT 1
    let item: Option<Encouragement> =
        sqlx::query_as("SELECT * FROM encouragements ORDER BY RANDOM() LIMIT 1")
            .fetch_optional(&state.0)
            .await?;
    Ok(item)
}

/// 根据当前连续天数智能选择鼓励语等级
///
/// PRD §4.2 模块七 & 分阶段计划 Sprint 5：鼓励语个性化规则
/// - 连续 1 天 → normal 普通
/// - 连续 3 天 → advanced 进阶
/// - 连续 7 天 → highlight 高亮
/// - 该等级无鼓励语时降级到 normal
#[tauri::command]
pub async fn random_encouragement_by_streak(
    streak: i64,
    state: State<'_, DbPool>,
) -> AppResult<Option<Encouragement>> {
    // 根据连续天数确定等级
    let level = if streak >= 7 {
        "highlight"
    } else if streak >= 3 {
        "advanced"
    } else {
        "normal"
    };

    // 优先从对应等级抽取
    let item: Option<Encouragement> =
        sqlx::query_as("SELECT * FROM encouragements WHERE level = ? ORDER BY RANDOM() LIMIT 1")
            .bind(level)
            .fetch_optional(&state.0)
            .await?;

    // 对应等级无鼓励语时降级到 normal
    if item.is_none() && level != "normal" {
        let fallback: Option<Encouragement> =
            sqlx::query_as("SELECT * FROM encouragements WHERE level = 'normal' ORDER BY RANDOM() LIMIT 1")
                .fetch_optional(&state.0)
                .await?;
        return Ok(fallback);
    }

    // 仍无则从全部抽取
    if item.is_none() {
        let any: Option<Encouragement> =
            sqlx::query_as("SELECT * FROM encouragements ORDER BY RANDOM() LIMIT 1")
                .fetch_optional(&state.0)
                .await?;
        return Ok(any);
    }

    Ok(item)
}

/// 抽取庆祝鼓励语（全部目标完成时使用）
#[tauri::command]
pub async fn random_celebration_encouragement(
    state: State<'_, DbPool>,
) -> AppResult<Option<Encouragement>> {
    let item: Option<Encouragement> =
        sqlx::query_as("SELECT * FROM encouragements WHERE level = 'celebration' ORDER BY RANDOM() LIMIT 1")
            .fetch_optional(&state.0)
            .await?;

    // 无庆祝鼓励语时降级到 highlight
    if item.is_none() {
        let fallback: Option<Encouragement> =
            sqlx::query_as("SELECT * FROM encouragements WHERE level = 'highlight' ORDER BY RANDOM() LIMIT 1")
                .fetch_optional(&state.0)
                .await?;
        return Ok(fallback);
    }

    Ok(item)
}

/// 获取连续完成天数统计
///
/// PRD §4.2 模块七 & 分阶段计划开发注意事项：
/// - "每天至少完成一个任务"才计入连续
/// - 当天无任务则"不中断也不计入"
/// - 当天有任务但未完成则中断
///
/// 实现逻辑（从今天往前推）：
/// 1. 若今日有任务但未完成任何 → current_streak = 0
/// 2. 若今日无任务 → 从昨日开始往前统计
/// 3. 若今日已完成 → 从今日开始往前统计
/// 4. 遇到"有任务但未完成"的日期 → 中断
/// 5. 遇到"无任务"的日期 → 跳过（不中断）
#[tauri::command]
pub async fn get_streak(state: State<'_, DbPool>) -> AppResult<StreakInfo> {
    let today = chrono::Local::now().date_naive();

    // 查询所有有任务的日期及其完成情况
    // day_has_task: 当天是否有任务
    // day_completed: 当天是否至少完成一个任务
    let rows: Vec<(String, i64, i64)> = sqlx::query_as(
        "SELECT plan_date,
                COUNT(*) as task_count,
                SUM(CASE WHEN status = 'done' THEN 1 ELSE 0 END) as done_count
         FROM tasks
         WHERE plan_date IS NOT NULL AND status != 'skipped'
         GROUP BY plan_date",
    )
    .fetch_all(&state.0)
    .await?;

    use std::collections::HashMap;
    let mut day_map: HashMap<chrono::NaiveDate, (bool, bool)> = HashMap::new();
    for (date_str, task_count, done_count) in rows {
        if let Ok(d) = chrono::NaiveDate::parse_from_str(&date_str, "%Y-%m-%d") {
            let has_task = task_count > 0;
            let completed = done_count > 0;
            day_map.insert(d, (has_task, completed));
        }
    }

    // 计算当前连续天数
    let mut current_streak: i64 = 0;
    // cursor 初始化为昨日（所有分支都会从昨日开始往前推）
    let mut cursor = today - chrono::Duration::days(1);

    // 今日特殊处理：若今日无任务，从昨日开始；若今日有任务但未完成，中断
    let today_entry = day_map.get(&today);
    let completed_today = today_entry.map(|(_, c)| *c).unwrap_or(false);

    match today_entry {
        None => {
            // 今日无任务，从昨日开始（cursor 已是昨日）
        }
        Some((true, false)) => {
            // 今日有任务但未完成 → 中断
            current_streak = 0;
            // cursor 已是昨日，但 today_unfinished 会跳过循环
        }
        Some((true, true)) => {
            // 今日已完成
            current_streak = 1;
        }
        _ => {}
    }

    // 如果今日有任务但未完成，current_streak 已为 0，跳过循环
    let today_unfinished = matches!(today_entry, Some((true, false)));
    if !today_unfinished {
        // 往前推算连续天数
        loop {
            let entry = day_map.get(&cursor);
            match entry {
                None => {
                    // 无任务日，跳过（不中断）
                    cursor = cursor - chrono::Duration::days(1);
                }
                Some((true, true)) => {
                    // 有任务且完成 → 连续+1
                    current_streak += 1;
                    cursor = cursor - chrono::Duration::days(1);
                }
                Some((true, false)) => {
                    // 有任务但未完成 → 中断
                    break;
                }
                _ => {
                    cursor = cursor - chrono::Duration::days(1);
                }
            }

            // 防止无限循环（最多回溯 10 年）
            if (today - cursor).num_days() > 3650 {
                break;
            }
        }
    }

    // 计算 longest_streak：遍历所有有任务的日期
    let mut longest_streak: i64 = 0;
    let mut temp_streak: i64 = 0;
    let mut last_date: Option<chrono::NaiveDate> = None;

    let mut sorted_dates: Vec<chrono::NaiveDate> = day_map.keys().copied().collect();
    sorted_dates.sort();

    for d in &sorted_dates {
        let (has_task, completed) = day_map[d];
        if !has_task {
            continue;
        }
        if completed {
            // 检查与上一个日期的连续性（允许中间有无任务日）
            let should_continue = match last_date {
                None => true,
                Some(last) => {
                    // 从 last 到 d 之间，所有有任务的日期都应已完成
                    // 简化处理：只要日期递增且中间没有"有任务但未完成"的日期
                    let mut check = last + chrono::Duration::days(1);
                    let mut ok = true;
                    while check < *d {
                        if let Some((ht, comp)) = day_map.get(&check) {
                            if *ht && !*comp {
                                ok = false;
                                break;
                            }
                        }
                        check = check + chrono::Duration::days(1);
                    }
                    ok
                }
            };
            if should_continue {
                temp_streak += 1;
            } else {
                temp_streak = 1;
            }
            last_date = Some(*d);
            if temp_streak > longest_streak {
                longest_streak = temp_streak;
            }
        } else {
            // 有任务但未完成 → 中断
            temp_streak = 0;
            last_date = Some(*d);
        }
    }

    // 确保 longest_streak 至少等于 current_streak
    if current_streak > longest_streak {
        longest_streak = current_streak;
    }

    Ok(StreakInfo {
        current_streak,
        longest_streak,
        completed_today,
    })
}
