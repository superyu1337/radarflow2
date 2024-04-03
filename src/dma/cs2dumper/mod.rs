#![allow(dead_code)]
#![allow(overflowing_literals)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

mod client_mod;
mod engine2_mod;
mod offsets_mod;

pub use client_mod::cs2_dumper::schemas::client_dll as client;
pub use engine2_mod::cs2_dumper::schemas::engine2_dll as engine;
pub use offsets_mod::cs2_dumper::offsets as offsets;