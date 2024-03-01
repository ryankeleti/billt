//mod app;
mod db;
mod search;

use std::{env, path::Path};

use reqwest::blocking::Client;

//use ratatui::prelude::{CrosstermBackend, Terminal};

//use app::App;
//use db::Db;

use search::*;

//const DB_PATH: &'static str = "db.json";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    //println!("{:?}", serde_json::to_string(&chrono::Utc::now()));
    //let db_path = Path::new(DB_PATH);
    //let db = Db::read(&db_path)?;
    //let mut app = App::new(db, &db_path);

    let args: Vec<_> = env::args().collect();
    if args.len() == 1 {
        return Ok(());
    }
    for arg in &args[1..] {
        let path = Path::new(arg).with_extension("csv");
        let mut wtr = csv::Writer::from_path(&path)?;
        println!("Search: {}", arg);
        let bills = get_search(&client, None, Year::All, arg)?;
        //for (i, bill) in bills.into_iter().enumerate() {
        //    db.bills.insert(i as u32, db::Entry { bill });
        //}

        for bill in bills {
            wtr.serialize(bill)?;
        }
        println!("Saved to {}", path.display());
    }

    /*
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stderr(), crossterm::terminal::EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    app.run(&mut terminal)?;

    //db.write(&db_path)?;

    crossterm::execute!(std::io::stderr(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    */

    Ok(())
}
