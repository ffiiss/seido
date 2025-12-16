use std::*;

struct Shared {
    exiting: bool,
    remains: [time::Instant; 8],
}

pub struct BitExciter {
    thread: Option<thread::JoinHandle<()>>,
    shared: sync::Arc<sync::Mutex<Shared>>,
    cvar: sync::Arc<sync::Condvar>,
}

impl Drop for BitExciter {
    fn drop(&mut self) {
        {
            let mut shared = self.shared.lock().unwrap();
            shared.exiting = true;
            self.cvar.notify_one();
        }
        self.thread.take().unwrap().join().ok();
    }
}

impl BitExciter {
    pub fn new(setter: Box<dyn FnMut(u8) -> () + Send>) -> Self {
        let shared = sync::Arc::new(sync::Mutex::new(Shared {
            exiting: false,
            remains: [time::Instant::now(); 8],
        }));
        let cvar = sync::Arc::new(sync::Condvar::new());

        let thread = thread::spawn({
            let shared = shared.clone();
            let cvar = cvar.clone();
            move || BitExciter::process(&shared, &cvar, setter)
        });

        BitExciter {
            thread: Some(thread),
            shared: shared,
            cvar: cvar,
        }
    }

    pub fn excite(&self, n: usize, dur: time::Duration) {
        let until = time::Instant::now() + dur;

        let mut shared = self.shared.lock().unwrap();
        shared.remains[n] = cmp::max(shared.remains[n], until);
        self.cvar.notify_one();
    }

    fn process(
        shared: &sync::Mutex<Shared>,
        cvar: &sync::Condvar,
        mut setter: Box<dyn FnMut(u8) -> () + Send>,
    ) {
        let mut prev_bits = 0x00;
        let mut shared = shared.lock().unwrap();
        while !shared.exiting {
            let now = time::Instant::now();

            let mut next_bits = 0;
            for r in shared.remains.iter() {
                next_bits = (next_bits >> 1) | if *r <= now { 0x00 } else { 0x80 };
            }

            if next_bits != prev_bits {
                setter(next_bits);
                #[cfg(debug_assertions)]
                eprintln!("{:?}, {:08b}", now, next_bits);
                prev_bits = next_bits;
            }

            shared = match shared.remains.into_iter().filter(|r| *r > now).min() {
                Some(until) => cvar.wait_timeout(shared, until - now).unwrap().0,
                None => cvar.wait(shared).unwrap(),
            };
        }

        setter(0);
    }
}
