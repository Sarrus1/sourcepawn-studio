use anyhow::Result;
use crossbeam::channel::Receiver;
use itertools::Itertools;
use lsp_server::{Connection, Response};
use lsp_types::{
    notification::{DidOpenTextDocument, Exit, Initialized},
    request::ResolveCompletionItem,
    request::{Completion, Initialize, Shutdown},
    ClientCapabilities, CompletionItem, CompletionParams, CompletionResponse,
    DidOpenTextDocumentParams, InitializeParams, InitializedParams, Location, Position, Range,
    TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams, Url, WorkspaceFolder,
};
use std::{
    env,
    fs::File,
    io,
    path::{Path, PathBuf},
    sync::Once,
    thread::JoinHandle,
    time::Duration,
};
use tempfile::{tempdir, TempDir};
use zip::ZipArchive;

use super::{LspClient, Server};
use store::options::Options;

#[derive(Debug)]
pub enum InternalMessage {
    OptionsRequested,
}

#[derive(Debug)]
pub struct Fixture {
    pub documents: Vec<Document>,
}

impl Fixture {
    pub fn parse(input: &str) -> Fixture {
        let mut documents = Vec::new();

        let mut start = 0;
        if !input.is_empty() {
            for end in input
                .match_indices("%!")
                .skip(1)
                .map(|(i, _)| i)
                .chain(std::iter::once(input.len()))
            {
                documents.push(Document::parse(&input[start..end]));
                start = end;
            }
        }

        Self { documents }
    }

    pub fn write_files(&self, dir: &Path) {
        for document in &self.documents {
            let text = String::from(&document.text);
            let path = dir.join(&document.path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &text).unwrap();
        }
    }

