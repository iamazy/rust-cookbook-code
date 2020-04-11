extern crate slog;

use raft::eraftpb::ConfState;
use raft::prelude::*;
use raft::storage::MemStorage;
use slog::{Drain, Logger};

use std::collections::HashMap;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::{Duration, Instant};

type ProposeCallback = Box<dyn Fn() + Send>;

enum Msg {
    Propose {
        id: u8,
        cb: ProposeCallback,
    },
    #[allow(dead_code)]
    Raft(Message),
}

pub fn single_node() {
    let storage = MemStorage::new_with_conf_state(ConfState::from((vec![1], vec![])));

    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain)
        .chan_size(4096)
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();

    let logger = slog::Logger::root(drain, slog::o!("tag" => format!("[{}]",1)));

    let cfg = Config {
        id: 1,
        election_tick: 10,
        heartbeat_tick: 3,
        max_size_per_msg: 1024 * 1024 * 1024,
        max_inflight_msgs: 256,
        applied: 0,
        ..Default::default()
    };

    // create the raft node
    let mut r = RawNode::new(&cfg, storage).unwrap();

    let (sender, receiver) = mpsc::channel();

    // use another thread to propose a raft request
    send_propose(logger.clone(), sender);

    // loop forever to drive the raft
    let mut t = Instant::now();
    let mut timeout = Duration::from_millis(100);

    // use a hashmap to hold the `propose` callbacks
    let mut cbs = HashMap::new();

    loop {
        match receiver.recv_timeout(timeout) {
            Ok(Msg::Propose { id, cb }) => {
                cbs.insert(id, cb);
                r.propose(vec![], vec![id]).unwrap();
            }
            Ok(Msg::Raft(m)) => r.step(m).unwrap(),
            Err(RecvTimeoutError::Timeout) => (),
            Err(RecvTimeoutError::Disconnected) => return,
        }

        let d = t.elapsed();
        t = Instant::now();
        if d >= timeout {
            timeout = Duration::from_millis(100);
            // we drive raft every 100ms
            r.tick();
        } else {
            timeout -= d;
        }
        on_ready(&mut r, &mut cbs);
    }
}

fn on_ready(r: &mut RawNode<MemStorage>, cbs: &mut HashMap<u8, ProposeCallback>) {
    if !r.has_ready() {
        return;
    }

    let mut ready = r.ready();
    let is_leader = r.raft.leader_id == r.raft.id;
    if is_leader {
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            //here we only have one peer, so can ignore this.
        }
    }

    if !raft::is_empty_snap(ready.snapshot()) {
        // This is a snapshot,we need to apply the snapshot at first.
        r.mut_store()
            .wl()
            .apply_snapshot(ready.snapshot().clone())
            .unwrap();
    }

    if !ready.entries().is_empty() {
        //append entries to the raft log
        r.mut_store().wl().append(ready.entries()).unwrap();
    }

    if let Some(hs) = ready.hs() {
        r.mut_store().wl().set_hardstate(hs.clone());
    }

    if !is_leader {
        let msgs = ready.messages.drain(..);
        for msg in msgs {
            // send messages to other peers
        }
    }

    if let Some(committed_entries) = ready.committed_entries.take() {
        let mut last_apply_index = 0;
        for entry in committed_entries {
            // mostly, you need to save the last apply index to resume appling
            // after restart. here we just ignore this because we use a memory storage
            last_apply_index = entry.index;
            if entry.data.is_empty() {
                // empty entry, when the peer becomes leader it will send an empty entry.
                continue;
            }

            if entry.get_entry_type() == EntryType::EntryNormal {
                if let Some(cb) = cbs.remove(entry.data.get(0).unwrap()) {
                    cb();
                }
            }

            // TODO: handle EntryConfChange
        }
    }

    //advance the raft
    r.advance(ready);
}

fn send_propose(logger: Logger, sender: mpsc::Sender<Msg>) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(10));
        let (s1, r1) = mpsc::channel::<u8>();
        slog::info!(logger, "propose a request");

        sender
            .send(Msg::Propose {
                id: 1,
                cb: Box::new(move || {
                    s1.send(0).unwrap();
                }),
            })
            .unwrap();

        let n = r1.recv().unwrap();
        assert_eq!(n, 0);
        slog::info!(logger, "receive the propose callback");
    });
}
