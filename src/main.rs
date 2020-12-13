#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

lazy_static! {
    static ref REG: &'static str = r"\s*x(\d)+\s*";
    static ref SEP: &'static str = r"\s*,\s*";
    static ref NUM: &'static str = r"\s*(\d+)\s*";
    static ref LAB: &'static str = r"\s*(\w+)\s*";
    static ref COM: &'static str = r"\s*(//.*)?";
    static ref NOP_STR: String = format!(r"^\s*nop\s*{c}$", c=*COM);
    static ref NOP_REGEX: Regex = Regex::new(&NOP_STR).unwrap(); // nop
    static ref LD_STR: String = format!(r"^\s*ld\s+{r}{s}{n}\({r}\)\s*{c}$", r=*REG, s=*SEP, n=*NUM, c=*COM);
    static ref LD_REGEX: Regex = Regex::new(&LD_STR).unwrap(); // ld x5, 40(x6)
    static ref SD_STR: String = format!(r"^\s*sd\s+{r}{s}{n}\({r}\)\s*{c}$", r=*REG, s=*SEP, n=*NUM, c=*COM);
    static ref SD_REGEX: Regex = Regex::new(&SD_STR).unwrap(); // sd x5, 40(x6)
    static ref AND_STR: String = format!(r"^\s*and\s+{r}{s}{r}{s}{r}{c}$", r=*REG, s=*SEP, c=*COM);
    static ref AND_REGEX: Regex = Regex::new(&AND_STR).unwrap(); // and x5, x6, x7
    static ref OR_STR: String = format!(r"^\s*or\s+{r}{s}{r}{s}{r}{c}$", r=*REG, s=*SEP, c=*COM);
    static ref OR_REGEX: Regex = Regex::new(&OR_STR).unwrap(); // or x5, x6, x7
    static ref ADD_STR: String = format!(r"^\s*add\s+{r}{s}{r}{s}{r}{c}$", r=*REG, s=*SEP, c=*COM);
    static ref ADD_REGEX: Regex = Regex::new(&ADD_STR).unwrap(); // add x5, x6, x7
    static ref SUB_STR: String = format!(r"^\s*sub\s+{r}{s}{r}{s}{r}{c}$", r=*REG, s=*SEP, c=*COM);
    static ref SUB_REGEX: Regex = Regex::new(&SUB_STR).unwrap(); // sub x5, x6, x7
    static ref BEQ_STR: String = format!(r"^\s*beq\s+{r}{s}{r}{s}{l}{c}$", r=*REG, s=*SEP, l=*LAB, c=*COM);
    static ref BEQ_REGEX: Regex = Regex::new(&BEQ_STR).unwrap(); // beq x5, x6, Label
    static ref BLT_STR: String = format!(r"^\s*blt\s+{r}{s}{r}{s}{l}{c}$", r=*REG, s=*SEP, l=*LAB, c=*COM);
    static ref BLT_REGEX: Regex = Regex::new(&BLT_STR).unwrap(); // blt x5, x6, Label
    static ref LABEL_STR: String = format!(r"^{l}:{c}$", l=*LAB, c=*COM);
    static ref LABEL_REGEX: Regex = Regex::new(&LABEL_STR).unwrap(); // Label:
}

#[derive(StructOpt, Debug)]
#[structopt(name = "risc-v-assembler")]
struct Opt {
    #[structopt(parse(from_os_str))]
    asm: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    obj: Option<PathBuf>,

    #[structopt(long)]
    padding: Option<usize>,
}

fn main() {
    let opt = Opt::from_args();
    let asm = fs::read_to_string(&opt.asm).unwrap();
    let mut labels = HashMap::new();
    let mut instructions = Vec::new();
    for line in asm.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        } else if NOP_REGEX.is_match(line) {
            instructions.push((0, None));
        } else if let Some(inst) = parse_ld(line) {
            instructions.push((inst, None));
        } else if let Some(inst) = parse_sd(line) {
            instructions.push((inst, None));
        } else if let Some(inst) = parse_and(line) {
            instructions.push((inst, None));
        } else if let Some(inst) = parse_or(line) {
            instructions.push((inst, None));
        } else if let Some(inst) = parse_add(line) {
            instructions.push((inst, None));
        } else if let Some(inst) = parse_sub(line) {
            instructions.push((inst, None));
        } else if let Some((inst, label)) = parse_beq(line) {
            instructions.push((inst, Some(label)));
        } else if let Some((inst, label)) = parse_blt(line) {
            instructions.push((inst, Some(label)));
        } else if let Some(caps) = LABEL_REGEX.captures(line) {
            labels.insert(caps[1].to_string(), instructions.len());
        } else {
            panic!("Invalid Instruction: `{}`", line);
        }
    }

    let mut instructions = transform_labels(instructions, labels);

    if let Some(size) = opt.padding {
        if instructions.len() > size {
            eprintln!("Warning: the padding length is smaller than the number of instructions.")
        }
        while instructions.len() < size {
            instructions.push(0);
        }
    }

    let obj_path = match opt.obj {
        Some(obj) => obj,
        None => {
            let mut path = opt.asm.clone();
            path.set_extension("obj");
            path
        }
    };
    let mut obj = File::create(obj_path).unwrap();
    for inst in instructions {
        writeln!(&mut obj, "{:0>32b}", inst).unwrap();
    }
}

