use std::{
    collections::{HashMap, HashSet},
    usize,
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
        class: Result<(usize, usize), String>,
    },
    /// for class the representation is (file_number, tag_number)
    Object {
        name: String,
        class: Result<(usize, usize), String>,
    },
}

impl AvailableTag {
    pub fn get_name(&self) -> &String {
        match self {
            AvailableTag::Class { name } => name,
            AvailableTag::Function { name, class: _ } => name,
            AvailableTag::Object { name, class: _ } => name,
        }
    }

    pub fn is_class(&self) -> bool {
        if let AvailableTag::Class { name: _ } = self {
            return true;
        }
        return false;
    }

    pub fn needed_class(&self) -> Option<&String> {
        match self {
            AvailableTag::Class { name: _ } => None,
            AvailableTag::Function { name: _, class } => match class {
                Ok(_) => None,
                Err(c) => Some(c),
            },
            AvailableTag::Object { name: _, class } => match class {
                Ok(_) => None,
                Err(c) => Some(c),
            },
        }
    }

    pub fn put_class_data(&mut self, file_tag_i: (usize, usize)) {
        match self {
            AvailableTag::Class { name: _ } => {}
            AvailableTag::Function { name: _, class } => *class = Ok(file_tag_i),
            AvailableTag::Object { name: _, class } => *class = Ok(file_tag_i),
        }
    }
}

// println!("\n\n-----raw imports-----\n\n");
// for (file_i, file_imports) in &raw_imports {
//     println!("\nfor file {}", all_files.get(file_i).unwrap());
//     for import in file_imports {
//         match import {
//             Import::File(path) => println!("f-->{}", path),
//             Import::Module(path) => println!("m-->{}", path),
//             Import::Package(path) => println!("p-->{}", path),
//         }
//     }
// }
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

    println!("-------- all_files --------");
    for (f, f_p) in all_files.iter().enumerate() {
        println!("({}) {}", f, f_p);
    }

    println!("-------- raw_imports --------");
    for (file, imports) in &raw_imports {
        println!("for file {}", file);
        for import in imports {
            println!("\t{}", import);
        }
    }

    let mut all_tags: HashMap<usize, Vec<AvailableTag>> = HashMap::new();
    let mut children_tags: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    // initial formation of tags list and tags hierarchy
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
                        class: Err(fun.class_name.clone()),
                    });

                if let Some(parent_class) = scope_to_class_tag.get(&fun.parent_scope) {
                    children_tags
                        .entry((f, parent_class.clone()))
                        .or_default()
                        .push((f, all_tags[&f].len() - 1));
                }
            });
            file_hard_data.3.iter().for_each(|ob| {
                all_tags
                    .entry(f)
                    .or_insert_with(Vec::new)
                    .push(AvailableTag::Object {
                        name: ob.name.clone(),
                        class: Err(ob.class_name.clone()),
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

    // trying to create connections between tag_class and some actual class that may exist
    let mut changes = HashMap::new();
    for (f, _) in all_files.iter().enumerate() {
        let file_tags = all_tags.get(&f).unwrap();

        let imported_files = match raw_imports.get(&f) {
            Some(fi) => fi,
            None => &Vec::new(),
        };

        println!("--> for file {}", f);
        all_tags
            .iter()
            // tags from imported files
            .filter(|(imported_file, _)| imported_files.contains(imported_file))
            .for_each(|(imported_file, imported_tags)| {
                for (i_t_i, i_t) in imported_tags.iter().enumerate() {
                    if !i_t.is_class() {
                        continue;
                    }
                    println!("--> checking {:?}", i_t);
                    for (matched_tag_index, t) in file_tags.iter().enumerate() {
                        println!(
                            "  for {:?} {}",
                            t,
                            t.needed_class().unwrap_or(&"!".to_string()) == i_t.get_name()
                        );
                        if t.needed_class().unwrap_or(&"!".to_string()) == i_t.get_name() {
                            changes.insert((f, matched_tag_index), (*imported_file, i_t_i));
                        }
                    }
                }
            });
    }

    // bake changes
    all_tags.iter_mut().for_each(|(f, file_tags)| {
        file_tags.iter_mut().enumerate().for_each(|(tag_i, tag)| {
            let tag_key = (*f, tag_i);
            if changes.contains_key(&tag_key) {
                tag.put_class_data(changes.get(&tag_key).unwrap().clone());
            }
        });
    });

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
) -> HashMap<usize, Vec<usize>> {
    let mut all_imports: HashMap<usize, Vec<usize>> = HashMap::new();

    for (f, file) in all_files.iter().enumerate() {
        println!("{}", file);
        match read_imports::get_imported_files(project_path, file) {
            Ok(imports) => {
                for import in imports {
                    let import_path = match import {
                        Import::File(path) => path,
                        Import::Module(path) => path,
                        Import::Package(path) => path,
                    };
                    if let Some(import_index) = all_files.iter().position(|i| **i == import_path) {
                        all_imports.entry(f).or_default().push(import_index);
                    }
                }
            }
            Err(e) => {
                println!("error in imports for {} => {}", file, e);
            }
        }
    }

    all_imports
}
