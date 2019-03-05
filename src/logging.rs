// Copyright 2019 Gregory Meyer
//
// Permission is hereby granted, free of charge, to any person
// obtaining a copy of this software and associated documentation
// files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy,
// modify, merge, publish, distribute, sublicense, and/or sell copies
// of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS
// BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
// ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{
    env, mem,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime},
};

use log::{info, LevelFilter, Log, Metadata, Record};
use parking_lot::Mutex;

pub fn init() {
    init_with_polling_period(DEFAULT_PERIOD);
}

pub fn init_with_polling_period(polling_period: Duration) {
    let logger = Box::new(AsyncLogger::with_polling_period(polling_period));
    let (max_level, defaulted) = get_max_level();

    log::set_boxed_logger(logger).unwrap();
    log::set_max_level(max_level);

    if let Some(maybe_unparsed) = defaulted {
        if let Some(unparsed) = maybe_unparsed {
            info!(
                "couldn't parse '{}' as a maximum logging level, using INFO",
                unparsed
            );
        } else {
            info!("no maximum logging level provided, using INFO");
        }
    }
}

// returns None if not defaulted
// returns Some(None) if defaulted because no env var was found
// returns Some(Some(s)) if defaulted because env var couldn't be parsed
fn get_max_level() -> (LevelFilter, Option<Option<String>>) {
    let level_str = match env::var("RUST_LOG") {
        Ok(l) => l,
        Err(_) => return (DEFAULT_LOG_LEVEL, Some(None)),
    };

    match level_str.to_lowercase().parse() {
        Ok(l) => (l, None),
        Err(_) => (DEFAULT_LOG_LEVEL, Some(Some(level_str))),
    }
}

const DEFAULT_LOG_LEVEL: LevelFilter = LevelFilter::Info;

struct AsyncLogger {
    sink: Arc<Sink>,
    keep_running: Arc<AtomicBool>,
    sink_thread: Option<JoinHandle<()>>, // so we can move out of sink_thread and join
}

const DEFAULT_PERIOD: Duration = Duration::from_millis(100);

impl AsyncLogger {
    fn with_polling_period(polling_period: Duration) -> AsyncLogger {
        let sink = Arc::new(Sink::new());
        let keep_running = Arc::new(AtomicBool::new(true));

        let child_sink = sink.clone();
        let child_keep_running = keep_running.clone();

        let sink_thread = Some(thread::spawn(move || {
            while child_keep_running.load(Ordering::Relaxed) {
                child_sink.pop();
                thread::sleep(polling_period);
            }
        }));

        AsyncLogger {
            sink,
            keep_running,
            sink_thread,
        }
    }

    fn format(record: &Record) -> String {
        if let Some(module) = record.module_path() {
            if module != record.target() {
                return format!(
                    "[{} {} {}/{}] {}\n",
                    humantime::format_rfc3339(SystemTime::now()),
                    record.level(),
                    module,
                    record.target(),
                    record.args()
                );
            }
        }

        format!(
            "[{} {} {}] {}\n",
            humantime::format_rfc3339(SystemTime::now()),
            record.level(),
            record.target(),
            record.args()
        )
    }
}

impl Log for AsyncLogger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
        // metadata.level() >= Level::Info
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        self.sink.push(AsyncLogger::format(record))
    }

    fn flush(&self) {
        self.sink.pop();
    }
}

impl Drop for AsyncLogger {
    fn drop(&mut self) {
        self.keep_running.store(false, Ordering::Relaxed);
        self.sink_thread.take().unwrap().join().unwrap();
    }
}

struct Sink {
    queue: Mutex<String>,
}

impl Sink {
    fn new() -> Sink {
        Sink {
            queue: Mutex::new(String::new()),
        }
    }

    fn push(&self, msg: String) {
        let mut queue = self.queue.lock();
        queue.push_str(&msg);
    }

    fn pop(&self) {
        let mut messages = String::new();

        {
            let mut queue = self.queue.lock();

            if queue.is_empty() {
                return;
            }

            mem::swap(&mut *queue, &mut messages);
        }

        Sink::print(&messages)
    }

    fn print(messages: &str) {
        if messages.is_empty() {
            return;
        }

        eprint!("{}", messages);
    }
}

impl Drop for Sink {
    fn drop(&mut self) {
        Sink::print(self.queue.get_mut());
    }
}
