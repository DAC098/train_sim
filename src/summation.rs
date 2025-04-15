//! functions for calculating summations and utility structs for ease of use
//! with the summations

/// defines something that can be called with a single argument and then return
/// a single value
pub trait Callable<T> {
    /// calls the struct with the given value and a value of the same type
    fn call(&self, given: T) -> T;
}

impl<F, T> Callable<T> for F
where
    F: Fn(T) -> T + 'static,
    T: Sized,
{
    fn call(&self, given: T) -> T {
        self(given)
    }
}

/// provides interpolated lookups between values stored
///
/// each index of the table is considered the x value and each value stored at
/// that index is considered the y value.
///
/// if the desired x value lands on an index then the value stored at that index
/// will be returned. otherwise it will return the interpolated value between
/// the floor of the given x value and the next available one.
///
/// by default, it will expect all provided x values to be retrievable in some
/// way and will panic if it cannot retrieve the necessary indexs from the
/// table.
///
/// you can manually fill the lookup table from an empty vector
/// ```
/// use Callable;
///
/// let mut lt = InterpolateLookup::from(Vec::new());
/// // list of y values available
/// lt.push(0.0);
/// lt.push(1.5);
/// lt.push(3.0);
///
/// // the requested y value at the given point x
/// lt.call(1.5);
/// ```
///
/// or fill it with a vector of filled values
/// ```
/// let mut lt = InterpolateLookup::from(vec![0.0, 1.5, 3.0]);
///
/// lt.call(1.5);
/// ```
#[derive(Debug, Clone)]
pub struct InterpolateLookup {
    lookup: Vec<f64>,
}

impl InterpolateLookup {
    /// attempt to retrieve a value from the lookup table with the given index
    ///
    /// the [`f64`] will be cast to a [`usize`] and then attempt to retrieve a
    /// copied value
    pub fn try_get_index(&self, given: f64) -> Option<f64> {
        self.lookup.get(given as usize).copied()
    }

    /// retrieve a value from the lookup table with the given index
    ///
    /// panics if the desired index is not found in the lookup table
    pub fn get_index(&self, given: f64) -> f64 {
        self.try_get_index(given)
            .expect("failed to retrieve given index")
    }

    /// returns the current length of the lookup table
    pub fn len(&self) -> usize {
        self.lookup.len()
    }

    /// adds a new value to the end of the lookup table
    pub fn push(&mut self, given: f64) {
        self.lookup.push(given);
    }
}

impl From<Vec<f64>> for InterpolateLookup {
    fn from(given: Vec<f64>) -> Self {
        Self { lookup: given }
    }
}

impl Callable<f64> for InterpolateLookup {
    fn call(&self, x: f64) -> f64 {
        let x0 = x.floor();

        // check to see if the given x is a whole number, if so then dont
        // interpolate and instead just retrieve the value at that index
        // if possible
        if x0 == x {
            return self.get_index(x);
        }

        let x1 = x0 + 1.0;

        let y0 = self.get_index(x0);
        let y1 = self.get_index(x1);

        // if x1 is always 1 greater than x0 then it can be removed and just be
        // 1, otherwise this: y0 + (x - x0) * ((y1 - y0) / (x1 - x0))
        y0 + (x - x0) * (y1 - y0)
    }
}

/// performs a left riemann summation with the given callable
pub fn left_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized,
{
    assert_ne!(iterations, 0);

    let step = (upper - lower) / (iterations as f64);
    let mut sum = 0.0;

    for iter in 0..iterations {
        let i = iter as f64;
        let x = lower + i * step;

        sum += cb.call(x);
    }

    sum * step
}

/// performs a midpoint riemann summation with the given callable
pub fn mid_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized,
{
    assert_ne!(iterations, 0);

    let step = (upper - lower) / (iterations as f64);
    let half = step / 2.0;
    let mut sum = 0.0;

    for iter in 0..iterations {
        let i = iter as f64;
        let x = (lower + i * step) + half;

        sum += cb.call(x);
    }

    sum * step
}

/// performs a right riemann summation with the given callable
pub fn right_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized,
{
    assert_ne!(iterations, 0);

    let step = (upper - lower) / (iterations as f64);
    let mut sum = 0.0;

    for iter in 0..iterations {
        let i = (iter + 1) as f64;
        let x = lower + i * step;

        sum += cb.call(x);
    }

    sum * step
}

/// performs a trapezoidal summation with the given callable
pub fn trapezoidal<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized,
{
    assert_ne!(iterations, 0);

    let step = (upper - lower) / (iterations as f64);
    let mut sum = (cb.call(upper) + cb.call(lower)) / 2.0;

    for iter in 0..iterations {
        let i = iter as f64;
        let x = lower + i * step;

        sum += cb.call(x);
    }

    sum * step
}

/// performs simpsons summation with the given callable
pub fn simpsons<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized,
{
    assert_ne!(iterations, 0);

    let step = (upper - lower) / (iterations as f64);
    let mut sum = 0.0;

    for iter in 0..=iterations {
        let i = iter as f64;
        let x = lower + i * step;
        let res = cb.call(x);

        if iter == 0 || iter == iterations {
            sum += res;
        } else if iter % 2 == 1 {
            sum += 4.0 * res;
        } else {
            sum += 2.0 * res;
        }
    }

    step * sum / 3.0
}

#[cfg(test)]
mod test {
    use approx::assert_relative_eq;

    use super::*;

    fn simple_curve(x: f64) -> f64 {
        4.0 * x - x * x
    }

    #[test]
    fn left() {
        let calc = left_riemann(0.0, 4.0, 4, &simple_curve);

        assert_relative_eq!(calc, 10.0);
    }

    #[test]
    fn mid() {
        let calc = mid_riemann(0.0, 4.0, 4, &simple_curve);

        assert_relative_eq!(calc, 11.0);
    }

    #[test]
    fn right() {
        let calc = right_riemann(0.0, 4.0, 4, &simple_curve);

        assert_relative_eq!(calc, 10.0);
    }

    #[test]
    fn trap() {
        let calc = trapezoidal(0.0, 4.0, 4, &simple_curve);

        assert_relative_eq!(calc, 10.0);
    }

    #[test]
    fn interpolate() {
        let lookup = InterpolateLookup::from(vec![0.0, 1.0, 2.0]);

        assert_relative_eq!(lookup.call(0.0), 0.0);
        assert_relative_eq!(lookup.call(1.0), 1.0);
        assert_relative_eq!(lookup.call(0.5), 0.5);
        assert_relative_eq!(lookup.call(1.5), 1.5);
    }
}
