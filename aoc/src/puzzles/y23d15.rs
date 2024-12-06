use std::{iter::zip, str::FromStr};

use lazy_errors::{prelude::*, Result};

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
struct Step {
    id: u8,
    op: Operation,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
struct Lens {
    label: String,
    focal: u8,
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
enum Operation {
    Insert(Lens),
    Remove(String),
}

impl FromStr for Step {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let [label, focal_maybe] = s
            .split(['=', '-'])
            .collect::<Vec<_>>()
            .try_into()
            .map_err(|_| err!("Input doesn't match pattern: '{s}'"))?;

        let i = label.len();
        let op = &s[i..i + 1];

        let id = hash(label);
        let label = label.to_owned();
        let op = match op {
            "=" => {
                let focal = focal_maybe.parse().or_wrap_with(|| {
                    format!("Invalid focal width '{focal_maybe}'")
                })?;

                Operation::Insert(Lens { label, focal })
            }
            "-" => {
                if !focal_maybe.is_empty() {
                    return Err(err!(
                        "Operation '-' doesn't accept parameter '{focal_maybe}'"
                    ));
                }

                Operation::Remove(label)
            }
            others => return Err(err!("Invalid operation '{others}'")),
        };

        Ok(Step { id, op })
    }
}

pub fn parse(input: &str) -> Result<Vec<String>> {
    Ok(input
        .trim()
        .split(',')
        .map(str::to_owned)
        .collect())
}

pub fn part1(steps: &[String]) -> Result<u64> {
    Ok(steps
        .iter()
        .map(|step| hash(step))
        .map(Into::<u64>::into)
        .sum())
}

pub fn part2(steps: &[String]) -> Result<u64> {
    // Well, this is very clunky, but as of 2024 there doesn't seem to be
    // a better way to initialize an array of `Vec<T>` if `T` is `!Copy`…
    let mut boxes: [Vec<Lens>; 256] = vec![vec![]; 256]
        .try_into()
        .map_err(|_| err!("Failed to setup boxes"))?;

    for step in steps.iter().map(|s| Step::from_str(s)) {
        let Step { id, op } = step?;
        let the_box = &mut boxes[usize::from(id)];
        match op {
            Operation::Insert(lens) => {
                let slot = the_box
                    .iter()
                    .position(|e| e.label == lens.label)
                    .and_then(|i| the_box.get_mut(i));
                match slot {
                    Some(slot) => *slot = lens,
                    None => the_box.push(lens),
                }
            }
            Operation::Remove(label) => the_box.retain(|e| e.label != label),
        }
    }

    Ok(zip(0u64.., boxes)
        .map(|(b, slots)| {
            zip(0u64.., slots)
                .map(|(s, lens)| {
                    let focal = u64::from(lens.focal);
                    (b + 1) * (s + 1) * focal
                })
                .sum::<u64>()
        })
        .sum())
}

fn hash(s: &str) -> u8 {
    s.as_bytes().iter().fold(0, |acc, &e| {
        let acc = u16::from(acc);
        let e = u16::from(e);
        ((acc + e) * 17) as u8 // modulo 256 as per specs
    })
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y23, D15, "1")?;

        let p0 = parse(&input)?;
        let p1 = part1(&p0)?;
        let p2 = part2(&p0)?;

        assert_eq!(p1, 1320);
        assert_eq!(p2, 145);
        Ok(())
    }
}
