mod file_walk;
mod serialization;

use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead},
    path::Path,
    rc::{Rc, Weak},
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

#[derive(Serialize, Clone)]
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

#[derive(Clone)]
pub struct ClassEntry {
    pub name: String,
    pub parent_scope: usize,
    pub class_scope: usize,
}

#[derive(Clone)]
pub struct FunctionEntry {
    pub name: String,
    pub parent_scope: usize,
    pub function_scope: usize,
    pub class_name: String,
}

#[derive(Clone)]
pub struct ObjectEntry {
    pub name: String,
    pub parent_scope: usize,
    pub class_name: String,
    // default value
}

fn get_key(e: &ClassEntry, scopes: &Vec<ScopeEntry>) {}

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
    let mut files = tags
        .into_iter()
        .map(|tag| &tag.file_name)
        .collect::<Vec<&String>>();
    files.sort();
    HashSet::from_iter(files.into_iter())
}

pub async fn get_all_hard_data<'a>(
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

    for (i, file_path) in all_files.iter().enumerate() {
        let mut tags: Vec<&TagEntry> = Vec::new();
        for t in all_tags {
            if t.file_name == **file_path {
                tags.push(t);
            }
        }

        // Process the file and collect the data
        let file_data = file_walk(file_path, &tags);
        all_data.insert(*file_path, file_data);

        // Calculate progress and send it
        let progress = ((i + 1) as f32 / total_files as f32) * 100.0; // Use floating-point division
        progress_indication(progress as u8); // Cast to u8 for progress indication
    }

    all_data
}
