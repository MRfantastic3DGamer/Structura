use std::{
    collections::{HashMap, HashSet},
    path,
};

mod read_imports;
use read_imports::Import;

use crate::tag_entry::{ClassEntry, FunctionEntry, ObjectEntry, ScopeEntry};

enum AvailableTag {
    Class(ClassEntry),
    Function(FunctionEntry),
    Object(ObjectEntry),
}

pub fn evaluate_all_available_tags<'a>(project_path: &String, all_files: &'a HashSet<&'a String>) {
    let raw_imports = read_all_imports(project_path, all_files);
    println!("\n\n-----raw imports-----\n\n");
    for file in &raw_imports {
        let file_path = file.0;
        let imports = file.1;
        println!("\nfor file {}", file_path);
        for import in imports {
            match import {
                Import::File(path) => println!("f-->{}", path),
                Import::Module(path) => println!("m-->{}", path),
                Import::Package(path) => println!("p-->{}", path),
            }
        }
    }
}

fn read_all_imports<'a>(
    project_path: &String,
    all_files: &'a HashSet<&'a String>,
) -> HashMap<&'a String, Vec<Import>> {
    let mut all_imports = HashMap::new();

    for file in all_files {
        println!("{}", file);
        match read_imports::get_imported_files(project_path, file) {
            Ok(imports) => {
                all_imports.insert(*file, imports);
            }
            Err(e) => {
                println!("error in imports for {} => {}", file, e);
            }
        }
    }

    all_imports
}

fn evaluate_available_tags<'a>(
    imports: Vec<Import>,
    hard_data: HashMap<
        &String,
        (
            Vec<ScopeEntry>,
            Vec<ClassEntry>,
            Vec<FunctionEntry>,
            Vec<ObjectEntry>,
        ),
    >,
) -> Result<Vec<AvailableTag>, &str> {
    let mut available_tag: Vec<AvailableTag> = Vec::new();
    for import in imports {
        match import {
            Import::File(path) => {
                let file_tags_option = hard_data.get(&path);
                match file_tags_option {
                    Some(file_tags) => {
                        let mut available_tag: Vec<AvailableTag> = Vec::new();
                        // let scopes = file_tags.0;
                    }
                    None => {
                        return Err("data for this file not found");
                    }
                }
            }
            Import::Module(path) => {
                // todo: recursively look for all files inside and add them all
            }
            Import::Package(name) => {
                // leave empty and enable mailability so that new functions / objects / classes can later be attached to this package
            }
        }
    }
    Ok(available_tag)
}
