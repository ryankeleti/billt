use std::{collections::BTreeMap, fmt, fs, io, path::Path};

use serde::{Deserialize, Serialize};
//use chrono::{DateTime, Utc};

use crate::search::Bill;

#[derive(Serialize, Deserialize)]
pub struct Db {
    pub bills: BTreeMap<u32, Entry>,
    pub saved_searches: Vec<String>,
}

impl Db {
    pub fn read(path: &Path) -> Result<Self, DbError> {
        let data = fs::read(path)?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub fn write(&self, path: &Path) -> Result<(), DbError> {
        fs::copy(path, path.with_extension("json.bkp"))?;
        fs::write(path, serde_json::to_vec_pretty(self)?)?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Entry {
    pub bill: Bill,
    //pub last_checked: DateTime<Utc>
}

#[derive(Debug)]
pub enum DbError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for DbError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for DbError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DbError {}
