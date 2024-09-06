use itertools::Either;
use num::Integer;

pub trait Mean {
    fn mean(&self, right: &Self) -> Self;
}

pub trait Median<T>
where
    Self: AsRef<[T]>,
    T: Mean + Copy,
{
    fn median(&self) -> Option<T> {
        match middle(&self.as_ref())? {
            Either::Left(middle) => Some(*middle),
            Either::Right((left, right)) => Some(T::mean(left, right)),
        }
    }
}

impl<T, U> Median<T> for U
where
    U: AsRef<[T]> + ?Sized,
    T: Mean + Copy,
{
}

pub fn min_med_max_sorted<T, U>(slice: &U) -> Option<(T, T, T)>
where
    T: Mean + Copy,
    U: AsRef<[T]> + Median<T> + ?Sized,
{
    let slice = slice.as_ref();
    let min = *slice.first()?;
    let max = *slice.last()?;
    let med = slice.median()?;

    Some((min, med, max))
}

fn middle<T, U>(container: &U) -> Option<Either<&T, (&T, &T)>>
where
    U: AsRef<[T]>,
{
    let slice = container.as_ref();
    let len = slice.len();
    if len == 0 {
        None
    } else if len.is_odd() {
        Some(Either::Left(&slice[len / 2]))
    } else {
        let a = &slice[len / 2 - 1];
        let b = &slice[len / 2];
        Some(Either::Right((a, b)))
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use super::*;

    #[derive(Debug, Copy, Clone)]
    struct TestData(i32);

    impl Mean for TestData {
        fn mean(&self, right: &Self) -> Self {
            let inner = (self.0 + right.0) / 2;
            Self(inner)
        }
    }

    #[test_case(&[], None; "Empty")]
    #[test_case(&[42], Some(Either::Left(&42)); "Single element")]
    #[test_case(&[0, 42], Some(Either::Right((&0, &42))); "Two elements")]
    #[test_case(
        &[0, 42, 69, 666, 1337], Some(Either::Left(&69));
        "Odd number of elements")]
    #[test_case(
        &[0, 1, 42, 69, 666, 1337], Some(Either::Right((&42, &69)));
        "Even number of elements")]
    fn middle(slice: &[i32], expectation: Option<Either<&i32, (&i32, &i32)>>) {
        assert_eq!(expectation, super::middle(&slice))
    }

    #[test_case(&[], None; "Empty slice")]
    #[test_case(
        &[0, 42, 1337], Some((0, 42, 1337));
       "Odd number of elements")]
    #[test_case(
        &[0, 0, 42, 1337], Some((0, 21, 1337));
       "Even number of elements without decimals")]
    #[test_case(
        &[0, 1, 42, 1337], Some((0, 21, 1337));
        "Even number of positive elements with decimals rounded down to zero")]
    #[test_case(
        &[-1337, -42, -1, 0], Some((-1337, -21, 0));
        "Even number of negative elements with decimals rounded up to zero")]
    fn min_med_max_sorted(slice: &[i32], expected: Option<(i32, i32, i32)>) {
        let sut: Vec<TestData> = slice
            .iter()
            .map(|&i| TestData(i))
            .collect();

        let actual = super::min_med_max_sorted(&sut)
            .map(|(min, med, max)| (min.0, med.0, max.0));

        assert_eq!(expected, actual);
    }
}
