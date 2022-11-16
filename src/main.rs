mod context;
mod fishing_context;
mod geometry;
mod observation;

use context::ContextState;
use fishing_context::FishingContext;
use observation::Observation;
use std::env;
use std::error;
use std::time::Instant;

fn main() -> Result<(), Box<dyn error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Bad number of arguments: <input_csv_file_path> <output_result_path>");
        std::process::exit(1);
    }

    println!("Reading and parsing input CSV file...");
    let observations = Observation::from_csv(&args[1])?;

    println!("Particle filtering...");
    let start = Instant::now();
    let mut ctx: FishingContext = FishingContext {
        observations: observations.clone(),
        nb_of_particles: 100,
        samples: Vec::new(),
        sigma: 100.0f64,
        alpha: 100000.0f64,
    };
    let states: Vec<ContextState> = ctx.particle_filter();
    let duration = start.elapsed();
    println!("Particle filtering took {:?} milliseconds", duration);

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
        "Context --> correct: {}, false: {}",
        correct_context, false_context
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
