#![allow(dead_code, unused_imports, unused_mut, unused_variables)]
mod fishing_context;
mod geometry;
mod observation;
mod particle;
mod random_generator;
mod utils;

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
    let mut ctx = FishingContext::new(
        observations.as_slice(),
        100,
        5.0,
        (3.31, 1.19),
        (1.36, 0.89),
        51,
    );
    let states: Vec<Observation> = ctx.particle_filter();
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

    wtr.write_record(&[
        "x",
        "y",
        "time",
        "heading",
        "speed",
        "context",
        "distanceToShore",
    ])?;

    for state in states {
        wtr.serialize((
            state.pos.x,
            state.pos.y,
            state.time,
            state.heading,
            state.speed,
            state.context,
            state.distance_to_shore,
        ))?;
    }

    wtr.flush()?;

    Ok(())
}
