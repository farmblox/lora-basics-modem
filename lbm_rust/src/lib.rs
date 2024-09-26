#![no_std]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
use core::concat;
use core::env;
use core::include;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/smtc_bindings.rs"));
