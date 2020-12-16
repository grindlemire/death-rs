use super::error::Error;
use crossbeam_channel::after;
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use signal_hook::iterator::Signals;
use std::{fmt::Debug, thread::spawn, time::Duration};

pub trait Life: Debug {
    fn run(&self, done: Receiver<()>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

#[derive(Debug)]
pub struct Death {
    // the channel listening for OS signals
    signals: Receiver<()>,

    // the list of worker threads we are tracking
    closers: Vec<Option<Closer>>,

    // the timeout to wait for the children to shut down
    timeout: Duration,

    // used to signal to the workers they need to shut down
    signal_closed: Sender<Result<(), Error>>,

    // used to signal the main thread that a worker has shut down successfully
    receive_closed: Receiver<Result<(), Error>>,
}

#[derive(Debug)]
struct Closer {
    close: Sender<()>,
}

impl Death {
    pub fn new(signals: &[i32], timeout: Duration) -> Result<Death, Error> {
        let (signal_closed, receive_closed) = unbounded();
        Ok(Death {
            signals: Death::register_signals(signals)?,
            closers: Vec::new(),
            timeout: timeout,
            signal_closed: signal_closed,
            receive_closed: receive_closed,
        })
    }

    pub fn give_life<T>(&mut self, runner: T) -> &mut Death
    where
        T: Life + std::marker::Send + 'static,
    {
        let (send_done, receive_done) = bounded(0);
        let closer = Some(Closer { close: send_done });
        let signaler = self.signal_closed.clone();

        spawn(move || {
            // This is a hack to force the std::error:Error into death::error::Error
            // while keeping the translation transparent to the caller
            let result = match runner.run(receive_done) {
                Ok(r) => Ok(r),
                Err(e) => Err(Error::from(e)),
            };
            let _ = signaler.send(result);
        });

        self.closers.push(closer);
        self
    }

    pub fn wait_for_death(&mut self) -> Vec<Error> {
        loop {
            select! {
                recv(self.signals) -> _ => {
                    return self.send_shutdown();
                }
            }
        }
    }

    fn send_shutdown(&mut self) -> Vec<Error> {
        let mut errors = Vec::new();

        // send shutdown signal
        self.closers.iter_mut().for_each(|c| {
            let _ = c.as_ref().unwrap().close.send(());
        });

        // initialize timeout and wait for responses
        let timeout = after(self.timeout);
        let mut waiting = self.closers.len() as i32;
        'receive_output: loop {
            select! {
                recv(self.receive_closed) -> result => {
                    waiting = waiting - 1;
                    let err = match result {
                        // first strip off the crossbeam error and handle it
                        Ok(returned) => match returned {
                            // then handle the error returned from the worker
                            Err(e) => Err(e),
                            _ => Ok(()),
                        },
                        Err(t) => Err(Error::Channel(t)),
                    };

                    match err {
                        Err(e) => errors.push(e.into()),
                        _ => (),
                    };

                    if waiting <= 0 {
                        break 'receive_output;
                    }
                }

                recv(timeout) -> _ => {
                    errors.push(Error::TimedOut(waiting));
                    break 'receive_output;
                }
            }
        }
        errors
    }

    fn register_signals(signals: &[i32]) -> Result<Receiver<()>, Error> {
        let (sender, receiver) = bounded(100);
        let signals = Signals::new(signals)?;

        spawn(move || {
            for _sig in signals.forever() {
                let _ = sender.send(());
            }
        });

        Ok(receiver)
    }
}