    pub fn setup(&self, client: &LspClient, dir: &Path) {
        for document in &self.documents {
            let text = String::from(&document.text);
            let path = dir.join(&document.path);

            let uri = Url::from_file_path(&path).unwrap();
            let language_id = "sourcepawn".to_string();

            client
                .send_notification::<DidOpenTextDocument>(DidOpenTextDocumentParams {
                    text_document: TextDocumentItem::new(uri, language_id, 0, text),
                })
                .unwrap();
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub path: PathBuf,
    pub text: String,
    pub cursor: Option<Position>,
    pub ranges: Vec<Range>,
}

impl Document {
    pub fn parse(input: &str) -> Self {
        let mut lines = Vec::new();

        let (path, input) = input
            .trim()
            .strip_prefix("%! ")
            .map(|input| input.split_once('\n').unwrap_or((input, "")))
            .unwrap();

        let mut ranges = Vec::new();
        let mut cursor = None;

        for line in input.lines() {
            if line.chars().all(|c| matches!(c, ' ' | '^' | '|' | '!')) && !line.is_empty() {
                let index = (lines.len() - 1) as u32;

                cursor = cursor.or_else(|| {
                    let character = line.find('|')?;
                    Some(Position::new(index, character as u32))
                });

                if let Some(start) = line.find('!') {
                    let position = Position::new(index, start as u32);
                    ranges.push(Range::new(position, position));
                }

                if let Some(start) = line.find('^') {
                    let end = line.rfind('^').unwrap() + 1;
                    ranges.push(Range::new(
                        Position::new(index, start as u32),
                        Position::new(index, end as u32),
                    ));
                }
            } else {
                lines.push(line);
            }
        }

        Self {
            path: PathBuf::from(path),
            text: lines.join("\n"),
            cursor,
            ranges,
        }
    }
}

static LOGGER: Once = Once::new();

#[derive(Debug)]
pub struct TestBed {
    fixture: Fixture,
    locations: Vec<Location>,
    _temp_dir: TempDir,
    temp_dir_path: PathBuf,
    pub internal_rx: Receiver<InternalMessage>,
    client: LspClient,
    client_thread: Option<JoinHandle<()>>,
    server_thread: Option<JoinHandle<()>>,
}

impl Drop for TestBed {
    fn drop(&mut self) {
        log::trace!("Dropping server.");
        let _ = self.client.send_request::<Shutdown>(());
        let _ = self.client.send_notification::<Exit>(());
        self.client_thread.take().unwrap().join().unwrap();
        self.server_thread.take().unwrap().join().unwrap();
    }
}

impl TestBed {
    pub fn new(fixture: &str, add_sourcemod: bool) -> Result<Self> {
        LOGGER.call_once(|| {
            if option_env!("TEST_LOG") == Some("1") {
                fern::Dispatch::new()
                    .level(log::LevelFilter::Trace)
                    .chain(std::io::stderr())
                    .apply()
                    .unwrap()
            }
        });

        let fixture = Fixture::parse(fixture);

        let temp_dir = tempdir()?;
        let temp_dir_path = temp_dir.path().canonicalize()?;

        let temp_sm_dir = tempdir()?;
        let temp_sm_dir_path = temp_sm_dir.path().canonicalize()?;

        let locations: Vec<Location> = fixture
            .documents
            .iter()
            .flat_map(|document| {
                let uri = Url::from_file_path(temp_dir_path.join(&document.path)).unwrap();
                document
                    .ranges
                    .iter()
                    .map(move |range| Location::new(uri.clone(), *range))
            })
            .collect();

        let (server_conn, client_conn) = Connection::memory();
        let (internal_tx, internal_rx) = crossbeam::channel::unbounded();

        let client = LspClient::new(client_conn.sender);

        let server_thread =
            std::thread::spawn(move || Server::new(server_conn, false).run().unwrap());
        let client_thread = {
            let client = client.clone();
            std::thread::spawn(move || {
                let destination = temp_sm_dir_path.clone();
                for message in &client_conn.receiver {
                    match message {
                        lsp_server::Message::Request(request) => {
                            if request.method == "workspace/configuration" {
                                let mut options = Options::default();
                                if add_sourcemod {
                                    let current_dir = env::current_dir().unwrap();
                                    let sourcemod_path =
                                        current_dir.join("test_data/sourcemod.zip");
                                    unzip_file(&sourcemod_path, destination.to_str().unwrap())
                                        .unwrap();
                                    options
                                        .includes_directories
                                        .push(destination.clone().join("include/"));
                                }
                                client
                                    .send_response(Response::new_ok(request.id, vec![options]))
                                    .unwrap();
                                std::thread::sleep(Duration::from_millis(100));
                                internal_tx.send(InternalMessage::OptionsRequested).unwrap()
                            } else {
                                client
                                    .send_error(
                                        request.id,
                                        lsp_server::ErrorCode::MethodNotFound,
                                        "Method not found".into(),
                                    )
                                    .unwrap();
                            }
                        }
                        lsp_server::Message::Response(response) => {
                            client.recv_response(response).unwrap();
                        }
                        lsp_server::Message::Notification(_) => {}
                    }
                }
            })
        };
        fixture.write_files(&temp_dir_path);

        Ok(TestBed {
            fixture,
            locations,
            _temp_dir: temp_dir,
            temp_dir_path,
            client,
            internal_rx,
            client_thread: Some(client_thread),
            server_thread: Some(server_thread),
        })
    }

    pub fn initialize(&self, capabilities: ClientCapabilities) -> Result<()> {
        self.client
            .send_request::<Initialize>(InitializeParams {
                capabilities,
                workspace_folders: Some(vec![WorkspaceFolder {
                    uri: Url::from_file_path(self.temp_dir_path.clone()).unwrap(),
                    name: "test".to_string(),
                }]),
                ..Default::default()
            })
            .unwrap();
        self.client
            .send_notification::<Initialized>(InitializedParams {})
            .unwrap();

        self.fixture.setup(&self.client, self.directory());

        Ok(())
    }

    pub fn client(&self) -> &LspClient {
        &self.client
    }

    pub fn cursor(&self) -> Option<TextDocumentPositionParams> {
        let (document, cursor) = self
            .fixture
            .documents
            .iter()
            .find_map(|document| document.cursor.map(|cursor| (document, cursor)))?;

        let uri = Url::from_file_path(self.temp_dir_path.join(&document.path)).unwrap();
        let id = TextDocumentIdentifier::new(uri);
        Some(TextDocumentPositionParams::new(id, cursor))
    }

    #[allow(unused)]
    pub fn locations(&self) -> &[Location] {
        &self.locations
    }

    pub fn directory(&self) -> &Path {
        &self.temp_dir_path
    }

    #[allow(unused)]
    pub fn documents(&self) -> &[Document] {
        &self.fixture.documents
    }
}

pub fn complete(fixture: &str) -> Vec<CompletionItem> {
    let test_bed = TestBed::new(fixture, true).unwrap();
    test_bed
        .initialize(
            serde_json::from_value(serde_json::json!({
                "textDocument": {
                    "completion": {
                        "completionItem": {
                            "documentationFormat": ["plaintext", "markdown"]
                        }
                    }
                },
                "workspace": {
                    "configuration": true,
                    "workspace_folders": true
                }
            }))
            .unwrap(),
        )
        .unwrap();
    let text_document_position = test_bed.cursor().unwrap();
    test_bed
        .internal_rx
        .recv_timeout(Duration::from_secs(10))
        .unwrap();
    let items = match test_bed
        .client()
        .send_request::<Completion>(CompletionParams {
            text_document_position,
            partial_result_params: Default::default(),
            work_done_progress_params: Default::default(),
            context: None,
        })
        .unwrap()
    {
        Some(CompletionResponse::Array(items)) => items,
        Some(CompletionResponse::List(list)) => list.items,
        None => Vec::new(),
    };

    items
        .into_iter()
        .map(|item| match item.data {
            Some(_) => {
                let mut item = test_bed
                    .client()
                    .send_request::<ResolveCompletionItem>(item)
                    .unwrap();

                item.data = None;
                item
            }
            None => item,
        })
        .sorted_by(|item1, item2| item1.label.cmp(&item2.label))
        .collect()
}

pub fn unzip_file(zip_file_path: &PathBuf, destination: &str) -> Result<(), io::Error> {
    let file = File::open(zip_file_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let dest_path = format!("{}/{}", destination, file.name());

        if file.is_dir() {
            std::fs::create_dir_all(&dest_path)?;
        } else {
            if let Some(parent_dir) = std::path::Path::new(&dest_path).parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            let mut dest_file = File::create(&dest_path)?;
            io::copy(&mut file, &mut dest_file)?;
        }
    }

    Ok(())
}
