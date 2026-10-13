//! Hubbub Desktop — Tauri application shell.
//!
//! This is a thin shell that:
//! - Sets up the Tauri window and IPC
//! - Delegates all business logic to `hubbub-app` crate
//! - Contains no business logic itself

pub mod commands;
pub mod events;

use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;

use hubbub_app::{AppConfig, AppService};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let app_handle = app.handle();
            let app_dir = app_handle
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("./hubbub_data"));

            let config = AppConfig {
                workspace_dir: app_dir,
                ..Default::default()
            };

            let service = tauri::async_runtime::block_on(async { AppService::init(config).await })
                .map_err(|e| {
                    Box::new(std::io::Error::other(e.to_string())) as Box<dyn std::error::Error>
                })?;

            app.manage(Arc::new(service));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::get_version,
            commands::list_conversations,
            commands::create_conversation,
            commands::get_conversation,
            commands::list_messages,
            commands::list_agents,
            commands::set_agent_model,
            commands::upsert_agent,
            commands::delete_agent,
            commands::set_provider_key,
            commands::has_provider_key,
            commands::delete_provider_key,
            commands::send_message,
            commands::cancel_run,
            commands::list_reports,
            commands::read_report,
            commands::write_report,
            commands::list_documents,
            commands::search_documents,
            commands::list_audit_logs,
            commands::get_model_usage_stats,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            eprintln!("Failed to start Hubbub: {e}");
            std::process::exit(1);
        });
}
