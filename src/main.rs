use clap::Parser;
use csv::Writer;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,

    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct SampleRecord {
    name: String,
    billing_code: String,

    #[serde(
        default,
        with = "price_serde",
        rename(deserialize = "negotiated_rates")
    )]
    avg_rate: f32,
}

mod price_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[derive(Deserialize, Debug, Default)]
    struct NegotiatedPricesRecord {
        negotiated_prices: Vec<NegotiatedRateRecord>,
    }

    #[derive(Deserialize, Debug, Default)]
    struct NegotiatedRateRecord {
        negotiated_rate: f32,
    }

    pub fn serialize<S>(data: &f32, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_f32(*data)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        let data: Vec<NegotiatedPricesRecord> = Deserialize::deserialize(deserializer)?;
        let prices: Vec<f32> = data
            .iter()
            .flat_map(|i| &i.negotiated_prices)
            .map(|i| i.negotiated_rate)
            .collect();
        let sum = prices.iter().copied().sum::<f32>();
        Ok(sum / prices.len() as f32)
    }
}

fn convert_to_csv(input: &mut dyn BufRead, output: &mut dyn Write) -> Result<(), String> {
    let mut line = String::new();
    let mut wtr = Writer::from_writer(output);
    while input
        .read_line(&mut line)
        .map_err(|e| format!("Error reading from input: {e}"))?
        > 0
    {
        let tmp: SampleRecord =
            serde_json::from_str(&line).map_err(|e| format!("Error parsing: {e}"))?;
        if tmp.avg_rate <= 30. {
            wtr.serialize(&tmp)
                .map_err(|e| format!("Error serializing: {e}"))?;
            wtr.flush()
                .map_err(|e| format!("Error flushing writes: {e}"))?;
        }
        line.clear();
    }
    Ok(())
}

// TODO: Use proper error type.
// TODO: BufRead buffer size as const.
fn main() -> Result<(), String> {
    let args = Args::parse();
    let input: &mut dyn BufRead = if let Some(file) = args.input {
        let file = File::open(file).map_err(|e| format!("File open error: {e}"))?;
        &mut BufReader::new(file)
    } else {
        &mut io::stdin().lock()
    };
    let output: &mut dyn Write = if let Some(file) = args.output {
        let file = File::create(file).map_err(|e| format!("File create error: {e}"))?;
        &mut BufWriter::new(file)
    } else {
        &mut io::stdout().lock()
    };
    convert_to_csv(input, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANY_JSON: &str = r#"{"negotiation_arrangement":"capitation","name":"1REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT","billing_code":"98975","negotiated_rates":[{"negotiated_prices":[{"negotiated_rate":18.26,"service_code":["02","10","11"]}]}]}
{"negotiation_arrangement":"capitation","name":"2REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT","billing_code":"98975","negotiated_rates":[{"negotiated_prices":[{"negotiated_rate":18.26,"service_code":["02","10","11"]}]}]}
{"negotiation_arrangement":"capitation","name":"3REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT","billing_code":"98975","negotiated_rates":[{"negotiated_prices":[{"negotiated_rate":18.26,"service_code":["02","10","11"]}]}]}"#;
    const SAMPLE_MANY_CSV: &str = r#"name,billing_code,avg_rate
"1REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT",98975,18.26
"2REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT",98975,18.26
"3REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT",98975,18.26
"#;
    #[test]
    fn testmany() {
        let mut writer = Vec::<_>::new();
        assert!(convert_to_csv(&mut String::from(SAMPLE_MANY_JSON).as_bytes(), &mut writer).is_ok());
        assert_eq!(
            str::from_utf8(&writer).expect("valid write"),
            String::from(SAMPLE_MANY_CSV)
        );
    }

    const SAMPLE_MULTIPLE_AVG_JSON: &str = r#"{"negotiation_arrangement":"capitation","name":"1REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT","billing_code":"98975","negotiated_rates":[{"negotiated_prices":[{"negotiated_rate":12.44,"service_code":["02","10","11"]}, {"negotiated_rate":24.94,"service_code":["02","10","11"]}]}]}
{"negotiation_arrangement":"capitation","name":"1REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT","billing_code":"98975","negotiated_rates":[{"negotiated_prices":[{"negotiated_rate":92.44,"service_code":["02","10","11"]}, {"negotiated_rate":24.94,"service_code":["02","10","11"]}]}]}
"#;
    const SAMPLE_MULTIPLE_AVG_CSV: &str = r#"name,billing_code,avg_rate
"1REMOTE THERAPEUTIC MONITORING (EG, RESPIRATORY SYSTEM STATUS, MUSCULOSKELETAL SYSTEM STATUS, TX ADHERENCE, TX RESPONSE); INITIAL SET-UP & PATIENT EDUCATION ON USE OF EQUIPMENT",98975,18.69
"#;

    #[test]
    fn test_multiple_avg_exclude_large() {
        let mut writer = Vec::<_>::new();
        assert!(convert_to_csv(
            &mut String::from(SAMPLE_MULTIPLE_AVG_JSON).as_bytes(),
            &mut writer,
        ).is_ok());
        assert_eq!(
            str::from_utf8(&writer).expect("valid write"),
            String::from(SAMPLE_MULTIPLE_AVG_CSV)
        );
    }
}
