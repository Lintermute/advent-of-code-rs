use core::str::FromStr;

use lazy_errors::{prelude::*, Result};
use rayon::prelude::*;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Input {
    registers: [Register; 3],
    program:   Program,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct Register(u64);

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct Program {
    words:  Vec<u8>,
    instrs: Vec<Instruction>,
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Instruction {
    Div(RegisterId, ComboOperand),
    Bxl(LiteralOperand),
    Bst(ComboOperand),
    Bxc,
    Jnz(LiteralOperand),
    Out(ComboOperand),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub struct LiteralOperand(u64);

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Operand {
    Literal(u64),
    Combo(ComboOperand),
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum ComboOperand {
    Literal(u8),
    Register(RegisterId),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Hash, Eq, Ord)]
pub enum RegisterId {
    A = 0,
    B = 1,
    C = 2,
}

impl FromStr for Input {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        let mut lines = input.lines();

        let registers = (&mut lines)
            .take(3)
            .map(|line| line.parse())
            .collect::<Result<Vec<_>>>()?
            .try_into()
            .map_err(|_| err!("Failed to parse registers"))?;

        let program = lines
            .nth(1)
            .ok_or_else(|| err!("No program found"))
            .and_then(|line| line.parse())?;

        Ok(Self { registers, program })
    }
}

impl FromStr for Register {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let [reg_id, value] = line
            .split(": ")
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| err!("Failed to parse register: '{line}'"))?;

        let _ = reg_id;
        let v = u64::from_str(value).or_wrap_with(|| -> Error {
            err!("Failed to parse register value '{value}'")
        })?;

        Ok(Register(v))
    }
}

impl FromStr for Program {
    type Err = Error;

    fn from_str(line: &str) -> Result<Self> {
        let words = line
            .strip_prefix("Program: ")
            .ok_or_else(|| err!("Failed to parse programs: '{line}'"))?
            .split(',')
            .map(|word| {
                u8::from_str(word).or_wrap_with(|| -> Error {
                    err!("Failed to parse WORD '{word}'")
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let instrs = words
            .chunks(2)
            .map(|chunks| Instruction::try_from((&chunks[0], &chunks[1])))
            .collect::<Result<Vec<_>>>()?;

        Ok(Program { words, instrs })
    }
}

impl TryFrom<(&u8, &u8)> for Instruction {
    type Error = Error;

    fn try_from((opcode, operand): (&u8, &u8)) -> Result<Self> {
        match opcode {
            0 => Ok(Instruction::Div(RegisterId::A, operand.try_into()?)),
            1 => Ok(Instruction::Bxl(operand.try_into()?)),
            2 => Ok(Instruction::Bst(operand.try_into()?)),
            3 => Ok(Instruction::Jnz(operand.try_into()?)),
            4 => Ok(Instruction::Bxc),
            5 => Ok(Instruction::Out(operand.try_into()?)),
            6 => Ok(Instruction::Div(RegisterId::B, operand.try_into()?)),
            7 => Ok(Instruction::Div(RegisterId::C, operand.try_into()?)),
            _ => Err(err!("Invalid opcode {opcode}")),
        }
    }
}

impl TryFrom<&u8> for LiteralOperand {
    type Error = Error;

    fn try_from(value: &u8) -> Result<Self> {
        Ok(LiteralOperand((*value).into()))
    }
}

impl TryFrom<&u8> for ComboOperand {
    type Error = Error;

    fn try_from(value: &u8) -> Result<Self> {
        match value {
            0 => Ok(ComboOperand::Literal(0)),
            1 => Ok(ComboOperand::Literal(1)),
            2 => Ok(ComboOperand::Literal(2)),
            3 => Ok(ComboOperand::Literal(3)),
            4 => Ok(ComboOperand::Register(RegisterId::A)),
            5 => Ok(ComboOperand::Register(RegisterId::B)),
            6 => Ok(ComboOperand::Register(RegisterId::C)),
            7 => Err(err!("Operand 7 is reserved")),
            v => Err(err!("Invalid value '{v}'")),
        }
    }
}

pub fn parse(input: &str) -> Result<Input> {
    input.parse()
}

pub fn part1(input: &Input) -> Result<String> {
    use itertools::Itertools;

    let reg = input.registers.map(|r| r.0);

    Ok(run(reg, &input.program.instrs, None)
        .unwrap()
        .iter()
        .map(|k| k.to_string())
        .join(","))
}

pub fn part2(input: &Input) -> Result<u64> {
    const BATCH_SIZE: u64 = 10_000_000;
    const BATCH_MAX: u64 = u64::MAX / BATCH_SIZE;

    let reg = input.registers.map(|r| r.0);

    (0..BATCH_MAX)
        .find_map(|batch| {
            dbg!(batch);
            ((batch * BATCH_SIZE)..((batch + 1) * BATCH_SIZE))
                .into_par_iter()
                .find_map_first(|init| {
                    let mut reg = reg;
                    reg[0] = init;

                    run(reg, &input.program.instrs, Some(&input.program.words))
                        .map(|_| init)
                })
        })
        .ok_or_else(|| err!("Failed to find initial value for register A"))
}

fn run(
    mut reg: [u64; 3],
    instrs: &[Instruction],
    out_exp: Option<&[u8]>,
) -> Option<Vec<u8>> {
    let mut ip = 0;
    let mut out = vec![];

    while let Some(instr) = instrs.get(ip) {
        match instr {
            Instruction::Div(register_id, combo_operand) => {
                reg[(*register_id) as usize] =
                    reg[0] >> combo_operand.value(&reg);
            }
            Instruction::Bxl(literal_operand) => reg[1] ^= literal_operand.0,
            Instruction::Bst(combo_operand) => {
                reg[1] = combo_operand.value(&reg) % 8;
            }
            Instruction::Bxc => {
                reg[1] ^= reg[2];
            }
            Instruction::Jnz(literal_operand) => {
                if reg[0] != 0 {
                    ip = 2 * literal_operand.0 as usize;
                    continue;
                }
            }
            Instruction::Out(combo_operand) => {
                let val = combo_operand.value(&reg) % 8;
                out.push(val as u8);
            }
        }

        if let Some(exp) = out_exp {
            if !exp.starts_with(&out) {
                return None;
            }
        }

        ip += 1;
    }

    if let Some(exp) = out_exp {
        if exp == out {
            Some(out)
        } else {
            None
        }
    } else {
        Some(out)
    }
}

impl ComboOperand {
    fn value(&self, registers: &[u64; 3]) -> u64 {
        match self {
            ComboOperand::Literal(v) => (*v).into(),
            ComboOperand::Register(id) => registers[(*id) as usize],
        }
    }
}

#[cfg(test)]
mod tests {
    use lazy_errors::Result;
    use test_case::test_case;

    use crate::{day::*, fs::Config, year::*};

    #[test_case(Y24, D17, "1", "4,6,3,5,6,3,5,2,1,0")]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p1_example(y: Year, d: Day, label: &str, expected: &str) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part1(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }

    #[test_case(Y24, D17, "2", 117440)]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn p2_example(y: Year, d: Day, label: &str, expected: u64) -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(y, d, label)?;
        let input = super::parse(&input)?;
        let result = super::part2(&input)?;
        assert_eq!(result, expected);
        Ok(())
    }
}
