use crate::random_generator::{
    random_normal, random_uniform, random_uniform_range, random_usize_uniform_range,
};
use crate::{
    geometry::Point,
    markov_graph::{read_graph_from_file, MarkovGraph},
    observation::Observation,
    particle::{Particle, ParticleContextType},
};

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};

#[derive(Debug)]
pub struct FishingContext {
    observations: Vec<Observation>,
    nb_of_particles: u16,
    particles: Vec<Particle>,
    sigma: f64,
    sailing_normal_speed_distr: (f64, f64),
    fishing_normal_speed_distr: (f64, f64),
    context_smoothing_window_size: usize,
    markov_graph: MarkovGraph<ParticleContextType>,
    is_record_history: bool,
    history_file_path: String,
}

impl FishingContext {
    pub fn new(
        observations: &[Observation],
        nb_of_particles: u16,
        sigma: f64,
        sailing_normal_speed_distr: (f64, f64),
        fishing_normal_speed_distr: (f64, f64),
        context_window_size: usize,
        history_file: Option<String>,
    ) -> FishingContext {
        let markov_graph: MarkovGraph<ParticleContextType> = read_graph_from_file("src/graph.txt");
        println!("\nHere is the Markov graph: \n{}", markov_graph);

        FishingContext {
            observations: observations.to_vec(),
            nb_of_particles: nb_of_particles,
            particles: Vec::new(),
            sigma: sigma,
            sailing_normal_speed_distr: sailing_normal_speed_distr,
            fishing_normal_speed_distr: fishing_normal_speed_distr,
            context_smoothing_window_size: context_window_size,
            markov_graph,
            is_record_history: history_file.is_some(),
            history_file_path: history_file.unwrap_or_default(),
        }
    }

