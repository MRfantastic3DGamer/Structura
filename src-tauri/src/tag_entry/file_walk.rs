// mod crate::evaluate_imports::languages_constants::get_language;

use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    u128,
};

use regex::Regex;

use super::{ClassEntry, FunctionEntry, ObjectEntry, ScopeEntry, TagEntry};

macro_rules! build_regex_vec_from_res {
    ($res:expr) => {{
        let (_, regex_res) = $res; // Destructure the tuple
        if regex_res.is_none() {
            eprintln!("couldn't get the regex for the file {:?}", $res);
            return None;
        }
        let regex_strs = regex_res.unwrap(); // Unwrap the Option<&&[&str]>
        let mut regex_vec = vec![];

        for regex_str in regex_strs.into_iter() {
            if let Ok(regex) = Regex::new(regex_str) {
                regex_vec.push(regex);
            } else {
                eprintln!("couldn't build regex for {}", regex_str);
            }
        }
        regex_vec
    }};
}

pub fn file_walk(
    file_path: &String,
    file_tags: &Vec<&TagEntry>,
) -> (
    Vec<ScopeEntry>,
    Vec<ClassEntry>,
    Vec<FunctionEntry>,
    Vec<ObjectEntry>,
) {
    let res = brackets_walk(file_path, file_tags);
    res
}

// TODO: note the section after = as its own scope
fn brackets_walk(
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
    let file: File;
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
    let mut scope_stack: Vec<usize> = Vec::new();
    let mut line_num: u128 = 0;
    let mut scope_scout_tag = '_';

    // region create file level scope
    scope_entries.push(ScopeEntry {
        file_name: file_path.clone(),
        start_line: 0,
        start_col: 0,
        end_line: 0,
        end_col: 0,
        parent_scope: u128::MAX,
        children_scop: Vec::new(),
    });
    scope_stack.push(0);
    // endregion

    let buf_reader = io::BufReader::new(&file);
    for (line_index, line) in buf_reader.lines().enumerate() {
        let line_content = line.unwrap();

        for t in tags {
            // todo: also look for if else elif switch etc

            if line_content.find(&t.reg_ex).is_some() {
                // TODO:search for all the data types first
                let class_name_regex = Regex::new(r"(\w+) ").unwrap();

                let class_name = match class_name_regex.captures(&line_content) {
                    Some(n) => n[0].to_string(),
                    None => "None".to_string(),
                }
                .trim()
                .to_string();

                // let line_parts: Vec<&str> = line_content.trim().split(" ").collect();
                // let class_name = line_parts[0].to_string();
                match t.tag.as_str() {
                    "c" => {
                        let parents = find_parents(&line_content);
                        println!("parents -> {:?}", &parents);
                        let new_class_entry = ClassEntry {
                            name: t.tag_name.clone(),
                            class_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx
                            } else {
                                usize::MAX // Indicate no parent (root scope)
                            },
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx
                            } else {
                                usize::MAX // Indicate no parent (root scope)
                            },
                            parents: parents,
                        };
                        class_entries.push(new_class_entry);
                        scope_scout_tag = 'c';
                    }
                    "f" => {
                        let new_fn_entry = FunctionEntry {
                            name: t.tag_name.clone(),
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx
                            } else {
                                usize::MAX // Indicate no parent (root scope)
                            },
                            function_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx
                            } else {
                                usize::MAX // Indicate no parent (root scope)
                            },
                            class_name: class_name,
                        };
                        function_entries.push(new_fn_entry);
                        scope_scout_tag = 'f';
                    }
                    "m" => {
                        let new_obj_entry = ObjectEntry {
                            name: t.tag_name.clone(),
                            parent_scope: if let Some(&parent_idx) = scope_stack.last() {
                                parent_idx
                            } else {
                                usize::MAX // Indicate no parent (root scope)
                            },
                            class_name: class_name,
                        };
                        object_entries.push(new_obj_entry);
                        // TODO: scope_scout_tag = 'm';
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
                            class_entry.class_scope = new_scope_idx
                        }
                    }
                    'f' => {
                        if let Some(function_entry) = function_entries.last_mut() {
                            function_entry.function_scope = new_scope_idx
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

    // update the ending values of class scope
    if let Some(scope_idx) = scope_stack.pop() {
        scope_entries[scope_idx].end_line = line_num;
        scope_entries[scope_idx].end_col = u128::MAX;
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
        println!(
            "\t{} {} in {} of {}",
            f.class_name, f.name, f.parent_scope, f.function_scope
        );
    }
    println!("objects");
    for m in &object_entries {
        println!("\t{} {} in {}", m.class_name, m.name, m.parent_scope);
    }

    (
        scope_entries,
        class_entries,
        function_entries,
        object_entries,
    )
}

fn find_parents(line: &String) -> Vec<String> {
    // Corrected regex pattern to capture access specifiers and class names.
    let parent_regex = Regex::new(r"(public|private|protected)\s+(\w+)").unwrap();

    parent_regex
        .captures_iter(line)
        .filter_map(|cap| cap.get(2))
        .map(|p| p.as_str().to_string())
        .collect()
}

pub fn language_file_walk(file_path: &String) -> Option<bool> {
    let file_text = match fs::read_to_string(file_path) {
        Ok(f) => f,
        Err(_) => String::new(),
    };

    let assignments_regex =
        build_regex_vec_from_res!(crate::data::get_regex_assignments(file_path));
    let class_regex = build_regex_vec_from_res!(crate::data::get_regex_class(file_path));
    let funs_regex = build_regex_vec_from_res!(crate::data::get_regex_fun(file_path));
    let _interfaces_regex = build_regex_vec_from_res!(crate::data::get_regex_interface(file_path));
    let objs_regex = build_regex_vec_from_res!(crate::data::get_regex_object(file_path));

    println!();
    for a in assignments_regex {
        for caps in a.captures_iter(&file_text) {
            println!("for obj {}", caps.get(1).map(|m| m.as_str()).unwrap_or(""));
            println!("-->{}", caps.get(0).map(|m| m.as_str()).unwrap_or(""));
            println!();
        }
    }
    for c in class_regex {
        for caps in c.captures_iter(&file_text) {
            let class_name = &caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let class_parents = &caps.get(2).map(|m| m.as_str()).unwrap_or("");

            println!("for Class: {}", class_name);
            println!("    parents: {}", class_parents);
            println!();
        }
    }
    for f in funs_regex {
        for caps in f.captures_iter(&file_text) {
            let return_type = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let function_name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let arguments = caps.get(3).map(|m| m.as_str()).unwrap_or("");

            println!("Function Name: {}", function_name);
            println!("  Return Type: {}", return_type);
            if !arguments.is_empty() {
                println!("  Arguments: {}", arguments);
            } else {
                println!("  Arguments: None");
            }
            println!();
        }
    }
    for o in objs_regex {
        for caps in o.captures_iter(&file_text) {
            let obj_class = &caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let obj_name = &caps.get(2).map(|m| m.as_str()).unwrap_or("");

            println!("object Class: {} , name: {}", obj_class, obj_name);
            println!();
        }
    }

    Some(true)
}
