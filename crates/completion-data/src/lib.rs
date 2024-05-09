//! Completion data for game events, scrapped from Alliedmodders.

use std::{borrow::Cow, io::Read};

use flate2::read::GzDecoder;
use fxhash::FxHashMap;
use once_cell::sync::Lazy;
use serde::Deserialize;

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database<'a>(#[serde(borrow)] FxHashMap<&'a str, Game<'a>>);

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
    #[serde(borrow)]
    events: Vec<Event<'a>>,
}

impl Game<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn events(&self) -> &[Event] {
        &self.events
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,
}

impl Event<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl<'a> Database<'a> {
    // FIXME: Get rid of the double borrow
    pub fn iter(&self) -> impl Iterator<Item = (&&str, &Game)> + '_ {
        self.0.iter()
    }

    pub fn get(&self, game: &str) -> Option<&Game> {
        self.0.get(game)
    }
}

/// Bytes of the compressed JSON data for events completion.
const EVENTS_JSON_GZ: &[u8] = include_bytes!("../data/events.json.gz");

/// The completion data database.
pub static DATABASE: Lazy<Database<'static>> = Lazy::new(|| {
    let mut decoder = GzDecoder::new(EVENTS_JSON_GZ);
    let json = Box::leak(Box::default());
    decoder.read_to_string(json).unwrap();
    let db: Database = serde_json::from_str(json).unwrap();

    db
});
