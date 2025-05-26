use std::fs::File; // Importing File for file operations
use std::io::{self, BufWriter, Write}; // Importing necessary traits for I/O operations

use clap::Parser; // Importing clap-trap for command-line argument parsing
use serde::Deserialize; // Importing serde for deserializing JSON data into Rust structs

#[derive(Parser, Debug)] // get ready to parse command-line arguments using clap-trap!
#[command(name = "Rust ETL")]
#[command(about = "Convert JSON to CSV with avg_rate filtering", long_about = None)]
struct Args {
    /// Optional input file (reads from STDIN if not available)
    #[arg(short, long)]
    input: Option<String>,

    /// Optional output file (writes to STDOUT if not available)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Debug, Deserialize)] // serde will help us deserialize JSON data into these structs
struct Record {
    name: String,
    billing_code: String,
    negotiated_rates: Vec<NegotiatedRate>,
}

#[derive(Debug, Deserialize)]
struct NegotiatedRate {
    negotiated_prices: Vec<NegotiatedPrice>,
}

#[derive(Debug, Deserialize)]
struct NegotiatedPrice {
    negotiated_rate: f64,
}

// Time for the main method, the program's entry point!
fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Initialize the logger for debugging
    let args = Args::parse();

    // Instantiate our reader and writer based on command-line arguments
    let reader: Box<dyn io::Read> = match args.input {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };

    let writer: Box<dyn Write> = match args.output {
        Some(path) => Box::new(BufWriter::new(File::create(path)?)),
        None => Box::new(io::stdout()),
    };

    let mut scribe = csv::Writer::from_writer(writer);
    scribe.write_record(&["name", "billing_code", "avg_rate"])?;


    // Using this to stream the JSON file one object at a time isntead of loading everything into memory at once. Then, iterate through fields and print values
    let stream = serde_json::Deserializer::from_reader(reader).into_iter::<Record>();

    log::info!("Starting to process records...");
    // Iterate through each record in the stream
    for (i, result) in stream.enumerate() {
        match result {
            Ok(record) => {
                let rates: Vec<f64> = record.negotiated_rates
                    .iter()
                    .flat_map(|r| r.negotiated_prices.iter().map(|p| p.negotiated_rate))
                    .collect();

                if rates.is_empty() {
                    log::warn!("Record {} skipped: no rates found", i);
                    continue;
                }

                let avg_rate: f64 = rates.iter().sum::<f64>() / rates.len() as f64;

                if avg_rate <= 30.0 {
                    scribe.write_record(&[
                        record.name.as_str(),
                        record.billing_code.as_str(),
                        format!("{:.2}", avg_rate).as_str(),
                    ])?;
                } else {
                    log::info!("Record {} excluded (avg_rate = {:.2})", i, avg_rate);
                }
            }
            Err(err) => {
                log::error!("Failed to parse record {}: {}", i, err);
            }
        }
    }

    scribe.flush()?; // Ensure all data is written to the output
    log::info!("Processing complete. Output written successfully!");
    Ok(()) // Return Ok if everything went well!
}