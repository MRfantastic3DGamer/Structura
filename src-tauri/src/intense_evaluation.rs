use crate::{data::*, evaluate_imports::read_all_imports};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::usize;
use std::{collections::HashMap, fmt, fs};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StatefulClassConnection {
    Undiscovered(String),
    Connected(usize, usize),
    DataType(usize),
}
impl StatefulClassConnection {
    pub fn connect(&mut self, connectable: ClassConnectable) -> bool {
        match connectable.connection {
            Self::Undiscovered(_) => {}
            Self::Connected(f, t) => {
                *self = StatefulClassConnection::Connected(f, t);
                return true;
            }
            Self::DataType(t) => {
                *self = StatefulClassConnection::DataType(t);
                return true;
            }
        }
        false
    }
}
pub struct ClassConnectable {
    file: usize,
    scope: usize,
    query: String,
    connection: StatefulClassConnection,
}

/// start, end, parent, \[bracket/curly/square/root\], content
pub struct SCOPE(usize, usize, usize, i8, String);

/// start, \[access rout, \[args\]\]
pub struct CHILDACCESS(usize, Vec<(String, Vec<String>)>);

/// start, name, vars_scope
pub struct FUNCTIONCALL(usize, String, usize);

/// lhs(start, str), rhs(start, str)
pub struct EQUATION((usize, String), (usize, String));

/// scope, name, [(parent_scope, parents)]
pub struct CLASS(usize, String, Vec<(String, String)>);

/// scope, name, return_type, [args], name_pos
pub struct FUNCTION(usize, String, String, Vec<(String, String)>, usize);

/// scope, [imports as args], [args]
pub struct LAMBDA(usize, Vec<(String, String)>, Vec<(String, String)>);

/// parent_scope, name, type
pub struct OBJECT(usize, String, String);

impl CHILDACCESS {}
impl CLASS {}
impl FUNCTION {
    /// name, return_type, args_types
    pub fn get_signature(&self) -> (&String, &String, Vec<String>) {
        return (
            &self.1,
            &self.2,
            self.3.iter().map(|x| x.0.clone()).collect(),
        );
    }
}
impl FUNCTIONCALL {}
impl LAMBDA {}
impl OBJECT {}

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

/////////////////////////////////////////////////////////////////////////////////////////////
/// START ///////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////

