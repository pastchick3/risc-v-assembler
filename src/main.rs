#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

lazy_static! {
    static ref REG: &'static str = r"\s*x(\d)+\s*";
    static ref SEP: &'static str = r"\s*,\s*";
    static ref NUM: &'static str = r"\s*(\d+)\s*";
    static ref LD_STR: String = format!(r"^\s*ld\s+{r}{s}{n}\({r}\)\s*$", r=*REG, s=*SEP, n=*NUM);
    static ref LD_REGEX: Regex = Regex::new(&LD_STR).unwrap(); // ld x5, 40(x6)
    static ref SD_STR: String = format!(r"^\s*sd\s+{r}{s}{n}\({r}\)\s*$", r=*REG, s=*SEP, n=*NUM);
    static ref SD_REGEX: Regex = Regex::new(&SD_STR).unwrap(); // sd x5, 40(x6)
    static ref AND_STR: String = format!(r"^\s*and\s+{r}{s}{r}{s}{r}$", r=*REG, s=*SEP);
    static ref AND_REGEX: Regex = Regex::new(&AND_STR).unwrap(); // and x5, x6, x7
    static ref OR_STR: String = format!(r"^\s*or\s+{r}{s}{r}{s}{r}$", r=*REG, s=*SEP);
    static ref OR_REGEX: Regex = Regex::new(&OR_STR).unwrap(); // or x5, x6, x7
    static ref ADD_STR: String = format!(r"^\s*add\s+{r}{s}{r}{s}{r}$", r=*REG, s=*SEP);
    static ref ADD_REGEX: Regex = Regex::new(&ADD_STR).unwrap(); // add x5, x6, x7
    static ref SUB_STR: String = format!(r"^\s*sub\s+{r}{s}{r}{s}{r}$", r=*REG, s=*SEP);
    static ref SUB_REGEX: Regex = Regex::new(&SUB_STR).unwrap(); // sub x5, x6, x7
    static ref BEQ_STR: String = format!(r"^\s*beq\s+{r}{s}{r}{s}{n}$", r=*REG, s=*SEP, n=*NUM);
    static ref BEQ_REGEX: Regex = Regex::new(&BEQ_STR).unwrap(); // beq x5, x6, 100
}

#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
struct Opt {
    #[structopt(long, parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();
    let file = fs::read_to_string(opt.file).unwrap();
    let mut instructions = Vec::new();
    for line in file.lines() {
        if let Some(inst) = parse_ld(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_ld(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_sd(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_and(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_or(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_add(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_sub(line) {
            instructions.push(inst);
        } else if let Some(inst) = parse_beq(line) {
            instructions.push(inst);
        } else {
            panic!("Invalid Instruction: `{}`", line);
        }
    }
}

fn parse_ld(line: &str) -> Option<u32> {
    if let Some(caps) = LD_REGEX.captures(line) {
        let target: u32 = caps[1].parse().unwrap();
        let offset: u32 = caps[2].parse().unwrap();
        let base: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0000011;
        instruction |= target << 7;
        instruction |= 0b011 << 12;
        instruction |= base << 15;
        instruction |= offset << 20;
        Some(instruction)
    } else {
        None
    }
}

fn parse_sd(line: &str) -> Option<u32> {
    if let Some(caps) = SD_REGEX.captures(line) {
        let target: u32 = caps[1].parse().unwrap();
        let offset: u32 = caps[2].parse().unwrap();
        let base: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b0100011;
        instruction |= (offset & 0b11111) << 5;
        instruction |= 0b011 << 12;
        instruction |= target << 15;
        instruction |= base << 20;
        instruction |= (offset & !0b11111) << 25;
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

fn parse_beq(line: &str) -> Option<u32> {
    if let Some(caps) = BEQ_REGEX.captures(line) {
        let rs1: u32 = caps[1].parse().unwrap();
        let rs2: u32 = caps[2].parse().unwrap();
        let imm: u32 = caps[3].parse().unwrap();
        let mut instruction: u32 = 0;
        instruction |= 0b1100111;
        instruction |= rs1 << 15;
        instruction |= rs2 << 20;
        instruction |= (imm & 0b1111) << 8;
        instruction |= (imm & 0b111110000) << 25;
        instruction |= (imm & 0b10000000000) << 8;
        instruction |= (imm & 0b100000000000) << 31;
        Some(instruction)
    } else {
        None
    }
}
