mod sound;
mod timer;

use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    Emitter, Manager,
};
use timer::TimerEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let sound = std::sync::Arc::new(sound::SoundPlayer::new());
    let engine = TimerEngine::new(sound);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .manage(engine)
        .invoke_handler(tauri::generate_handler![
            timer::get_timer_state,
            timer::start_timer,
            timer::pause_timer,
            timer::toggle_timer,
            timer::reset_timer,
            timer::skip_session,
            timer::set_preset,
            timer::set_sessions_goal,
            sound::set_muted,
        ])
        .setup(|app| {
            // Start timer tick loop
            let engine = app.state::<TimerEngine>();
            engine.start_tick_loop(app.handle().clone());

            // Autostart plugin
            #[cfg(desktop)]
            {
                use tauri_plugin_autostart::MacosLauncher;
                app.handle().plugin(tauri_plugin_autostart::init(
                    MacosLauncher::LaunchAgent,
                    None,
                ))?;
            }

            // System tray
            let toggle_item =
                MenuItem::with_id(app, "toggle", "Start/Pause", true, None::<&str>)?;
            let reset_item = MenuItem::with_id(app, "reset", "Reset", true, None::<&str>)?;
            let skip_item = MenuItem::with_id(app, "skip", "Skip", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &toggle_item,
                    &reset_item,
                    &skip_item,
                    &separator,
                    &show_item,
                    &quit_item,
                ],
            )?;

            let app_handle = app.handle().clone();
            TrayIconBuilder::new()
                .icon(tauri::image::Image::from_bytes(include_bytes!(
                    "../icons/tray-icon.png"
                ))?)
                .icon_as_template(true)
                .menu(&menu)
                .tooltip("Pie Timer")
                .on_menu_event(move |_app, event| {
                    let engine = app_handle.state::<TimerEngine>();
                    match event.id().as_ref() {
                        "toggle" => {
                            let state = engine.do_toggle();
                            let _ = app_handle.emit("timer:tick", &state);
                        }
                        "reset" => {
                            let state = engine.do_reset();
                            let _ = app_handle.emit("timer:tick", &state);
                        }
                        "skip" => {
                            let state = engine.do_skip();
                            let _ = app_handle.emit("timer:tick", &state);
                        }
                        "show" => {
                            if let Some(window) = _app.get_webview_window("main") {
                                let _ = window.unminimize();
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        "quit" => {
                            _app.exit(0);
                        }
                        _ => {}
                    }
                })
                .build(app)?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
