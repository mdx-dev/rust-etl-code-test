# Rust ETL Code Test

Fork this repo for this test. When you are done submit a PR against this repo.

Given the sample data provided, convert to csv in the format specified:

`name, billing_code, avg_rate` where `avg_rate` is the average of all `negotiated_rate` values for each record. Exclude records with an `avg_rate` greater than 30.

- Feel free to use any tools or libraries of your choice.
- The program should be as fast as possible.
- The program should accept inputs of unbounded size.
- The program should accept input from a file or STDIN.
- Output should be written to a file or STDOUT.

## Project Documentation

### Requirements

- Rust 1.70 or higher
- Cargo (Rust's package manager)

### Usage

The program can be run in several ways:

1. Process a file and output to a file:

```bash
cargo run --release -- -i sample.jsonl -o output.csv
```

2. Process from STDIN and output to STDOUT:

```bash
cargo run --release
```

3. Process a file and output to STDOUT:

```bash
cargo run --release -- -i sample.jsonl
```

4. Process from STDIN and output to a file:

```bash
cargo run --release -- -o output.csv
```

#### Performance Optimizations

- **Streaming Processing**: Processes data line by line
- **Buffered I/O**: Uses buffered readers and writers
- **Memory Management**: Minimal memory footprint through streaming

#### Error Handling

- Gracefully handles JSON parsing errors
- Provides detailed error reporting
- Continues processing despite individual record errors

### Dependencies

- `serde`: For JSON deserialization
- `csv`: For CSV output
- `clap`: For command-line argument parsing
- `tokio`: For async runtime support
