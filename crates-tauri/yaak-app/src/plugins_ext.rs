//! Tauri-specific plugin management code.
//!
//! This module contains all Tauri integration for the plugin system:
//! - Plugin initialization and lifecycle management
//! - Tauri commands for plugin search/install/uninstall
//! - Plugin update checking

use crate::PluginContextExt;
use crate::error::Result;
use crate::models_ext::QueryManagerExt;
use log::{error, info};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::path::BaseDirectory;
use tauri::plugin::{Builder, TauriPlugin};
use tauri::{
    AppHandle, Emitter, Manager, RunEvent, Runtime, State, WebviewWindow, command,
    is_dev,
};
use yaak_models::models::Plugin;
use yaak_models::util::UpdateSource;
use yaak_plugins::api::{
    PluginSearchResponse,
    search_plugins,
};
use yaak_plugins::events::{Color, Icon, PluginContext, ShowToastRequest};
use yaak_plugins::install::{delete_and_uninstall, download_and_install};
use yaak_plugins::manager::PluginManager;
use yaak_tauri_utils::api_client::yaak_api_client;

static EXITING: AtomicBool = AtomicBool::new(false);

// ============================================================================
// Tauri Commands
// ============================================================================

#[command]
pub async fn cmd_plugins_search<R: Runtime>(
    app_handle: AppHandle<R>,
    query: &str,
) -> Result<PluginSearchResponse> {
    let http_client = yaak_api_client(&app_handle)?;
    Ok(search_plugins(&http_client, query).await?)
}

#[command]
pub async fn cmd_plugins_install<R: Runtime>(
    window: WebviewWindow<R>,
    name: &str,
    version: Option<String>,
) -> Result<()> {
    let plugin_manager = Arc::new((*window.state::<PluginManager>()).clone());
    let http_client = yaak_api_client(window.app_handle())?;
    let query_manager = window.state::<yaak_models::query_manager::QueryManager>();
    let plugin_context = window.plugin_context();
    download_and_install(
        plugin_manager,
        &query_manager,
        &http_client,
        &plugin_context,
        name,
        version,
    )
    .await?;
    Ok(())
}

#[command]
pub async fn cmd_plugins_uninstall<R: Runtime>(
    plugin_id: &str,
    window: WebviewWindow<R>,
) -> Result<Plugin> {
    let plugin_manager = Arc::new((*window.state::<PluginManager>()).clone());
    let query_manager = window.state::<yaak_models::query_manager::QueryManager>();
    let plugin_context = window.plugin_context();
    Ok(delete_and_uninstall(plugin_manager, &query_manager, &plugin_context, plugin_id).await?)
}

// ============================================================================
// Tauri Plugin Initialization
// ============================================================================

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("yaak-plugins")
        .setup(|app_handle, _| {
            // Resolve paths for plugin manager
            let vendored_plugin_dir = app_handle
                .path()
                .resolve("vendored/plugins", BaseDirectory::Resource)
                .expect("failed to resolve plugin directory resource");

            let installed_plugin_dir = app_handle
                .path()
                .app_data_dir()
                .expect("failed to get app data dir")
                .join("installed-plugins");

            #[cfg(target_os = "windows")]
            let node_bin_name = "yaaknode.exe";
            #[cfg(not(target_os = "windows"))]
            let node_bin_name = "yaaknode";

            let node_bin_path = app_handle
                .path()
                .resolve(format!("vendored/node/{}", node_bin_name), BaseDirectory::Resource)
                .expect("failed to resolve yaaknode binary");

            let plugin_runtime_main = app_handle
                .path()
                .resolve("vendored/plugin-runtime", BaseDirectory::Resource)
                .expect("failed to resolve plugin runtime")
                .join("index.cjs");

            let dev_mode = is_dev();

            // Create plugin manager asynchronously
            let app_handle_clone = app_handle.clone();
            tauri::async_runtime::block_on(async move {
                let manager = PluginManager::new(
                    vendored_plugin_dir,
                    installed_plugin_dir,
                    node_bin_path,
                    plugin_runtime_main,
                    dev_mode,
                )
                .await;

                // Initialize all plugins after manager is created
                let bundled_dirs = manager
                    .list_bundled_plugin_dirs()
                    .await
                    .expect("Failed to list bundled plugins");

                // Ensure all bundled plugins make it into the database
                let db = app_handle_clone.db();
                for dir in &bundled_dirs {
                    if db.get_plugin_by_directory(dir).is_none() {
                        db.upsert_plugin(
                            &Plugin {
                                directory: dir.clone(),
                                enabled: true,
                                url: None,
                                ..Default::default()
                            },
                            &UpdateSource::Background,
                        )
                        .expect("Failed to upsert bundled plugin");
                    }
                }

                // Get all plugins from database and initialize
                let plugins = db.list_plugins().expect("Failed to list plugins from database");
                drop(db); // Explicitly drop the connection before await

                let errors =
                    manager.initialize_all_plugins(plugins, &PluginContext::new_empty()).await;

                // Show toast for any failed plugins
                for (plugin_dir, error_msg) in errors {
                    let plugin_name = plugin_dir.split('/').last().unwrap_or(&plugin_dir);
                    let toast = ShowToastRequest {
                        message: format!("Failed to start plugin '{}': {}", plugin_name, error_msg),
                        color: Some(Color::Danger),
                        icon: Some(Icon::AlertTriangle),
                        timeout: Some(10000),
                    };
                    if let Err(emit_err) = app_handle_clone.emit("show_toast", toast) {
                        error!("Failed to emit toast for plugin error: {emit_err:?}");
                    }
                }

                app_handle_clone.manage(manager);
            });

            Ok(())
        })
        .on_event(|app, e| match e {
            RunEvent::ExitRequested { api, .. } => {
                if EXITING.swap(true, Ordering::SeqCst) {
                    return; // Only exit once to prevent infinite recursion
                }
                api.prevent_exit();
                tauri::async_runtime::block_on(async move {
                    info!("Exiting plugin runtime due to app exit");
                    let manager: State<PluginManager> = app.state();
                    manager.terminate().await;
                    app.exit(0);
                });
            }
            _ => {}
        })
        .build()
}
