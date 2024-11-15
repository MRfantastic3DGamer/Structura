use crate::*;

pub static ASSIGNMENT: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        // simple assignment
        r"(\w+)\s*=\s*.*;",
        concat!(
            // access child
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+",
            // assign
            possible_spaces!(),r"=",possible_spaces!(),
            // access child
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+",
            // end
            possible_spaces!(), r";"
        ),
    ],
    "cpp" => &[
        // simple assignment
        r"(\w+)\s*=\s*.*;",
        concat!(
            // access child
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+",
            // assign
            possible_spaces!(),r"=",possible_spaces!(),
            // access child
            fun_call_or_word!(), possible_spaces!() , "(", possible_spaces!(), r"\.", possible_spaces!(), fun_call_or_word!(), or!(), r"->", possible_spaces!(), fun_call_or_word!(), possible_spaces!(), ")+",
            // end
            possible_spaces!(), r";"
        ),
    ],
};
