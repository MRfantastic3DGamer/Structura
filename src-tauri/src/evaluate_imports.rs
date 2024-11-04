use std::{
    collections::{HashMap, HashSet},
    path,
};

mod read_imports;
use read_imports::Import;

use crate::tag_entry::{ClassEntry, FunctionEntry, ObjectEntry, ScopeEntry};

#[derive(Debug)]
enum AvailableTag {
    Class {
        name: String,
    },
    /// for class the representation is (file_number, tag_number)
    Function {
        name: String,
        class: Option<(usize, usize)>,
    },
    /// for class the representation is (file_number, tag_number)
    Object {
        name: String,
        class: Option<(usize, usize)>,
    },
}

pub fn evaluate_all_available_tags<'a>(
    project_path: &String,
    all_files: &'a HashSet<&'a String>,
    all_hard_data: HashMap<
        &'a String,
        (
            Vec<ScopeEntry>,
            Vec<ClassEntry>,
            Vec<FunctionEntry>,
            Vec<ObjectEntry>,
        ),
    >,
) {
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

    let mut all_tags: HashMap<usize, Vec<AvailableTag>> = HashMap::new();
    let mut children_tags: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for (f, file_path) in all_files.iter().enumerate() {
        if let Some(file_hard_data) = all_hard_data.get(file_path) {
            let mut scope_to_class_tag = HashMap::new();

            // add the entry
            file_hard_data.1.iter().enumerate().for_each(|(i, c)| {
                all_tags
                    .entry(f)
                    .or_insert_with(Vec::new)
                    .push(AvailableTag::Class {
                        name: c.name.clone(),
                    });

                scope_to_class_tag.insert(c.class_scope, i);
            });
            file_hard_data.2.iter().for_each(|fun| {
                all_tags
                    .entry(f)
                    .or_insert_with(Vec::new)
                    .push(AvailableTag::Function {
                        name: fun.name.clone(),
                        class: None,
                    });

                if let Some(parent_class) = scope_to_class_tag.get(&fun.parent_scope) {
                    children_tags
                        .entry((f, parent_class.clone()))
                        .or_default()
                        .push((f, all_tags[&f].len() - 1));

                    println!(
                        "added fn {},{} to {},{}",
                        f,
                        all_tags.len() - 1,
                        f,
                        parent_class
                    );
                }
            });
            file_hard_data.3.iter().for_each(|ob| {
                all_tags
                    .entry(f)
                    .or_insert_with(Vec::new)
                    .push(AvailableTag::Object {
                        name: ob.name.clone(),
                        class: None,
                    });

                if let Some(parent_class) = scope_to_class_tag.get(&ob.parent_scope) {
                    children_tags
                        .entry((f, parent_class.clone()))
                        .or_default()
                        .push((f, all_tags[&f].len() - 1));
                }
            });
        }
    }

    println!("\n\n---------- file wise tags ----------\n\n");
    for (f, file_path) in all_files.iter().enumerate() {
        println!("for file {}->{}", f, file_path);
        if let Some(file_tags) = all_tags.get(&f) {
            file_tags.iter().for_each(|t| println!("{:?}", t));
        }
    }

    println!("\n\n---------- children table ----------\n\n");
    for ((pf, ps), c) in children_tags.iter() {
        println!("from {:?}", all_tags[pf][*ps]);
        c.iter().for_each(|(cf, ck)| {
            println!("   to {:?}", all_tags[cf][*ck]);
        });
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
