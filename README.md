Unicode character "confusable" detection and "skeleton" computation, specified by the
[Unicode Standard Annex #39](http://www.unicode.org/reports/tr39/). These functions are
for working with strings that appear nearly identical once rendered, but do not
compare as equal.

[Documentation](https://docs.rs/unicode_skeleton/*/unicode_skeleton)

```rust
extern crate unicode_skeleton;

use unicode_skeleton::{skeleton_chars, confusable};

fn main() {
    assert_eq!(skeleton_chars("𝔭𝒶ỿ𝕡𝕒ℓ").collect::<String>(), "paypal");
    assert!(confusable("ℝ𝓊𝓈𝓉", "Rust"));
}
```


# crates.io

Adding the following to your `Cargo.toml` to use:

```toml
[dependencies]
unicode_skeleton = "0.1.0"
```
