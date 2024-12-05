use std::collections::HashMap;

use itertools::Itertools;
use lazy_errors::Result;

pub struct Instr {
    map:     HashMap<u8, Vec<u8>>,
    updates: Vec<Vec<u8>>,
}

pub fn parse(input: &str) -> Result<Instr> {
    use core::str::FromStr;

    let mut lines = input.lines();

    let map = (&mut lines)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let [l, r] = line
                .split('|')
                .collect::<Vec<_>>()
                .try_into()
                .unwrap();
            let l = u8::from_str(l).unwrap();
            let r = u8::from_str(r).unwrap();
            (l, r)
        })
        .into_group_map();

    let updates: Vec<_> = lines
        .map(|line| {
            line.split(',')
                .map(|k| u8::from_str(k).unwrap())
                .collect::<Vec<_>>()
        })
        .collect();

    Ok(Instr { map, updates })
}

pub fn part1(data: &Instr) -> Result<u64> {
    let sum = data
        .updates
        .iter()
        .filter(|pages| is_correct(pages, &data.map))
        .map(|pages| pages[pages.len() / 2])
        .map(u64::from)
        .sum();
    Ok(sum)
}

pub fn part2(data: &Instr) -> Result<u64> {
    let sum = data
        .updates
        .iter()
        .filter(|pages| !is_correct(pages, &data.map))
        .map(|pages| sort(pages, &data.map))
        .map(|pages| pages[pages.len() / 2])
        .map(u64::from)
        .sum();
    Ok(sum)
}

fn is_correct(pages: &[u8], rules: &HashMap<u8, Vec<u8>>) -> bool {
    pages
        .iter()
        .enumerate()
        .all(|(i, &page)| {
            let Some(after) = rules.get(&page) else {
                return true;
            };
            after.iter().all(|p| {
                if let Some(j) = pages.iter().position(|x| *x == *p) {
                    i < j
                } else {
                    true
                }
            })
        })
}

fn sort(pages: &[u8], rules: &HashMap<u8, Vec<u8>>) -> Vec<u8> {
    let mut pages = pages.to_vec();
    loop {
        let mut swapped = false;
        for i in 0..pages.len() {
            let page = pages[i];
            let Some(after) = rules.get(&page) else {
                continue;
            };
            if let Some(j) = after
                .iter()
                .filter_map(|p| pages.iter().position(|x| *x == *p))
                .min()
            {
                if j < i {
                    pages.swap(i, j);
                    swapped = true;
                }
            }
        }
        if !swapped {
            break;
        }
    }
    pages
}

#[cfg(test)]
mod tests {
    use crate::{day::*, fs::Config, year::*};

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D05, "1")?;

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 143);
        assert_eq!(p2, 123);
        Ok(())
    }
}
