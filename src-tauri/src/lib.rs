pub mod commands;
pub mod core;
pub mod features;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    let builder = tauri::Builder::default().setup(|app| {
        if cfg!(debug_assertions) {
            app.handle().plugin(
                tauri_plugin_log::Builder::default()
                    .level(log::LevelFilter::Info)
                    .build(),
            )?;
        }

        let handle = app.handle().clone();
        tokio::spawn(async move {
            let db_pool = core::db::init_db(&handle)
                .await
                .expect("Failed to initialize database");
            core::db::run_migrations(&db_pool)
                .await
                .expect("Database migrations failed");
            handle.manage(db_pool);
        });

        Ok(())
    });

    builder
        .invoke_handler(tauri::generate_handler![
            commands::project_commands::create_project,
            commands::project_commands::get_project,
            commands::project_commands::list_projects,
            commands::project_commands::update_project,
            commands::project_commands::delete_project,
            commands::task_commands::create_task,
            commands::task_commands::get_task,
            commands::task_commands::list_tasks,
            commands::task_commands::update_task,
            commands::task_commands::delete_task,
            commands::task_commands::list_inbox_tasks,
            commands::checkpoint_commands::create_checkpoint,
            commands::checkpoint_commands::get_checkpoint,
            commands::checkpoint_commands::list_checkpoints_for_task,
            commands::checkpoint_commands::update_checkpoint,
            commands::checkpoint_commands::delete_checkpoint,
            commands::activity_commands::create_activity,
            commands::activity_commands::get_activity,
            commands::activity_commands::list_activities,
            commands::activity_commands::update_activity,
            commands::activity_commands::delete_activity,
            commands::tag_commands::create_tag,
            commands::tag_commands::get_tag,
            commands::tag_commands::list_tags,
            commands::tag_commands::update_tag,
            commands::tag_commands::delete_tag,
            commands::link_commands::link_task_to_activity,
            commands::link_commands::unlink_task_from_activity,
            commands::link_commands::add_tag_to_project,
            commands::link_commands::remove_tag_from_project,
            commands::link_commands::add_tag_to_task,
            commands::link_commands::remove_tag_from_task,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
