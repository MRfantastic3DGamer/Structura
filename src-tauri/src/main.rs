use serde_json::json;
use std::str;
use tauri::Runtime;

mod data;
mod project_data;
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

    let project_data = match project_data::get_project_data() {
        Some(data) => data,
        None => {
            let new_data = project_data::create_project_data(
                project_path.clone(),
                tags_path.clone(),
            ).await;
            project_data::set_project_data(new_data.clone());
            new_data
        }
    };

    // Convert data to JSON format
    let (imports_json, tags_json, children_json) = evaluate_imports::jsonify_evaluated_data(
        &project_data.raw_imports,
        &project_data.all_tags,
        &project_data.children_tags,
    );

    let project_hierarchy = json!([
        project_data.all_files,
        imports_json,
        tags_json,
        children_json
    ]);

    // Emit project structure
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

    // Emit intense data
    let intense_data_json = json!({
        "custom_classes": project_data.custom_classes,
        "accessible_scopes": project_data.accessible_scopes,
        "scoped_connectable_s": project_data.scoped_connectables,
    });

    // Emit to frontend
    let intense_emit_result = window.emit("intense_data", intense_data_json);

    match intense_emit_result {
        Ok(_) => {}
        Err(e) => {
            eprintln!("couldn't emit the intense data properly due to \n\terror : {}", e);
        }
    }
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

#[tauri::command]
async fn submit_query<R: Runtime>(
    query: String,
    numbers: Vec<i32>,
    window: tauri::Window<R>,
) -> Result<(), String> {
    // Process the query and numbers here
    println!("Received query: {}", query);
    println!("Received files: {:?}", numbers);

    // Example: Calculate the sum of numbers
    let sum: i32 = numbers.iter().sum();

    // Prepare response data
    let response = json!({
        "original_query": query,
        "numbers": numbers,
        "sum": sum,
        "message": "Query processed successfully"
    });

    // Emit the response back to the frontend
    window.emit("query_response", response)
        .map_err(|e| format!("Failed to emit query response: {}", e))?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            request_project_structure,
            save_project_structure,
            del_project_structure,
            request_project_data_flow,
            save_project_data_flow,
            del_project_data_flow,
            submit_query
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