fn parse_ld(line: &str) -> Option<u32> {
    if let Some(caps) = LD_REGEX.captures(line) {
        let rd: u32 = caps[1].parse().unwrap();
        let imm: u32 = caps[2].parse().unwrap();
        let rs1: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0000011;
        instruction |= rd << 7;
        instruction |= 0b011 << 12;
        instruction |= rs1 << 15;
        instruction |= imm << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_sd(line: &str) -> Option<u32> {
    if let Some(caps) = SD_REGEX.captures(line) {
        let rs2: u32 = caps[1].parse().unwrap();
        let imm: u32 = caps[2].parse().unwrap();
        let rs1: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0100011;
        instruction |= (imm & 0b00000000_00000000_00000000_00011111) << 7;
        instruction |= 0b011 << 12;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        instruction |= (imm & 0b00000000_00000000_00001111_11100000) << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_and(line: &str) -> Option<u32> {
    if let Some(caps) = AND_REGEX.captures(line) {
        let rd: u32 = caps[1].parse().unwrap();
        let rs1: u32 = caps[2].parse().unwrap();
        let rs2: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0110011;
        instruction |= rd << 7;
        instruction |= 0b111 << 12;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_or(line: &str) -> Option<u32> {
    if let Some(caps) = OR_REGEX.captures(line) {
        let rd: u32 = caps[1].parse().unwrap();
        let rs1: u32 = caps[2].parse().unwrap();
        let rs2: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0110011;
        instruction |= rd << 7;
        instruction |= 0b110 << 12;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_add(line: &str) -> Option<u32> {
    if let Some(caps) = ADD_REGEX.captures(line) {
        let rd: u32 = caps[1].parse().unwrap();
        let rs1: u32 = caps[2].parse().unwrap();
        let rs2: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0110011;
        instruction |= rd << 7;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_sub(line: &str) -> Option<u32> {
    if let Some(caps) = SUB_REGEX.captures(line) {
        let rd: u32 = caps[1].parse().unwrap();
        let rs1: u32 = caps[2].parse().unwrap();
        let rs2: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0110011;
        instruction |= rd << 7;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        instruction |= 1u32 << 30;
        Some(instruction)
    } else {
        None
    }
}

fn parse_beq(line: &str) -> Option<(u32, String)> {
    if let Some(caps) = BEQ_REGEX.captures(line) {
        let rs1: u32 = caps[1].parse().unwrap();
        let rs2: u32 = caps[2].parse().unwrap();
        let label: String = caps[3].to_string();
        let mut instruction: u32 = 0;
        instruction |= 0b1100011;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        Some((instruction, label))
    } else {
        None
    }
}

fn parse_blt(line: &str) -> Option<(u32, String)> {
    if let Some(caps) = BLT_REGEX.captures(line) {
        let rs1: u32 = caps[1].parse().unwrap();
        let rs2: u32 = caps[2].parse().unwrap();
        let label: String = caps[3].to_string();
        let mut instruction: u32 = 0;
        instruction |= 0b1100011;
        instruction |= 0b100 << 12;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        Some((instruction, label))
    } else {
        None
    }
}

fn transform_labels(
    instructions: Vec<(u32, Option<String>)>,
    labels: HashMap<String, usize>,
) -> Vec<u32> {
    instructions
        .into_iter()
        .enumerate()
        .map(|(i, (mut inst, label))| {
            if let Some(label) = label {
                if let Some(j) = labels.get(&label) {
                    let imm: u32 = ((j - i) * 4).try_into().unwrap();
                    inst |= (imm & 0b00000000_00000000_00000000_00011110) << 7;
                    inst |= (imm & 0b00000000_00000000_00000111_11100000) << 20;
                    inst |= (imm & 0b00000000_00000000_00001000_00000000) >> 4;
                    inst |= (imm & 0b00000000_00000000_00010000_00000000) << 19;
                    inst
                } else {
                    panic!("Invalid Label: `{}`", &label);
                }
            } else {
                inst
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ld() {
        let instruction = parse_ld("ld x5, 40(x6)").unwrap();
        assert_eq!(instruction, 0b000000101000_00110_011_00101_0000011);
    }

    #[test]
    fn sd() {
        let instruction = parse_sd("sd x5, 40(x6)").unwrap();
        assert_eq!(instruction, 0b0000001_00101_00110_011_01000_0100011);
    }

    #[test]
    fn and() {
        let instruction = parse_and("and x5, x6, x7").unwrap();
        assert_eq!(instruction, 0b0000000_00111_00110_111_00101_0110011);
    }

    #[test]
    fn or() {
        let instruction = parse_or("or x5, x6, x7").unwrap();
        assert_eq!(instruction, 0b0000000_00111_00110_110_00101_0110011);
    }

    #[test]
    fn add() {
        let instruction = parse_add("add x5, x6, x7").unwrap();
        assert_eq!(instruction, 0b0000000_00111_00110_000_00101_0110011);
    }

    #[test]
    fn sub() {
        let instruction = parse_sub("sub x5, x6, x7").unwrap();
        assert_eq!(instruction, 0b0100000_00111_00110_000_00101_0110011);
    }

    #[test]
    fn beq() {
        let (inst, label) = parse_beq("beq x5, x6, Label").unwrap();
        let instructions = vec![(inst, Some(label))];
        let mut labels = HashMap::new();
        labels.insert("Label".to_string(), 2);
        let instructions = transform_labels(instructions, labels);
        assert_eq!(instructions[0], 0b0000000_00110_00101_000_01000_1100011);
    }

    #[test]
    fn blt() {
        let (inst, label) = parse_blt("blt x5, x6, Label").unwrap();
        let instructions = vec![(inst, Some(label))];
        let mut labels = HashMap::new();
        labels.insert("Label".to_string(), 2);
        let instructions = transform_labels(instructions, labels);
        assert_eq!(instructions[0], 0b0000000_00110_00101_100_01000_1100011);
    }
}
