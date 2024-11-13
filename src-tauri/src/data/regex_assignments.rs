pub static ASSIGNMENT: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        r"(\w+)\s*=\s*.*;",
    ],
    "cpp" => &[
        r"(\w+)\s*=\s*.*;",
    ],
};
