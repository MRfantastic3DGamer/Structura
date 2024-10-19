use serde::Serializer;

pub fn u128_as_string<S>(x: &u128, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(&x.to_string())
}

pub fn vec_u128_as_string<S>(x: &Vec<u128>, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let str_vec: Vec<String> = x.iter().map(|&n| n.to_string()).collect();
    s.serialize_some(&str_vec)
}
