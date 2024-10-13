use std::path::Path;

pub fn get_imported_files(file_path: &str) -> Result<Vec<String>, String> {
    // Determine the file extension
    let path = Path::new(file_path);
    let extension = match path.extension() {
        Some(ext) => ext.to_str().unwrap_or(""),
        None => return Err("Could not determine file extension.".to_string()),
    };

    // Read the file content
    let content = match std::fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return Err("Could not read the file.".to_string()),
    };

    // Match the file extension to a programming language and extract imports
    let imported_files = match extension {
        "rs" => extract_imports_rust(&content),
        "py" => extract_imports_python(&content),
        "js" | "ts" => extract_imports_js_ts(&content),
        "java" => extract_imports_java(&content),
        "c" | "cpp" | "h" => extract_imports_c_cpp(&content),
        "php" => extract_imports_php(&content),
        _ => return Err("Unsupported or unknown file type.".to_string()),
    }
    .into_iter()
    .map(|f| resolve_import_paths(file_path, f))
    .collect::<Vec<String>>();

    Ok(imported_files)
}

fn resolve_import_paths(base_file_path: &str, import: String) -> String {
    // Get the base file's parent directory
    let base_path = Path::new(base_file_path)
        .parent()
        .unwrap_or_else(|| Path::new(""));

    // Join the base path with the relative import path
    let mut full_path = base_path.join(import);

    // Normalize the path, resolving `..` and `.` components
    full_path = full_path.canonicalize().unwrap_or(full_path);

    // Convert the path to a string and replace backslashes with forward slashes
    let full_path_str = full_path.to_string_lossy().to_string();

    // Ensure the path only uses one type of separator (here we choose '/')
    let normalized_path = full_path_str.replace("\\", "/");

    normalized_path
}

// Function to extract imports for Rust
fn extract_imports_rust(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("mod ") || line.trim().starts_with("extern crate ") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.trim_end_matches(';').to_string());
            }
        }
    }
    imports
}

// Function to extract imports for Python
fn extract_imports_python(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("import ") || line.trim().starts_with("from ") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.to_string());
            }
        }
    }
    imports
}

// Function to extract imports for JavaScript/TypeScript
fn extract_imports_js_ts(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("import ") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.trim_matches(&['"', '\'', '{', '}'][..]).to_string());
            }
        }
    }
    imports
}

// Function to extract imports for Java
fn extract_imports_java(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("import ") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.trim_end_matches(';').to_string());
            }
        }
    }
    imports
}

// Function to extract includes for C/C++
fn extract_imports_c_cpp(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("#include ") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.trim_matches(&['"', '<', '>'][..]).to_string());
            }
        }
    }
    imports
}

// Function to extract includes for PHP
fn extract_imports_php(content: &str) -> Vec<String> {
    let mut imports = Vec::new();
    for line in content.lines() {
        if line.trim().starts_with("include") || line.trim().starts_with("require") {
            if let Some(import) = line.split_whitespace().nth(1) {
                imports.push(import.trim_matches(&['"', '\'', ';'][..]).to_string());
            }
        }
    }
    imports
}
