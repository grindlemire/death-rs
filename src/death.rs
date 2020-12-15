use super::error::{CloseTimedOutError, DeathError};
use crossbeam_channel::after;
use crossbeam_channel::{bounded, select, unbounded, Receiver, Sender};
use signal_hook::iterator::Signals;
use std::{error::Error, fmt::Debug, thread::spawn, time::Duration};

pub trait Life: Debug {
    fn run(&self, done: Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn id(&self) -> String;
}

#[derive(Debug)]
pub struct Death {
    signals: Receiver<()>,
    closers: Vec<Option<Closer>>,
    timeout: Duration,

    signal_closed: Sender<Result<(), Box<dyn Error + Send + Sync>>>,
    receive_closed: Receiver<Result<(), Box<dyn Error + Send + Sync>>>,
}

#[derive(Debug)]
struct Closer {
    id: String,
    close: Sender<()>,
}

impl Death {
    pub fn new(signals: &[i32], timeout: Duration) -> Result<Death, Box<dyn Error>> {
        let (signal_closed, receive_closed) = unbounded();
        Ok(Death {
            signals: Death::register_signals(signals)?,
            closers: Vec::new(),
            timeout: timeout,
            signal_closed: signal_closed,
            receive_closed: receive_closed,
        })
    }

    pub fn give_life<T: 'static>(&mut self, runner: T) -> &Death
    where
        T: Life + std::marker::Send,
    {
        let (send_done, receive_done) = bounded(0);
        let closer = Some(Closer {
            id: runner.id(),
            close: send_done,
        });
        let signaler = self.signal_closed.clone();

        spawn(move || {
            let _ = signaler.send(runner.run(receive_done));
        });

        self.closers.push(closer);
        self
    }

    pub fn wait_for_death(&mut self) -> Vec<Box<dyn Error>> {
        loop {
            select! {
                recv(self.signals) -> _ => {
                    return self.send_shutdown();
                }
            }
        }
    }

    fn send_shutdown(&mut self) -> Vec<Box<dyn Error>> {
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
                            Err(e) => Err(DeathError::new(e)),
                            _ => Ok(()),
                        },
                        Err(t) => Err(DeathError::new(t.into())),
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
                    errors.push(CloseTimedOutError::new(waiting).into());
                    break 'receive_output;
                }
            }
        }
        errors
    }

    fn register_signals(signals: &[i32]) -> Result<Receiver<()>, Box<dyn Error>> {
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
