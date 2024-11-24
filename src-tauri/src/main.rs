use serde_json::json;
use std::str;
use tauri::Runtime;

mod data;
mod evaluate_imports;
mod intense_evaluation;
mod tag_entry;

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// region requests

/// create the project structure as :
///
/// for each file fi in F
///
///     fi -> ([S],[T],{t->(fj,tj)})
///
/// where **F**:`files`, **S**:`scopes`, **T**:`tags`
///
/// sorting is done by `F(name),S(starting_point),T(definition_order)` for easy indexing
#[tauri::command]
async fn request_project_structure<R: Runtime>(
    project_path: String,
    tags_path: String,
    window: tauri::Window<R>,
) {
    let emit_process_progress_status = |key: &str, progress: u8| {
        window.emit("progress", json!((key, progress))).unwrap();
    };

    let tags_result = match tag_entry::get_tags_data(tags_path) {
        Ok(res) => res,
        Err(_) => Vec::new(),
    };
    // F
    let all_files = tag_entry::get_all_files(&tags_result)
        .into_iter()
        .collect::<Vec<&String>>();
    let hard_data =
        tag_entry::get_all_hard_data(&all_files, &tags_result, emit_process_progress_status).await;
    let (raw_imports, all_tags, children_tags) = evaluate_imports::evaluate_all_hard_data(
        &project_path,
        &all_files,
        hard_data,
        emit_process_progress_status,
    );

    let (imports_json, tags_json, children_json) =
        evaluate_imports::jsonify_evaluated_data(&raw_imports, &all_tags, &children_tags);

    let project_hierarchy = json!([all_files, imports_json, tags_json, children_json]);

    let structure_emit_result = window.emit("project_structure", project_hierarchy);
    match structure_emit_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!(
                "couldn't emit the structure data properly due to \n\terror : {}",
                e
            );
        }
    }

    println!("\n\n------ intense extract ------\n\n");
    intense_evaluation::evaluate(&project_path, &all_files);
    // for f_p in all_files {
    //     language_file_intense_extract(f_p);
    // }
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
