use std::{collections::HashMap, fmt, str::FromStr};

use reqwest::blocking::{Client, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

// FIXME don't store these in the source, obviously :p
const API_KEY: &str = "20232b096f015a6417529824ddf70b14";
const API_URL: &str = "https://api.legiscan.com";

fn build_prefix(client: &Client) -> RequestBuilder {
    client.get(API_URL).query(&[("key", API_KEY)])
}

// Annoying workaround because #[serde(flatten)] doesn't work with csv crate.
#[derive(Serialize)]
pub struct BillCsvRow<'a>(Bill, Query<'a>);

#[derive(Serialize)]
pub struct BillCsvRowWithExtraStuff<'a>(Bill, ExtraBillStuff, Query<'a>);

#[derive(Serialize)]
struct Query<'a> {
    query: &'a str,
}

impl<'a> BillCsvRow<'a> {
    pub fn new(bill: Bill, query: &'a str) -> Self {
        Self(bill, Query { query })
    }
}

impl<'a> BillCsvRowWithExtraStuff<'a> {
    pub fn new(bill: Bill, extra: ExtraBillStuff, query: &'a str) -> Self {
        Self(bill, extra, Query { query })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BillResult {
    pub status: String,
    pub bill: ExtraBillStuff,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExtraBillStuff {
    pub state_link: Option<String>,
    pub status: Status,
    pub status_date: Option<String>,
    pub description: Option<String>,
}

// Option stuff is kinda bad, but I want to ignore when some fields
// are randomly null from API.

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bill {
    pub relevance: u32,
    pub state: Option<String>,
    pub bill_number: Option<String>,
    pub bill_id: u32,
    pub change_hash: Option<String>,
    pub url: Option<String>,
    pub last_action: Option<String>,
    pub last_action_date: Option<String>,
    pub title: Option<String>,
    pub text_url: Option<String>,
    pub research_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Summary {
    page: Option<String>,
    range: Option<String>,
    relevancy: Option<String>,
    count: u32,
    page_current: u32,
    page_total: u32,
}

#[derive(Deserialize)]
struct SearchResult {
    status: String,
    searchresult: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize_repr)]
#[repr(u8)]
pub enum Status {
    NA = 0,
    Introduced = 1,
    Engrossed = 2,
    Enrolled = 3,
    Passed = 4,
    Vetoed = 5,
    Failed = 6,
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

pub fn get_bill(client: &Client, bill_id: u32) -> reqwest::Result<ExtraBillStuff> {
    let params = &[("op", "getBill"), ("id", &bill_id.to_string())];
    build_prefix(client)
        .query(params)
        .send()?
        .json::<BillResult>()
        .map(|r| r.bill)
}
