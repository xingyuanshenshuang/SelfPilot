mod commands;
mod db;
mod error;
mod services;

use sqlx::sqlite::SqlitePoolOptions;
use tauri::Manager;

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // 获取应用数据目录并创建
            let app_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_dir)?;

            // 构建 SQLite 数据库连接
            let db_path = app_dir.join("selfpilot.db");
            let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

            // 初始化连接池并执行迁移
            let pool = tauri::async_runtime::block_on(async {
                let pool = SqlitePoolOptions::new()
                    .max_connections(5)
                    .connect(&db_url)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                sqlx::migrate!("./migrations")
                    .run(&pool)
                    .await
                    .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

                Ok::<_, Box<dyn std::error::Error>>(pool)
            })?;

            // 将连接池注入 Tauri State
            app.manage(db::DbPool(pool));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 目标相关
            commands::goal::create_goal,
            commands::goal::list_goals,
            commands::goal::get_goal,
            commands::goal::delete_goal,
            commands::goal::auto_split,
            // 任务相关
            commands::task::create_task,
            commands::task::complete_task,
            commands::task::skip_task,
            commands::task::list_today_tasks,
            commands::task::list_overdue_tasks,
            commands::task::list_tasks_by_goal,
            // 进度相关
            commands::progress::get_goal_progress,
            commands::progress::get_all_goals_progress,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
