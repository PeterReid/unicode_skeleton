// To run: rustc confusables_to_data.rs && confusables_to_data > ..\src\data.rs

use std::fs::File;
use std::io::Read;
use std::u32;
use std::char;
use std::collections::BTreeMap;

fn main() {
    let mut confusables = String::new();
    File::open("confusables.txt")
        .expect("Failed to open confusables.txt")
        .read_to_string(&mut confusables)
        .expect("Failed to read confusables.txt");

    // First, put all the lines into a BTreeMap to get them unique and sorted
    let mut inputs_to_outputs = BTreeMap::new();

    for line in confusables.split("\n").skip(1) { // Skip the first line since it contains a unicode marker
        let line = if let Some(comment_begin) = line.find('#') {
                &line[..comment_begin]
            } else {
                line
            };
        if line.len() > 0 {
            let mut line_chunks = line.split(" ;\t");
            let from = line_chunks.next().expect("Failed to parse line");
            let tos = line_chunks.next().expect("Failed to parse line");

            let from = u32::from_str_radix(from, 16).expect("Failed to parse `from` as hex");
            let old = inputs_to_outputs.insert(from, tos);
            assert!(old.is_none());
        }
    }

    let mut input_and_output_indices = Vec::new();
    let mut outputs = Vec::new();
    for (from, tos) in inputs_to_outputs {
        assert!(outputs.len() < 0xffff);

        input_and_output_indices.push( (from, outputs.len() as u16) );
        for to in tos.split(" ") {
            let to = u32::from_str_radix(to, 16).expect("Failed to parse `to` as hex");
            outputs.push(char::from_u32(to).expect("Invalid codepoint"));
        }
    }

    // Ensure that no prototype contains characters that themselves need to be translated to another prototype
    for output in outputs.iter() {
        assert!(input_and_output_indices.binary_search_by_key(&(*output as u32), |x| x.0).is_err());
    }

    println!("pub static INPUT_AND_OUTPUT_INDICES: [(u32, u16); {}] = {:?};", input_and_output_indices.len(), input_and_output_indices);
    println!("pub static OUTPUTS: [char; {}] = {:?};", outputs.len(), outputs);

}