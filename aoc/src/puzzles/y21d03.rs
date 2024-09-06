use lazy_errors::{prelude::*, Result};

use crate::parser;

pub fn parse(input: String) -> Result<Vec<String>> {
    parser::parse_all(input.lines()).collect()
}

pub fn part1(numbers: &[String]) -> Result<usize> {
    let count = numbers.len();

    let counts_per_bit_pos: Vec<usize> = numbers
        .iter()
        .map(|line| {
            line.chars()
                .map(|ch| {
                    ch.to_digit(2)
                        .ok_or_else(|| err!("Bad digit: {}", ch))
                        .and_then(|k: u32| usize::try_from(k).or_wrap())
                })
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .reduce(|l, r| {
            l.into_iter()
                .zip(r)
                .map(|(l, r)| l + r)
                .collect()
        })
        .ok_or_else(|| err!("List of numbers was empty"))?;

    let bits = counts_per_bit_pos
        .iter()
        .map(|&c| if c <= count / 2 { '0' } else { '1' })
        .collect::<String>();

    let gamma = usize::from_str_radix(&bits, 2).or_wrap()?;
    let epsilon = 2usize.pow(12) - 1 - gamma;
    Ok(gamma * epsilon)
}

pub fn part2(numbers: &[String]) -> Result<usize> {
    let oxy = reduce(numbers, true)?;
    let co2 = reduce(numbers, false)?;

    let oxy = usize::from_str_radix(&oxy, 2).or_wrap()?;
    let co2 = usize::from_str_radix(&co2, 2).or_wrap()?;

    Ok(oxy * co2)
}

fn reduce(numbers: &[String], keep_most_common: bool) -> Result<String> {
    let digits = if let Some(any) = numbers.first() {
        any.len()
    } else {
        return Err(err!("Input list is empty"));
    };

    let mut numbers = Vec::from(numbers);

    for pos in 0..=digits {
        let ones = numbers
            .iter()
            .map(|k| {
                k.chars()
                    .nth(pos)
                    .ok_or_else(|| err!("Too short: {}", k))
            })
            .collect::<Result<Vec<char>>>()?
            .into_iter()
            .filter(|&b| b == '1')
            .count();

        let zeroes = numbers.len() - ones;

        let digit_to_keep = match (keep_most_common, zeroes <= ones) {
            (true, true) => '1',
            (true, false) => '0',
            (false, true) => '0',
            (false, false) => '1',
        };

        numbers.retain(|n| {
            n.chars()
                .nth(pos)
                .map(|ch| ch == digit_to_keep)
                .unwrap_or(false)
        });

        if numbers.len() <= 1 {
            break;
        }
    }

    match numbers.len() {
        0 => Err(err!("Did not find the number you were looking for.")),
        1 => numbers.pop().ok_or_else(|| err!("wat")),
        _ => Err(err!("Too many numbers left")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_handles_empty_list() {
        assert!(reduce(&[], true)
            .is_err_and(|e| e.to_string() == "Input list is empty"));
    }
}
