use std::default::Default;
use std::fmt::{Display, Formatter, Result as FmtResult};
use std::time::Duration;

pub struct Timing {
    min: Duration,
    max: Duration,
    total: Duration,
    counted: u32,
}

impl Timing {
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
                "min {:#?} max {:#?} avg {:#?} total: {:#?}",
                self.min, self.max, avg, self.total
            )
        } else {
            write!(f, "{:#?}", self.total)
        }
    }
}
