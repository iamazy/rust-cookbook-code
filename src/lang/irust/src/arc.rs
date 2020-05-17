use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

fn main() {
    // 1. Cloning references
    let foo = Arc::new(vec![1.0, 2.0, 3.0]);
    // The two syntaxes below are equivalent
    let a = foo.clone();
    let b = Arc::clone(&foo);
    println!("{:p}", a);
    println!("{:p}", b);

    // 2. Deref behavior
    let arc = Arc::new(());
    // downgrade to `Week<T>` which cannot auto-dereference to T
    Arc::downgrade(&arc);

    // 3. Sharing some immutable data between threads
    let five = Arc::new(5);
    for _ in 0..10 {
        let five = Arc::clone(&five);
        thread::spawn(move || {
            println!("{:?}", five);
        });
    }

    // 4. Sharing a mutable `AtomicUsize`
    let val = Arc::new(AtomicUsize::new(5));
    for _ in 0..10 {
        let val = Arc::clone(&val);
        thread::spawn(move || {
            let v = val.fetch_add(1, Ordering::SeqCst);
            println!("{:?}", v);
        });
    }
}
