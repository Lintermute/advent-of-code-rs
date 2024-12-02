use std::cmp::Ordering;

use itertools::Itertools;
use lazy_errors::Result;

use crate::parser;

pub fn parse(input: &str) -> Result<Vec<usize>> {
    parser::parse_each(input.lines()).collect()
}

pub fn part1(numbers: &[usize]) -> Result<usize> {
    let result = numbers
        .iter()
        .tuple_windows()
        .map(|(old, new)| new.cmp(old))
        .filter(|ordering| matches!(ordering, Ordering::Greater))
        .count();

    Ok(result)
}

pub fn part2(numbers: &[usize]) -> Result<usize> {
    let result = numbers
        .iter()
        .tuple_windows()
        .map(|(first, second, third)| first + second + third)
        .tuple_windows()
        .map(|(old, new)| new.cmp(&old))
        .filter(|ordering| matches!(ordering, Ordering::Greater))
        .count();

    Ok(result)
}
