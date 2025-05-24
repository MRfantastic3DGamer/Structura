use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;
use serde_json::json;

use crate::tag_entry;
use crate::evaluate_imports::{self, ProgramTag};
use crate::intense_evaluation;
use crate::intense_evaluation::StatefulClassConnection;
use crate::tag_entry::{TagEntry, ScopeEntry, ClassEntry, FunctionEntry, ObjectEntry};

#[derive(Default, Serialize, Clone)]
pub struct ProjectData {
    pub tags_data: Vec<TagEntry>,
    pub all_files: Vec<String>,
    pub hard_data: HashMap<String, (Vec<ScopeEntry>, Vec<ClassEntry>, Vec<FunctionEntry>, Vec<ObjectEntry>)>,
    pub raw_imports: HashMap<usize, Vec<usize>>,
    pub all_tags: HashMap<usize, Vec<ProgramTag>>,
    pub children_tags: HashMap<(usize, usize), Vec<(usize, usize)>>,
    pub custom_classes: HashMap<usize, Vec<(String, usize)>>,
    pub accessible_scopes: HashMap<usize, HashMap<usize, Vec<(usize, usize)>>>,
    pub scoped_connectables: HashMap<usize, HashMap<usize, HashMap<String, StatefulClassConnection>>>,
}

lazy_static! {
    static ref PROJECT_DATA: Mutex<Option<ProjectData>> = Mutex::new(None);
}

pub fn get_project_data() -> Option<ProjectData> {
    PROJECT_DATA.lock().unwrap().clone()
}

pub fn set_project_data(data: ProjectData) {
    *PROJECT_DATA.lock().unwrap() = Some(data);
}

pub fn clear_project_data() {
    *PROJECT_DATA.lock().unwrap() = None;
}

pub async fn create_project_data(
		project_path: String,
		tags_path: String,
) -> ProjectData {

	//

	let tags_result = match tag_entry::get_tags_data(tags_path) {
			Ok(res) => res,
			Err(_) => Vec::new(),
	};
	// F
	let all_files = tag_entry::get_all_files(&tags_result)
			.into_iter()
			.cloned()
			.collect::<Vec<String>>();
	let all_files_refs = all_files.iter().collect::<Vec<&String>>();
	let hard_data_ref =
			tag_entry::get_all_hard_data(&all_files_refs, &tags_result).await;
	let hard_data: HashMap<String, (Vec<ScopeEntry>, Vec<ClassEntry>, Vec<FunctionEntry>, Vec<ObjectEntry>)> =
		hard_data_ref.into_iter()
			.map(|(k, v)| (k.clone(), v))
			.collect();
	let all_files_refs = all_files.iter().collect::<Vec<&String>>();
	let hard_data_refs: HashMap<&String, (Vec<ScopeEntry>, Vec<ClassEntry>, Vec<FunctionEntry>, Vec<ObjectEntry>)> =
		hard_data.iter().map(|(k, v)| (k, v.clone())).collect();
	let (raw_imports, all_tags, children_tags) = evaluate_imports::evaluate_all_hard_data(
			&project_path,
			&all_files_refs,
			hard_data_refs,
	);

	let (imports_json, tags_json, children_json) =
			evaluate_imports::jsonify_evaluated_data(&raw_imports, &all_tags, &children_tags);

	let project_hierarchy = json!([all_files, imports_json, tags_json, children_json]);

	println!("\n\n------ intense extract ------\n\n");
	let all_files_refs = all_files.iter().collect::<Vec<&String>>();
	let (custom_classes, accessible_scopes, scoped_connectable_s) =
			intense_evaluation::evaluate(&project_path, &all_files_refs);

	// Serialize data
	let intense_data_json = json!({
			"custom_classes": custom_classes,
			"accessible_scopes": accessible_scopes,
			"scoped_connectable_s": scoped_connectable_s,
	});

	//
	ProjectData {
		tags_data: tags_result,
		all_files,
		hard_data,
		raw_imports,
		all_tags,
		children_tags,
		custom_classes,
		accessible_scopes,
    scoped_connectables: scoped_connectable_s,
	}
}