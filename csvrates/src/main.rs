use std::env;
use std::fs;
use std::io::Write;
use std::io::{BufRead, BufReader, Read};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct NegotiatedPrices {
    negotiated_rate: f64,
}
#[derive(Debug, Deserialize)]
struct NegotiatedRates {
    negotiated_prices: Vec<NegotiatedPrices>,
}
#[derive(Debug, Deserialize)]
struct Record {
    billing_code: String,
    name: String,
    negotiated_rates: Vec<NegotiatedRates>,
}

#[derive(serde::Serialize)]
struct CsvRow<'a> {
    name: &'a str,
    billing_code: &'a str,
    avg_rate: f64,
}

fn main() -> std::io::Result<()> {
    let input = env::args().nth(1);
    let output = env::args().nth(2);
    match (input, output) {
        (Some(filename), Some(outfile)) => process_lines(
            fs::File::open(filename).expect("can open input file"),
            fs::File::create(outfile).expect("can open output file"),
        ),
        (Some(filename), None) => process_lines(
            fs::File::open(filename).expect("can open input file"),
            std::io::stdout(),
        ),
        (_, _) => process_lines(std::io::stdin(), std::io::stdout()),
    }
}

/// go through and read and write
fn process_lines<R: Read, W: Write>(reader: R, writer: W) -> std::io::Result<()> {
    let mut csv_writer = csv::Writer::from_writer(writer);
    let buffer = BufReader::new(reader);
    for line in buffer.lines() {
        // this panics on a bad record, but we could use if let if we wanted to be less strict
        let rec = serde_json::from_str::<Record>(&line.expect("line reading works"))
            .expect("valid record json");
        // note we skip
        if let Some(average_negotiated_rate) = calc_average_negotiated_rate(&rec.negotiated_rates) {
            if average_negotiated_rate > 30.0 {
                continue;
            }
            let csv_row = create_csv_row(&rec, average_negotiated_rate);
            csv_writer.serialize(csv_row)?
        }
    }
    Ok(())
}

/// this outputs the csv line by line
/// unfortunately this is coupled to the csv writer type (File vs io)
fn create_csv_row(rec: &Record, average_negotiated_rate: f64) -> CsvRow {
    CsvRow {
        name: &rec.name,
        billing_code: &rec.billing_code,
        // note this is an f64, so not formatted to 2 decimal places
        avg_rate: average_negotiated_rate,
    }
}

// protects against missing rates, but not against negatives or other possible JSON oddities
fn calc_average_negotiated_rate(negotiated_rates: &[NegotiatedRates]) -> Option<f64> {
    let mut sum = 0.0;
    let mut count_of_rates = 0.0;
    for nr in negotiated_rates {
        for np in nr.negotiated_prices.iter() {
            sum += np.negotiated_rate;
            count_of_rates += 1.0;
        }
    }
    if count_of_rates > 0.0 {
        Some(sum / count_of_rates)
    } else {
        None
    }
}
