mod context;
mod fishing_context;
mod geometry;
mod observation;
mod random_generator;

use context::ContextState;
use fishing_context::FishingContext;
use observation::Observation;
use std::env;
use std::error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        return Err("Bad number of arguments: <input_csv_file_path> <output_result_path>".into());
    }

    println!("Reading and parsing input CSV file...");
    let observations = Observation::from_csv(&args[1])?;

    println!("Particle filtering...");
    let start = Instant::now();
    let mut ctx: FishingContext = FishingContext {
        observations: observations.clone(),
        nb_of_particles: 100,
        samples: Vec::new(),
        sigma: 5.0,
        alpha: 1.0,
        sailing_normal_speed_distr: (3.31, 1.19),
        fishing_normal_speed_distr: (1.36, 0.89),
    };
    let states: Vec<ContextState> = ctx.particle_filter();
    let duration = start.elapsed();
    println!("Particle filtering took {:?}", duration);

    println!("Analyzing results...");
    let mut correct_context: u32 = 0;
    let mut false_context: u32 = 0;
    for i in 0..states.len() {
        if states[i].context == observations[i].context {
            correct_context += 1;
        } else {
            false_context += 1;
        }
    }
    println!(
        "Context --> correct: {}, false: {}. Success rate: {}",
        correct_context,
        false_context,
        correct_context as f32 / (correct_context + false_context) as f32
    );

    println!("Writing result to output file...");
    let mut wtr = csv::Writer::from_path(&args[2])?;

    wtr.write_record(&["x", "y", "heading", "speed", "context"])?;

    for state in states {
        wtr.serialize((
            state.pos.x,
            state.pos.y,
            state.heading,
            state.speed,
            state.context,
        ))?;
    }

    wtr.flush()?;

    Ok(())
}
