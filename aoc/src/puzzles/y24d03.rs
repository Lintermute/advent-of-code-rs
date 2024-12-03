use lazy_errors::Result;

pub fn parse(input: &str) -> Result<Vec<(u16, u16)>> {
    use itertools::Itertools;
    use lazy_regex::regex;

    use crate::parser::{parse_substrs, regex_matches, Rect};
    use core::str::FromStr;

    let instrs = |line| regex_matches(line, regex!(r"mul\(\d{1,3},\d{1,3}\)"));
    parse_substrs(input.lines(), instrs)
        .map_ok(|(_, y): (Rect, String)| y)
        .map_ok(|x| {
            let p = x.find(',').unwrap();
            let n = x.len();
            let l = &x[4..p];
            let r = &x[p + 1..n - 1];

            // dbg!(l, r);
            let l = u16::from_str(l).unwrap();
            let r = u16::from_str(r).unwrap();
            (l, r)
        })
        .collect()
}

pub fn part1(_data: &[(u16, u16)]) -> Result<u64> {
    let sum = _data
        .iter()
        .map(|&(l, r)| {
            let l = u64::from(l);
            let r = u64::from(r);
            l * r
        })
        .sum();

    Ok(sum)
}

pub fn part2(_data: &[(u16, u16)]) -> Result<u64> {
    Ok(0)
}

#[cfg(test)]
mod tests {
    use crate::{
        fs::Config,
        ident::{D03, Y24},
    };

    use super::*;

    #[test]
    #[cfg_attr(miri, ignore)] // Because of `read_workspace_dir_from_cargo`
    fn example_1() -> Result<()> {
        let config = Config::from_env_or_defaults()?;
        let input = config.read_example_puzzle_input(Y24, D03, "1")?;
        // let input = config
        //     .read_personal_puzzle_input(Y24, D03)?
        //     .unwrap();

        let p0 = super::parse(&input)?;
        let p1 = super::part1(&p0)?;
        let p2 = super::part2(&p0)?;

        assert_eq!(p1, 161);
        assert_eq!(p2, 0);
        Ok(())
    }
}
