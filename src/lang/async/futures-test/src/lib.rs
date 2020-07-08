#[cfg(test)]
mod tests {
    use futures::executor::ThreadPool;
    use futures::channel::mpsc;
    use futures::StreamExt;

    #[test]
    fn thread_pool() {
        let pool = ThreadPool::new().expect("Failed to build pool");
        let (tx, rx) = mpsc::unbounded::<i32>();

        let fut_values = async {
            let fut_tx_result = async move {
                (0..=100).for_each(|v| {
                    tx.unbounded_send(v).expect("Failed to send");
                })
            };
            pool.spawn_ok(fut_tx_result);
            let fut_values = rx.map(|v| v * 2).collect();
            fut_values.await
        };

        let values: Vec<i32> = futures::executor::block_on(fut_values);
        println!("Values = {:?}", values);
    }
}
