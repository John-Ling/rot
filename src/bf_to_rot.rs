use std::fs;
use std::env;
use std::collections::HashMap;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];

    let token_map: HashMap<char, &str> = HashMap::from([
        ('<', "767"),
        ('>', "676"),
        ('+', "67"),
        ('-', "76"),
        ('.', "667"),
        (',', "776"),
        ('[', "6677"),
        (']', "7766"),
    ]);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    let mut output_lines: Vec<String> = Vec::new();

    for line in contents.lines() {
        let encoded_tokens: Vec<&str> = line
            .chars()
            .filter_map(|c| token_map.get(&c).copied())
            .collect();

        output_lines.push(encoded_tokens.join(" "));
    }

    let output_contents = output_lines.join("\n");

    let output_path = format!(
        "{}.rot",
        std::path::Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(file_path)
    );

    fs::write(&output_path, output_contents).expect("Should have been able to write the file");

    println!("Written to {}", output_path);
}
