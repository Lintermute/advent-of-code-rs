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

pub mod day3 {
    use core::fmt::Display;

    use aoc::puzzles::y24d03;

    pub fn part1(input: &str) -> impl Display {
        let data = y24d03::parse(input).unwrap();
        y24d03::part1(&data).unwrap()
    }

    pub fn part2(input: &str) -> impl Display {
        let data = y24d03::parse(input).unwrap();
        y24d03::part2(&data).unwrap()
    }
}

pub mod day4 {
    use core::fmt::Display;

    use aoc::puzzles::y24d04;

    pub fn part1(input: &str) -> impl Display {
        let data = y24d04::parse(input).unwrap();
        y24d04::part1(&data).unwrap()
    }

    pub fn part2(input: &str) -> impl Display {
        let data = y24d04::parse(input).unwrap();
        y24d04::part2(&data).unwrap()
    }
}
