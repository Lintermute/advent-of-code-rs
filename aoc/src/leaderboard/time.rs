use std::{
    cmp::Ordering,
    fmt::{Debug, Display},
    time::Duration,
};

use itertools::Itertools;
use lazy_errors::{prelude::*, Result};
use num::integer::{average_ceil, div_rem};

use crate::leaderboard::min_med_max::Mean;

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum Time {
    Exactly(Duration),
    Forever,
}

impl PartialOrd for Time {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Time {
    fn cmp(&self, other: &Self) -> Ordering {
        use Time::*;
        match (self, other) {
            (Exactly(s), Exactly(o)) => s.cmp(o),
            (Exactly(_), Forever) => Ordering::Less,
            (Forever, Exactly(_)) => Ordering::Greater,
            (Forever, Forever) => Ordering::Equal,
        }
    }
}

impl Mean for Time {
    fn mean(&self, right: &Self) -> Self {
        match (self, right) {
            (Time::Exactly(l), Time::Exactly(r)) => {
                let l_secs = l.as_secs();
                let r_secs = r.as_secs();
                let mean_secs = average_ceil(l_secs, r_secs);
                Time::Exactly(Duration::from_secs(mean_secs))
            }
            _ => Time::Forever,
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Time::Exactly(t) => {
                let (h, rem) = div_rem(t.as_secs(), 60 * 60);
                let (m, s) = div_rem(rem, 60);
                let string = format!("{h:02}:{m:02}:{s:02}");
                Display::fmt(&string, f)
            }
            Time::Forever => Display::fmt(&">24h", f),
        }
    }
}

impl TryFrom<&str> for Time {
    type Error = Error;

