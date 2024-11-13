pub static OBJECT: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        // r"class\s+(\w+)(?:\s*:\s*(\w+))?",
    ],
    "cpp" => &[
        r"(\w[\w\s\*&:<>]*)\s+(\w+)\s*=\s*.*;",
    ],
};
