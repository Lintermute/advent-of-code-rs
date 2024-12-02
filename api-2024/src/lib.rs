pub mod day1 {
    use core::fmt::Display;

    use aoc::puzzles::*;

    pub fn part1(input: &str) -> impl Display {
        let data = y24d01::parse(input.to_owned()).unwrap();
        y24d01::part1(&data).unwrap()
    }

    pub fn part2(input: &str) -> impl Display {
        let data = y24d01::parse(input.to_owned()).unwrap();
        y24d01::part2(&data).unwrap()
    }
}
