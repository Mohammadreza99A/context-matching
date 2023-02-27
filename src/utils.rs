use num_traits::Float;
use rand_distr::num_traits;

pub fn linspace<T: Float + std::convert::From<u16>>(l: T, h: T, n: usize) -> Vec<T> {
    let size: T = (n as u16 - 1)
        .try_into()
        .expect("too many elements: max is 2^16");
    let dx = (h - l) / size;

    (1..=n)
        .scan(-dx, |a, _| {
            *a = *a + dx;
            Some(*a)
        })
        .collect()
}