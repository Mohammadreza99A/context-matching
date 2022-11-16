use rand::{distributions::Uniform, thread_rng, Rng};
use rand_distr::{Distribution, Normal};

use crate::{
    context::{ContextState, ContextType},
    geometry::Point,
    observation::Observation,
};

#[derive(Debug)]
pub struct FishingContext {
    pub observations: Vec<Observation>,
    pub nb_of_particles: u16,
    pub samples: Vec<ContextState>,
    pub sigma: f64,
    pub alpha: f64,
}

impl FishingContext {
    pub fn particle_filter(&mut self) -> Vec<ContextState> {
        let mut rng = thread_rng();
        let uniform_between_0_1 = Uniform::new(0.0f64, 1.0f64);
        let uniform_between_0_360 = Uniform::new(0.0f64, 360.0f64);

        let mut res_states: Vec<ContextState> = Vec::new();
        // let mut samples: Vec<ContextState> = Vec::new();

        // Generate the first sample
        for _ in 0..self.nb_of_particles {
            let direction: Point = Point {
                x: rng.sample(uniform_between_0_1),
                y: rng.sample(uniform_between_0_1),
            };
            let heading: f64 = rng.sample(uniform_between_0_360);
            let state: ContextState = ContextState {
                pos: self.observations[0].pos,
                direction,
                heading,
                speed: 0.0f64,
                context: ContextType::SAILING,
            };
            let updated_state: ContextState = self.update(&state, self.observations[1].time);
            self.samples.push(updated_state);
        }

        // Generate rest of the samples based on observations
        for i in 0..self.observations.len() - 1 {
            // self.samples = self.updates(self.samples.as_slice(), self.observations[i + 1].time);
            // self.samples = self.updates(self.observations[i + 1].time);
            self.updates(self.observations[i + 1].time);
            let weights: Vec<f64> =
                // self.importance_sampling(samples.as_slice(), &self.observations[i + 1]);
                self.importance_sampling(&self.observations[i + 1]);
            let max_weight: f64 = weights.iter().copied().fold(f64::NAN, f64::max);
            let max_weight_index: usize = weights.iter().position(|&r| r == max_weight).unwrap();
            res_states.push(self.samples[max_weight_index]);
            // samples = self.resample(samples.as_slice(), weights.as_slice());
            // self.samples = self.resample(weights.as_slice());
            self.resample(weights.as_slice());
        }

        res_states
    }

    // fn updates(&self, samples: &[ContextState], time_diff: f64) -> Vec<ContextState> {
    // fn updates(&self, time_diff: f64) -> Vec<ContextState> {
    fn updates(&mut self, time_diff: f64) {
        let mut new_samples: Vec<ContextState> = Vec::new();

        for sample in &self.samples {
            new_samples.push(self.update(&sample, time_diff));
        }

        self.samples = new_samples;
        // new_samples
    }

    fn update(&self, sample: &ContextState, time_diff: f64) -> ContextState {
        let mut rng = thread_rng();
        let uniform_between_0_1 = Uniform::new(0.0f64, 1.0f64);

        // Update heading
        // randomly  (uniform) chosen from -0.4 to +0.4 radians
        // (appx 22.91 degree) from previous heading
        let heading_uniform = Uniform::new(sample.heading - 22.91f64, sample.heading + 22.91f64);
        let new_heading: f64 = rng.sample(heading_uniform);

        // Update context
        // 10% chance that the context changes to the other one
        let context_change_prob: f64 = rng.sample(uniform_between_0_1);
        let mut new_context: ContextType = sample.context;
        if context_change_prob < 0.1f64 {
            match sample.context {
                ContextType::FISHING => new_context = ContextType::SAILING,
                ContextType::SAILING => new_context = ContextType::FISHING,
            }
        }

        // Update speed
        // The speed is set according to the context
        let new_speed: f64 = match sample.context {
            ContextType::FISHING => {
                let mut rng = thread_rng();
                // let normal = Normal::new(1.36f64, 0.89f64).unwrap();
                let normal = Normal::new(3.5232835282106914f64, 1.8926981846601172f64).unwrap();
                normal.sample(&mut rng)
            }
            ContextType::SAILING => {
                let mut rng = thread_rng();
                // let normal = Normal::new(3.31f64, 1.19f64).unwrap();
                let normal = Normal::new(5.8554435066212704f64, 1.5104887135097356f64).unwrap();
                normal.sample(&mut rng)
            }
        };

        // Update position
        let new_dir_x: f64 = sample.direction.x - 0.4 + 2.0 * rng.sample(uniform_between_0_1) * 0.4;
        let new_dir_y: f64 = sample.direction.y - 0.4 + 2.0 * rng.sample(uniform_between_0_1) * 0.4;
        let mut new_dir: Point = Point {
            x: new_dir_x,
            y: new_dir_y,
        };
        let norm: f64 = 1.0f64 / self.norm(&new_dir);
        new_dir = Point {
            x: new_dir.x * norm,
            y: new_dir.y * norm,
        };
        let new_pos_x: f64 = sample.pos.x + (new_speed * time_diff * new_dir.x);
        let new_pos_y: f64 = sample.pos.y + (new_speed * time_diff * new_dir.y);
        let new_pos: Point = Point {
            x: new_pos_x,
            y: new_pos_y,
        };

        ContextState {
            pos: new_pos,
            direction: new_dir,
            heading: new_heading,
            speed: new_speed,
            context: new_context,
        }
    }

    // fn importance_sampling(&self, states: &[ContextState], observation: &Observation) -> Vec<f64> {
    fn importance_sampling(&self, observation: &Observation) -> Vec<f64> {
        let mut weighted_samples: Vec<f64> = Vec::new();

        for state in &self.samples {
            weighted_samples.push(self.alpha * self.calc_emission_prob(observation, &state))
        }

        weighted_samples
    }

    // fn resample(&self, states: &[ContextState], weights: &f64]) -> Vec<ContextState> {
    // fn resample(&self, weights: &[f64]) -> Vec<ContextState> {
    fn resample(&mut self, weights: &[f64]) {
        let mut new_samples: Vec<ContextState> = Vec::new();
        let mut t = 0.0f64;
        let mut k: Vec<f64> = vec![0.0f64; weights.len()];

        for i in 0..weights.len() {
            t += weights[i];
            k[i] = t;
        }

        for _ in 0..self.nb_of_particles {
            let mut t2 = 0.0f64;
            if t > 0.0 {
                let mut rng = thread_rng();
                let uniform_dist = Uniform::new(0.0, t);
                t2 = rng.sample(uniform_dist);
            }

            let mut j: usize = 0;
            while k[j] < t2 {
                j += 1;
            }
            new_samples.push(self.samples[j]);
        }

        self.samples = new_samples;
        // new_samples
    }

    fn calc_emission_prob(&self, observation: &Observation, state: &ContextState) -> f64 {
        let p: Point = Point {
            x: observation.pos.x - state.pos.x,
            y: observation.pos.y - state.pos.y,
        };
        let two_pi: f64 = 2.0f64 * 3.141592653589793f64;
        let gc: f64 = self.norm(&p);
        let first_term: f64 = 1.0 / (two_pi.sqrt() * self.sigma);
        let second_term: f64 = -0.5 * (gc / self.sigma).powf(2.0);
        first_term * second_term.exp()
    }

    fn norm(&self, point: &Point) -> f64 {
        let f = point.x.powf(2.0) + point.y.powf(2.0);
        f.sqrt()
    }
}
