# death-rs

A lightweight thread management library for easily handling concurrency in rust.

## Why?

I am a big fan of the golang [life](https://github.com/vrecan/life) and [death](https://github.com/vrecan/death) libraries and their concurrency paradigm for managing go routines in microservices. This library follows a similar philosophy and is intended to facilitate clean services that manage independent threads and properly encapsulate business logic without worrying about the nitty gritty details of thread lifecycles.

## What does `death-rs` do?

-   Gives a simple and lightweight interface for creating worker threads to do some business logic
-   Captures signals from the OS and propagates them down to the worker threads to allow each worker thread to clean up and stop gracefully.
-   Handles the concurrency for a graceful shutdown.

## tl;dr quickstart

1. Implement the `Life` trait for your struct
    ```rs
    pub trait Life: std::fmt::Debug {
        // run is called in a separate thread. Done receives a signal when to stop the thread
        fn run(&self, done: Receiver<()>) -> Result<(), Box<dyn Error + Send + Sync>>;
        // id is used to identify the worker thread
        fn id(&self) -> String;
    }
    ```
2. Initialize death
    ```rs
    let mut d: Death = Death::new(
        // pass the signals to death you want to catch
        &[SIGINT, SIGTERM],
        // pass the timeout to wait for the workers to gracefully stop
        Duration::from_millis(800),
    )?
    ```
3. Register your worker with death (it will handle all the async)
    ```rs
    d.give_life(Worker::new());
    ```
4. Call `wait_for_death` in the main thread to block until you receive an interrupt signal
    ```rs
    // errors here is a vector of errors returned from the workers
    let errors = d.wait_for_death();
    ```
5. Do your business logic in your threads and communicate via channels

**For a working example see [examples/simple.rs](./examples/simple.rs)**

## To run the example:

```
cargo run --example simple
```

and press `ctrl-c` after it boots up.

# Release Process

## Rules for release branches:

-   If you are releasing a new major version you need to branch off of master into a branch `release-branch.v#` (example `release-branch.v2` for a 2.x release)
-   If you are releasing a minor or patch update to an existing major release make sure to merge master into the release branch

## Rules for tagging and publishing the release

When you are ready to publish the release make sure you...

1. Merge your changes into the correct release branch.
2. Check out the release branch locally (example: `git pull origin release-branch.v3`)
3. Create a new tag for the specific release version you will publish (example: `git tag v3.0.1`)
4. Push the tag up to github (example: `git push origin v3.0.1`)
5. Go to the release tab in github
6. Select the target branch as the release branch and type in the tag name (tagname should include `v` so example: `v3.0.1`)
7. Write a title and a well worded description on exactly what is in this change
8. Click publish release
