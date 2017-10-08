// Copyright 2017 Peter Reid. See the COPYRIGHT file at the top-level
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
pub fn confusable<A, B>(a: A, b: B) -> bool
    where A: IntoIterator<Item=char>, B: IntoIterator<Item=char>
{
    let mut skeleton_a = skeleton_chars(a);
    let mut skeleton_b = skeleton_chars(b);

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
pub struct SkeletonChars<I>(
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

/// Create an `Iterator` over the chararacters of the skeleton of the provided string.
///
/// # Examples
/// ```Rust
/// skeleton_chars("𝔭𝒶ỿ𝕡𝕒ℓ").collect::<String>(); // "paypal"
/// skeleton_chars(['𝒶', '𝒷', '𝒸']).collect::<String>();  "abc"
/// ```   
pub fn skeleton_chars<I: IntoIterator<Item=char>>(i: I) -> SkeletonChars<I::IntoIter> {
    SkeletonChars::new(i.into_iter())
}


#[cfg(test)]
mod tests {
    use super::{skeleton_chars, confusable};

    #[test]
    fn skeleton_char_cases() {
        assert_eq!(skeleton_chars("\u{0441}").collect::<String>(), "\u{0063}");
        assert_eq!(skeleton_chars("𝔭𝒶ỿ𝕡𝕒ℓ").collect::<String>(), "paypal");
        assert_eq!(skeleton_chars("ℝ𝓊𝓈𝓉").collect::<String>(), "Rust");

        assert_eq!(skeleton_chars(['𝒶', '𝒷', '𝒸']).collect::<String>(), "abc");
    }

    #[test]
    fn confusables() {
        assert!(confusable("ℝ𝓊𝓈𝓉", "Rust"));
        assert!(!confusable("ℝ𝓊𝓈", "Rust"));
        assert!(!confusable("ℝ𝓊𝓈𝓉", "Rus"));
        assert!(!confusable("Rast", "Rust"));
    }
}
