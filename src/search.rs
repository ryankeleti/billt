use std::{collections::HashMap, fmt, str::FromStr};

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

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum Year {
    All,
    Current,
    Recent,
    Prior,
    Exact(u32),
}

impl fmt::Display for Year {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Year::All => "all".fmt(f),
            Year::Current => "current".fmt(f),
            Year::Recent => "recent".fmt(f),
            Year::Prior => "prior".fmt(f),
            Year::Exact(year) => year.fmt(f),
        }
    }
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

#[derive(PartialEq, Eq, Debug)]
pub struct ParseYearError(&'static str);

impl fmt::Display for ParseYearError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl std::error::Error for ParseYearError {}

impl FromStr for Year {
    type Err = ParseYearError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "all" => Ok(Year::All),
            "current" => Ok(Year::Current),
            "recent" => Ok(Year::Recent),
            "prior" => Ok(Year::Prior),
            _ => match s.parse::<u32>() {
                Ok(year) if year > 1900 => Ok(Year::Exact(year)),
                Ok(_) => Err(ParseYearError("exact year should be > 1900")),
                Err(_) => Err(ParseYearError("could not parse exact year")),
            },
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

        let params = &[
            ("op", "getSearch"),
            ("state", state.unwrap_or("ALL")),
            ("year", &year.to_string()),
            ("query", query),
        ];

        println!("search_params = {:?}", params);

        build_prefix(client)
            .query(params)
            .send()?
            .json::<SearchResult>()?
    };

    for (k, item) in res.searchresult {
        if k == "summary" {
            match serde_json::from_value::<Summary>(item) {
                Err(_) => {
                    // No results.
                    return Ok(Vec::new());
                }
                Ok(summary) => {
                    println!("{:#?}", summary);
                    return get_search_until(client, state, year, query, summary.page_total);
                }
            }
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
