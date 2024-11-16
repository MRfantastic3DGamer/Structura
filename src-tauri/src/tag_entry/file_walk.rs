mod language_scanners;
use super::{ClassEntry, FunctionEntry, ObjectEntry, ScopeEntry, TagEntry};
use crate::data::{
    get_regex_access_child, get_regex_assignments, get_regex_class, get_regex_fun,
    get_regex_lambda, get_regex_object,
};
use regex::Regex;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
    u128,
};

macro_rules! build_regex_vec_from_res {
    ($res:expr) => {{
        let (_, regex_res) = $res;
        if regex_res.is_none() {
            eprintln!("couldn't get the regex for the file {:?}", $res);
            return None;
        }
        let regex_strs = regex_res.unwrap();
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

/// works for only cpp and needs to be redesigned for other languages
pub fn language_file_intense_extract(file_path: &String) -> Option<bool> {
    println!();
    println!("{}", file_path);

    let file_text = match fs::read_to_string(file_path) {
        Ok(f) => f,
        Err(_) => String::new(),
    };

    // scopes evaluation

    // start, end, parent
    let mut scope_entries: Vec<[usize; 3]> = Vec::new();
    let mut scope_stack: Vec<usize> = Vec::new();
    // region create file level scope

    scope_entries.push([0, 0, usize::MAX]);
    scope_stack.push(0);
    // endregion

    for (c_i, c) in file_text.chars().enumerate() {
        if c == '{' {
            scope_entries.push([
                c_i,
                0,
                if let Some(&parent_idx) = scope_stack.last() {
                    parent_idx
                } else {
                    usize::MAX
                },
            ]);
            let new_scope_idx = scope_entries.len() - 1;
            scope_stack.push(new_scope_idx);
        } else if c == '}' {
            if let Some(scope_idx) = scope_stack.pop() {
                // Update the end char idx for the last scope on the stack
                scope_entries[scope_idx][1] = c_i;
            } else {
                println!("Unmatched closing brace at {}", c_i);
            }
        }
    }
    drop(scope_stack);

    println!("Scopes");
    scope_entries
        .iter()
        .for_each(|x| println!("start:{}, end:{}, parent{}", x[0], x[1], x[2]));

    // -------------------------------------------------------------------------------------------------------//
    // MATCHING ALL THE PATTERNS TO NARROW DOWN SEARCH FOR ALL THE THINGS
    // -------------------------------------------------------------------------------------------------------//
    let access_children_regex = build_regex_vec_from_res!(get_regex_access_child(file_path));
    let assignments_regex = build_regex_vec_from_res!(get_regex_assignments(file_path));
    let class_regex = build_regex_vec_from_res!(get_regex_class(file_path));
    let funs_regex = build_regex_vec_from_res!(get_regex_fun(file_path));
    // let interfaces_regex = build_regex_vec_from_res!(get_regex_interface(file_path));
    let lambdas_regex = build_regex_vec_from_res!(get_regex_lambda(file_path));
    let objs_regex = build_regex_vec_from_res!(get_regex_object(file_path));

    for a in access_children_regex {
        for caps in a.find_iter(&file_text) {
            println!("{}", caps.as_str());
        }
        println!();
    }
    println!("assignments");
    for a in assignments_regex {
        // println!("for regex:{}", a.as_str());
        for caps in a.captures_iter(&file_text) {
            // caps.iter().for_each(|x| {
            //     print!("{:?}", x);
            // });
            if let Some(res) = caps
                .iter()
                .filter(|m_o| {
                    if let Some(m) = m_o {
                        return m.as_str().contains('=');
                    }
                    false
                })
                .next()
            {
                if res.is_none() {
                    continue;
                }
                let eq_match = res.unwrap();
                let equation = eq_match.as_str();
                if let Some(eq_pos) = equation.find('=') {
                    let lhs = equation[..eq_pos].to_string();
                    let rhs = equation
                        .chars()
                        .skip(eq_pos + 1)
                        .collect::<String>()
                        .trim()
                        .to_string();
                    let mut parent_scope = 0;
                    let def_start_pos = eq_match.start();
                    for (s_i, s) in scope_entries.iter().enumerate() {
                        if def_start_pos > s[0] && s[0] > scope_entries[parent_scope][0] {
                            parent_scope = s_i;
                        }
                    }
                    println!("lhs:{}, rhs:{}, inside:({})", lhs, rhs, parent_scope);
                }
            }
            println!();
        }
    }
    println!("classes");
    for c in class_regex {
        for caps in c.captures_iter(&file_text) {
            // caps.iter().for_each(|x| print!("{x:?}"));
            if let Some(class_def_m) = &caps.get(0) {
                let class_def = class_def_m.as_str();
                if let Some(parents_def_pos) = class_def.find(':') {
                    let class_part = class_def[..parents_def_pos]
                        .trim()
                        .strip_prefix("class")
                        .unwrap()
                        .to_string();
                    println!("{}", class_part);
                    let class_parents = &caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let mut comma_pos = Vec::new();
                    class_parents.chars().enumerate().for_each(|(i, c)| {
                        if c == ',' {
                            comma_pos.push(i);
                        }
                    });
                    let mut parent_type_and_names = Vec::new();
                    let mut prev_comma_pos = 0 as usize;
                    comma_pos.iter().for_each(|p| {
                        parent_type_and_names
                            .push(class_parents[prev_comma_pos..p.clone()].trim().to_string());
                        prev_comma_pos = p + 1;
                    });
                    parent_type_and_names.push(class_parents[prev_comma_pos..].trim().to_string());
                    let mut parents: Vec<[String; 2]> = Vec::new();
                    parent_type_and_names.iter().for_each(|t_n| {
                        if let Some(sep_pos) = t_n.rfind(' ') {
                            let arg_type = t_n[..sep_pos].trim().to_string();
                            let arg_name = t_n[sep_pos + 1..].trim().to_string();
                            parents.push([arg_type, arg_name]);
                        }
                    });
                    print!("  parents: {:?}", parents);
                } else {
                    let class_part = class_def[..class_def.len() - 1].trim().to_string();
                    if let Some(class_name) = class_part.strip_prefix("class") {
                        print!("{} with no parents", class_name.trim().to_string());
                    }
                }

                let scope_start_pos = class_def_m.end();
                let class_scope = scope_entries
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x[0] == scope_start_pos - 1)
                    .find_map(|x| {
                        if x.1[0] == scope_start_pos - 1 {
                            return Some(x.0);
                        }
                        None
                    });
                println!("with scope as ({:?})", class_scope);
            } else {
                // eprintln!("couldn't parse for class\n{}", )
            }
            println!();
        }
    }
    println!("functions");
    for f in funs_regex {
        // println!("regex {}", f.as_str());
        for caps in f.captures_iter(&file_text) {
            // println!("captured: {:?}", caps);
            caps.iter().for_each(|x| {
                if let Some(m) = x {
                    let m_str = m.as_str();
                    let def = m_str[..m_str.len()].to_string();
                    let bo_pos_o = def.find("(");
                    let bc_pos_o = def.rfind(")");
                    if bo_pos_o.is_none() || bc_pos_o.is_none() {
                        return;
                    }
                    let bo_pos = bo_pos_o.unwrap();
                    let bc_pos = bc_pos_o.unwrap();
                    let type_and_name = def[..bo_pos].to_string();
                    let type_name_separator_pos = type_and_name.rfind(" ").unwrap_or(0);
                    let (type_, name) = (
                        type_and_name[..type_name_separator_pos].to_string(),
                        type_and_name[type_name_separator_pos + 1..].to_string(),
                    );
                    let inside_b = def[bo_pos + 1..bc_pos].to_string();
                    let mut comma_pos = Vec::new();
                    inside_b.chars().enumerate().for_each(|(i, c)| {
                        if c == ',' {
                            comma_pos.push(i);
                        }
                    });
                    let mut args_type_and_names = Vec::new();
                    let mut prev_comma_pos = 0 as usize;
                    comma_pos.iter().for_each(|p| {
                        args_type_and_names
                            .push(inside_b[prev_comma_pos..p.clone()].trim().to_string());
                        prev_comma_pos = p + 1;
                    });
                    args_type_and_names.push(inside_b[prev_comma_pos..].trim().to_string());
                    let mut args: Vec<[String; 2]> = Vec::new();
                    args_type_and_names.iter().for_each(|t_n| {
                        if let Some(sep_pos) = t_n.rfind(' ') {
                            let arg_type = t_n[..sep_pos].trim().to_string();
                            let arg_name = t_n[sep_pos + 1..].trim().to_string();
                            args.push([arg_type, arg_name]);
                        }
                    });

                    let scope_start_pos = m.end();

                    let fun_scope = scope_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| x[0] == scope_start_pos - 1)
                        .find_map(|x| {
                            if x.1[0] == scope_start_pos - 1 {
                                return Some(x.0);
                            }
                            None
                        });
                    // .map(|x| x.0)
                    // .collect::<Vec<usize>>()
                    // .get(0)
                    // .unwrap_or(&usize::MAX);
                    print!(
                        "fun:{} return:{}, with args:{:?}, and scope as ({:?})",
                        name, type_, args, fun_scope
                    );
                    println!();
                }
            });
        }
    }
    // println!("interfaces");
    // for i in interfaces_regex {
    //     for caps in i.captures_iter(&file_text) {
    //         caps.iter().for_each(|m| {
    //             print!("{:?}", m);
    //         });
    //         println!();
    //     }
    // }
    println!("lambdas");
    for l in lambdas_regex {
        for caps in l.captures_iter(&file_text) {
            caps.iter().for_each(|m| {
                print!("{:?}", m);
            });
            println!();
        }
    }
    println!("objects");
    for o in objs_regex {
        for caps in o.captures_iter(&file_text) {
            let obj_class = &caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let obj_name = &caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let mut parent_scope = 0;
            let def_start_pos = &caps.get(1).unwrap().start();
            for (s_i, s) in scope_entries.iter().enumerate() {
                if *def_start_pos > s[0] && s[0] > scope_entries[parent_scope][0] {
                    parent_scope = s_i;
                }
            }
            println!(
                "object Class: {} , name: {} inside: {}",
                obj_class, obj_name, parent_scope
            );
            println!();
        }
    }

    Some(true)
}
