use crate as soroban_sdk;
use soroban_sdk::{
    contracttype,
};

#[contracttype]
pub enum Udt {
    Aaa(u32, bool),
}