pub fn evaluate(project_path: &String, all_files: &Vec<&String>) {
    let mut intense_info = Vec::new();
    for file in all_files {
        if let Some(info) = language_file_intense_extract(file) {
            intense_info.push(info);
        }
    }

    let scoped_data = create_scope_availability(project_path, all_files, &intense_info);
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// READING FILES ///////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////

fn language_file_intense_extract(
    file_path: &String,
) -> Option<(
    Vec<SCOPE>,
    Vec<CHILDACCESS>,
    Vec<EQUATION>,
    Vec<CLASS>,
    Vec<FUNCTION>,
    Vec<FUNCTIONCALL>,
    Vec<LAMBDA>,
    Vec<OBJECT>,
)> {
    println!();
    println!("{}", file_path);

    let file_text = match fs::read_to_string(file_path) {
        Ok(f) => f,
        Err(_) => String::new(),
    };

    // scopes evaluation

    // start, end, parent
    let mut scope_entries: Vec<SCOPE> = Vec::new();
    let mut scope_stack: Vec<usize> = Vec::new();
    // region create file level scope

    scope_entries.push(SCOPE(0, file_text.len(), usize::MAX, 3, file_text.clone()));
    scope_stack.push(0);

    for (c_i, c) in file_text.chars().enumerate() {
        if c == '(' || c == '{' || c == '[' {
            scope_entries.push(SCOPE(
                c_i,
                0,
                if let Some(&parent_idx) = scope_stack.last() {
                    parent_idx
                } else {
                    usize::MAX
                },
                match c {
                    '(' => 0,
                    '{' => 1,
                    '[' => 2,
                    _ => 5,
                },
                "".to_string(),
            ));
            let new_scope_idx = scope_entries.len() - 1;
            scope_stack.push(new_scope_idx);
        } else if c == ')' || c == '}' || c == ']' {
            if let Some(scope_idx) = scope_stack.pop() {
                scope_entries[scope_idx].1 = c_i;
            } else {
                eprintln!("Unmatched closing brace at {}", c_i);
            }
        }
    }
    drop(scope_stack);
    let scopes_len = scope_entries.len();
    for i in 0..scopes_len {
        let s = scope_entries.get(i).unwrap();
        scope_entries[i].4 = file_text[s.0 + 1..s.1].chars().collect();
    }

    // -------------------------------------------------------------------------------------------------------//
    // MATCHING ALL THE PATTERNS TO NARROW DOWN SEARCH FOR ALL THE THINGS
    // -------------------------------------------------------------------------------------------------------//
    let access_children_regex = build_regex_vec_from_res!(get_regex_access_child(file_path));
    let function_call_regex = build_regex_vec_from_res!(get_regex_function_call(file_path));
    let assignments_regex = build_regex_vec_from_res!(get_regex_assignments(file_path));
    let class_regex = build_regex_vec_from_res!(get_regex_class(file_path));
    let funs_regex = build_regex_vec_from_res!(get_regex_fun(file_path));
    // let interfaces_regex = build_regex_vec_from_res!(get_regex_interface(file_path));
    let lambdas_regex = build_regex_vec_from_res!(get_regex_lambda(file_path));
    let objs_regex = build_regex_vec_from_res!(get_regex_object(file_path));

    let mut access_children_entries: Vec<CHILDACCESS> = Vec::new();
    for a in access_children_regex {
        for caps in a.find_iter(&file_text) {
            let re = regex::Regex::new(r"\.|->").unwrap();
            let access_rout = re
                .split(caps.as_str().trim())
                .map(|s| (s.to_string(), vec![]))
                .collect();
            let res = CHILDACCESS(caps.start(), access_rout);
            access_children_entries.push(res);
        }
        println!();
    }
    let mut equation_entries: Vec<EQUATION> = Vec::new();
    for a in assignments_regex {
        for caps in a.captures_iter(&file_text) {
            println!();
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
                    let rhs_with_colon = equation
                        .chars()
                        .skip(eq_pos + 1)
                        .collect::<String>()
                        .trim()
                        .to_string();
                    let rhs = rhs_with_colon[..rhs_with_colon.len() - 1]
                        .trim()
                        .to_string();
                    let mut accessed_lhs: Option<usize> = None;
                    let mut accessed_rhs: Option<usize> = None;
                    let rhs_offset = equation.find(&rhs_with_colon).unwrap();
                    access_children_entries
                        .iter()
                        .enumerate()
                        .for_each(|(i, x)| {
                            if x.0 == eq_match.start() {
                                accessed_lhs = Some(i);
                            }
                            if x.0 == eq_match.start() + rhs_offset {
                                accessed_rhs = Some(i);
                            }
                        });
                    equation_entries.push(EQUATION(
                        (eq_match.start(), lhs),
                        (eq_match.start() + rhs_offset, rhs),
                    ));
                }
            }
        }
    }
    let mut class_entries: Vec<CLASS> = Vec::new();
    for c in class_regex {
        for caps in c.captures_iter(&file_text) {
            // caps.iter().for_each(|x| print!("{x:?}"));
            if let Some(class_def_m) = &caps.get(0) {
                let mut class_name = "".to_string();
                let mut class_parents = vec![];

                let class_def = class_def_m.as_str();
                if let Some(parents_def_pos) = class_def.find(':') {
                    let class_part = class_def[..parents_def_pos]
                        .trim()
                        .strip_prefix("class")
                        .unwrap()
                        .to_string();
                    class_name = class_part.trim().to_string();
                    let c_parents = &caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    class_parents = extract_args(c_parents.to_string());
                } else {
                    let class_part = class_def[..class_def.len() - 1].trim().to_string();
                    if let Some(c_name) = class_part.strip_prefix("class") {
                        class_name = c_name.trim().to_string();
                    }
                }

                let scope_start_pos = class_def_m.end();
                let class_scope = scope_entries
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.0 == scope_start_pos - 1)
                    .find_map(|x| {
                        if x.1 .0 == scope_start_pos - 1 {
                            return Some(x.0);
                        }
                        None
                    })
                    .unwrap();
                class_entries.push(CLASS(class_scope, class_name, class_parents));
            } else {
                // eprintln!("couldn't parse for class\n{}", )
            }
        }
    }
    let mut function_entries: Vec<FUNCTION> = Vec::new();
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
                    let args = extract_args(inside_b);

                    let scope_start_pos = m.end();

                    let fun_scope = scope_entries
                        .iter()
                        .enumerate()
                        .filter(|(_, x)| x.0 == scope_start_pos - 1)
                        .find_map(|x| {
                            if x.1 .0 == scope_start_pos - 1 {
                                return Some(x.0);
                            }
                            None
                        })
                        .unwrap();
                    function_entries.push(FUNCTION(
                        fun_scope,
                        name,
                        type_,
                        args,
                        m.start() + type_name_separator_pos + 1,
                    ));
                }
            });
        }
    }
    let mut lambda_entries: Vec<LAMBDA> = Vec::new();
    for l in lambdas_regex {
        for caps in l.captures_iter(&file_text) {
            if let Some(def_match) = caps.get(0) {
                let def_str = def_match.as_str();
                let imports_end_pos = def_str.find("]").unwrap();
                let imports_str = def_str[1..imports_end_pos].to_string();
                let imports = extract_args(imports_str);
                let args_start_pos = def_str.find("(").unwrap();
                let args_end_pos = def_str.rfind(")").unwrap();
                let args_str = def_str[args_start_pos + 1..args_end_pos].to_string();
                let args = extract_args(args_str);

                let scope_start_pos = def_match.end();
                let fun_scope = scope_entries
                    .iter()
                    .enumerate()
                    .filter(|(_, x)| x.0 == scope_start_pos - 1)
                    .find_map(|x| {
                        if x.1 .0 == scope_start_pos - 1 {
                            return Some(x.0);
                        }
                        None
                    })
                    .unwrap();
                lambda_entries.push(LAMBDA(fun_scope, imports, args));
            }
        }
    }
    let mut function_call_entries: Vec<FUNCTIONCALL> = Vec::new();
    for f in function_call_regex {
        for caps in f.find_iter(&file_text) {
            let mut is_declaration = false;
            for f_n in &function_entries {
                if caps.start() == f_n.4 {
                    is_declaration = true;
                    break;
                }
            }
            if is_declaration {
                continue;
            }
            if let Some(vars_scope) = scope_entries.iter().enumerate().find_map(|(s_i, s)| {
                if s.0 == caps.end() - 1 {
                    return Some(s_i);
                }
                return None;
            }) {
                let fn_name = caps.as_str()[..caps.as_str().len() - 1].chars().collect();
                function_call_entries.push(FUNCTIONCALL(caps.start(), fn_name, vars_scope));
            } else {
                println!("error in finding vars_scope for query :{:?}", caps);
            }
        }
    }
    let mut object_entries: Vec<OBJECT> = Vec::new();
    for o in objs_regex {
        for caps in o.captures_iter(&file_text) {
            if let Some(_match) = caps.get(0) {
                let match_str = _match.as_str()[.._match.as_str().len() - 1]
                    .trim()
                    .to_string();
                if let Some(type_name_space_pos) = match_str.rfind(" ") {
                    let type_str = match_str[..type_name_space_pos].trim().to_string();
                    if type_str == "public".to_string() || type_str == "private".to_string() {
                        continue;
                    }
                    let name = match_str[type_name_space_pos..].trim().to_string();
                    let parent_scope = find_parent(&_match.start(), &scope_entries);
                    object_entries.push(OBJECT(parent_scope, name, type_str));
                }
            }
        }
    }

    log_entries("scopes", &scope_entries);
    log_entries("accessing children", &access_children_entries);
    log_entries("equations", &equation_entries);
    log_entries("classes", &class_entries);
    log_entries("functions", &function_entries);
    log_entries("functions calls", &function_call_entries);
    log_entries("lambdas", &lambda_entries);
    log_entries("objects", &object_entries);
    Some((
        scope_entries,
        access_children_entries,
        equation_entries,
        class_entries,
        function_entries,
        function_call_entries,
        lambda_entries,
        object_entries,
    ))
}

