// Copyright 2017 Peter Reid. See the COPYRIGHT
// directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Transforms a unicode string by replacing unusual characters with
//! similar-looking common characters, as specified by the
//! [Unicode Standard Annex #39](http://www.unicode.org/reports/tr39/).
//! For example, "ℝ𝓊𝓈𝓉" will be transformed to "Rust".
//! This simplified string is called the "skeleton".
//!
//! ```Rust
//! use unicode_skeleton::UnicodeSkeleton;
//!
//! "ℝ𝓊𝓈𝓉".skeleton_chars().collect::<String>() // "Rust"
//! ```
//!
//! Strings are considered "confusable" if they have the same skeleton.
//! For example, "ℝ𝓊𝓈𝓉" and "Rust" are confusable.
//!
//! ```Rust
//! use unicode_skeleton::confusable;
//!
//! confusable("ℝ𝓊𝓈𝓉", "Rust") // true
//! ```
//!
//! The translation to skeletons is based on
//! Unicode Security Mechanisms for UTR #39 version 10.0.0.
extern crate unicode_normalization;

use std::char;
use std::iter::FlatMap;
use std::slice;
use std::str::Chars;
use std::option;

use unicode_normalization::Decompositions;
use unicode_normalization::UnicodeNormalization;

mod data;

enum PrototypeCharsIterator {
    One(Option<char>),
    Slice(slice::Iter<'static, char>),
}

impl PrototypeCharsIterator {
    pub fn new(c: char) -> PrototypeCharsIterator {
        if let Ok(input_index) = data::INPUT_AND_OUTPUT_INDICES.binary_search_by_key(&(c as u32), |entry| entry.0) {
            let output_index_start = data::INPUT_AND_OUTPUT_INDICES[input_index].1 as usize;
            let output_index_end = data::INPUT_AND_OUTPUT_INDICES.get(input_index+1).map(|x| x.1 as usize).unwrap_or(data::OUTPUTS.len());
            let prototype_chars = &data::OUTPUTS[output_index_start..output_index_end];
            PrototypeCharsIterator::Slice(prototype_chars.iter())
        } else {
            PrototypeCharsIterator::One(Some(c))
        }
    }
}

impl Iterator for PrototypeCharsIterator {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        match self {
            &mut PrototypeCharsIterator::One(ref mut x) => x.take(),
            &mut PrototypeCharsIterator::Slice(ref mut xs) => xs.next().map(|c| *c),
        }
    }
}

type DecompositionsToPrototypeChars<I> = FlatMap<Decompositions<I>, PrototypeCharsIterator, fn(char) -> PrototypeCharsIterator>;
type DecomposeSingleChar = Decompositions<option::IntoIter<char>>;

/// Test if two strings have the same "skeleton", and thus could be visually
/// confused for each another.
pub fn confusable<A, B, AI, BI>(a: A, b: B) -> bool
    where A: UnicodeSkeleton<AI>, B: UnicodeSkeleton<BI>, AI: Iterator<Item=char>, BI: Iterator<Item=char>
{
    let mut skeleton_a = a.skeleton_chars();
    let mut skeleton_b = b.skeleton_chars();

    loop {
        match (skeleton_a.next(), skeleton_b.next()) {
            (None, None) => {
                return true;
            }
            (a, b) => {
                if a != b {
                    return false;
                }
            }
        }
    }
}

/// An iterator over the characters of the skeleton of a unicode string.
/// This is retrieved via the `UnicodeSkeleton` trait.
pub struct SkeletonChars<I: Iterator<Item=char>>(
    FlatMap<DecompositionsToPrototypeChars<I>, DecomposeSingleChar, fn(char) -> Decompositions<option::IntoIter<char>>>
);

impl<I: Iterator<Item=char>> Iterator for SkeletonChars<I> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.0.next()
    }
}

impl<I: Iterator<Item=char>> SkeletonChars<I> {
    fn new(source: I) -> SkeletonChars<I> {
        SkeletonChars(
            source
                .nfd()
                .flat_map(PrototypeCharsIterator::new as fn(char) -> PrototypeCharsIterator)
                .flat_map(|x| Some(x).into_iter().nfd()) )
    }
}

/// Method for retrieving a `SkeletonChars` from a `str` or other `char` iterator.
pub trait UnicodeSkeleton<I: Iterator<Item=char>> {
    /// Retrieve an iterater of the characters of the provided char sequence's skeleton
    ///
    /// # Examples
    /// ```Rust
    /// "𝔭𝒶ỿ𝕡𝕒ℓ".skeleton_chars().collect::<String>(); // "paypal"
    /// ['𝒶', '𝒷', '𝒸'].iter().map(|c| *c).collect::<String>();  "abc"
    fn skeleton_chars(self) -> SkeletonChars<I>;
}

impl<I: Iterator<Item=char>> UnicodeSkeleton<I> for I {
    fn skeleton_chars(self) -> SkeletonChars<I> {
        SkeletonChars::new(self)
    }
}

impl<'a> UnicodeSkeleton<Chars<'a>> for &'a str {
    fn skeleton_chars(self) -> SkeletonChars<Chars<'a>> {
        SkeletonChars::new(self.chars())
    }
}

#[cfg(test)]
mod tests {
    use super::{UnicodeSkeleton, confusable};

    #[test]
    fn skeleton_char_cases() {
        assert_eq!("\u{0441}".skeleton_chars().collect::<String>(), "\u{0063}");
        assert_eq!("𝔭𝒶ỿ𝕡𝕒ℓ".skeleton_chars().collect::<String>(), "paypal");
        assert_eq!("ℝ𝓊𝓈𝓉".skeleton_chars().collect::<String>(), "Rust");

        assert_eq!(['𝒶', '𝒷', '𝒸'].iter().map(|c| *c).skeleton_chars().collect::<String>(), "abc");
    }

    #[test]
    fn confusables() {
        assert!(confusable("ℝ𝓊𝓈𝓉", "Rust"));
        assert!(!confusable("ℝ𝓊𝓈", "Rust"));
        assert!(!confusable("ℝ𝓊𝓈𝓉", "Rus"));
        assert!(!confusable("Rast", "Rust"));
    }
}
