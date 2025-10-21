// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

mod analysis;
mod getdata;
mod getfreddata;
mod portfolio;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            getdata::make_request,
            portfolio::get_portfolio,
            portfolio::set_portfolio,
            getfreddata::get_fred_data,
            analysis::get_analysed_results,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
