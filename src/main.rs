use clap::Parser;
use rand::distr::Alphanumeric;
use rand::distr::Distribution;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::process;
mod tokens;
use crate::ir_nodes::IRCleanup;
use crate::ir_nodes::IRDecrement;
use crate::ir_nodes::IRIncrement;
use crate::ir_nodes::IRInput;
use crate::ir_nodes::IRJumpBck;
use crate::ir_nodes::IRJumpFwd;
use crate::ir_nodes::IROutput;
use crate::tokens::TokenType;
mod ir_nodes;
use crate::ir_nodes::IRLeft;
use crate::ir_nodes::IRNode;
use crate::ir_nodes::IRRight;
use tempfile::NamedTempFile;

/// Brainfuck variant for the brainrotted
#[derive(Parser, Debug)]
#[command(about, long_about = None)]
struct Args {
    /// Output x86-64 assembly instead of emulating execution
    #[arg(short, long)]
    assembly_only: bool,

    /// Generate an x86-64 executable instead of emulating execution
    #[arg(short, long)]
    non_portable: bool,

    #[arg(short, long)]
    output: Option<String>,

    input: String,
}

fn main() {
    let args = Args::parse();
    // Generate x86-64 assembly instead of emulating execution
    let non_portable = args.non_portable;
    let assembly_only = args.assembly_only;
    let file_path: String = args.input;

    if file_path.is_empty() {
        println!("No file path");
        process::exit(1);
    }

    let contents = fs::read_to_string(file_path).expect("Should have been able to read the file");
    let tokens = tokenise(contents);
    if !assembly_only && !non_portable {
        emulate_execution(tokens);
        process::exit(0);
    }

    let mut temp_assembly_file = match NamedTempFile::new() {
        Ok(t) => t,
        Err(_) => process::exit(1),
    };

    let output: String = match args.output {
        Some(s) => s,
        None => {
            println!("-a or -n require an output destination");
            process::exit(1);
        }
    };

    if assembly_only {
        let path = std::path::Path::new(&output);
        let mut file = match std::fs::File::create(&path) {
            Ok(file) => file,
            Err(why) => panic!("Error opening file for write: {}", why),
        };

        match file.write_all(generate_assembly(tokens).as_bytes()) {
            Ok(_) => process::exit(0),
            Err(why) => panic!("Failed to write to write: {}", why),
        }
    }

    if temp_assembly_file
        .write_all(generate_assembly(tokens).as_bytes())
        .is_err()
    {
        process::exit(1);
    }

    let temp_object_file = match NamedTempFile::new() {
        Ok(t) => t,
        Err(why) => panic!("Error opening file for write: {}", why),
    };

    let _ = std::process::Command::new("nasm")
        .arg("-f")
        .arg("elf64")
        .arg("-o")
        .arg(temp_object_file.path())
        .arg(temp_assembly_file.path())
        .status()
        .expect("compilation successful");

    let _ = std::process::Command::new("ld")
        .arg("-o")
        .arg(output)
        .arg(temp_object_file.path())
        .status()
        .expect("linking successful");

    drop(temp_assembly_file);
    drop(temp_object_file);
}

fn tokenise(content: String) -> Vec<TokenType> {
    let token_map = HashMap::from([
        // "Documentation as code" haha
        ("676", TokenType::RIGHT),    // Move data pointer forwards
        ("767", TokenType::LEFT),     // Move data pointer backwards
        ("67", TokenType::INCREMENT), // Increment value at data pointer
        ("76", TokenType::DECREMENT), // Increment value at data pointer
        ("667", TokenType::OUTPUT),   // Output value at data pointer
        ("776", TokenType::INPUT),    // Set value at data pointer to input of one character
        ("6677", TokenType::JUMPFWD), // If value at data pointer is 0, move instruction pointer to instruction after the nearest 7766
        ("7766", TokenType::JUMPBCK), // Move instruction pointer back to the nearest 6677 before it
    ]);

    let mut tokens: Vec<TokenType> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let words: Vec<&str> = line.split(" ").collect();
        for word in &words {
            match token_map.get(word) {
                Some(token_type) => tokens.push(*token_type),
                None => {} // Ignore any other words
            }
        }
    }
    tokens
}

fn emulate_execution(tokens: Vec<TokenType>) {
    let mut instruction_pointer = 0;
    let mut data_pointer: usize = 0;
    let mut cells_length: usize = 30000;
    let mut cells: Vec<u8> = vec![0; cells_length];
    let mut input_buffer: Vec<u8> = Vec::new();
    let mut buffer_pos = 0;

    let mut jump_table: HashMap<usize, usize> = HashMap::new();
    // Use stack to ensure each jump forward has a corresponding jump back
    // and store positions for jump forward commands
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
            _ => {}
        }
    }

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

