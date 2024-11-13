pub static CLASS: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        r"class\s+(\w+)(?:\s*:\s*([\w\s,]+))?",
    ],
    "cpp" => &[
        r"class\s+(\w+)(?:\s*:\s*([\w\s,]+))?",
    ],
};
