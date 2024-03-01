use std::time::Duration;

use base_db::{FileExtension, SourceDatabase, SubGraph};
use crossbeam::channel::Sender;
use fxhash::FxHashMap;
use hir_def::DefDatabase;
use ide_db::{FxIndexMap, RootDatabase};
use salsa::{Cancelled, Database, ParallelDatabase, Snapshot};
use vfs::FileId;

#[derive(Debug)]
pub struct ParallelPrimeCachesProgress {
    /// the projects that we are currently priming.
    pub projects_currently_indexing: Vec<String>,
    /// the total number of projects we want to prime.
    pub projects_total: usize,
    /// the total number of projects that have finished priming
    pub projects_done: usize,
}

pub(crate) fn parallel_prime_caches<F>(
    db: &RootDatabase,
    num_worker_threads: u8,
    cb: &(dyn Fn(ParallelPrimeCachesProgress) + Sync),
    file_id_to_name: F,
) where
    F: Fn(FileId) -> Option<String> + Sync + std::panic::UnwindSafe,
{
    let graph = db.graph();
    let projects_to_prime = graph.subgraphs_with_roots();
    let projects_to_prime = projects_to_prime
        .into_iter()
        .filter(|(_, subgraph)| subgraph.root.extension == FileExtension::Sp)
        .collect::<FxHashMap<_, _>>();

    enum ParallelPrimeCacheWorkerProgress {
        BeginProject { file_id: FileId, file_name: String },
        EndProject { file_id: FileId },
    }

    let (work_sender, progress_receiver) = {
        let (progress_sender, progress_receiver) = crossbeam::channel::unbounded();
        let (work_sender, work_receiver): (Sender<(SubGraph, String)>, _) =
            crossbeam::channel::unbounded();
        let prime_caches_worker = move |db: Snapshot<RootDatabase>| {
            while let Ok((subgraph, file_name)) = work_receiver.recv() {
                let file_id = subgraph.root.file_id;
                progress_sender
                    .send(ParallelPrimeCacheWorkerProgress::BeginProject { file_id, file_name })?;

                subgraph.nodes.iter().for_each(|node| {
                    db.file_def_map(node.file_id);
                });

                progress_sender.send(ParallelPrimeCacheWorkerProgress::EndProject { file_id })?;
            }

            Ok::<_, crossbeam::channel::SendError<_>>(())
        };

        for _ in 0..num_worker_threads {
            let worker = prime_caches_worker.clone();
            let db = db.snapshot();

            stdx::thread::Builder::new(stdx::thread::ThreadIntent::Worker)
                .allow_leak(true)
                .spawn(move || Cancelled::catch(|| worker(db)))
                .expect("failed to spawn thread");
        }

        (work_sender, progress_receiver)
    };

    let projects_total = projects_to_prime.len();
    let mut projects_done = 0;

    // an index map is used to preserve ordering so we can sort the progress report in order of
    // "longest crate to index" first
    let mut projects_currently_indexing =
        FxIndexMap::with_capacity_and_hasher(num_worker_threads as _, Default::default());

    while projects_done < projects_total {
        db.unwind_if_cancelled();

        for subgraph in projects_to_prime.values().cloned() {
            let file_id = subgraph.root.file_id;
            work_sender
                .send((subgraph, file_id_to_name(file_id).unwrap_or_default()))
                .ok();
        }

        // recv_timeout is somewhat a hack, we need a way to from this thread check to see if the current salsa revision
        // is cancelled on a regular basis. workers will only exit if they are processing a task that is cancelled, or
        // if this thread exits, and closes the work channel.
        let worker_progress = match progress_receiver.recv_timeout(Duration::from_millis(10)) {
            Ok(p) => p,
            Err(crossbeam::channel::RecvTimeoutError::Timeout) => {
                continue;
            }
            Err(crossbeam::channel::RecvTimeoutError::Disconnected) => {
                // our workers may have died from a cancelled task, so we'll check and re-raise here.
                db.unwind_if_cancelled();
                break;
            }
        };
        match worker_progress {
            ParallelPrimeCacheWorkerProgress::BeginProject { file_id, file_name } => {
                projects_currently_indexing.insert(file_id, file_name);
            }
            ParallelPrimeCacheWorkerProgress::EndProject { file_id } => {
                projects_currently_indexing.swap_remove(&file_id);
                projects_done += 1;
            }
        };

        let progress = ParallelPrimeCachesProgress {
            projects_currently_indexing: projects_currently_indexing.values().cloned().collect(),
            projects_done,
            projects_total,
        };

        cb(progress);
    }
}
