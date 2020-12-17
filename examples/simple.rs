use crossbeam_channel::{select, Receiver};
use death_rs::death::{Death, Life};
use error::Error;
use log::error;
use log::info;
use rand::Rng;
use signal_hook::{SIGINT, SIGTERM};
use simple_logger::SimpleLogger;
use std::thread::sleep;
use std::time::Duration;
use std::{error, process::exit};

fn main() {
    SimpleLogger::new().init().unwrap();

    let mut d: Death = match Death::new(
        // pass the signals to death you want to catch
        &[SIGINT, SIGTERM],
        // pass the timeout to wait for the workers to gracefully stop
        Duration::from_millis(800),
    ) {
        Ok(d) => d,
        Err(e) => panic!("Error creating death: {}", e),
    };

    // Create n workers and register them with death.
    // This will spin them up in their own threads.
    for i in 0..10 {
        let worker = Worker::new(i);
        d.give_life(worker);
    }

    // block the main thread waiting for a signal.
    // wait_for_death will return a list of all the errors
    // propagated back from the children (and a timeout if they didn't close
    // fast enough).
    let errors = d.wait_for_death();
    errors.iter().for_each(|e| error!("{}", e));
    if errors.len() as i32 > 0 {
        exit(1)
    }
}

#[derive(thiserror::Error, Debug)]
enum MyError {
    #[error("my error")]
    Err,
}

#[derive(Debug)]
struct Worker {
    id: i32,
}

impl Worker {
    fn new(i: i32) -> Worker {
        Worker { id: i }
    }
}

impl Life for Worker {
    fn run(&mut self, done: Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Running {}", self.id);
        loop {
            select! {
                recv(done) -> _ => {
                    let mut rng = rand::thread_rng();
                    let timeout = rng.gen_range(0, 1000);
                    sleep(Duration::from_millis(timeout));
                    info!("shut down {} with {} delay", self.id, timeout);
                    return Err(MyError::Err.into());
                }
            }
        }
    }
}
