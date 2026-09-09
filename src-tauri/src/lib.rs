mod assets;
#[cfg(windows)]
mod explorer;

use tauri::{Emitter, Manager};

#[tauri::command]
fn get_selected_paths() -> Vec<String> {
    #[cfg(windows)]
    {
        explorer::get_explorer_selected()
    }

    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

#[tauri::command]
fn inspect_path(path: String) -> assets::InspectResult {
    assets::inspect(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .app_name("moew")
                .build(),
        )
        .setup(|app| {
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{Code, Modifiers, ShortcutState};
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["ctrl+alt+space"])?
                        .with_handler(|app, shortcut, event| {
                            if event.state == ShortcutState::Pressed
                                && shortcut
                                    .matches(Modifiers::CONTROL | Modifiers::ALT, Code::Space)
                            {
                                let paths = get_selected_paths();
                                if let Some(first) = paths.first() {
                                    let result = assets::inspect(first);
                                    if result.is_asset {
                                        if let Some(window) = app.get_webview_window("main") {
                                            let _ = window.show();
                                            let _ = window.set_focus();
                                        }

                                        let _ = app.emit("preview", result);
                                    }
                                }
                            }
                        })
                        .build(),
                )?;
            }

            #[cfg(not(debug_assertions))]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.hide();
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_selected_paths, inspect_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
