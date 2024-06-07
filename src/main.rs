mod search;

use std::{env, path::Path};

use reqwest::blocking::Client;

use search::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        return Ok(());
    }

    let client = Client::new();
    for arg in &args[1..] {
        let path = Path::new(arg).with_extension("csv");
        let mut wtr = csv::Writer::from_path(&path)?;
        println!("Search: {}", arg);
        let bills = get_search(&client, None, Year::All, arg)?;
        for bill in bills {
            wtr.serialize(bill)?;
        }
        println!("Saved to {}", path.display());
    }

    Ok(())
}
