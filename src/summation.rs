pub trait Callable<T> {
    fn call(&self, given: T) -> T;
}

impl<F, T> Callable<T> for F
where
    F: Fn(T) -> T + 'static,
    T: Sized
{
    fn call(&self, given: T) -> T {
        self(given)
    }
}

pub struct InterpolateLookup {
    lookup: Vec<f64>
}

impl InterpolateLookup {
    pub fn try_get_index(&self, given: f64) -> Option<f64> {
        self.lookup.get(given as usize).copied()
    }

    pub fn get_index(&self, given: f64) -> f64 {
        self.try_get_index(given)
            .expect("failed to retrieve given index")
    }

    pub fn len(&self) -> usize {
        self.lookup.len()
    }

    pub fn inner(&self) -> &[f64] {
        self.lookup.as_slice()
    }
}

impl From<Vec<f64>> for InterpolateLookup {
    fn from(given: Vec<f64>) -> Self {
        InterpolateLookup {
            lookup: given
        }
    }
}

impl Callable<f64> for InterpolateLookup {
    fn call(&self, x: f64) -> f64 {
        if x.floor() == x {
            return self.get_index(x);
        }

        let x0 = x.floor();
        let x1 = x0 + 1.0;

        let y0 = self.get_index(x0);
        let y1 = self.get_index(x1);

        y0 + (x - x0) * (y1 - y0)
    }
}

pub type Summation<T> = for<'a> fn(T, T, u32, &'a (dyn Callable<T> + 'a)) -> T;

pub fn left_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized
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

pub fn mid_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized
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

pub fn right_riemann<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized
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

pub fn trapezoidal<T>(lower: f64, upper: f64, iterations: u32, cb: &T) -> f64
where
    T: Callable<f64> + ?Sized
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
