use tauri::api::file;

mod tag_entry;

use tag_entry::TagEntry;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// region output
fn emit_project_structure() {}
fn emit_project_data_flow() {}
// endregion

// region requests
#[tauri::command]
fn request_project_structure(tags_path: String) {
    let tags_result = match tag_entry::get_tags_data(tags_path) {
        Ok(res) => res,
        Err(_) => Vec::new(),
    };
    // let found_files: HashSet<String> = tag_entry::get_all_files(&tags_result);

    let mut all_imports = tag_entry::get_all_imports(&tags_result);
}
#[tauri::command]
fn save_project_structure(tags_path: &str) {}
#[tauri::command]
fn del_project_structure(tags_path: &str) {}

#[tauri::command]
fn request_project_data_flow(tags_path: &str) {}
#[tauri::command]
fn save_project_data_flow(tags_path: &str) {}
#[tauri::command]
fn del_project_data_flow(tags_path: &str) {}
// endregion interface

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            request_project_structure,
            save_project_structure,
            del_project_structure,
            request_project_data_flow,
            save_project_data_flow,
            del_project_data_flow
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
