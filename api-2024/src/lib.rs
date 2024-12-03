pub mod day1 {
    use core::fmt::Display;

    use aoc::puzzles::*;

    pub fn part1(input: &str) -> impl Display {
        let data = y24d01::parse(input).unwrap();
        y24d01::part1(&data).unwrap()
    }

    pub fn part2(input: &str) -> impl Display {
        let data = y24d01::parse(input).unwrap();
        y24d01::part2(&data).unwrap()
    }
}

pub mod day2 {
    use core::fmt::Display;

    use aoc::puzzles::*;

    pub fn part1(input: &str) -> impl Display {
        let data = y24d02::parse(input).unwrap();
        y24d02::part1(&data).unwrap()
    }

    pub fn part2(input: &str) -> impl Display {
        let data = y24d02::parse(input).unwrap();
        y24d02::part2(&data).unwrap()
    }
}
