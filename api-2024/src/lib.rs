macro_rules! api {
    ($day:literal, $solver1:path, $solver2:path, $parser:expr) => {
        paste::item! {
            pub mod [< day $day >] {
                use core::fmt::Display;

                use aoc::puzzles::*;

                pub fn part1(input: &str) -> impl Display {
                    let data = $parser(input).unwrap();
                    $solver1(&data).unwrap()
                }

                pub fn part2(input: &str) -> impl Display {
                    let data = $parser(input).unwrap();
                    $solver2(&data).unwrap()
                }
            }
        }
    };
}

api!(1, y24d01::part1, y24d01::part2, y24d01::parse);
api!(2, y24d02::part1, y24d02::part2, y24d02::parse);
api!(3, y24d03::part1, y24d03::part2, y24d03::parse);
api!(4, y24d04::part1, y24d04::part2, y24d04::parse);
api!(5, y24d05::part1, y24d05::part2, y24d05::parse);
api!(6, y24d06::part1, y24d06::part2, y24d06::parse);
