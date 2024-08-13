use std::{env, iter, thread};
use std::io::{Read, Write};
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use rand::{Rng, SeedableRng};
use rand::rngs::SmallRng;
use waydows_unix_socket::{UnixListener, UnixStream};

fn run_every_second(iterations_per_second: f64, mut f: impl FnMut() -> ControlFlow<()>) {
    let interval = Duration::from_secs_f64(1.0 / iterations_per_second);
    let mut next_time = Instant::now();

    loop {
        match f() {
            ControlFlow::Continue(()) => {},
            ControlFlow::Break(()) => break,
        }

        next_time += interval;

        if let Some(wait_time) = next_time.checked_duration_since(Instant::now()) {
            thread::sleep(wait_time)
        } else {
            next_time = Instant::now()
        }
    }
}

fn screen(width: usize, height: usize, thread_num: usize, rng: &mut impl Rng) -> Vec<u8> {
    let now = Instant::now();
    let mut screen = vec![0; width * height];
    rng.fill_bytes(&mut screen);

    // println!("thread {thread_num}: create and fill screen with random characters took {:?}", now.elapsed());

    screen
}

#[derive(Default)]
struct RunningAverage {
    count: u32,
    total: Duration,
}

impl RunningAverage {
    fn update(&mut self, duration: Duration) {
        self.count += 1;
        self.total += duration;
    }

    fn get(&self) -> Option<Duration> {
        if self.count == 0 {
            None
        } else {
            Some(self.total / self.count)
        }
    }
}

fn client(path: PathBuf, width: usize, height: usize) {
    let mut stream = UnixStream::connect(&path).unwrap();
    let mut buf = vec![0; width * height];
    let average = Mutex::new(RunningAverage::default());
    
    thread::scope(|s| {
        s.spawn(|| loop {
            thread::sleep(Duration::from_secs(1));
            println!("average: {:?}", average.lock().unwrap().get())
        });

        loop {
            let now = Instant::now();
            stream.read_exact(&mut buf).unwrap();
            average.lock().unwrap().update(now.elapsed());
        }
    })
}

fn server(path: PathBuf, width: usize, height: usize, fps: f64) {
    let listener = UnixListener::bind(&path).unwrap();

    thread::scope(|s| {
        let (screen_sender, screen_receiver) = crossbeam::channel::bounded(fps.round() as usize);

        let mut thread_rng = rand::thread_rng();
        (0..thread::available_parallelism().unwrap().get())
            .map(|num| (num, SmallRng::from_rng(&mut thread_rng).unwrap()))
            .for_each(|(num, mut rng)| {
                let screen_sender = screen_sender.clone();
                s.spawn(move || {
                    loop {
                        screen_sender.send(screen(width, height, num, &mut rng)).unwrap()
                    }
                });
            });

        println!("listening for incoming streams");

        loop {
            let (mut stream, addr) = listener.accept().unwrap();
            let screen_receiver = screen_receiver.clone();
            println!("new client {stream:?} {addr:?}");

            s.spawn(move || {
                run_every_second(fps, move || {
                    match stream.write_all(&screen_receiver.recv().unwrap()) {
                        Ok(()) => ControlFlow::Continue(()),
                        Err(_) => ControlFlow::Break(()),
                    }
                })
            });
        }
    })
}

fn main() {
    let mut args = env::args().skip(1);
    let kind = args.next().unwrap();
    let path: PathBuf = args.next().unwrap().into();
    let width = args.next().unwrap().parse().unwrap();
    let height = args.next().unwrap().parse().unwrap();
    let fps = args.next().unwrap().parse().unwrap();

    if kind == "client" {
        client(path, width, height);
    } else if kind == "server" {
        server(path, width, height, fps);
    } else {
        eprintln!("unknown kind {kind}");
        std::process::exit(1);
    }
}