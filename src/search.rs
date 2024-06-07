use std::collections::HashMap;

use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};

const API_KEY: &str = "20232b096f015a6417529824ddf70b14";
const API_URL: &str = "https://api.legiscan.com";

fn build_prefix(client: &Client) -> RequestBuilder {
    client.get(API_URL).query(&[("key", API_KEY)])
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bill {
    pub relevance: u32,
    pub state: String,
    pub bill_number: String,
    pub bill_id: u32,
    pub change_hash: String,
    pub url: String,
    pub text_url: String,
    pub research_url: String,
    pub last_action_date: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Summary {
    page: String,
    range: String,
    relevancy: String,
    count: u32,
    page_current: u32,
    page_total: u32,
}

#[derive(Deserialize)]
struct SearchResult {
    status: String,
    searchresult: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Copy)]
pub enum Year {
    All,
    Current,
    Recent,
    Prior,
    Exact(u32),
}

impl From<Year> for u32 {
    fn from(year: Year) -> u32 {
        match year {
            Year::All => 1,
            Year::Current => 2,
            Year::Recent => 3,
            Year::Prior => 4,
            Year::Exact(n) => n,
        }
    }
}

pub fn get_search_page(
    client: &Client,
    state: Option<&str>,
    year: Year,
    query: &str,
    page: u32,
) -> reqwest::Result<Vec<Bill>> {
    println!("get page {page}");

    let year: u32 = year.into();
    let res = build_prefix(client)
        .query(&[
            ("op", "getSearch"),
            ("state", state.unwrap_or("ALL")),
            ("year", &year.to_string()),
            ("query", query),
            ("page", &page.to_string()),
        ])
        .send()?
        .json::<SearchResult>()?;

    let mut bills = Vec::new();

    for (k, bill) in res.searchresult {
        if k != "summary" {
            match serde_json::from_value(bill) {
                Ok(bill) => bills.push(bill),
                Err(e) => println!("{e}"),
            }
        }
    }

    Ok(bills)
}

pub fn get_search(
    client: &Client,
    state: Option<&str>,
    year: Year,
    query: &str,
) -> reqwest::Result<Vec<Bill>> {
    let res = {
        let year: u32 = year.into();

        build_prefix(client)
            .query(&[
                ("op", "getSearch"),
                ("state", state.unwrap_or("ALL")),
                ("year", &year.to_string()),
                ("query", query),
            ])
            .send()?
            .json::<SearchResult>()?
    };

    for (k, item) in res.searchresult {
        if k == "summary" {
            let summary: Summary = serde_json::from_value(item).unwrap();
            println!("{:#?}", summary);
            return get_search_until(client, state, year, query, summary.page_total);
        }
    }

    unreachable!()
}

pub fn get_search_until(
    client: &Client,
    state: Option<&str>,
    year: Year,
    query: &str,
    page_total: u32,
) -> reqwest::Result<Vec<Bill>> {
    let mut bills: Vec<Bill> = Vec::new();

    for page in 1..=(page_total as usize) {
        let page_bills = get_search_page(client, state, year, query, page as u32)?;
        bills.extend(page_bills);
    }

    bills.sort_by_key(|bill| bill.relevance);
    bills.reverse();

    Ok(bills)
}
