use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#[cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[derive(Debug)]
struct TagEntry {
    tag_name: String,
    file_name: String,
    reg_ex: String,
    tag: String,
    context: String,
}

fn walk_project(project_path: &str) -> io::Result<Vec<TagEntry>> {
    let mut tags = Vec::new();

    // Open the file
    let path = Path::new(project_path);
    let file = File::open(path)?;

    // Read the file line by line
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?;  // Handle potential I/O errors
        if !line.starts_with("!") {  // Skip comment lines in the tags file
            let parts: Vec<&str> = line.split('\t').collect();  // Split the line by tabs
            if parts.len() >= 3 {
                let tag_name = parts[0].to_string();
                let file_name = parts[1].to_string();
                let reg_ex = parts[2].to_string();
                let tag = if parts.len()>=4 {parts[3].to_string()} else {"".to_string()};
                let context = if parts.len()>=5 {parts[4].to_string()} else {"".to_string()};
                tags.push(TagEntry {
                    tag_name    :   tag_name,
                    file_name   :   file_name,
                    reg_ex      :   reg_ex,
                    tag         :   tag,
                    context     :   context,
                });
            }
        }
    }
    Ok(tags)
}

// region output
fn emit_project_structure() {}
fn emit_project_data_flow() {}
// endregion

// region requests
#[tauri::command]
fn request_project_structure(project_path: &str) {
    let _ = walk_project(project_path);
}
#[tauri::command]
fn save_project_structure(project_path: &str) {
}
#[tauri::command]
fn del_project_structure(project_path: &str) {
}


#[tauri::command]
fn request_project_data_flow(project_path: &str) {
    let _ = walk_project(project_path);
}
#[tauri::command]
fn save_project_data_flow(project_path: &str) {
}
#[tauri::command]
fn del_project_data_flow(project_path: &str) {
}
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