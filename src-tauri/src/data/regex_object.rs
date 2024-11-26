use crate::*;

pub static OBJECT: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        r"(\w[\w\s\*&:<>]*)\s+(\w+)\s*=.*;",
    ],
    "cpp" => &[
        concat!(
            r"(", word!(), r"\s+" , r")", "+",
            word!(), possible_spaces!(), either_or!(",", "=", ";", r"\)")
        )
    ],
};
