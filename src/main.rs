#![allow(unused)]
mod search;

use std::{env, path::Path};

use clap::Parser;
use reqwest::blocking::Client;

use search::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();

    let query = args.query;
    println!("Search: {}", query);
    let bills = get_search(&client, args.state.as_deref(), args.year, &query)?;

    // Save result to CSV.
    let path = Path::new(&query).with_extension("csv");
    let mut wtr = csv::Writer::from_path(&path)?;
    for bill in bills {
        wtr.serialize(bill)?;
    }
    println!("Saved to {}", path.display());

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    query: String,

    #[arg(short, long, default_value_t = Year::All)]
    year: Year,

    #[arg(short, long)]
    state: Option<String>,
}
