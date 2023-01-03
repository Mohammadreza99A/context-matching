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
}

impl FishingContext {
    pub fn new(
        observations: &[Observation],
        nb_of_particles: u16,
        sigma: f64,
        sailing_normal_speed_distr: (f64, f64),
        fishing_normal_speed_distr: (f64, f64),
    ) -> FishingContext {
        FishingContext {
            observations: observations.to_vec(),
            nb_of_particles: nb_of_particles,
            particles: Vec::new(),
            sigma: sigma,
            sailing_normal_speed_distr: sailing_normal_speed_distr,
            fishing_normal_speed_distr: fishing_normal_speed_distr,
        }
    }

    pub fn particle_filter(&mut self) -> Vec<Particle> {
        let mut res_states: Vec<Particle> = Vec::new();

        // Generate initial particles
        for _ in 0..self.nb_of_particles {
            let mut random_context = ParticleContextType::SAILING;
            if (random_uniform() > 0.5) {
                random_context = ParticleContextType::FISHING;
            }
            let particle: Particle = Particle {
                pos: self.observations[0].pos,
                direction: Point {
                    x: random_uniform(),
                    y: random_uniform(),
                },
                heading: self.observations[0].heading,
                speed: self.observations[0].speed,
                context: random_context,
                weight: 1.0 / self.nb_of_particles as f64,
                memory: Vec::new(),
            };
            self.particles.push(particle);
        }

        // self.particle_filter_steps(self.observations[1]);
        for i in 1..self.observations.len() {
            self.particle_filter_steps(self.observations[i]);
        }
        // println!("{:#?}", self.particles);

        res_states
    }

    fn particle_filter_steps(&mut self, observation: Observation) {
        // Sampling
        let resampled_particles = self.resample();
        self.particles = resampled_particles;

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

            // Applying the motion model to generate new particle based on
            // previous one and drawn sample context-state above
            let new_particle = self.update(observation, &self.particles[i]);
            self.particles[i] = new_particle;
        }

        // Measuring or importance weight
        let weighted_new_particles = self.importance_sampling(&observation);
        self.particles = weighted_new_particles;

        // Resampling
        let resampled_new_particles = self.resample();
        self.particles = resampled_new_particles;
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
        let new_heading = random_uniform_range(particle.heading - 22.91, particle.heading + 22.91);
        // if new_heading > 360.0 || new_heading < 0.0 {
        //     println!("{:#?}", new_heading)
        // }

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
        let new_dir_x = particle.direction.x - 0.4 + 2.0 * random_uniform() * 0.4;
        let new_dir_y = particle.direction.y - 0.4 + 2.0 * random_uniform() * 0.4;
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

        Particle {
            pos: new_pos,
            direction: new_dir,
            heading: new_heading,
            speed: new_speed,
            weight: 0.0,
            context: particle.context,
            memory: Vec::new(),
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
