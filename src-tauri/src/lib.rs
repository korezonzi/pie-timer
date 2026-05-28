mod timer;

use tauri::Manager;
use timer::TimerEngine;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let engine = TimerEngine::new();

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
        ])
        .setup(|app| {
            let engine = app.state::<TimerEngine>();
            engine.start_tick_loop(app.handle().clone());
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
