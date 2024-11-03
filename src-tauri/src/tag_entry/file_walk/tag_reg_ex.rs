use regex::Regex;

enum TagType {
    Class,
    Function {
        input_types: [u128; 100], // class numbers
        output_type: u128,        // class number
    },
    Object {
        class: u128, // class number
    },
}

enum TagScope {
    LocalScope, // if the entry is defined inside {} or something
    Public,
    Private,
    Protected,
}

enum TagEntry {
    Cpp {
        tag_name: String,
        line: u128,
        tag_type: TagType,
        scope: TagScope,
    },
}

fn search_all_objects(language: String, code: String) {
    let object = Regex::new(r"[a-zA-Z_][a-zA-Z0-9_]\s+[a-zA-Z_][a-zA-Z0-9_][=;]").unwrap();
    let objs = object.find_iter(&code);
    for obj in objs {
        println!("{}", obj.as_str());
    }
}
