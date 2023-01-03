use crate::random_generator::{random_normal, random_uniform, random_uniform_range};

use crate::{
    context::{ContextState, ContextType},
    geometry::Point,
    observation::Observation,
};

#[derive(Debug)]
pub struct FishingContext {
    observations: Vec<Observation>,
    nb_of_particles: u16,
    samples: Vec<ContextState>,
    sigma: f64,
    sailing_normal_speed_distr: (f64, f64),
    fishing_normal_speed_distr: (f64, f64),
    context_smoothing_window: Vec<(u16, u16)>, // (# of sailings, # of fishings)
    context_smoothing_window_size: usize,
}

impl FishingContext {
    pub fn new(
        observations: &[Observation],
        nb_of_particles: u16,
        sigma: f64,
        sailing_normal_speed_distr: (f64, f64),
        fishing_normal_speed_distr: (f64, f64),
        context_window_size: usize,
    ) -> FishingContext {
        FishingContext {
            observations: observations.to_vec(),
            nb_of_particles: nb_of_particles,
            samples: Vec::new(),
            sigma: sigma,
            sailing_normal_speed_distr: sailing_normal_speed_distr,
            fishing_normal_speed_distr: fishing_normal_speed_distr,
            context_smoothing_window: Vec::new(),
            context_smoothing_window_size: context_window_size,
        }
    }

    pub fn particle_filter(&mut self) -> Vec<ContextState> {
        let mut res_states: Vec<ContextState> = Vec::new();

        // Generate the first sample
        for _ in 0..self.nb_of_particles {
            let direction: Point = Point {
                x: random_uniform(),
                y: random_uniform(),
            };
            let heading: f64 = random_uniform_range(0.0, 360.0);
            let state: ContextState = ContextState {
                pos: self.observations[0].pos,
                direction,
                heading,
                speed: 0.0,
                context: ContextType::SAILING,
            };
            let updated_state: ContextState =
                self.update(self.observations[0], &state, self.observations[1].time);
            self.samples.push(updated_state);
        }

        // Generate rest of the samples based on observations
        for i in 0..self.observations.len() - 1 {
            self.updates(self.observations[i], self.observations[i + 1].time);
            let weights: Vec<f64> = self.importance_sampling(&self.observations[i + 1]);
            let max_weight_index = self.find_max_weight_index(weights.as_slice());
            res_states.push(self.samples[max_weight_index]);
            // if self.context_smoothing_window.len() >= self.context_smoothing_window_size {
            //     let ctx = self.smooth_context(self.samples[max_weight_index].context);
            //     let index = res_states.len() - (self.context_smoothing_window_size / 2);
            //     res_states[index].context = ctx;
            // }
            self.resample(weights.as_slice());
        }

        res_states
    }

    fn updates(&mut self, observation: Observation, time_diff: f64) {
        let mut new_samples: Vec<ContextState> = Vec::new();
        let mut sailing_context_count: u16 = 0;
        let mut fishing_context_count: u16 = 0;

        for sample in &self.samples {
            let updated_sample: ContextState = self.update(observation, &sample, time_diff);
            new_samples.push(updated_sample);

            match sample.context {
                ContextType::SAILING => sailing_context_count += 1,
                ContextType::FISHING => fishing_context_count += 1,
            }
        }

        let context_window_elem = (sailing_context_count, fishing_context_count);
        if self.context_smoothing_window.len() < self.context_smoothing_window_size {
            self.context_smoothing_window.push(context_window_elem);
        } else {
            self.context_smoothing_window.drain(0..1);
            self.context_smoothing_window.push(context_window_elem);
        }
        self.samples = new_samples;
    }

