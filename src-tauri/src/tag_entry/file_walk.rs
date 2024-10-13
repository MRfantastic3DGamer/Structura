use std::{
    fs::File,
    io::{self, BufRead},
    path::Path,
};

use super::{ScopeEntry, TagEntry};

pub fn file_walk(file_path: &String, file_tags: &Vec<TagEntry>) -> Vec<ScopeEntry> {
    indentation_walk(file_path, file_tags)
}

fn indentation_walk(file_path: &String, file_tags: &Vec<TagEntry>) -> Vec<ScopeEntry> {
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
            return Vec::new();
        }
    }

    // Read the file line by line
    let reader = io::BufReader::new(file);

    let mut scope_entries: Vec<ScopeEntry> = Vec::new();
    let mut scope_stack: Vec<usize> = Vec::new(); // Stack to track current open scope indexes
    let mut line_num: u128 = 0;

    // Traverse through the lines
    for (line_index, line) in reader.lines().enumerate() {
        let line_content = line.unwrap();
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
                        .push(new_scope_idx as u128);
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

    scope_entries
}

fn brackets_walk() {}
