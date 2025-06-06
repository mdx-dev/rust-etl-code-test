use clap::Parser;
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input file path (if not provided, reads from stdin)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Output file path (if not provided, writes to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct InputRecord {
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

#[derive(Debug, Serialize)]
struct OutputRecord {
    name: String,
    billing_code: String,
    avg_rate: f64,
}

fn calculate_average_rate(record: &InputRecord) -> Option<f64> {
    let mut total_rate = 0.0;
    let mut rate_count = 0;
    
    for negotiated_rate in &record.negotiated_rates {
        for price in &negotiated_rate.negotiated_prices {
            total_rate += price.negotiated_rate;
            rate_count += 1;
        }
    }
    
    if rate_count > 0 {
        Some(total_rate / rate_count as f64)
    } else {
        None
    }
}

fn process_records(reader: Box<dyn BufRead>, writer: &mut Box<dyn Write>) -> io::Result<()> {
    let mut csv_writer = Writer::from_writer(writer);
    let mut stats = ProcessingStats::default();

    for line in reader.lines() {
        stats.total_lines += 1;
        let line = line?;
        
        match serde_json::from_str::<InputRecord>(&line) {
            Ok(record) => {
                if let Some(avg_rate) = calculate_average_rate(&record) {
                    if avg_rate <= 30.0 {
                        csv_writer.serialize(OutputRecord {
                            name: record.name,
                            billing_code: record.billing_code,
                            avg_rate,
                        })?;
                        stats.successful_records += 1;
                    }
                }
            }
            Err(e) => {
                stats.error_count += 1;
                println!("Error parsing line {}: {}", stats.total_lines, e);
            }
        }
    }

    print_processing_summary(&stats);

    csv_writer.flush()?;
    Ok(())
}

#[derive(Default)]
struct ProcessingStats {
    total_lines: usize,
    error_count: usize,
    successful_records: usize,
}

fn print_processing_summary(stats: &ProcessingStats) {
    println!("\nProcessing Summary:");
    println!("Total lines processed: {}", stats.total_lines);
    println!("Parsing errors: {}", stats.error_count);
    println!("Records with avg_rate <= 30: {}", stats.successful_records);
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    // Setup input
    let input: Box<dyn BufRead> = match args.input {
        Some(path) => Box::new(BufReader::new(File::open(path)?)),
        None => Box::new(BufReader::new(io::stdin())),
    };

    // Setup output
    let mut output: Box<dyn Write> = match args.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    process_records(input, &mut output)
} 