    fn update(
        &self,
        observation: Observation,
        sample: &ContextState,
        time_diff: f64,
    ) -> ContextState {
        // Update heading
        // randomly  (uniform) chosen from -0.4 to +0.4 radians
        // (appx 22.91 degree) from previous heading
        let new_heading = random_uniform_range(sample.heading - 22.91, sample.heading + 22.91);

        // Update context
        // 10% chance that the context changes to the other one
        let context_change_prob = random_uniform();
        let mut new_context: ContextType = sample.context;
        if context_change_prob < 0.1 {
            match sample.context {
                ContextType::FISHING => new_context = ContextType::SAILING,
                ContextType::SAILING => new_context = ContextType::FISHING,
            }
        }

        // Update speed
        // The speed is set according to the context
        let new_speed = match sample.context {
            ContextType::FISHING => random_normal(
                self.fishing_normal_speed_distr.0,
                self.fishing_normal_speed_distr.1,
            ),
            ContextType::SAILING => random_normal(
                self.sailing_normal_speed_distr.0,
                self.sailing_normal_speed_distr.1,
            ),
        };

        // Update position
        let new_dir_x = sample.direction.x - 0.4 + 2.0 * random_uniform() * 0.4;
        let new_dir_y = sample.direction.y - 0.4 + 2.0 * random_uniform() * 0.4;
        let mut new_dir = Point {
            x: new_dir_x,
            y: new_dir_y,
        };
        let norm = 1.0 / new_dir.norm();
        new_dir.x = new_dir.x * norm;
        new_dir.y = new_dir.y * norm;
        let new_pos = Point {
            // x: sample.pos.x + (new_speed * time_diff * new_dir.x),
            x: observation.pos.x + (new_speed * time_diff * new_dir.x),
            // y: sample.pos.y + (new_speed * time_diff * new_dir.y),
            y: observation.pos.y + (new_speed * time_diff * new_dir.y),
        };

        ContextState {
            pos: new_pos,
            direction: new_dir,
            heading: new_heading,
            speed: new_speed,
            context: new_context,
        }
    }

    fn importance_sampling(&self, observation: &Observation) -> Vec<f64> {
        let mut weighted_samples: Vec<f64> = Vec::new();

        for state in &self.samples {
            let mut weight = self.calc_emission_prob(observation, &state);

            if state.context == ContextType::FISHING {
                let scaled_distance_from_shore = Point::scaled_weighted_distance_from_line(
                    observation.distance_to_shore,
                    self.observations[0].distance_to_shore,
                    1.0f64,
                );
                weight = weight * scaled_distance_from_shore;
            }

            weighted_samples.push(weight);
        }

        weighted_samples
    }

    fn resample(&mut self, weights: &[f64]) {
        let mut new_samples: Vec<ContextState> = Vec::new();
        let mut t = 0.0f64;
        let mut k: Vec<f64> = vec![0.0; weights.len()];

        for i in 0..weights.len() {
            t += weights[i];
            k[i] = t;
        }

        for _ in 0..self.nb_of_particles {
            let mut t2 = 0.0f64;
            if t > 0.0 {
                t2 = random_uniform_range(0.0, t);
            }

            let mut j: usize = 0;
            while k[j] < t2 {
                j += 1;
            }
            new_samples.push(self.samples[j]);
        }

        self.samples = new_samples;
    }

    fn smooth_context(&mut self, current_context: ContextType) -> ContextType {
        let mid_index = self.context_smoothing_window_size / 2;

        let mut left_hand_sailing = 0;
        let mut left_hand_fishing = 0;
        let mut right_hand_sailing = 0;
        let mut right_hand_fishing = 0;

        for i in 0..mid_index {
            left_hand_sailing += self.context_smoothing_window[i].0;
            left_hand_fishing += self.context_smoothing_window[i].1;
        }

        for i in mid_index + 1..self.context_smoothing_window_size {
            right_hand_sailing += self.context_smoothing_window[i].0;
            right_hand_fishing += self.context_smoothing_window[i].1;
        }

        let sailing_total = left_hand_sailing + right_hand_sailing;
        let fishing_total = left_hand_fishing + right_hand_fishing;

        if sailing_total > fishing_total {
            if self.context_smoothing_window[mid_index].0 >= 50 {
                return ContextType::SAILING;
            }
        } else {
            if self.context_smoothing_window[mid_index].1 >= 50 {
                return ContextType::FISHING;
            }
        }

        current_context
    }

    fn find_max_weight_index(&self, weights: &[f64]) -> usize {
        let max_weight: f64 = weights.iter().copied().fold(f64::NAN, f64::max);
        weights.iter().position(|&r| r == max_weight).unwrap()
    }

    fn calc_emission_prob(&self, observation: &Observation, state: &ContextState) -> f64 {
        let p: Point = Point {
            x: observation.pos.x - state.pos.x,
            y: observation.pos.y - state.pos.y,
        };
        let two_pi = 2.0f64 * std::f64::consts::PI;
        let gc = p.norm();
        let first_term = 1.0 / (two_pi.sqrt() * self.sigma);
        let second_term = -0.5 * (gc / self.sigma).powf(2.0);
        first_term * second_term.exp()
    }
}
