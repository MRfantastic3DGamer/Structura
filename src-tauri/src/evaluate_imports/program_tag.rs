use super::languages_constants::get_data_types;

#[derive(Debug)]
pub enum ClassType {
    Undiscovered(String),
    Connected(usize, usize),
    DataType(usize),
}

impl ClassType {
    pub fn new(path: &str, class: String) -> ClassType {
        let mut n = ClassType::Undiscovered(class.clone());
        n.set_as_data_type(path, class);
        n
    }

    pub fn needed_class(&self) -> Option<&String> {
        if let ClassType::Undiscovered(name) = self {
            return Some(name);
        }
        return None;
    }

    pub fn set_as_data_type(&mut self, path: &str, class: String) -> bool {
        if let Some(data_types) = get_data_types(&path.to_string()) {
            for (i, t) in data_types.iter().enumerate() {
                if class == t.to_string() {
                    *self = ClassType::DataType(i);
                    return true;
                }
            }
        }
        return false;
    }

    pub fn set_class(&mut self, file: usize, t: usize) {
        *self = ClassType::Connected(file, t);
    }
}

#[derive(Debug)]
pub enum ProgramTag {
    Class {
        name: String,
        parents: Vec<ClassType>,
    },
    /// for class the representation is (file_number, tag_number)
    Function { name: String, class: ClassType },
    /// for class the representation is (file_number, tag_number)
    Object { name: String, class: ClassType },
}

impl ProgramTag {
    pub fn get_name(&self) -> &String {
        match self {
            ProgramTag::Class { name, parents: _ } => name,
            ProgramTag::Function { name, class: _ } => name,
            ProgramTag::Object { name, class: _ } => name,
        }
    }

    pub fn is_class(&self) -> bool {
        if let ProgramTag::Class {
            name: _,
            parents: _,
        } = self
        {
            return true;
        }
        return false;
    }

    pub fn needed_class(&self) -> Vec<Option<&String>> {
        match self {
            ProgramTag::Class { name: _, parents } => {
                parents.iter().map(|p| p.needed_class()).collect()
            }
            ProgramTag::Function { name: _, class } => vec![class.needed_class()],
            ProgramTag::Object { name: _, class } => vec![class.needed_class()],
        }
    }

    pub fn put_class_data(&mut self, file_tag_i: Vec<(usize, usize, usize)>) {
        match self {
            ProgramTag::Class { name: _, parents } => {
                file_tag_i.iter().for_each(|(class_i, file, tag)| {
                    parents.get_mut(*class_i).unwrap().set_class(*file, *tag);
                });
            }
            ProgramTag::Function { name: _, class } => {
                class.set_class(file_tag_i[0].1, file_tag_i[0].2)
            }
            ProgramTag::Object { name: _, class } => {
                class.set_class(file_tag_i[0].1, file_tag_i[0].2)
            }
        }
    }
}
