use crate::*;

pub static LAMBDA: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        // concat!(
        //     r"[]", anything_inside_brackets!(), possible_spaces!(), r"(->|)", possible_spaces!(), word!(), possible_spaces!(), r"{"
        // ),
    ],
    "cpp" => &[
        // concat!(
        //     r"[]", anything_inside_brackets!(), possible_spaces!(), r"(->|)", possible_spaces!(), word!(), possible_spaces!(), r"{"
        // ),
    ],
};