    fn try_from(text: &str) -> Result<Self> {
        let err_bad_pattern = || {
            Error::from_message(format!(
                "Input does not match pattern hh:mm:ss: '{text}'"
            ))
        };

        match text {
            "" => Err(err_bad_pattern()),
            ">24h" => Ok(Time::Forever),
            _ => text
                .split(':')
                .map(|k| {
                    k.parse::<u64>()
                        .or_wrap_with(|| format!("'{k}' is not a number"))
                })
                .collect::<Result<Vec<_>>>()
                .and_then(|vec| {
                    vec.into_iter()
                        .collect_tuple()
                        .ok_or_else(err_bad_pattern)
                })
                .and_then(|(h, m, s)| {
                    if m >= 60 {
                        return Err(err!("'{m}' not in range 00..60"));
                    }

                    if s >= 60 {
                        return Err(err!("'{s}' not in range 00..60"));
                    }

                    Ok(Time::Exactly(Duration::from_secs(
                        s + 60 * m + 60 * 60 * h,
                    )))
                }),
        }
        .or_wrap_with(|| "Invalid time")
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;

    use crate::leaderboard::min_med_max::min_med_max_sorted;

    use super::*;

    #[test_case("00:00:00")]
    #[test_case("00:00:01")]
    #[test_case("00:01:00")]
    #[test_case("01:00:00")]
    #[test_case("01:02:03")]
    #[test_case("10:20:30")]
    #[test_case("12:34:56")]
    fn time_parse_format_roundtrip(time: &str) -> Result<()> {
        assert_eq!(time, Time::try_from(time)?.to_string());
        Ok(())
    }

    #[test_case("     >24h", Time::Forever)]
    #[test_case(" 00:00:00", Time::Exactly(Duration::from_secs(0)))]
    fn formatting_time_supports_padding(expected_output: &str, time: Time) {
        assert_eq!(format!("{time:>9}"), expected_output);
    }

    #[test_case("00:00:00", 0)]
    #[test_case("00:00:42", 42)]
    #[test_case("00:42:00", 42*60)]
    #[test_case("42:00:00", 42*60*60)]
    fn parse_time_returns_exact_time(
        time: &str,
        total_seconds: u64,
    ) -> Result<()> {
        let duration = Duration::from_secs(total_seconds);
        let expectation = Time::Exactly(duration);
        assert_eq!(Time::try_from(time)?, expectation);
        Ok(())
    }

    #[test]
    fn parse_time_returns_forever() -> Result<()> {
        assert_eq!(Time::try_from(">24h")?, Time::Forever);
        Ok(())
    }

    #[test_case("", "hh:mm:ss"; "empty input")]
    #[test_case("00:00", "hh:mm:ss"; "missing tokens")]
    #[test_case("0A:00:00", "number"; "non-decimal number")]
    #[test_case("00:-1:00", "number"; "negative number")]
    #[test_case("00:60:00", "00..60"; "minutes out of range")]
    #[test_case("00:00:60", "00..60"; "seconds out of range")]
    fn parse_time_fails(time: &str, expected_err_msg: &str) -> Result<()> {
        let err_msg = Time::try_from(time)
            .unwrap_err()
            .to_string();

        if !err_msg.to_uppercase().contains("TIME") {
            return Err(err!("Bad error message: {err_msg}"));
        }

        if !err_msg.contains(expected_err_msg) {
            return Err(err!(
                "Expected '{}' to be part of error message '{}'",
                expected_err_msg,
                err_msg,
            ));
        }

        Ok(())
    }

    #[test_case("00:00:00", "00:00:00", "00:00:00")]
    #[test_case("00:00:30", "00:00:00", "00:00:15")]
    #[test_case("00:00:00", "00:00:30", "00:00:15")]
    #[test_case("00:01:00", "00:00:00", "00:00:30")]
    #[test_case("00:00:00", "00:01:00", "00:00:30")]
    #[test_case("02:03:00", "01:00:00", "01:31:30")]
    #[test_case("01:00:00", "02:03:00", "01:31:30")]
    #[test_case(">24h", "00:00:00", ">24h")]
    #[test_case("00:00:00", ">24h", ">24h")]
    #[test_case(">24h", ">24h", ">24h")]
    fn average(a: &str, b: &str, exp: &str) -> Result<()> {
        let a = Time::try_from(a)?;
        let b = Time::try_from(b)?;
        let exp = Time::try_from(exp)?;
        assert_eq!(exp, a.mean(&b));
        Ok(())
    }

    #[test_case("00:00:00", "00:00:01", Ordering::Less)]
    #[test_case("00:00:00", "00:01:00", Ordering::Less)]
    #[test_case("00:00:59", "00:01:00", Ordering::Less)]
    #[test_case("00:00:00", "01:00:00", Ordering::Less)]
    #[test_case("00:00:59", "01:00:00", Ordering::Less)]
    #[test_case("00:59:59", "01:00:00", Ordering::Less)]
    #[test_case("00:00:00", "00:00:00", Ordering::Equal)]
    #[test_case("00:00:01", "00:00:00", Ordering::Greater)]
    #[test_case("00:01:00", "00:00:00", Ordering::Greater)]
    #[test_case("00:01:00", "00:00:59", Ordering::Greater)]
    #[test_case("01:00:00", "00:00:00", Ordering::Greater)]
    #[test_case("01:00:00", "00:00:59", Ordering::Greater)]
    #[test_case("01:00:00", "00:59:59", Ordering::Greater)]
    #[test_case("01:00:00", ">24h", Ordering::Less)]
    #[test_case(">24h", "01:00:00", Ordering::Greater)]
    #[test_case(">24h", ">24h", Ordering::Equal)]
    fn compare(a: &str, b: &str, exp: Ordering) -> Result<()> {
        let a = Time::try_from(a)?;
        let b = Time::try_from(b)?;

        // Use `partial_cmp` here to test `Ord` and `PartialOrd` in one go
        assert_eq!(Some(exp), a.partial_cmp(&b));

        Ok(())
    }

    #[test_case(
        &["00:00:00", "00:00:00", "00:00:00"],
        "00:00:00", "00:00:00", "00:00:00")]
    #[test_case(
        &["01:00:00", "00:00:01", "00:01:00"],
        "00:00:01", "00:01:00", "01:00:00")]
    #[test_case(
        &["01:00:00", "01:00:00", "00:00:00"],
        "00:00:00", "01:00:00", "01:00:00")]
    #[test_case(
        &[">24h", "00:00:00", ">24h"],
        "00:00:00", ">24h", ">24h")]
    #[test_case(
        &[">24h", "00:00:00", "01:00:00", ">24h"],
        "00:00:00", ">24h", ">24h")]
    fn min_med_max(
        slice: &[&str],
        min: &str,
        med: &str,
        max: &str,
    ) -> Result<()> {
        let mut vec = slice
            .iter()
            .map(|str| Time::try_from(*str))
            .collect::<Result<Vec<_>>>()?;

        vec.sort_unstable();

        let min = Time::try_from(min)?;
        let med = Time::try_from(med)?;
        let max = Time::try_from(max)?;
        assert_eq!(min_med_max_sorted(&vec), Some((min, med, max)));
        Ok(())
    }
}
