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

pub struct UnconfusableChars<I: Iterator<Item=char>>(
    FlatMap<DecompositionsToPrototypeChars<I>, DecomposeSingleChar, fn(char) -> Decompositions<option::IntoIter<char>>>
);

impl<I: Iterator<Item=char>> Iterator for UnconfusableChars<I> {
    type Item = char;

    fn next(&mut self) -> Option<char> {
        self.0.next()
    }
}

impl<I: Iterator<Item=char>> UnconfusableChars<I> {
    fn new(source: I) -> UnconfusableChars<I> {
        UnconfusableChars(
            source
                .nfd()
                .flat_map(PrototypeCharsIterator::new as fn(char) -> PrototypeCharsIterator)
                .flat_map(|x| Some(x).into_iter().nfd()) )
    }
}

pub trait UnicodeUnconfuse<I: Iterator<Item=char>> {
    fn unconfusable_chars(self) -> UnconfusableChars<I>;
}

impl<I: Iterator<Item=char>> UnicodeUnconfuse<I> for I {
    fn unconfusable_chars(self) -> UnconfusableChars<I> {
        UnconfusableChars::new(self)
    }
}

impl<'a> UnicodeUnconfuse<Chars<'a>> for &'a str {
    fn unconfusable_chars(self) -> UnconfusableChars<Chars<'a>> {
        UnconfusableChars::new(self.chars())
    }
}

#[cfg(test)]
mod tests {
    use super::UnicodeUnconfuse;

    #[test]
    fn it_works() {
        assert_eq!("\u{0441}".unconfusable_chars().collect::<String>(), "\u{0063}");
        assert_eq!("𝔭𝒶ỿ𝕡𝕒ℓ".unconfusable_chars().collect::<String>(), "paypal");
    }
}
