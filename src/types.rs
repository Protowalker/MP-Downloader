use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Or<T, U> {
    First(T),
    Second(U),
}

pub type OrVec<T> = Or<T, Vec<T>>;
