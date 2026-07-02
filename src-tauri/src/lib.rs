use tauri::menu::{Menu, MenuItem, Submenu};
use tauri::{Emitter, Manager};

#[cfg(debug_assertions)]
fn with_dev_plugins(
  builder: tauri::Builder<tauri::Wry>,
) -> tauri::Builder<tauri::Wry> {
  builder.plugin(
    tauri_plugin_mcp_bridge::Builder::new()
      .bind_address("127.0.0.1")
      .build(),
  )
}

#[cfg(not(debug_assertions))]
fn with_dev_plugins(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
  builder
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  with_dev_plugins(
    tauri::Builder::default()
      .plugin(tauri_plugin_fs::init())
      .plugin(tauri_plugin_dialog::init())
      .plugin(tauri_plugin_shell::init())
      .plugin(tauri_plugin_process::init()),
  )
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      let save_item = MenuItem::with_id(app, "file-save", "Save", true, None::<&str>)?;
      let load_item = MenuItem::with_id(app, "file-load", "Load", true, None::<&str>)?;
      let export_item = MenuItem::with_id(app, "file-export", "Export Save", true, None::<&str>)?;
      let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
      let file_menu = Submenu::with_items(
        app,
        "File",
        true,
        &[&save_item, &load_item, &export_item, &quit_item],
      )?;

      let manage_mods = MenuItem::with_id(app, "mods-manage", "Manage Mods", true, None::<&str>)?;
      let open_mods = MenuItem::with_id(app, "mods-open-folder", "Open Mods Folder", true, None::<&str>)?;
      let mods_menu = Submenu::with_items(app, "Mods", true, &[&manage_mods, &open_mods])?;

      let menu = Menu::with_items(app, &[&file_menu, &mods_menu])?;
      app.set_menu(menu)?;

      if let Some(window) = app.get_webview_window("main") {
        let window_for_close = window.clone();
        window.on_window_event(move |event| {
          if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            let _ = window_for_close.emit("close-requested", ());
            api.prevent_close();
          }
        });
      }

      Ok(())
    })
    .on_menu_event(|app, event| {
      let _ = app.emit("menu-action", event.id().as_ref());
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
