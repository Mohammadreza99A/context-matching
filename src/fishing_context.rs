use crate::random_generator::{
    random_normal, random_uniform, random_uniform_range, random_usize_uniform_range,
};
use crate::utils::linspace;
use crate::{
    geometry::Point,
    markov_graph::{read_graph_from_file, MarkovGraph},
    observation::Observation,
    particle::{Particle, ParticleContextType, ParticleHistory},
};
use rand::seq::SliceRandom;
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
    history: Vec<ParticleHistory>,
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

        let mut is_record_history = false;
        let mut file_path = "".to_string();
        if let Some(f) = history_file {
            is_record_history = true;
            file_path = f;
        }

        FishingContext {
            observations: observations.to_vec(),
            nb_of_particles: nb_of_particles,
            particles: Vec::new(),
            sigma: sigma,
            sailing_normal_speed_distr: sailing_normal_speed_distr,
            fishing_normal_speed_distr: fishing_normal_speed_distr,
            context_smoothing_window_size: context_window_size,
            markov_graph,
            is_record_history,
            history_file_path: file_path,
            history: Vec::new(),
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
            // Open history file to write into it
            let history_file =
                File::create(&self.history_file_path).expect("failed to create file");

            // Wrap the file in a buffered writer
            let mut writer = BufWriter::new(history_file);

            writer
                .write("position,direction,heading,speed,weight,context\n".as_bytes())
                .expect("failed to write to file");

            // Add initial particles to history
            self.add_to_history(&mut writer);

            for i in 1..self.observations.len() {
                self.particle_filter_steps(self.observations[i]);

                // Add particles to history
                self.add_to_history(&mut writer);
            }

            // Flush the buffer to ensure that any remaining data is written to the file
            writer.flush().expect("failed to flush buffer");
        } else {
            for i in 1..self.observations.len() {
                self.particle_filter_steps(self.observations[i]);
            }
        }

        self.calc_optimal_sequence()
    }

    fn add_to_history(&mut self, writer: &mut BufWriter<File>) {
        let particle_history = ParticleHistory {
            particles: self
                .particles
                .iter()
                .cloned()
                .map(|v| v.clone())
                .collect::<Vec<Particle>>(),
        };

        // Convert history to string
        let history_str = format!("{}", particle_history);

        // Write the particle history string to the buffered writer
        writer
            .write(history_str.as_bytes())
            .expect("failed to write to file");

        self.history.push(particle_history);
    }

    fn particle_filter_steps(&mut self, observation: Observation) {
        // Sampling
        self.particles = self.resample();

        // Update/Drift & Diffuse
        for i in 0..self.particles.len() {
            // Drawing a sample context-state based on transition probabilities
            let mut new_context: ParticleContextType = self.particles[i].context;
            // 10% chance that the context changes to another one
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
        // let new_heading = self.generate_new_random_heading(&particle, distance);
        // let new_heading = self.generate_new_random_heading2(&particle);
        let new_heading = self.calc_new_heading(&particle);

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

    fn generate_new_random_heading(&self, particle: &Particle, distance: f64) -> f64 {
        let coast = self.observations[0].pos;
        let man_angle = 22.91;

        if (particle.pos.x == coast.x && particle.pos.y == coast.y)
            || particle.context == ParticleContextType::Fishing
        {
            let mut heading_low = ((particle.heading - man_angle) + 360.0) % 360.0;
            let mut heading_high = (particle.heading + man_angle) % 360.0;
            if heading_high < heading_low {
                let heading_tmp = heading_high;
                heading_high = heading_low;
                heading_low = heading_tmp;
            }
            let new_heading = random_uniform_range(heading_low, heading_high);
            return new_heading;
        }

        // Find the slope of the line from coast to particle's current position
        // Let's call this line slope
        let slope = (particle.pos.y - coast.y) / (particle.pos.x - coast.x);

        // Find the negative reciprocal of the slope to get the slope of line
        // which is perpendicular at particle's point. Let's call this perp_slope
        let perp_slope = -1.0 / slope;

        // Find the y-intercept of perp_slope using the particle's pos and perp_slope
        let b_perp = particle.pos.y - perp_slope * particle.pos.x;

        // Now let's define the cone which has all the possible headings from
        // -0.4 radians to 0.4 radians of the current heading
        let mut heading_low = particle.heading - man_angle;
        let mut heading_high = particle.heading + man_angle;
        if particle.context == ParticleContextType::GoFishing {
            if heading_low > 90.0 && heading_low < 270.0 {
                heading_low = particle.heading - 180.0;
            }
            if heading_low > 90.0 && heading_low < 270.0 {
                heading_high = particle.heading - 180.0;
            }
        }
        if particle.context == ParticleContextType::GoToPort {
            if !(heading_low > 90.0 && heading_low < 270.0) {
                heading_low = particle.heading - 180.0;
            }
            if !(heading_high > 90.0 && heading_high < 270.0) {
                heading_high = particle.heading - 180.0;
            }
        }
        heading_low = (heading_low % 360.0 + 360.0) % 360.0;
        heading_high = (heading_high % 360.0 + 360.0) % 360.0;
        let mut cone: Vec<f64> = linspace::<f64>(heading_low, heading_high, 50);

        // Define the range of x and y values for the cone based on the angle
        // and distance from particle's position
        let mut x_cone = Vec::new();
        let mut y_cone = Vec::new();
        for angle in cone.clone() {
            x_cone.push(particle.pos.x + distance * angle.cos());
            y_cone.push(particle.pos.y + distance * angle.sin());
        }

        // Check if the cone is on the far side of the perp_line and if not
        // only take the part which is
        let mut far_side_indices: Vec<usize> = Vec::new();
        for i in 0..x_cone.len() {
            let x = x_cone[i];
            let y = y_cone[i];
            if (perp_slope * x + b_perp) < y {
                far_side_indices.push(i);
            }
        }

        // Now select a random new heading from the cone
        let new_heading: f64;
        if far_side_indices.len() == 0 {
            let index = random_usize_uniform_range(0, cone.len());
            new_heading = cone[index];
            // random_heading = random_usize_uniform_range(0, x_cone.len() - 1);
        } else {
            new_heading = cone[*far_side_indices.choose(&mut rand::thread_rng()).unwrap()];
        }

        new_heading
    }

    fn calc_new_heading(&self, particle: &Particle) -> f64 {
        let coast = self.observations[0].pos;
        let man_angle = 22.91;

        if particle.context == ParticleContextType::Fishing {
            let mut heading_low = ((particle.heading - man_angle) + 360.0) % 360.0;
            let mut heading_high = (particle.heading + man_angle) % 360.0;
            if heading_high < heading_low {
                let heading_tmp = heading_high;
                heading_high = heading_low;
                heading_low = heading_tmp;
            }
            let new_heading = random_uniform_range(heading_low, heading_high);
            return new_heading;
        }

        let mut heading_low = particle.heading - man_angle;
        let mut heading_high = particle.heading + man_angle;
        if particle.context == ParticleContextType::GoFishing {
            if heading_low > 90.0 && heading_low < 270.0 {
                heading_low = (180.0 - heading_low) % 360.0;
            }
            if heading_low > 90.0 && heading_low < 270.0 {
                heading_high = (180.0 - heading_high) % 360.0;
            }
        }
        if particle.context == ParticleContextType::GoToPort {
            if !(heading_low > 90.0 && heading_low < 270.0) {
                heading_low = (180.0 - heading_low) % 360.0;
                // heading_low = particle.heading - 180.0;
                // heading_low = (heading_low % 360.0 + 360.0) % 360.0;
            }
            if !(heading_high > 90.0 && heading_high < 270.0) {
                heading_high = (180.0 - heading_high) % 360.0;
                // heading_high = particle.heading - 180.0;
                // heading_high = (heading_high % 360.0 + 360.0) % 360.0;
            }
        }

        if heading_high < heading_low {
            let heading_tmp = heading_high;
            heading_high = heading_low;
            heading_low = heading_tmp;
        }
        let new_heading = random_uniform_range(heading_low, heading_high);

        new_heading
    }

    fn generate_new_random_heading2(&self, particle: &Particle) -> f64 {
        let coast = self.observations[0].pos;

        if particle.pos.x == coast.x && particle.pos.y == coast.y {
            let heading_low = particle.heading - 0.4;
            let heading_high = particle.heading + 0.4;

            return random_uniform_range(heading_low, heading_high);
        }

        // let theta = ((particle.pos.x - coast.x) / (particle.pos.y - coast.y)).atan();
        let theta = ((coast.y - particle.pos.y) / (coast.x - particle.pos.x)).atan();

        // let new_heading: f64 = match particle.context {
        //     ParticleContextType::Fishing => {
        //         random_uniform_range(particle.heading - 0.4, particle.heading + 0.4)
        //     }
        //     ParticleContextType::GoFishing => random_uniform_range(theta - 0.4, theta + 0.4),
        //     ParticleContextType::GoToPort => {
        //         let pi = std::f64::consts::PI;
        //         random_uniform_range(theta - pi - 0.4, theta - pi + 0.4)
        //     }
        // };

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
        let norm = 1.0 / new_dir.norm();
        new_dir = new_dir * norm;

        new_dir
    }

    fn calc_optimal_sequence(&self) -> Vec<Observation> {
        let mut smoothing_window: Vec<(u16, u16, u16)> = Vec::new();

        let mut optimal_sequence: Vec<Observation> = Vec::new();

        for i in 0..self.observations.len() {
            let mut obs_memory: Vec<ParticleContextType> = Vec::new();
            for j in 0..self.particles.len() {
                obs_memory.push(self.particles[j].memory[i]);
            }

            let mut go_fishing_count: u16 = 0;
            let mut fishing_count: u16 = 0;
            let mut go_to_port_count: u16 = 0;

            for memory in obs_memory {
                match memory {
                    ParticleContextType::GoFishing => go_fishing_count += 1,
                    ParticleContextType::Fishing => fishing_count += 1,
                    ParticleContextType::GoToPort => go_to_port_count += 1,
                }
            }

            // Majority context
            let majority_context =
                if go_fishing_count > fishing_count && go_fishing_count > go_to_port_count {
                    ParticleContextType::GoFishing
                } else if fishing_count > go_fishing_count && fishing_count > go_to_port_count {
                    ParticleContextType::Fishing
                } else {
                    ParticleContextType::GoToPort
                };

            let obs_with_context = Observation {
                pos: self.observations[i].pos,
                time: self.observations[i].time,
                heading: self.observations[i].heading,
                speed: self.observations[i].speed,
                context: majority_context,
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

        let mut go_fishing_total = 0;
        let mut fishing_total = 0;
        let mut go_to_port_total = 0;

        for i in 0..mid_index {
            go_fishing_total += smoothing_window[i].0;
            fishing_total += smoothing_window[i].1;
            go_to_port_total += smoothing_window[i].2;
        }

        for i in mid_index + 1..self.context_smoothing_window_size {
            go_fishing_total += smoothing_window[i].0;
            fishing_total += smoothing_window[i].1;
            go_to_port_total += smoothing_window[i].2;
        }

        if go_fishing_total > fishing_total && go_fishing_total > go_to_port_total {
            if smoothing_window[mid_index].0 >= 50 {
                return ParticleContextType::GoFishing;
            }
        }
        if fishing_total > go_fishing_total && fishing_total > go_to_port_total {
            if smoothing_window[mid_index].1 >= 50 {
                return ParticleContextType::Fishing;
            }
        }
        if go_to_port_total > go_fishing_total && go_to_port_total > fishing_total {
            if smoothing_window[mid_index].2 >= 50 {
                return ParticleContextType::GoToPort;
            }
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
