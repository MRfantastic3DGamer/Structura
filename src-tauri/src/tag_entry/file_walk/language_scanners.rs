// pub trait LanguageHandler {
//     fn scan(path: &String);
// }

// pub struct LanguageScanner<HANDLER: LanguageHandler> {
//     pub regex_class: String,
//     handler: HANDLER,
// }

// pub enum FileComponent {
//     Assignment {
//         lhs: Vec<String>,
//         rhs: Vec<String>,
//     },
//     Class {
//         name: String,
//         parents: String,
//     },
//     Function {
//         name: String,
//         return_type: String,
//         args: Vec<[String; 2]>,
//     },
//     Interface {
//         name: String,
//     },
//     Lambda {},
//     Object {
//         name: String,
//         obj_class: String,
//     },
// }
