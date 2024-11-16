use crate::*;

pub static CLASS: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        concat!(
            r"class",r"\s+", word!(), r"(?:\s*:\s*([\w\s,]+))?", possible_spaces!(), r"\{",
        ),
    ],
    "cpp" => &[
        concat!(
            r"class",r"\s+", word!(), r"(?:\s*:\s*([\w\s,]+))?", possible_spaces!(), r"\{",
        ),
    ],
};
