extern crate crossbeam;

use crossbeam::crossbeam_channel;
use image::{ImageBuffer, Pixel, Rgb};
use lazy_static::lazy_static;
use num::complex::Complex;
use ring::digest::{Context, Digest, SHA256};
use std::fs::File;
use std::io::{BufReader, Error, Read};
use std::path::Path;
use std::sync::mpsc::{channel, RecvError};
use std::sync::Mutex;
use std::{thread, time};
use threadpool::ThreadPool;
use walkdir::WalkDir;

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
    // lock默认在block末尾释放，加上{}会让临界区更短
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

fn compute_digest<P: AsRef<Path>>(filepath: P) -> Result<(Digest, P), Error> {
    let mut buf_reader = BufReader::new(File::open(&filepath)?);
    let mut context = Context::new(&SHA256);
    let mut buffer = [0; 1024];

    loop {
        let count = buf_reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.update(&buffer[..count]);
    }
    Ok((context.finish(), filepath))
}

pub fn calculate_sha256_sum_of_iso_concurrently() -> Result<(), std::io::Error> {
    let pool = ThreadPool::new(num_cpus::get());

    let (tx, rx) = channel();

    for entry in WalkDir::new("/Users/iamazy/Downloads")
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| !e.path().is_dir())
    {
        let path = entry.path().to_owned();
        let tx = tx.clone();
        pool.execute(move || {
            let digest = compute_digest(path);
            tx.send(digest).expect("Could't not send data!");
        });
    }
    drop(tx);
    for t in rx.iter() {
        let (sha, path) = t?;
        println!("{:?}:{:?}", sha, path);
    }

    Ok(())
}

pub fn draw_fractal() -> Result<(), RecvError> {
    let (width, height) = (1920, 1080);
    let mut img = ImageBuffer::new(width, height);
    let iterations = 300;

    let c = Complex::new(-0.8, 0.156);
    let pool = ThreadPool::new(num_cpus::get());
    let (tx, rx) = channel();

    for y in 0..height {
        let tx = tx.clone();
        pool.execute(move || {
            for x in 0..width {
                let i = julia(c, x, y, width, height, iterations);
                let pixel = wavelength_to_rgb(380 + i * 400 / iterations);
                tx.send((x, y, pixel)).expect("Could't send data!");
            }
        });
    }

    for _ in 0..(width * height) {
        let (x, y, pixel) = rx.recv()?;
        img.put_pixel(x, y, pixel);
    }

    let _ = img.save("output.png");
    Ok(())
}

fn julia(c: Complex<f32>, x: u32, y: u32, width: u32, height: u32, max_iter: u32) -> u32 {
    let width = width as f32;
    let height = height as f32;

    let mut z = Complex {
        re: 3.0 * (x as f32 - 0.5 * width) / width,
        im: 2.0 * (y as f32 - 0.5 * height) / height,
    };

    let mut i = 0;
    for t in 0..max_iter {
        if z.norm() > 2.0 {
            break;
        }
        z = z * z + c;
        i = t;
    }
    i
}

fn normalize(color: f32, factor: f32) -> u8 {
    ((color * factor).powf(0.8) * 255.) as u8
}

fn wavelength_to_rgb(wavelength: u32) -> Rgb<u8> {
    let wave = wavelength as f32;
    let (r, g, b) = match wavelength {
        // 数字后面加个点表示转为浮点型
        // ..= 表示左闭右闭区间
        380..=439 => ((440. - wave) / (440. - 380.), 0.0, 1.0),
        440..=489 => (0.0, (wave - 440.) / (490. - 440.), 1.0),
        490..=509 => (0.0, 1.0, (510. - wave) / (510. - 490.)),
        510..=579 => ((wave - 510.) / (580. - 510.), 1.0, 0.0),
        580..=644 => (1.0, (645. - wave) / (645. - 580.), 0.0),
        645..=780 => (1.0, 0.0, 0.0),
        _ => (0.0, 0.0, 0.0),
    };

    let factor = match wavelength {
        380..=419 => 0.3 + 0.7 * (wave - 380.) / (420. - 380.),
        701..=780 => 0.3 + 0.7 * (780. - wave) / (780. - 700.),
        _ => 1.0,
    };

    let (r, g, b) = (
        normalize(r, factor),
        normalize(g, factor),
        normalize(b, factor),
    );
    Rgb::from_channels(r, g, b, 0)
}
