use criterion::{criterion_group, criterion_main, Criterion};

use aoc::{
    ident::{Day, Year},
    puzzles::*,
    Config, D01, D02, D03, Y21, Y23, Y24,
};

macro_rules! bench {
    ($year:ident, $day:ident, $solver1:path, $solver2:path) => {
        paste::item! {
            fn [< $year:lower $day:lower p1>](c: &mut Criterion) {
                let id = stringify!([< $year:lower $day:lower p1>]);
                let input = read_input_or_panic($year, $day);
                c.bench_function(&id, |b| {
                    b.iter(|| $solver1(criterion::black_box(&input)))
                });
            }

            fn [< $year:lower $day:lower p2>](c: &mut Criterion) {
                let id = stringify!([< $year:lower $day:lower p2>]);
                let input = read_input_or_panic($year, $day);
                c.bench_function(&id, |b| {
                    b.iter(|| $solver2(criterion::black_box(&input)))
                });
            }

            criterion_group!(
                [< $year:lower $day:lower>],
                [< $year:lower $day:lower p1>],
                [< $year:lower $day:lower p2>]);
        }
    };

    ($year:ident, $day:ident, $solver1:path, $solver2:path, $parser:expr) => {
        paste::item! {
            fn [< $year:lower $day:lower p0>](c: &mut Criterion) {
                let id = stringify!([< $year:lower $day:lower p0>]);
                let input = read_input_or_panic($year, $day);
                c.bench_function(&id, |b| {
                    b.iter(|| {
                        $parser(criterion::black_box(&input))
                    })
                });
            }

            fn [< $year:lower $day:lower p1>](c: &mut Criterion) {
                let id = stringify!([< $year:lower $day:lower p1>]);
                let input = read_input_or_panic($year, $day);
                let data = $parser(&input).unwrap();
                c.bench_function(&id, |b| {
                    b.iter(|| $solver1(criterion::black_box(&data)))
                });
            }

            fn [< $year:lower $day:lower p2>](c: &mut Criterion) {
                let id = stringify!([< $year:lower $day:lower p2>]);
                let input = read_input_or_panic($year, $day);
                let data = $parser(&input).unwrap();
                c.bench_function(&id, |b| {
                    b.iter(|| $solver2(criterion::black_box(&data)))
                });
            }

            criterion_group!(
                [< $year:lower $day:lower>],
                [< $year:lower $day:lower p0>],
                [< $year:lower $day:lower p1>],
                [< $year:lower $day:lower p2>]);
        }
    };
}

fn read_input_or_panic(y: Year, d: Day) -> String {
    Config::from_env_or_defaults()
        .unwrap()
        .read_personal_puzzle_input(y, d)
        .unwrap()
        .expect("Personal puzzle input not found")
}

bench!(Y21, D02, y21d02::part1, y21d02::part2);
bench!(Y23, D03, y23d03::part1, y23d03::part2, y23d03::parse);
bench!(Y24, D01, y24d01::part1, y24d01::part2, y24d01::parse);
bench!(Y24, D02, y24d02::part1, y24d02::part2, y24d02::parse);
bench!(Y24, D03, y24d03::part1, y24d03::part2, y24d03::parse);
criterion_main!(y21d02, y23d03, y24d01, y24d02, y24d03);
