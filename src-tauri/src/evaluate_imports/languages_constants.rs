use std::path::Path;

use phf::phf_map;

/// data types for each language
pub static DATA_TYPES: phf::Map<&'static str, &'static [&'static str]> = phf_map! {
    "c" => &[
        "void",
        "char",
        "signed char",
        "unsigned char",
        "short",
        "short int",
        "signed short",
        "signed short int",
        "unsigned short",
        "unsigned short int",
        "int",
        "signed",
        "signed int",
        "unsigned",
        "unsigned int",
        "long",
        "long int",
        "signed long",
        "signed long int",
        "unsigned long",
        "unsigned long int",
        "long long",
        "long long int",
        "signed long long",
        "signed long long int",
        "unsigned long long",
        "unsigned long long int",
        "float",
        "double",
        "long double",
    ],
    "cpp" => &[
        "void",
        "char",
        "signed char",
        "unsigned char",
        "short",
        "short int",
        "signed short",
        "signed short int",
        "unsigned short",
        "unsigned short int",
        "int",
        "signed",
        "signed int",
        "unsigned",
        "unsigned int",
        "long",
        "long int",
        "signed long",
        "signed long int",
        "unsigned long",
        "unsigned long int",
        "long long",
        "long long int",
        "signed long long",
        "signed long long int",
        "unsigned long long",
        "unsigned long long int",
        "float",
        "double",
        "long double",
    ],
};

pub fn get_language(file_path: &String) -> Option<&str> {
    Path::new(file_path)
        .extension()
        .and_then(|ext| ext.to_str())
}

pub fn get_data_types(file_path: &String) -> Option<&&[&str]> {
    // Determine the extension of the file.
    if let Some(extension) = get_language(file_path) {
        // Match the file extension to a language and return the data types from the DATA_TYPES map.
        match extension {
            "c" => DATA_TYPES.get("c"),
            "cpp" | "cc" | "cxx" | "h" | "hpp" => DATA_TYPES.get("cpp"),
            _ => None, // Return None if the language is not found.
        }
    } else {
        None // Return None if there is no extension.
    }
}
