mod data_types;
mod regex_assignments;
mod regex_class;
mod regex_fun;
mod regex_interface;
mod regex_object;

use std::path::Path;

pub fn get_language(file_path: &String) -> Option<&str> {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

fn get_data_for_extension<'a>(
    file_path: &'a String,
    map: &'a phf::Map<&'static str, &'static [&'static str]>,
) -> (Option<&'a str>, Option<&'a &'static [&'static str]>) {
    if let Some(extension) = get_language(file_path) {
        match extension {
            "c" => (Some(extension), map.get("c")),
            "cpp" | "cc" | "cxx" | "h" | "hpp" => (Some(extension), map.get("cpp")),
            _ => (None, None),
        }
    } else {
        (None, None)
    }
}

pub fn get_data_types(file_path: &String) -> Option<&&[&str]> {
    get_data_for_extension(file_path, &data_types::DATA_TYPES).1
}

pub fn get_regex_assignments(file_path: &String) -> (Option<&str>, Option<&&[&str]>) {
    get_data_for_extension(file_path, &regex_assignments::ASSIGNMENT)
}

pub fn get_regex_class(file_path: &String) -> (Option<&str>, Option<&&[&str]>) {
    get_data_for_extension(file_path, &regex_class::CLASS)
}

pub fn get_regex_fun(file_path: &String) -> (Option<&str>, Option<&&[&str]>) {
    get_data_for_extension(file_path, &regex_fun::FUNCTION)
}

pub fn get_regex_interface(file_path: &String) -> (Option<&str>, Option<&&[&str]>) {
    get_data_for_extension(file_path, &regex_interface::INTERFACE)
}

pub fn get_regex_object(file_path: &String) -> (Option<&str>, Option<&&[&str]>) {
    get_data_for_extension(file_path, &regex_object::OBJECT)
}
