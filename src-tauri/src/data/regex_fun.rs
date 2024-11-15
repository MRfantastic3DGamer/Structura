use crate::*;

pub static FUNCTION: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        concat!(
            r"(\w[\w\s\*&:<>]*)\s+(\w+)", possible_spaces!(), anything_inside_brackets!(),
        ),
    ],
    "cpp" => &[
        concat!(
            r"(\w[\w\s\*&:<>]*)\s+(\w+)", possible_spaces!(), anything_inside_brackets!(),
        ),
    ],
};
