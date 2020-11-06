use std::sync::{Arc, Mutex, Condvar};
use std::thread;

fn main() {

    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    thread::spawn(move ||{
        let (lock, cvar) = &*pair2;
        let mut started = lock.lock().unwrap();
        *started = true;
        println!("111");
        cvar.notify_one();
    });

    let (lock, cvar) = &*pair;
    let mut started = lock.lock().unwrap();
    println!("233");
    while !*started {
        println!("133");
        started = cvar.wait(started).unwrap();
        println!("323");
    }

}
