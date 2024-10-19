use tauri::{api::file, window, Manager, Runtime};

mod tag_entry;

use tag_entry::TagEntry;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
//
// region payloads
#[derive(Clone, serde::Serialize)]
struct ProgressPayload {
    t: String,
    key: String,
    amount: f64,
}

#[derive(Clone, serde::Serialize)]
struct MessagePayload {
    t: String,
    value: String,
}

// endregion

// region output
fn emit_project_structure() {}
fn emit_project_data_flow() {}
// endregion

// region requests
#[tauri::command]
async fn request_project_structure<R: Runtime>(tags_path: String, window: tauri::Window<R>) {
    let emit_process_progress_status = |progress: u8| {
        println!("emitted progress as {}", progress);
        window.emit("progress", progress).unwrap();
    };

    let tags_result = match tag_entry::get_tags_data(tags_path) {
        Ok(res) => res,
        Err(_) => Vec::new(),
    };
    let all_files = tag_entry::get_all_files(&tags_result);
    let data =
        tag_entry::get_all_data(&all_files, &tags_result, emit_process_progress_status).await;
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
