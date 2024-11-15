use crate::*;

pub static ACCESS_CHILD: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        concat!(
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+"
        )
        ],
        "cpp" => &[
        concat!(
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+"
        )
    ],
};

// word = [a-zA-Z_][a-zA-Z0-9_]*
// fun_call / obj = \([^)]*\)|[a-zA-Z_][a-zA-Z0-9_]*
// access child = \.|->
