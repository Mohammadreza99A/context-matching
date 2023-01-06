use crate::random_generator::{random_normal, random_uniform, random_uniform_range};

use crate::{
    geometry::Point,
    observation::Observation,
    particle::{Particle, ParticleContextType},
};

#[derive(Debug)]
pub struct FishingContext {
    observations: Vec<Observation>,
    nb_of_particles: u16,
    particles: Vec<Particle>,
    sigma: f64,
    sailing_normal_speed_distr: (f64, f64),
    fishing_normal_speed_distr: (f64, f64),
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
            particles: Vec::new(),
            sigma: sigma,
            sailing_normal_speed_distr: sailing_normal_speed_distr,
            fishing_normal_speed_distr: fishing_normal_speed_distr,
            context_smoothing_window_size: context_window_size,
        }
    }

    pub fn particle_filter(&mut self) -> Vec<Observation> {
        // Generate initial particles
        for _i in 0..self.nb_of_particles {
            let mut random_context = ParticleContextType::SAILING;
            if random_uniform() > 0.5 {
                random_context = ParticleContextType::FISHING;
            }
            // if i > self.nb_of_particles / 2 {
            //     random_context = ParticleContextType::FISHING;
            // }
            let mut particle: Particle = Particle {
                pos: self.observations[0].pos,
                direction: Point {
                    x: self.observations[0].heading.sin(),
                    // x: random_uniform(),
                    y: self.observations[0].heading.cos(),
                    // y: random_uniform(),
                },
                heading: self.observations[0].heading,
                speed: self.observations[0].speed,
                context: random_context,
                weight: 1.0 / self.nb_of_particles as f64,
                memory: Vec::new(),
            };
            particle.memory.push(random_context);
            self.particles.push(particle);
        }

        for i in 1..self.observations.len() {
            self.particle_filter_steps(self.observations[i]);
        }

        self.calc_optimal_sequence()
    }

    fn particle_filter_steps(&mut self, observation: Observation) {
        // Sampling
        self.particles = self.resample();

        // Update/Drift & Diffuse
        for i in 0..self.particles.len() {
            // Drawing a sample context-state based on transition probabilities
            let mut new_context: ParticleContextType = self.particles[i].context;
            if random_uniform() < 0.1 {
                match new_context {
                    ParticleContextType::SAILING => new_context = ParticleContextType::FISHING,
                    ParticleContextType::FISHING => new_context = ParticleContextType::SAILING,
                }
            }
            self.particles[i].context = new_context;
            self.particles[i].memory.push(new_context);

            // Applying the motion model to generate new particle based on
            // previous one and drawn sample context-state above
            self.particles[i] = self.update(observation, &self.particles[i]);
        }

        // Assigning weights
        self.particles = self.importance_sampling(&observation);
        let mut weight_normalization: f64 = 0.0;
        for i in 0..self.particles.len() {
            weight_normalization += self.particles[i].weight;
        }
        for i in 0..self.particles.len() {
            self.particles[i].weight = self.particles[i].weight / weight_normalization;
        }
    }

    fn resample(&mut self) -> Vec<Particle> {
        let mut new_particles: Vec<Particle> = Vec::new();
        let mut t = 0.0f64;
        let mut k: Vec<f64> = vec![0.0; self.particles.len()];

        for i in 0..self.particles.len() {
            t += self.particles[i].weight;
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
            new_particles.push(self.particles[j].clone());
        }

        new_particles
    }

    fn update(&self, observation: Observation, particle: &Particle) -> Particle {
        let time_diff = observation.time;

        // Update heading
        // randomly  (uniform) chosen from -0.4 to +0.4 radians
        // (appx 22.91 degree) from previous heading
        let mut heading_low = ((particle.heading - 22.91) + 360.0) % 360.0;
        let mut heading_high = (particle.heading + 22.91) % 360.0;
        if heading_high < heading_low {
            let heading_tmp = heading_high;
            heading_high = heading_low;
            heading_low = heading_tmp;
        }
        let new_heading = random_uniform_range(heading_low, heading_high);

        // Update speed
        // The speed is set according to the context
        let new_speed = match particle.context {
            ParticleContextType::FISHING => random_normal(
                self.fishing_normal_speed_distr.0,
                self.fishing_normal_speed_distr.1,
            ),
            ParticleContextType::SAILING => random_normal(
                self.sailing_normal_speed_distr.0,
                self.sailing_normal_speed_distr.1,
            ),
        };

        // Update position
        let mut new_dir = Point {
            x: new_heading.sin(),
            y: new_heading.cos(),
        };
        let norm = 1.0 / new_dir.norm();
        new_dir.x = new_dir.x * norm;
        new_dir.y = new_dir.y * norm;
        let new_pos = Point {
            x: particle.pos.x + (new_speed * time_diff * new_dir.x),
            y: particle.pos.y + (new_speed * time_diff * new_dir.y),
        };

        Particle {
            pos: new_pos,
            direction: new_dir,
            heading: new_heading,
            speed: new_speed,
            weight: particle.weight,
            context: particle.context,
            memory: particle.memory.clone(),
        }
    }

    fn importance_sampling(&self, observation: &Observation) -> Vec<Particle> {
        let mut weighted_particles: Vec<Particle> = Vec::new();

        for particle in &self.particles {
            let weight = self.calc_emission_prob(observation, &particle);
            let weighted_new_particle = Particle {
                pos: particle.pos,
                direction: particle.direction,
                heading: particle.heading,
                speed: particle.speed,
                weight,
                context: particle.context,
                memory: particle.memory.clone(),
            };
            weighted_particles.push(weighted_new_particle);
        }

        weighted_particles
    }

    fn calc_optimal_sequence(&self) -> Vec<Observation> {
        let mut smoothing_window: Vec<(u16, u16)> = Vec::new();

        let mut optimal_sequence: Vec<Observation> = Vec::new();

        for i in 0..self.observations.len() {
            let mut obs_memory: Vec<ParticleContextType> = Vec::new();
            for j in 0..self.particles.len() {
                obs_memory.push(self.particles[j].memory[i]);
            }

            let mut sailing_count: u16 = 0;
            let mut fishing_count: u16 = 0;

            for memory in obs_memory {
                match memory {
                    ParticleContextType::SAILING => sailing_count += 1,
                    ParticleContextType::FISHING => fishing_count += 1,
                }
            }

            // Majority context
            let mut majority_context = ParticleContextType::SAILING;
            if sailing_count < fishing_count {
                majority_context = ParticleContextType::FISHING;
            }

            let obs_with_context = Observation {
                pos: self.observations[i].pos,
                time: self.observations[i].time,
                heading: self.observations[i].heading,
                speed: self.observations[i].speed,
                context: majority_context,
                distance_to_shore: self.observations[i].distance_to_shore,
            };
            optimal_sequence.push(obs_with_context);

            // For smoothing
            let context_window_elem = (sailing_count, fishing_count);
            if smoothing_window.len() < self.context_smoothing_window_size {
                smoothing_window.push(context_window_elem);
            } else {
                smoothing_window.drain(0..1);
                smoothing_window.push(context_window_elem);
            }

            // Smoothing
            if i >= self.context_smoothing_window_size {
                let current_context =
                    optimal_sequence[i - (self.context_smoothing_window_size / 2)].context;
                let smoothed_context =
                    self.smooth_context(current_context, smoothing_window.as_slice());
                optimal_sequence[i - (self.context_smoothing_window_size / 2)].context =
                    smoothed_context;
            }
        }

        optimal_sequence
    }

    fn smooth_context(
        &self,
        current_context: ParticleContextType,
        smoothing_window: &[(u16, u16)],
    ) -> ParticleContextType {
        let mid_index = self.context_smoothing_window_size / 2;

        let mut left_hand_sailing = 0;
        let mut left_hand_fishing = 0;
        let mut right_hand_sailing = 0;
        let mut right_hand_fishing = 0;

        for i in 0..mid_index {
            left_hand_sailing += smoothing_window[i].0;
            left_hand_fishing += smoothing_window[i].1;
        }

        for i in mid_index + 1..self.context_smoothing_window_size {
            right_hand_sailing += smoothing_window[i].0;
            right_hand_fishing += smoothing_window[i].1;
        }

        let sailing_total = left_hand_sailing + right_hand_sailing;
        let fishing_total = left_hand_fishing + right_hand_fishing;

        if sailing_total > fishing_total {
            if smoothing_window[mid_index].0 >= 50 {
                return ParticleContextType::SAILING;
            }
        } else {
            if smoothing_window[mid_index].1 >= 50 {
                return ParticleContextType::FISHING;
            }
        }

        current_context
    }

    fn calc_emission_prob(&self, observation: &Observation, particle: &Particle) -> f64 {
        let p: Point = Point {
            x: observation.pos.x - particle.pos.x,
            y: observation.pos.y - particle.pos.y,
        };
        let two_pi = 2.0f64 * std::f64::consts::PI;
        let gc = p.norm();
        let first_term = 1.0 / (two_pi.sqrt() * self.sigma);
        let second_term = -0.5 * (gc / self.sigma).powf(2.0);
        first_term * second_term.exp()
    }
}
