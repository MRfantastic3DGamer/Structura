use std::{
    fs::File,
    io::{self, BufRead, Seek, SeekFrom},
    path::Path,
};

use tauri::GlobPattern;

use super::{ClassEntry, FunctionEntry, ObjectEntry, ScopeEntry, TagEntry};

pub fn file_walk(
    file_path: &String,
    file_tags: &Vec<&TagEntry>,
) -> (
    Vec<ScopeEntry>,
    Vec<ClassEntry>,
    Vec<FunctionEntry>,
    Vec<ObjectEntry>,
) {
    let res = indentation_walk(file_path, file_tags);
    res
}

fn indentation_walk(
    file_path: &String,
    tags: &Vec<&TagEntry>,
) -> (
    Vec<ScopeEntry>,
    Vec<ClassEntry>,
    Vec<FunctionEntry>,
    Vec<ObjectEntry>,
) {
    // Open the file
    let path = Path::new(file_path);
    let file_r = File::open(path);
    let mut file: File;
    match file_r {
        Ok(f) => {
            file = f;
        }
        Err(_) => {
            println!("could not read this file {}", file_path);
            return (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        }
    }

    let mut scope_entries: Vec<ScopeEntry> = Vec::new();
    let mut class_entries: Vec<ClassEntry> = Vec::new();
    let mut function_entries: Vec<FunctionEntry> = Vec::new();
    let mut object_entries: Vec<ObjectEntry> = Vec::new();
    let mut scope_stack: Vec<usize> = Vec::new(); // Stack to track current open scope indexes
    let mut line_num: u128 = 0;
    let mut scope_scout_tag = '_';

    let buf_reader = io::BufReader::new(&file);
    for (line_index, line) in buf_reader.lines().enumerate() {
        let line_content = line.unwrap();

        for t in tags {
            // todo: also look for if else elif switch etc

            if line_content.find(&t.reg_ex).is_some() {
                match t.tag.as_str() {
                    "c" => {
                        let new_class_entry = ClassEntry {
                            name: t.tag_name.clone(),
                            class_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx as u128
                            } else {
                                u128::MAX // Indicate no parent (root scope)
                            },
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx as u128
                            } else {
                                u128::MAX // Indicate no parent (root scope)
                            },
                        };

                        class_entries.push(new_class_entry);
                        scope_scout_tag = 'c';
                    }
                    "f" => {
                        let new_fn_entry = FunctionEntry {
                            name: t.tag_name.clone(),
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx as u128
                            } else {
                                u128::MAX // Indicate no parent (root scope)
                            },
                            function_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx as u128
                            } else {
                                u128::MAX // Indicate no parent (root scope)
                            },
                        };
                        function_entries.push(new_fn_entry);
                        scope_scout_tag = 'f';
                    }
                    "m" => {
                        let new_obj_entry = ObjectEntry {
                            name: t.tag_name.clone(),
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx as u128
                            } else {
                                u128::MAX // Indicate no parent (root scope)
                            },
                        };
                        object_entries.push(new_obj_entry);
                        // scope_scout_tag = 'm'; TODO:
                    }
                    _ => {}
                }
            }
        }

        let mut col_num: u128 = 0;
        line_num = line_index as u128 + 1; // Line numbers start from 1

        // Traverse each character in the line
        for ch in line_content.chars() {
            col_num += 1;

            // If we encounter an opening brace '{', create a new ScopeEntry
            if ch == '{' {
                let new_scope = ScopeEntry {
                    file_name: file_path.clone(),
                    start_line: line_num,
                    start_col: col_num,
                    end_line: 0, // Placeholder, will be updated when the scope ends
                    end_col: 0,  // Placeholder, will be updated when the scope ends
                    parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                        parent_idx as u128
                    } else {
                        u128::MAX // Indicate no parent (root scope)
                    },
                    children_scop: Vec::new(),
                };

                // Add to scope_entries and push the index to the stack
                scope_entries.push(new_scope);
                let new_scope_idx = scope_entries.len() - 1;

                // If there's a parent scope, add the current scope as its child
                if let Some(&parent_idx) = scope_stack.last() {
                    scope_entries[parent_idx]
                        .children_scop
                        .push(new_scope_idx.clone() as u128);
                }

                // set this scope as class or function scope if it was being scouted for
                match scope_scout_tag {
                    'c' => {
                        if let Some(class_entry) = class_entries.last_mut() {
                            class_entry.class_scope = new_scope_idx as u128
                        }
                    }
                    'f' => {
                        if let Some(function_entry) = function_entries.last_mut() {
                            function_entry.function_scope = new_scope_idx as u128
                        }
                    }
                    _ => {}
                }

                // Push this scope's index onto the stack (to denote it's the current scope)
                scope_stack.push(new_scope_idx);
            }
            // If we encounter a closing brace '}', finalize the current ScopeEntry
            else if ch == '}' {
                if let Some(scope_idx) = scope_stack.pop() {
                    // Update the end line and column for the last scope on the stack
                    scope_entries[scope_idx].end_line = line_num;
                    scope_entries[scope_idx].end_col = col_num;
                } else {
                    println!(
                        "Unmatched closing brace at line {} column {}",
                        line_num, col_num
                    );
                }
            }
        }
    }

    println!("in {}", &file_path);
    println!("scopes");
    for s in &scope_entries {
        println!(
            "\t{}..{}, p->{}, c->({})",
            s.start_line,
            s.end_line,
            s.parent_scope,
            s.children_scop
                .iter()
                .map(|num| num.to_string()) // Convert each u128 to String
                .collect::<Vec<String>>()
                .join(",")
        );
    }
    println!("classes");
    for c in &class_entries {
        println!("\t{} in {} of {}", c.name, c.parent_scope, c.class_scope);
    }
    println!("functions");
    for f in &function_entries {
        println!("\t{} in {} of {}", f.name, f.parent_scope, f.function_scope);
    }
    println!("objects");
    for m in &object_entries {
        println!("\t{} in {}", m.name, m.parent_scope);
    }

    (
        scope_entries,
        class_entries,
        function_entries,
        object_entries,
    )
}

fn brackets_walk() {}
