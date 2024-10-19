mod file_walk;
mod imports_checking;
mod serialization;

use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path::Path,
    sync::Arc,
};

use file_walk::file_walk;
use serde::Serialize;
use serialization::{u128_as_string, vec_u128_as_string};
use tauri::window;

#[derive(Debug, Serialize)]
pub struct TagEntry {
    pub tag_name: String,
    pub file_name: String,
    pub reg_ex: String,
    pub tag: String,
    pub context: String,
}

#[derive(Serialize)]
pub struct ScopeEntry {
    pub file_name: String,
    #[serde(serialize_with = "u128_as_string")]
    pub start_line: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub start_col: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub end_line: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub end_col: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub parent_scope: u128,
    #[serde(serialize_with = "vec_u128_as_string")]
    pub children_scop: Vec<u128>,
}

#[derive(Serialize)]
pub struct ClassEntry {
    pub name: String,
    #[serde(serialize_with = "u128_as_string")]
    pub parent_scope: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub class_scope: u128,
}

#[derive(Serialize)]
pub struct FunctionEntry {
    pub name: String,
    #[serde(serialize_with = "u128_as_string")]
    pub parent_scope: u128,
    #[serde(serialize_with = "u128_as_string")]
    pub function_scope: u128,
}

#[derive(Serialize)]
pub struct ObjectEntry {
    pub name: String,
    #[serde(serialize_with = "u128_as_string")]
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

pub fn get_all_files<'a>(tags: &'a Vec<TagEntry>) -> HashSet<&'a String> {
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
                all_imports.insert(*file, i);
            }
            Err(e) => {
                println!("error in imports for {} => {}", file, e);
            }
        }
    }

    all_imports
}

async fn get_all_Scopes<'a>(
    all_files: &'a HashSet<&'a String>,
    all_tags: &'a Vec<TagEntry>,
    progress_indication: impl Fn(u8),
) -> HashMap<
    &'a String,
    (
        Vec<ScopeEntry>,
        Vec<ClassEntry>,
        Vec<FunctionEntry>,
        Vec<ObjectEntry>,
    ),
> {
    let mut all_data = HashMap::new();
    let total_files = all_files.len(); // Get the total number of files only once

    for (i, file) in all_files.iter().enumerate() {
        let mut tags: Vec<&TagEntry> = Vec::new();
        for t in all_tags {
            if t.file_name == **file {
                tags.push(t);
            }
        }

        // Process the file and collect the data
        let file_data = file_walk(file, &tags);
        all_data.insert(file.clone(), file_data);

        // Calculate progress and send it
        let progress = ((i + 1) as f32 / total_files as f32) * 100.0; // Use floating-point division
        progress_indication(progress as u8); // Cast to u8 for progress indication
    }

    all_data
}

pub async fn get_all_data<'f>(
    all_files: &'f HashSet<&'f String>,
    all_tags: &'f Vec<TagEntry>,
    progress_indication: impl Fn(u8),
) -> HashMap<
    &'f String,
    (
        Vec<ScopeEntry>,
        Vec<ClassEntry>,
        Vec<FunctionEntry>,
        Vec<ObjectEntry>,
    ),
> {
    let all_imports = get_all_imports(&all_files);
    let all_data: HashMap<
        &String,
        (
            Vec<ScopeEntry>,
            Vec<ClassEntry>,
            Vec<FunctionEntry>,
            Vec<ObjectEntry>,
        ),
    > = get_all_Scopes(&all_files, all_tags, progress_indication).await;
    all_data
}
