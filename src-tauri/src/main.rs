use serde_json::json;
use serde::Deserialize;
use std::{path::PathBuf, str};
use tauri::Runtime;
use tokio::time::{sleep, Duration};

mod data;
mod project_data;
mod evaluate_imports;
mod intense_evaluation;
mod tag_entry;
mod use_llama;
mod io_operations;

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
    // let emit_process_progress_status = |key: &str, progress: u8| {
    //     window.emit("progress", json!((key, progress))).unwrap();
    // };

    project_data::clear_project_data();

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
#[tauri::command]
async fn read_file_content_by_index(index: usize) -> Result<String, String> {
    let project_data = project_data::get_project_data().ok_or("Project data not initialized")?;
    let file_path = project_data
        .all_files
        .get(index)
        .ok_or("File index out of bounds")?;
    io_operations::read_text_from_file(file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}
// endregion interface

// region AI commands
#[tauri::command]
async fn generate_class<R: Runtime>(
    query: String,
    file_paths: Vec<String>,
    app: tauri::AppHandle<R>,
    window: tauri::Window<R>,
) -> Result<(), String> {
    Ok(())
}


#[derive(Deserialize)]
struct QueryPayload {
    query: String,
    context_files: Vec<usize>,
}

#[tauri::command]
async fn process_query_with_files<R: Runtime>(
    payload: String,
    window: tauri::Window<R>
) -> Result<String, String> {
    println!("Received payload: {}", payload);

    let parsed: QueryPayload = serde_json::from_str(&payload)
        .map_err(|e| format!("Failed to parse payload: {}", e))?;


    let context_files: Vec<usize> = parsed.context_files;

    let project_data = project_data::get_project_data().ok_or("Project data not initialized")?;

    let file_paths: Vec<String> = context_files
    .iter()
    .filter_map(|&i| project_data.all_files.get(i).cloned())
    .collect();

    println!("query: {}", parsed.query);
    println!("context_files_indexes: {:?}", context_files);
    println!("context files: {:?}", file_paths);

    let file_refs: Vec<&str> = file_paths.iter().map(String::as_str).collect();

    let res = use_llama::query_ollama(&parsed.query, &file_refs).await
        .map_err(|e| format!("Error querying Ollama: {}", e));

    if let Ok(ref files) = res {
        for file in files {
            let mut abs_path = PathBuf::from(&file.filePath);
            if abs_path.is_relative() {
                let project_root = PathBuf::from(&project_data.project_path);
                abs_path = project_root.join(&abs_path);
            }
            io_operations::write_text_to_file(
                abs_path,
                &file.fileContent,
            ).await.map_err(|e| format!("Failed to write file: {}", e))?;
        }
    };

    project_data::clear_project_data();

    sleep(Duration::from_millis(1000)).await;

    request_project_structure(
        project_data.project_path.clone(),
        "tags".to_string(),
        window,
    ).await;

    // Simulated logic:
    Ok(format!(
        "Result: {:#?}",
        res
    ))
}
// endregion AI commands


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
            submit_query,
            read_file_content_by_index,
            process_query_with_files
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
