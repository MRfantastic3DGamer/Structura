use std::path::{Path, PathBuf};

pub enum Import {
    ///     if a file is imported
    File(String),
    ///     if a module is imported
    Module(String),
    ///     if a pre built package is imported
    Package(String),
}

pub fn get_imported_files(project_path: &String, file_path: &str) -> Result<Vec<Import>, String> {
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
    .map(|f| resolve_import_paths(project_path, file_path, &f))
    .collect::<Vec<Import>>();

    Ok(imported_files)
}

fn resolve_import_paths(project_path: &String, file_path: &str, import: &String) -> Import {
    // Get the base file's parent directory
    let file_relative_path = Path::new(file_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(import);

    if file_relative_path.exists() {
        return format_path(file_relative_path);
    }

    let project_relative_path = Path::new(project_path)
        .parent()
        .unwrap_or_else(|| Path::new(""))
        .join(import);
    if project_relative_path.exists() {
        return format_path(project_relative_path);
    }

    return Import::Package(import.clone());
}

fn format_path(path: PathBuf) -> Import {
    let path_str = path
        .canonicalize()
        .unwrap_or(path.to_path_buf())
        .to_string_lossy()
        .to_string();

    Import::File(
        path_str
            .strip_prefix("\\\\?\\")
            .unwrap_or(&path_str)
            .to_string(),
    )
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
