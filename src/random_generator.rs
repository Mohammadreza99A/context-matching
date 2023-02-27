use rand::{distributions::Uniform, thread_rng, Rng};
use rand_distr::{Distribution, Normal};

pub fn random_uniform() -> f64 {
    let mut rng = thread_rng();
    let uniform = Uniform::new(0.0f64, 1.0f64);

    rng.sample(uniform)
}

pub fn random_uniform_range(low: f64, high: f64) -> f64 {
    let mut rng = thread_rng();
    let uniform = Uniform::new(low, high);

    rng.sample(uniform)
}

pub fn random_usize_uniform_range(low: usize, high: usize) -> usize {
    let mut rng = thread_rng();
    let uniform = Uniform::new(low, high);

    rng.sample(uniform)
}

pub fn random_normal(mean: f64, std_dev: f64) -> f64 {
    let mut rng = thread_rng();
    let normal = Normal::new(mean, std_dev).unwrap();

    normal.sample(&mut rng)
}
