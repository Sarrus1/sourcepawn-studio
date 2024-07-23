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

    #[serde(borrow)]
    note: Option<Cow<'a, str>>,

    #[serde(borrow)]
    attributes: Vec<Attribute<'a>>,
}

impl Event<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn note(&self) -> Option<&str> {
        self.note.as_deref()
    }

    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute<'a> {
    #[serde(borrow)]
    name: Cow<'a, str>,

    #[serde(borrow)]
    r#type: Cow<'a, str>,

    #[serde(borrow)]
    description: Option<Cow<'a, str>>,
}

impl Attribute<'_> {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
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

    pub fn get_event(&self, game: &str, event: &str) -> Option<&Event> {
        self.0.get(game)?.events.iter().find(|ev| ev.name == event)
    }

    pub fn get_events(&self, name: &str) -> Vec<(String, Event)> {
        let mut res = Vec::new();
        self.0.iter().for_each(|(_, game)| {
            if let Some(ev) = game.events().iter().find(|ev| ev.name() == name) {
                res.push((game.name().into(), ev.clone()));
            }
        });

        res
    }

    /// Returns all the generic events as a vector of owned [`Events`](Event).
    pub fn generic_events(&self) -> Vec<Event> {
        let mut res = Vec::new();
        let names = ["Generic Source", "Generic Source Server"];
        for name in names {
            res.extend(
                self.0
                    .get(name)
                    .expect("expected generic events")
                    .events
                    .iter()
                    .cloned(),
            )
        }

        res
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
