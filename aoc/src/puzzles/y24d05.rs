use std::collections::HashMap;

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};

pub struct PrintQueue {
    rules: HashMap<u8, Vec<u8>>,
    good:  Vec<Vec<u8>>,
    bad:   Vec<Vec<u8>>,
}

pub fn parse(input: &str) -> Result<PrintQueue> {
    let mut lines = input.lines();

    let rules = (&mut lines)
        .take_while(|line| !line.is_empty())
        .map(|line| {
            let [l, r] = line
                .split('|')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|_| err!("Invalid line: '{line}'"))?;

            let l = parse_page_number(l)?;
            let r = parse_page_number(r)?;

            Ok((l, r))
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .into_group_map();

    let (good, bad) = lines
        .map(|line| {
            line.split(',')
                .map(parse_page_number)
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .partition(|update| is_correct(update, &rules));

    Ok(PrintQueue { rules, good, bad })
}

pub fn part1(data: &PrintQueue) -> Result<u32> {
    Ok(data
        .good
        .iter()
        .map(|pages| pages[pages.len() / 2])
        .map(u32::from)
        .sum())
}

pub fn part2(data: &PrintQueue) -> Result<u32> {
    Ok(data
        .bad
        .iter()
        .map(|pages| sort(pages, &data.rules))
        .map(|pages| pages[pages.len() / 2])
        .map(u32::from)
        .sum())
}

fn parse_page_number(s: &str) -> Result<u8> {
    use core::str::FromStr;
    u8::from_str(s).or_wrap_with(|| format!("Not a page number: '{s}'"))
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
                    let e = pages.remove(i);
                    pages.insert(j, e);
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
