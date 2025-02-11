use itertools::Itertools;
use lazy_errors::Result;
use lazy_regex::regex;

use crate::parser::{parse_substrs, regex_matches, Point, Rect};

type SymbolId = usize;
type NumberId = usize;

pub struct Data {
    symbols:   Vec<(Point, char)>,
    /// Note that rectangles of `numbers` include the borders on the grid.
    numbers:   Vec<(Rect, u16)>,
    neighbors: Vec<(SymbolId, Vec<NumberId>)>,
}

pub fn parse(input: &str) -> Result<Data> {
    let (symbols, numbers) =
        rayon::join(|| parse_symbols(input), || parse_numbers(input));

    let symbols = symbols?;
    let numbers = numbers?;

    let neighbors = symbols
        .iter()
        .enumerate()
        .map(|(i, (p, _symbol))| {
            let positions = numbers
                .iter()
                .positions(|(rect, _number)| rect.contains(p))
                .collect();

            (i, positions)
        })
        .collect();

    Ok(Data {
        symbols,
        numbers,
        neighbors,
    })
}

fn parse_symbols(input: &str) -> Result<Vec<(Point, char)>> {
    let symbols = |line| regex_matches(line, regex!(r"[^\d\.]"));
    parse_substrs(input.lines(), symbols).try_collect()
}

fn parse_numbers(input: &str) -> Result<Vec<(Rect, u16)>> {
    let numbers = |line| regex_matches(line, regex!(r"\d+"));
    parse_substrs(input.lines(), numbers)
        .map(|e| {
            let (rect, number): (Rect, _) = e?;
            let grown = rect.grow()?;
            Ok((grown, number))
        })
        .try_collect()
}

pub fn part1(data: &Data) -> Result<u32> {
    Ok(data
        .neighbors
        .iter()
        .flat_map(|(_i_sym, i_nums)| i_nums.iter())
        .unique()
        .map(|&i| data.numbers[i].1)
        .map(u32::from)
        .sum())
}

pub fn part2(data: &Data) -> Result<u32> {
    Ok(data
        .neighbors
        .iter()
        .filter(|&&(i_sym, _)| data.symbols[i_sym].1 == '*')
        .filter(|(_, i_nums)| i_nums.len() == 2)
        .map(|(_, i_nums)| {
            i_nums
                .iter()
                .map(|&i| data.numbers[i].1)
                .map(u32::from)
                .product::<u32>()
        })
        .sum())
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y23, D03, "1")?;

        let p0 = parse(&input)?;
        let p1 = part1(&p0)?;
        let p2 = part2(&p0)?;

        assert_eq!(p1, 4361);
        assert_eq!(p2, 467835);
        Ok(())
    }
}
