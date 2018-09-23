#[macro_use]
mod parsers;
mod symbols;
mod token;

pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;

pub use self::parsers::file;
use self::symbols::{SymbolTable, SymbolType};
use self::token::Token;
use nom::types::CompleteStr;
use std::fs::File;
use std::io::Read;

#[derive(Debug)]
/// Responsible for parsing a raw string into bytecode for the VM.
/// Constructing the symbol table
/// Works in two phases:
///     * Phase 1 - Label Extraction and generation of the EPIE header
///     * Phase 2 - Generate bytecode for the body
pub struct Assembler {
    phase: AssemblerPhase,
    symbols: SymbolTable,
}
#[derive(Debug)]
pub enum AssemblerPhase {
    First,
}

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Option<Token>,
    label: Option<Token>,
    directive: Option<Token>,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
        }
    }

    pub fn assemble_file(&mut self, path: &str) -> Option<Vec<u8>> {
        let mut contents = String::new();

        File::open(path)
            .expect("Couldn't open the file")
            .read_to_string(&mut contents)
            .expect("Coudln't read to file");

        self.assemble(&contents)
    }

    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        match file(CompleteStr(raw)) {
            Ok((_, program)) => {
                // FIRST PHASE

                // Get the header
                let mut assembled_program = self.write_pie_header();
                // Extract labels
                self.extract_labels(&program);
                // SECOND PHASE
                let mut body = program.to_bytes(&self.symbols);
                // Merge the header with body
                assembled_program.append(&mut body);

                Some(assembled_program)
            }

            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                None
            }
        }
    }

    /// Go through every instruction and look for label declarations.
    /// When label found add it to symbol table, along with the byte we found the label at.
    fn extract_labels(&mut self, p: &Program) {
        for (i, instruction) in p.instructions.iter().enumerate() {
            if instruction.is_label() {
                if let Some(name) = instruction.label_name() {
                    self.symbols.add(name, (i * 4) + 64, SymbolType::Label);
                }
            }
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = Vec::with_capacity(PIE_HEADER_LENGTH);

        for byte in &PIE_HEADER_PREFIX[0..] {
            header.push(*byte);
        }

        while header.len() < PIE_HEADER_LENGTH {
            header.push(0);
        }

        header
    }
}

impl AssemblerInstruction {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut results = Vec::with_capacity(4);

        match self.opcode {
            Some(Token::Op(ref code)) => results.push(*code),

            _ => {
                // panic!("Non-opcode found in opcode field");
            }
        }

        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(ref op) => AssemblerInstruction::extract_operand(op, &mut results, symbols),
                None => (),
            }
        }

        while results.len() < 4 {
            results.push(0);
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            Token::Register(ref reg) => results.push(*reg),
            Token::Number(ref num) => {
                let byte1 = *num as u16;
                let byte2 = byte1 >> 8;

                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }

            Token::LabelUsage(ref label) => {
                if let Some(offset) = symbols.offset(label) {
                    let byte1 = offset;
                    let byte2 = offset >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                }
            }
            _ => panic!("opcode found in operand field"),
        }
    }

    fn is_label(&self) -> bool {
        self.label.is_some()
    }

    fn label_name(&self) -> Option<String> {
        match self.label.as_ref() {
            Some(&Token::LabelDeclaration(ref string)) => Some(string.clone()),
            _ => None,
        }
    }
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = Vec::with_capacity(self.instructions.len() * 4);

        for inst in self.instructions.iter() {
            program.append(&mut inst.to_bytes(symbols));
        }

        program
    }
}