    pub fn particle_filter(&mut self) -> Vec<Observation> {
        // Generate initial particles
        for _i in 0..self.nb_of_particles {
            let mut random_context = ParticleContextType::GoFishing;

            let mut particle: Particle = Particle {
                pos: self.observations[0].pos,
                direction: Point {
                    x: self.observations[0].heading.cos(),
                    y: self.observations[0].heading.sin(),
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

        if self.is_record_history {
            // Open history file and wrap the file in a buffered writer
            let mut writer = BufWriter::new(
                File::create(&self.history_file_path).expect("failed to create file"),
            );

            // Write headers to the file
            let headers = format!(
                "id,obs_ctx,{}",
                (1..=self.nb_of_particles)
                    .map(|i| format!("p_{}", i))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            writer
                .write(headers.as_bytes())
                .expect("failed to write to file");

            // Add initial particles to history
            write!(writer, "\n{},{}", 0, self.observations[0].context).unwrap();
            self.add_to_history(&mut writer);

            // Apply particle filtering for all observations
            for i in 1..self.observations.len() {
                self.particle_filter_steps(self.observations[i]);

                // Add particles to history
                write!(writer, "\n{},{}", i, self.observations[i].context).unwrap();
                self.add_to_history(&mut writer);
                writer.flush().expect("failed to flush buffer");
            }

            // Flush the buffer to ensure that any remaining data is written to the file
            writer.flush().expect("failed to flush buffer");
        } else {
            // Apply particle filtering for all observations
            for i in 1..self.observations.len() {
                self.particle_filter_steps(self.observations[i]);
            }
        }

        self.calc_optimal_sequence()
    }

    fn add_to_history(&self, writer: &mut BufWriter<File>) {
        for particle in &self.particles {
            write!(writer, ",{}", particle.context).unwrap();
        }
    }

    fn particle_filter_steps(&mut self, observation: Observation) {
        // Importance sampling
        self.particles = self.resample();

        // Update/Drift & Diffuse
        for i in 0..self.particles.len() {
            // Drawing a sample context-state based on transition probabilities
            // 10% chance that the context changes to another one
            let mut new_context: ParticleContextType = self.particles[i].context;
            if random_uniform() < 0.1 {
                new_context = self.markov_graph.get_dest(new_context, 0.1).unwrap();
            }

            // Add context to memory
            self.particles[i].context = new_context;
            self.particles[i].memory.push(new_context);

            // Applying the motion model to generate new particle based on
            // previous one and drawn sample context-state above
            self.particles[i] = self.update(observation, &self.particles[i]);
        }

        // Assigning weights
        self.particles = self.weight_measurement(&observation);
        let weight_sum = self.particles.iter().map(|p| p.weight).sum::<f64>();
        self.particles
            .iter_mut()
            .for_each(|p| p.weight /= weight_sum);
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

        // Update speed
        let new_speed = match particle.context {
            ParticleContextType::Fishing => random_normal(
                self.fishing_normal_speed_distr.0,
                self.fishing_normal_speed_distr.1,
            ),
            ParticleContextType::GoFishing => random_normal(
                self.sailing_normal_speed_distr.0,
                self.sailing_normal_speed_distr.1,
            ),
            ParticleContextType::GoToPort => random_normal(
                self.sailing_normal_speed_distr.0,
                self.sailing_normal_speed_distr.1,
            ),
        };
        let distance = new_speed * time_diff;

        // Update heading
        let new_heading = self.generate_new_random_heading(&particle);

        // Update direction
        let new_dir = self.calc_new_direction(new_heading);

        // Update position
        let new_pos = Point {
            x: particle.pos.x + (distance * new_dir.x),
            y: particle.pos.y + (distance * new_dir.y),
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

    fn weight_measurement(&self, observation: &Observation) -> Vec<Particle> {
        self.particles
            .iter()
            .map(|particle| {
                let weight = self.calc_emission_prob(observation, particle);
                Particle {
                    weight,
                    memory: particle.memory.clone(),
                    ..*particle
                }
            })
            .collect()
    }

    fn generate_new_random_heading(&self, particle: &Particle) -> f64 {
        let coast = self.observations[0].pos;

        if particle.pos.x == coast.x && particle.pos.y == coast.y {
            let heading_low = particle.heading - 0.4;
            let heading_high = particle.heading + 0.4;

            return random_uniform_range(heading_low, heading_high);
        }

        let theta = ((coast.y - particle.pos.y) / (coast.x - particle.pos.x)).atan();

        let new_heading: f64 = match particle.context {
            ParticleContextType::Fishing => {
                random_uniform_range(particle.heading - 0.4, particle.heading + 0.4)
            }
            ParticleContextType::GoFishing => {
                let heading_diff =
                    (theta - particle.heading.to_radians()) % (2.0 * std::f64::consts::PI);

                let low: f64;
                let high: f64;

                if heading_diff < std::f64::consts::PI {
                    low = 0.0;
                    high = 22.0;
                } else {
                    low = -22.0;
                    high = 0.0;
                }

                random_uniform_range(low, high) + particle.heading
            }
            ParticleContextType::GoToPort => {
                let heading_diff =
                    (theta - particle.heading.to_radians()) % (2.0 * std::f64::consts::PI);

                let low: f64;
                let high: f64;

                if heading_diff < std::f64::consts::PI {
                    low = -22.0;
                    high = 0.0;
                } else {
                    low = 0.0;
                    high = 22.0;
                }

                random_uniform_range(low, high) + particle.heading
            }
        };

        new_heading % 360.0
    }

    fn calc_new_direction(&self, heading: f64) -> Point {
        let mut new_dir = Point {
            x: heading.cos(),
            y: heading.sin(),
        };

        new_dir * (1.0 / new_dir.norm())
    }

    fn calc_optimal_sequence(&self) -> Vec<Observation> {
        let mut smoothing_window: Vec<(u16, u16, u16)> = Vec::new();

        let mut optimal_sequence: Vec<Observation> = Vec::new();

        let states = self.markov_graph.get_all_nodes();

        for i in 0..self.observations.len() {
            let obs_memory: Vec<ParticleContextType> =
                self.particles.iter().map(|p| p.memory[i]).collect();

            let mut states_count: HashMap<ParticleContextType, u16> = states
                .clone()
                .into_iter()
                .fold(HashMap::new(), |mut acc, ctx_type| {
                    acc.insert(ctx_type, 0);
                    acc
                });

            for memory in obs_memory {
                states_count
                    .entry(memory)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            // Majority context
            let (majority_context, _) = states_count
                .iter()
                .max_by_key(|&(_, v)| v)
                .expect("Map is empty");

            let obs_with_context = Observation {
                pos: self.observations[i].pos,
                time: self.observations[i].time,
                heading: self.observations[i].heading,
                speed: self.observations[i].speed,
                context: *majority_context,
            };
            optimal_sequence.push(obs_with_context);

            // // For smoothing
            // let context_window_elem = (go_fishing_count, fishing_count, go_to_port_count);
            // if smoothing_window.len() < self.context_smoothing_window_size {
            //     smoothing_window.push(context_window_elem);
            // } else {
            //     smoothing_window.drain(0..1);
            //     smoothing_window.push(context_window_elem);
            // }

            // // Smoothing
            // if i >= self.context_smoothing_window_size {
            //     let current_context =
            //         optimal_sequence[i - (self.context_smoothing_window_size / 2)].context;
            //     let smoothed_context =
            //         self.smooth_context(current_context, smoothing_window.as_slice());
            //     optimal_sequence[i - (self.context_smoothing_window_size / 2)].context =
            //         smoothed_context;
            // }
        }

        optimal_sequence
    }

    fn smooth_context(
        &self,
        current_context: ParticleContextType,
        smoothing_window: &[(u16, u16, u16)],
    ) -> ParticleContextType {
        let mid_index = self.context_smoothing_window_size / 2;

        let (go_fishing_total, fishing_total, go_to_port_total) = smoothing_window
            .iter()
            .take(mid_index)
            .chain(smoothing_window.iter().skip(mid_index + 1))
            .fold((0, 0, 0), |(gf, f, gp), &(gfi, fi, gpi)| {
                (gf + gfi, f + fi, gp + gpi)
            });

        if smoothing_window[mid_index].0 >= 50
            && go_fishing_total > fishing_total
            && go_fishing_total > go_to_port_total
        {
            return ParticleContextType::GoFishing;
        } else if smoothing_window[mid_index].1 >= 50
            && fishing_total > go_fishing_total
            && fishing_total > go_to_port_total
        {
            return ParticleContextType::Fishing;
        } else if smoothing_window[mid_index].2 >= 50
            && go_to_port_total > go_fishing_total
            && go_to_port_total > fishing_total
        {
            return ParticleContextType::GoToPort;
        }

        current_context
    }

    fn calc_emission_prob(&self, observation: &Observation, particle: &Particle) -> f64 {
        let p: Point = observation.pos - particle.pos;
        let two_pi = 2.0f64 * std::f64::consts::PI;
        let gc = p.norm();
        let first_term = 1.0 / (two_pi.sqrt() * self.sigma);
        let second_term = -0.5 * (gc / self.sigma).powf(2.0);
        first_term * second_term.exp()
    }
}
