use chrono::NaiveDate;
use uuid::Uuid;

use crate::db::models::{Goal, Task};
use crate::error::{AppError, AppResult};

/// 自动拆解目标为每日任务
///
/// PRD §4.2 模块二：
/// - 按截止日期剩余天数（不含今天，且 ≥ 1）将总量平均分配到每天
/// - 余数分配到前几天
/// - 每个任务记录"计划数量"和"单位"
/// - 任务从明天开始，到截止日为止
pub fn split_goal_into_tasks(goal: &Goal, today: NaiveDate) -> AppResult<Vec<Task>> {
    let deadline_str = goal.deadline.as_ref().ok_or_else(|| {
        AppError::Param("目标未设置截止日期，无法拆解".into())
    })?;

    let deadline = NaiveDate::parse_from_str(deadline_str, "%Y-%m-%d")
        .map_err(|e| AppError::Param(format!("截止日期格式错误: {}", e)))?;

    let remaining_days = (deadline - today).num_days();
    if remaining_days < 1 {
        return Err(AppError::Business(format!(
            "截止日期剩余天数不足（{}天），至少需要留出1天",
            remaining_days
        )));
    }

    let total = goal.total_qty;
    if total <= 0.0 {
        return Err(AppError::Business("目标总量必须大于0才能自动拆解".into()));
    }

    // 平均分配：base = floor(total / days)，余数分到前几天
    let base = (total / remaining_days as f64).floor();
    let remainder = (total - base * remaining_days as f64).round() as i64;

    let now = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    let mut tasks = Vec::new();
    let mut day_index = 0;

    for i in 0..remaining_days {
        let plan_qty = if i < remainder { base + 1.0 } else { base };

        // 跳过 plan_qty = 0 的任务（总量小于天数时）
        if plan_qty <= 0.0 {
            continue;
        }

        day_index += 1;
        let task_date = today + chrono::Duration::days(i + 1); // 从明天开始
        let task_id = Uuid::new_v4().to_string();
        let path = format!("/{}/{}", goal.id, task_id);

        tasks.push(Task {
            id: task_id,
            goal_id: goal.id.clone(),
            stage_id: None,
            parent_id: Some(goal.id.clone()),
            path,
            name: format!("{} - 第{}天", goal.name, day_index),
            plan_date: Some(task_date.format("%Y-%m-%d").to_string()),
            plan_qty,
            actual_qty: 0.0,
            unit: goal.unit.clone(),
            status: "pending".to_string(),
            is_manual: 0,
            source: "auto".to_string(),
            sort_order: i,
            created_at: now.clone(),
        });
    }

    Ok(tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_goal(id: &str, name: &str, deadline: &str, total_qty: f64, unit: &str) -> Goal {
        Goal {
            id: id.to_string(),
            name: name.to_string(),
            deadline: Some(deadline.to_string()),
            total_qty,
            unit: unit.to_string(),
            created_at: "2026-06-24T00:00:00".to_string(),
        }
    }

    #[test]
    fn test_split_even() {
        // 10 个任务，5 天，每天 2 个
        let goal = make_goal("g1", "背单词", "2026-06-29", 10.0, "个");
        let today = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
        let tasks = split_goal_into_tasks(&goal, today).unwrap();
        assert_eq!(tasks.len(), 5);
        for t in &tasks {
            assert_eq!(t.plan_qty, 2.0);
        }
        // 日期从 6/25 到 6/29
        assert_eq!(tasks[0].plan_date, Some("2026-06-25".to_string()));
        assert_eq!(tasks[4].plan_date, Some("2026-06-29".to_string()));
    }

    #[test]
    fn test_split_with_remainder() {
        // 10 个任务，3 天：4 + 3 + 3
        let goal = make_goal("g2", "看书", "2026-06-27", 10.0, "页");
        let today = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
        let tasks = split_goal_into_tasks(&goal, today).unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].plan_qty, 4.0); // 余数 1 分到第1天
        assert_eq!(tasks[1].plan_qty, 3.0);
        assert_eq!(tasks[2].plan_qty, 3.0);
    }

    #[test]
    fn test_split_zero_days() {
        let goal = make_goal("g3", "过期", "2026-06-24", 10.0, "个");
        let today = NaiveDate::from_ymd_opt(2026, 6, 24).unwrap();
        let result = split_goal_into_tasks(&goal, today);
        assert!(result.is_err());
    }
}
