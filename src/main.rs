#![allow(unused)]
mod search;

use std::path::Path;

use chrono::prelude::*;
use clap::{builder::NonEmptyStringValueParser, Parser};
use reqwest::blocking::Client;

use search::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = Client::new();

    let query = args.query;
    println!("Search: {}", query);
    println!("bill_status: {}", args.bill_status);

    let mut bills = get_search(&client, args.state.as_deref(), args.year, &query)?;

    if let Some(last_action_date) = args.last_action_date {
        println!("Filtering out results before {last_action_date}...");
        bills.retain(|bill| {
            if let Some(date) = &bill.last_action_date {
                *date >= last_action_date
            } else {
                false
            }
        });
    }

    if bills.is_empty() {
        println!("No results.");
    }

    // Save result to CSV.
    let local: DateTime<Local> = Local::now();
    let parts = vec![
        query.clone(),
        args.state.unwrap_or("US".to_string()),
        args.year.to_string(),
        "generated".to_string(),
        local.format("%Y-%m-%d_%H-%M-%S").to_string(),
    ];
    let path = parts.join("_");

    let path = Path::new(&path).with_extension("csv");
    let mut wtr = csv::Writer::from_path(&path)?;
    for bill in bills.into_iter() {
        if args.bill_status {
            match get_bill(&client, bill.bill_id) {
                Ok(extra) => {
                    // println!("{:#?}", extra);
                    wtr.serialize(BillCsvRowWithExtraStuff::new(bill, extra, &query))?;
                }
                Err(e) => {
                    eprintln!("{e}, continuing");
                }
            }
        } else {
            wtr.serialize(BillCsvRow::new(bill, &query))?;
        }
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

    #[arg(long)]
    last_action_date: Option<String>,

    #[arg(long, default_value_t = false)]
    bill_status: bool,
}
