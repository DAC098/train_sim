//! contains timing utility structs

use std::default::Default;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::{Duration, Instant};

/// collects timing information for convience
///
/// tracks the minimum, maximum, total, and count of the values provided to the
/// [`Timing::update`] function. can also be [`Display`]ed to show the minimum,
/// maximum, average, and total time values stored.
///
/// if the total timing information collected is only 1 then it will only
/// display the total time as all the values will be the same.
///
/// ```
/// let mut timing = Timing::default();
///
/// for _ in 0..10 {
///     let start = std::time::Instant::now();
///
///     // do something
///
///     timing.update(start.elapsed());
/// }
///
/// println!("timings: {timing}");
/// ```
pub struct Timing {
    min: Duration,
    max: Duration,
    total: Duration,
    counted: u32,
}

impl Timing {
    /// updates tracked values with the given duration
    pub fn update(&mut self, given: Duration) {
        if self.min > given {
            self.min = given;
        }

        if self.max < given {
            self.max = given;
        }

        self.total += given;
        self.counted += 1;
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            min: Duration::MAX,
            max: Duration::ZERO,
            total: Duration::ZERO,
            counted: 0,
        }
    }
}

impl Display for Timing {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        if self.counted > 1 {
            let avg = self.total / self.counted;

            write!(
                f,
                "min: {}.{:09}\nmax: {}.{:09}\navg: {}.{:09}\ntot: {}.{:09}",
                self.min.as_secs(),
                self.min.subsec_nanos(),
                self.max.as_secs(),
                self.max.subsec_nanos(),
                avg.as_secs(),
                avg.subsec_nanos(),
                self.total.as_secs(),
                self.total.subsec_nanos(),
            )
        } else {
            write!(
                f,
                "total: {}.{:09}",
                self.total.as_secs(),
                self.total.subsec_nanos()
            )
        }
    }
}

/// timer that will indicate if a certain amout of time has passed since the
/// previously stored value
///
/// by default, the duration of time that can pass is 10 seconds
///
/// ```
/// let mut timer = LogTimer::default();
///
/// // do work of some kind
///
/// if timer.update() {
///     println!("10 seconds have passed");
/// } else {
///     println!("10 seconds have not passed");
/// }
/// ```
pub struct LogTimer {
    /// the last recorded time from the update
    ///
    /// during initialization it will be from [`Instant::now`]
    last: Instant,
    /// total duration of time that can pass
    ///
    /// defaults to 10 seconds
    drtn: Duration,
}

impl LogTimer {
    /// checks the current timestamp with the internal time and updates if the
    /// difference is greater than the specified duration
    pub fn update(&mut self) -> bool {
        let now = Instant::now();

        if now - self.last > self.drtn {
            self.last = now;

            true
        } else {
            false
        }
    }
}

impl Default for LogTimer {
    fn default() -> Self {
        Self {
            last: Instant::now(),
            drtn: Duration::from_secs(10),
        }
    }
}
