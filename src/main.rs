use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::{self, BufReader, Read, Write};

#[derive(Deserialize)]
struct InputRecord {
    name: String,
    billing_code: String,
    negotiated_rate: f64,
}

#[derive(Serialize)]
struct OutputRecord {
    name: String,
    billing_code: String,
    avg_rate: f64,
}

fn process<R: Read, W: Write>(reader: R, writer: W) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::Reader::from_reader(reader);
    let mut wtr = csv::Writer::from_writer(writer);

    let mut grouped: HashMap<(String, String), Vec<f64>> = HashMap::new();

    for result in rdr.deserialize() {
        let record: InputRecord = result?;
        grouped
            .entry((record.name, record.billing_code))
            .or_default()
            .push(record.negotiated_rate);
    }

    for ((name, billing_code), rates) in grouped {
        let avg: f64 = rates.iter().sum::<f64>() / rates.len() as f64;
        if avg <= 30.0 {
            let out = OutputRecord {
                name,
                billing_code,
                avg_rate: avg,
            };
            wtr.serialize(out)?;
        }
    }

    wtr.flush()?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        let file = File::open(&args[1])?;
        let reader = BufReader::new(file);
        process(reader, io::stdout())?;
    } else {
        let stdin = io::stdin();
        process(stdin.lock(), io::stdout())?;
    }

    Ok(())
}