fn generate_assembly(tokens: Vec<TokenType>) -> String {
    let mut generated = String::from(
        "
    section .bss 
        termios_orig: resb 40
        termios_new:  resb 40
        cell resb 1
        cells resb 30000 
    section .text 
    global _start
    
    _start:
        mov r12, cells ; set r12 to be at the start address of cells
    ",
    );

    let ir = generate_ir(tokens);
    for node in ir {
        match node {
            IRNode::CLEANUP(_) => generated.push_str(
                format!("\nmov rax, 60\nmov rdi, 0\nsyscall").as_str(), // exit syscall
            ),
            IRNode::JUMPFWD(jump) => {
                generated.push_str(
                    format!(
                        "\n.{}_start:\n cmp byte [r12], 0\nje .{}_end",
                        jump.path_name, jump.path_name
                    )
                    .as_str(),
                );
            }
            IRNode::JUMPBCK(jump) => {
                generated.push_str(
                    format!(
                        "\ncmp byte [r12], 0\njne .{}_start\n.{}_end:",
                        jump.path_name, jump.path_name
                    )
                    .as_str(),
                );
            }
            IRNode::LEFT(left) => {
                generated.push_str(format!("\nsub r12, {}", left.distance).as_str())
            }
            IRNode::RIGHT(right) => {
                generated.push_str(format!("\nadd r12, {}", right.distance).as_str())
            }
            IRNode::INCREMENT(increment) => {
                generated.push_str(format!("\nadd byte [r12], {}", increment.amount).as_str());
            }
            IRNode::DECREMENT(decrement) => {
                generated.push_str(format!("\nsub byte [r12], {}", decrement.amount).as_str());
            }
            IRNode::OUTPUT(_) => {
                generated.push_str("\nmov rax, 1\nmov rdi, 1\nmov rsi, r12\nmov rdx, 1\nsyscall")
            } // Moves cell address into rsi then sys calls print
            IRNode::INPUT(_) => {
                generated.push_str("\nmov rax, 0\nmov rdi, 0\nmov rsi, r12\nmov rdx, 1\nsyscall")
            } // Reads character into r12
        }
    }

    return generated;
}

fn generate_ir(tokens: Vec<TokenType>) -> Vec<IRNode> {
    let mut ir: Vec<IRNode> = vec![];
    let mut branches = vec![];
    let mut data_pointer: usize = 0;
    let mut cells_length: usize = 30000;
    let mut cells: Vec<u8> = vec![0; cells_length];
    for token in tokens {
        // Keep track of the most recent node to compress if needed
        // i.e two identical operations are merged into one
        let current = ir.last_mut();
        match token {
            TokenType::LEFT => {
                data_pointer = data_pointer.saturating_sub(1);
                match current {
                    Some(IRNode::LEFT(left)) => {
                        left.distance += 1;
                    }
                    _ => ir.push(IRNode::LEFT(IRLeft { distance: 1 })),
                }
            }
            TokenType::RIGHT => {
                data_pointer += 1;
                if data_pointer >= cells_length {
                    cells_length *= 2;
                    cells.resize(cells_length, 0);
                }

                match current {
                    Some(IRNode::RIGHT(right)) => {
                        right.distance += 1;
                    }
                    _ => ir.push(IRNode::RIGHT(IRRight { distance: 1 })),
                }
            }
            TokenType::INCREMENT => {
                cells[data_pointer] = cells[data_pointer].wrapping_add(1);
                match current {
                    Some(IRNode::INCREMENT(increment)) => {
                        increment.amount += 1;
                    }
                    _ => ir.push(IRNode::INCREMENT(IRIncrement { amount: 1 })),
                }
            }
            TokenType::DECREMENT => {
                cells[data_pointer] = cells[data_pointer].wrapping_sub(1);
                match current {
                    Some(IRNode::DECREMENT(decrement)) => {
                        decrement.amount += 1;
                    }
                    _ => ir.push(IRNode::DECREMENT(IRDecrement { amount: 1 })),
                }
            }
            TokenType::INPUT => ir.push(IRNode::INPUT(IRInput {})),
            TokenType::OUTPUT => ir.push(IRNode::OUTPUT(IROutput {})),
            TokenType::JUMPFWD => {
                let path_name = random_name("path", 8);
                ir.push(IRNode::JUMPFWD(IRJumpFwd {
                    path_name: path_name.clone(),
                }));
                branches.push(path_name);
            }
            TokenType::JUMPBCK => ir.push(IRNode::JUMPBCK(IRJumpBck {
                path_name: branches.pop().unwrap(),
            })),
        }
    }
    ir.push(IRNode::CLEANUP(IRCleanup {}));
    ir
}

fn random_name(prefix: &str, len: usize) -> String {
    let rng = rand::rng();
    let sample = Alphanumeric;
    let suffix: String = sample
        .sample_iter(rng)
        .take(len)
        .map(|b| b as char)
        .collect();

    return format!("{prefix}_{suffix}");
}