fn extract_args(args_str: String) -> Vec<(String, String)> {
    let mut comma_pos = Vec::new();
    args_str.chars().enumerate().for_each(|(i, c)| {
        if c == ',' {
            comma_pos.push(i);
        }
    });
    let mut prev_comma_pos = 0 as usize;
    let mut args_type_and_names = Vec::new();
    comma_pos.iter().for_each(|p| {
        args_type_and_names.push(args_str[prev_comma_pos..p.clone()].trim().to_string());
        prev_comma_pos = p + 1;
    });
    args_type_and_names.push(args_str[prev_comma_pos..].trim().to_string());
    let mut args: Vec<(String, String)> = Vec::new();
    args_type_and_names.iter().for_each(|t_n| {
        if let Some(sep_pos) = t_n.rfind(' ') {
            let arg_type = t_n[..sep_pos].trim().to_string();
            let arg_name = t_n[sep_pos + 1..].trim().to_string();
            args.push((arg_type, arg_name));
        }
    });
    args
}

fn find_parent(start_pos: &usize, scope_entries: &Vec<SCOPE>) -> usize {
    let mut parent_scope = 0;
    for (s_i, s) in scope_entries.iter().enumerate() {
        if *start_pos > s.0 && s.0 > scope_entries[parent_scope].0 {
            parent_scope = s_i;
        }
    }
    return parent_scope;
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// ANALYZE INTENSE DATA ////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////

pub fn create_scope_availability(
    project_path: &String,
    all_files: &Vec<&String>,
    files_data: &Vec<(
        Vec<SCOPE>,
        Vec<CHILDACCESS>,
        Vec<EQUATION>,
        Vec<CLASS>,
        Vec<FUNCTION>,
        Vec<FUNCTIONCALL>,
        Vec<LAMBDA>,
        Vec<OBJECT>,
    )>,
) -> (
    HashMap<usize, Vec<(String, usize)>>,
    HashMap<usize, HashMap<usize, Vec<(usize, usize)>>>,
    HashMap<usize, HashMap<usize, HashMap<String, StatefulClassConnection>>>,
) {
    // file -> [files]
    let imported_files: HashMap<usize, Vec<usize>> = read_all_imports(project_path, all_files);
    // file -> scope -> query -> class_connection
    let mut scoped_connectable_s: HashMap<
        usize,
        HashMap<usize, HashMap<String, StatefulClassConnection>>,
    > = HashMap::new();
    // file -> scope -> [(file, scope)]
    let mut accessible_scopes: HashMap<usize, HashMap<usize, Vec<(usize, usize)>>> = HashMap::new();
    // file -> [(class_name, scope)]
    let mut custom_classes: HashMap<usize, Vec<(String, usize)>> = HashMap::new();

    let mut scope_parents: HashMap<usize, HashMap<usize, Vec<(usize, usize)>>> = HashMap::new();

    for (file, (scopes, _, _, classes, functions, _, lambdas, objects)) in
        files_data.iter().enumerate()
    {
        for c in classes {
            let c_scope = c.0;
            let c_p_scope = scopes[c_scope].2;
            custom_classes
                .entry(file)
                .or_insert_with(Vec::new)
                .push((c.1.clone(), c.0));

            scoped_connectable_s
                .entry(file)
                .or_insert_with(HashMap::new)
                .entry(c_p_scope)
                .or_insert_with(HashMap::new)
                .insert(
                    c.1.clone(),
                    StatefulClassConnection::Connected(file, custom_classes[&file].len() - 1),
                );
        }
        for f in functions {
            let f_scope = f.0;
            let f_vars_scope = find_next_scope(&f.4, scopes);
            accessible_scopes
                .entry(file)
                .or_insert_with(HashMap::new)
                .entry(f_scope)
                .or_insert_with(Vec::new)
                .push((file, f_vars_scope));
            let f_p_scope = scopes[f_scope].2;
            scoped_connectable_s
                .entry(file)
                .or_insert_with(HashMap::new)
                .entry(f_p_scope)
                .or_insert_with(HashMap::new)
                .insert(
                    f.1.clone(),
                    StatefulClassConnection::Undiscovered(f.2.clone()),
                );
            for (arg_t, arg) in &f.3 {
                scoped_connectable_s
                    .entry(file)
                    .or_insert_with(HashMap::new)
                    .entry(f_scope)
                    .or_insert_with(HashMap::new)
                    .insert(
                        arg.clone(),
                        StatefulClassConnection::Undiscovered(arg_t.clone()),
                    );
            }
        }
        for f in lambdas {
            let f_scope = f.0;

            for (arg_t, arg) in &f.2 {
                scoped_connectable_s
                    .entry(file)
                    .or_insert_with(HashMap::new)
                    .entry(f_scope)
                    .or_insert_with(HashMap::new)
                    .insert(
                        arg.clone(),
                        StatefulClassConnection::Undiscovered(arg_t.clone()),
                    );
            }
        }
        for o in objects {
            let o_p_scope = o.0;
            scoped_connectable_s
                .entry(file)
                .or_insert_with(HashMap::new)
                .entry(o_p_scope)
                .or_insert_with(HashMap::new)
                .insert(
                    o.1.clone(),
                    StatefulClassConnection::Undiscovered(o.2.clone()),
                );
        }
        for (si, _) in scopes.iter().enumerate() {
            let mut c = si;
            while c != usize::MAX {
                scope_parents
                    .entry(file)
                    .or_insert_with(HashMap::new)
                    .entry(si)
                    .or_insert_with(Vec::new)
                    .push((file, c));
                c = scopes.get(c).unwrap().2;
            }
        }
    }

    for (file, (scopes, _, _, _, _, _, _, _)) in files_data.iter().enumerate() {
        for (s_i, _) in scopes.iter().enumerate() {
            for i in imported_files.get(&file).unwrap_or(&vec![]) {
                accessible_scopes
                    .entry(file)
                    .or_insert_with(HashMap::new)
                    .entry(s_i)
                    .or_insert_with(Vec::new)
                    .push((i.clone(), 0));
            }
        }
    }
    for (file, scopes) in scope_parents {
        for (s, sp) in scopes {
            for parent in sp {
                accessible_scopes
                    .entry(file)
                    .or_insert_with(HashMap::new)
                    .entry(s)
                    .or_insert_with(Vec::new)
                    .push(parent);
            }
        }
    }

    // TODO: evaluate (scopes) and [scopes]
    // for (file, (scopes, access, equation, _, _, _, _, _)) in files_data.iter().enumerate() {}

    log_hashmap("imported files", &imported_files);
    log_nested_hashmap("accessible scopes", &accessible_scopes);
    log_deeply_nested_hashmap("scoped connectable(s)", &scoped_connectable_s);

    return (custom_classes, accessible_scopes, scoped_connectable_s);
}

fn find_next_scope(pos: &usize, scopes: &Vec<SCOPE>) -> usize {
    let mut res = 0;
    let mut res_dist = usize::MAX;
    for (s_i, s) in scopes.iter().enumerate() {
        if s.0 > *pos && s.0 - pos < res_dist {
            res_dist = s.0 - pos;
            res = s_i;
        }
    }
    return res;
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// LOGGING /////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////////////////////////////////////////////////

impl fmt::Display for SCOPE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
                f,
                "----------------------------------------------------\n\t{3}{0}<-->{1}{4}\n\t||parent:{2} {5}",
                self.0, self.1, self.2, match self.3 {
                    0 => '(',
                    1 => '{',
                    2 => '[',
                    3 => '.',
                    _ => '!',
                },match self.3 {
                    3 => '.',
                    2 => ']',
                    1 => '}',
                    0 => ')',
                    _ => '!',
                },
                match self.3 {
                    0 => self.4.clone(),
                    _ => "".to_string()
                }
            )
    }
}
impl fmt::Display for CHILDACCESS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "----------------------------------------------------\n\tstart:{}\n\taccess route:{:?}",
            self.0, self.1
        )
    }
}
impl fmt::Display for EQUATION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "----------------------------------------------------\n\t({}){} =({}){}",
            (self.0).0,
            (self.0).1,
            (self.1).0,
            (self.1).1
        )
    }
}
impl fmt::Display for CLASS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "----------------------------------------------------\n\t{1} : {2:?}\n\t||scope:{0:?}",
            self.0, self.1, self.2
        )
    }
}
impl fmt::Display for FUNCTION {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "----------------------------------------------------\n\t{0:?}||({4})name:{1}||({2:?})\n\t||scope:{3:?}", self.2, self.1, self.3, self.0, self.4)
    }
}
impl fmt::Display for FUNCTIONCALL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "----------------------------------------------------\n\t({}){} with vars_scope ({})",
            self.0, self.1, self.2
        )
    }
}
impl fmt::Display for LAMBDA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "----------------------------------------------------\n\tscope:{:?}\n\timports as args:{:?}\n\targs:{:?}",
                self.0, self.1, self.2)
    }
}
impl fmt::Display for OBJECT {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "----------------------------------------------------\n\t{2:?}|| {1}\n\t||parent:{0}",
            self.0, self.1, self.2
        )
    }
}
fn log_entries<T: fmt::Display>(label: &str, entries: &Vec<T>) {
    // println!();
    println!("{}", label);
    entries.iter().enumerate().for_each(|(i, entry)| {
        print!("({i})");
        println!("{}", entry);
    });
    println!("=======================================================");
}

