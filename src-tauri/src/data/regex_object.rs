pub static OBJECT: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        r"(\w[\w\s\*&:<>]*)\s+(\w+)\s*=.*;",
    ],
    "cpp" => &[
        r"(\w[\w\s\*&:<>]*)\s+(\w+)\s*=.*;",
    ],
};
