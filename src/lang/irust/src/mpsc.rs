use std::sync::mpsc::channel;
use std::thread;

fn main() {
    // 1. simple usage
    // Create a simple streaming channel
    let (tx, rx) = channel();
    thread::spawn(move || {
        tx.send(10).unwrap();
    });

    println!("received: {:?}", rx.recv().unwrap());

    // 2. shared usage
    // Create a shared channel that can be sent along from many threads
    // where tx is the sending half (tx for transmission), and rx is the
    // receiving half (rx for receiving)
    let (tx, rx) = channel();
    for i in 0..10 {
        let tx = tx.clone();
        thread::spawn(move || {
            tx.send(i).unwrap();
        });
    }

    for _ in 0..10 {
        let j = rx.recv().unwrap();
        println!("{}", j);
    }

    // 3. propagating panics
    let (tx, rx) = channel::<i32>();
    drop(tx);
    println!("{:?}", rx.recv().is_err());

    // synchronous channel
    use std::sync::mpsc::sync_channel;

    let (tx, rx) = sync_channel::<i32>(0);
    thread::spawn(move || {
        // This will wait for the parent thread to start receiving
        tx.send(53).unwrap();
    });
    println!("{:?}", rx.recv().unwrap());
}