/// Logs a HashMap with a specific format.
/// Each entry in the HashMap is logged with its key and value(s).
fn log_hashmap<K: Debug, V: Debug>(title: &str, map: &HashMap<K, V>) {
    println!("=== {} ===", title);
    for (key, value) in map {
        println!("{:?} -> {:?}", key, value);
    }
    println!();
}

/// Logs a nested HashMap with additional formatting for nested levels.
fn log_nested_hashmap<K: Debug, SK: Debug, SV: Debug>(
    title: &str,
    map: &HashMap<K, HashMap<SK, SV>>,
) {
    println!("=== {} ===", title);
    for (key, inner_map) in map {
        println!("{:?} -> {{", key);
        for (inner_key, inner_value) in inner_map {
            println!("\t{:?} -> {:?}", inner_key, inner_value);
        }
        println!("}}");
    }
    println!();
}

/// Logs a deeply nested HashMap.
fn log_deeply_nested_hashmap<K: Debug, SK: Debug, QK: Debug, QV: Debug>(
    title: &str,
    map: &HashMap<K, HashMap<SK, HashMap<QK, QV>>>,
) {
    println!("=== {} ===", title);
    for (key, inner_map) in map {
        println!("{:?} -> {{", key);
        for (inner_key, query_map) in inner_map {
            println!("\t{:?} -> {{", inner_key);
            for (query_key, query_value) in query_map {
                println!("\t\t{:?} -> {:?}", query_key, query_value);
            }
            println!("\t}}");
        }
        println!("}}");
    }
    println!();
}
