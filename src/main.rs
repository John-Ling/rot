use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq)]
enum TokenType {
    LEFT,
    RIGHT,
    INCREMENT,
    DECREMENT,
    OUTPUT,
    INPUT,
    JUMPFWD,
    JUMPBCK,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let file_path: &str = &args[1];

    let token_map = HashMap::from([
        ("676", TokenType::RIGHT),
        ("767", TokenType::LEFT),
        ("67", TokenType::INCREMENT),
        ("76", TokenType::DECREMENT),
        ("667", TokenType::OUTPUT),
        ("776", TokenType::INPUT),
        ("6677", TokenType::JUMPFWD),
        ("7766", TokenType::JUMPBCK),
    ]);

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");

    // Tokenise
    let mut tokens: Vec<TokenType> = Vec::new();
    let lines: Vec<&str> = contents.lines().collect();
    for line in &lines {
        let words: Vec<&str> = line.split(" ").collect();
        for word in &words {
            match token_map.get(word) {
                Some(token_type) => tokens.push(*token_type),
                None => {} // Ignore any other words
            }
        }
    }

    // Build jump table
    let mut jump_table: HashMap<usize, usize> = HashMap::new();

    // Use stack to ensure each jump forward has a corresponding jump back
    // And store positions for jump forward commands
    let mut stack: Vec<usize> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        match token {
            TokenType::JUMPFWD => {
                // Push position onto stack
                stack.push(index);
            }
            TokenType::JUMPBCK => {
                match stack.pop() {
                    Some(destination) => {
                        // Add opening and closing token to jump table
                        jump_table.insert(destination, index);
                        jump_table.insert(index, destination);
                    }
                    None => {
                        panic!("Couldn't find 6677 opening token");
                    }
                }
            }
            _ => {} // Ignore all other tokens
        }
    }

    // Execute
    let mut instruction_pointer = 0;
    let mut data_pointer: usize = 0;

    let mut cells_length: usize = 30000;
    let mut cells: Vec<u8> = vec![0; cells_length];

    let mut input_buffer: Vec<u8> = Vec::new();
    let mut buffer_pos = 0;

    while instruction_pointer < tokens.len() {
        match tokens[instruction_pointer] {
            TokenType::LEFT => data_pointer = data_pointer.saturating_sub(1),
            TokenType::RIGHT => {
                data_pointer += 1;
                if data_pointer >= cells_length {
                    cells_length *= 2;
                    cells.resize(cells_length, 0);
                }
            }
            TokenType::INCREMENT => cells[data_pointer] = cells[data_pointer].wrapping_add(1),
            TokenType::DECREMENT => cells[data_pointer] = cells[data_pointer].wrapping_sub(1),
            TokenType::INPUT => {
                // Refill buffer when exhausted
                if buffer_pos >= input_buffer.len() {
                    let mut line = String::new();
                    io::stdout().flush().expect("Failed to flush");
                    io::stdin().read_line(&mut line).expect("Failed to read");
                    input_buffer = line.bytes().collect();
                    buffer_pos = 0;
                }

                if buffer_pos < input_buffer.len() {
                    cells[data_pointer] = input_buffer[buffer_pos];
                    buffer_pos += 1;
                } else {
                    cells[data_pointer] = 0; // EOF
                }
            }
            TokenType::OUTPUT => print!("{}", cells[data_pointer] as char),
            TokenType::JUMPFWD => {
                if cells[data_pointer] == 0 {
                    instruction_pointer = jump_table[&instruction_pointer] - 1;
                }
            }
            TokenType::JUMPBCK => {
                if cells[data_pointer] != 0 {
                    instruction_pointer = jump_table[&instruction_pointer] - 1;
                }
            }
        }

        instruction_pointer += 1;
    }
}
