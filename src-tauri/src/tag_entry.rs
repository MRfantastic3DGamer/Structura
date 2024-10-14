use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use file_walk::file_walk;
use tauri::api::file;

mod file_walk;
mod imports_checking;

#[derive(Debug)]
pub struct TagEntry {
    pub tag_name: String,
    pub file_name: String,
    pub reg_ex: String,
    pub tag: String,
    pub context: String,
}

// TODO: associate them with key: u128,
pub struct ScopeEntry {
    pub file_name: String,
    pub start_line: u128,
    pub start_col: u128,
    pub end_line: u128,
    pub end_col: u128,
    pub parent_scope: u128,
    pub children_scop: Vec<u128>,
}

pub struct ClassEntry {
    pub name: String,
    pub class_scope: u128,
}

pub struct InterfaceEntry {
    pub name: String,
    pub class_scope: u128,
}

pub struct FunctionEntry {
    pub name: String,
    pub parent_scope: u128,
    pub function_scope: u128,
}

pub struct ObjectEntry {
    pub name: String,
    pub parent_scope: u128,
}

pub fn get_tags_data(tags_path: String) -> io::Result<Vec<TagEntry>> {
    let mut tags = Vec::new();

    // Open the file.
    let path = Path::new(&tags_path);
    let file = File::open(path)?;

    // Read the file line by line
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line?; // Handle potential I/O errors
        if !line.starts_with("!") {
            // Skip comment lines in the tags file
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() >= 3 {
                let tag_name = parts[0].to_string();
                let file_name = parts[1].to_string();
                let reg_ex = parts[2]
                    .to_string()
                    .get(2..parts[2].to_string().len() - 4)
                    .unwrap()
                    .trim()
                    .to_string();
                let tag = if parts.len() >= 4 {
                    parts[3].to_string()
                } else {
                    "".to_string()
                };
                let context = if parts.len() >= 5 {
                    parts[4].to_string()
                } else {
                    "".to_string()
                };
                tags.push(TagEntry {
                    tag_name: tag_name,
                    file_name: file_name,
                    reg_ex: reg_ex,
                    tag: tag,
                    context: context,
                });
            }
        }
    }
    Ok(tags)
}

pub fn get_all_files(tags: &Vec<TagEntry>) -> HashSet<&String> {
    tags.into_iter()
        .map(|tag| &tag.file_name)
        .collect::<HashSet<&String>>()
}

pub fn get_all_imports<'a>(all_files: &'a HashSet<&'a String>) -> HashMap<&'a String, Vec<String>> {
    let mut all_imports = HashMap::new();

    for file in all_files {
        println!("{}", file);
        match imports_checking::get_imported_files(file) {
            Ok(i) => {
                all_imports.insert(*file, i); // Insert the file reference and the imports vector
            }
            Err(e) => {
                println!("error in imports for {} => {}", file, e);
            }
        }
    }

    all_imports
}

fn get_all_Scopes<'a>(
    all_files: &'a HashSet<&'a String>,
    all_tags: &Vec<TagEntry>,
) -> HashMap<&'a String, Vec<ScopeEntry>> {
    let mut all_scopes = HashMap::new();

    for file in all_files {
        let mut tags: Vec<&TagEntry> = Vec::new();
        for t in all_tags {
            if t.file_name == **file {
                tags.push(t.clone());
            }
        }
        all_scopes.insert(*file, file_walk(file, &tags));
    }
    all_scopes
}

pub fn get_all_data(all_tags: &Vec<TagEntry>) {
    let all_files = get_all_files(all_tags);
    let all_imports = get_all_imports(&all_files);
    let all_scopes = get_all_Scopes(&all_files, &all_tags);
}
