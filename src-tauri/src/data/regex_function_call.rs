use crate::*;

pub static FUNCTION_CALL: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        concat!(
            word!(), possible_spaces!(), r"\("
        )
        ],
        "cpp" => &[
        concat!(
            word!(), possible_spaces!(), r"\("
        )
    ],
};
