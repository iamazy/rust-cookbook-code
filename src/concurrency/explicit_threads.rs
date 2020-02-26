extern crate crossbeam;

use crossbeam::crossbeam_channel;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{thread, time};

pub fn find_max(arr: &[i32]) -> Option<i32> {
    const THREADHOLD: usize = 2;

    if arr.len() <= THREADHOLD {
        return arr.iter().cloned().max();
    }
    let mid = arr.len() / 2;
    let (left, right) = arr.split_at(mid);

    crossbeam::scope(|s| {
        let thread_l = s.spawn(|_| find_max(left));
        let thread_r = s.spawn(|_| find_max(right));

        // find_max 返回值类型 Option<i32>
        let max_l = thread_l.join().unwrap()?;
        let max_r = thread_r.join().unwrap()?;
        Some(max_l.max(max_r))
    })
    .unwrap()
}

pub fn pass_data_in_two_threads() {
    let (snd, rcv) = crossbeam_channel::unbounded::<i32>();
    let n_msgs = 5;
    crossbeam::scope(|s| {
        s.spawn(|_| {
            for i in 0..n_msgs {
                snd.send(i).unwrap();
                thread::sleep(time::Duration::from_millis(100));
            }
        });
    })
    .unwrap();
    for _ in 0..n_msgs {
        let msg = rcv.recv().unwrap();
        println!("Received {}", msg);
    }
}

lazy_static! {
    static ref FRUIT: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn insert(fruit: &str) -> std::io::Result<()> {
    let mut db = FRUIT
        .lock()
        .map_err(|_| "Failed to acquire MutexGuard")
        .unwrap();
    db.push(fruit.to_string());
    Ok(())
}

pub fn maintain_global_mutable_state() -> std::io::Result<()> {
    insert("apple")?;
    insert("orange")?;
    insert("peach")?;
    {
        let db = FRUIT
            .lock()
            .map_err(|_| "Failed to acquire MutexGuard")
            .unwrap();
        db.iter()
            .enumerate()
            .for_each(|(i, item)| println!("{}:{}", i, item));
    }
    insert("grape")?;
    Ok(())
}
