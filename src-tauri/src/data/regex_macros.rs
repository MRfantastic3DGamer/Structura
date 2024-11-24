#[macro_export]
macro_rules! word {
    () => {
        r"[a-zA-Z_][a-zA-Z0-9_]*"
    };
}

#[macro_export]
macro_rules! possible_spaces {
    () => {
        r"\s*"
    };
}

#[macro_export]
macro_rules! or {
    () => {
        r"|"
    };
}

#[macro_export]
macro_rules! either_or {
    ($first:expr $(, $rest:expr)*) => {
        concat!("(", $first $(, "|", $rest)*, ")")
    };
}

#[macro_export]
macro_rules! in_bracket {
    ($($item:expr),+) => {
        concat!(r"\(", $($item),+, r"\)")
    };
}

#[macro_export]
macro_rules! anything_inside_brackets {
    () => {
        r"\([^)(]*\)"
    };
}
#[macro_export]
macro_rules! fun_call {
    () => {
        concat!(word!(), anything_inside_brackets!())
    };
}
// \(([^)]*)\)
#[macro_export]
macro_rules! fun_call_or_word {
    () => {
        concat!(r"(", fun_call!(), or!(), word!(), r")")
    };
}
