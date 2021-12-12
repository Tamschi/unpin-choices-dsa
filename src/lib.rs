//! A "data structures & algorithms" demo repository for a blog post. This isn't a well-structured package by itself.
//!
//! [![Zulip Chat](https://img.shields.io/endpoint?label=chat&url=https%3A%2F%2Fiteration-square-automation.schichler.dev%2F.netlify%2Ffunctions%2Fstream_subscribers_shield%3Fstream%3Dproject%252Funpin-choices-dsa)](https://iteration-square.schichler.dev/#narrow/stream/project.2Funpin-choices-dsa)
//!
//! # About the documentation
//!
//! ## Personal notes
//!
//! While much of the documentation in this library is already very verbose,
//! certain subjective comments are still indented using markdown quote blocks:
//!
//! > I'm excited to see how pinning will interact with future Rust language features.

#![doc(html_root_url = "https://docs.rs/unpin-choices-dsa/0.0.1")]
#![warn(clippy::pedantic, missing_docs)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![no_std]

#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod readme {}

extern crate alloc;

pub mod anti_pinned;
pub mod if_ready;
pub mod join_future;
pub mod pinned_pin;
pub mod pinned_pin_pins_items;
pub mod ready_or_never;

mod unchecked_tap;
