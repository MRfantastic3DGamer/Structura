use crate::*;

pub static LAMBDA: phf::Map<&'static str, &'static [&'static str]> = phf::phf_map! {
    "c" => &[
        concat!(
            r"\[(|(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r")(", possible_spaces!() ,r",", possible_spaces!() ,r"(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r"))*)\]\((|(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r")(", possible_spaces!() ,r",", possible_spaces!() ,r"(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r"))*)\)", possible_spaces!() ,r"\{",
        )
    ],
    "cpp" => &[
        concat!(
            r"\[(|(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r")(", possible_spaces!() ,r",", possible_spaces!() ,r"(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r"))*)\]\((|(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r")(", possible_spaces!() ,r",", possible_spaces!() ,r"(\&*\**", possible_spaces!() ,r"(", word!() ,r"\s+)+", word!() ,r"))*)\)", possible_spaces!() ,r"\{",
        )
    ],
};
