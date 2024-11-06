use tauri::Runtime;

mod evaluate_imports;
mod tag_entry;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// region output
// fn emit_project_structure() {}
// fn emit_project_data_flow() {}
// endregion

// region requests
#[tauri::command]
async fn request_project_structure<R: Runtime>(
    project_path: String,
    tags_path: String,
    window: tauri::Window<R>,
) {
    create_project_structure(project_path, tags_path, window).await;
}
#[tauri::command]
fn save_project_structure(_tags_path: &str) {}
#[tauri::command]
fn del_project_structure(_tags_path: &str) {}

#[tauri::command]
fn request_project_data_flow(_tags_path: &str) {}
#[tauri::command]
fn save_project_data_flow(_tags_path: &str) {}
#[tauri::command]
fn del_project_data_flow(_tags_path: &str) {}
// endregion interface

/// create the project structure as :
///
/// for each file fi in F
///
///     fi -> ([S],[T],{t->(fj,tj)})
///
/// where **F**:`files`, **S**:`scopes`, **T**:`tags`
///
/// sorting is done by `F(name),S(starting_point),T(definition_order)` for easy indexing
async fn create_project_structure<R: Runtime>(
    project_path: String,
    tags_path: String,
    window: tauri::Window<R>,
) {
    let emit_process_progress_status = |progress: u8| {
        window.emit("progress", progress).unwrap();
    };

    let tags_result = match tag_entry::get_tags_data(tags_path) {
        Ok(res) => res,
        Err(_) => Vec::new(),
    };
    // F
    let all_files = tag_entry::get_all_files(&tags_result);
    let hard_data =
        tag_entry::get_all_hard_data(&all_files, &tags_result, emit_process_progress_status).await;
    let _imported_tags =
        evaluate_imports::evaluate_all_available_tags(&project_path, &all_files, hard_data);
}

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
