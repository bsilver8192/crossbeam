use crossbeam::queue::SegQueue;
use std::thread;

mod message;

const MESSAGES: usize = 5_000_000;
const THREADS: usize = 4;

fn seq() {
    let q = SegQueue::new();

    for i in 0..MESSAGES {
        q.push(message::new(i));
    }

    for _ in 0..MESSAGES {
        q.pop().unwrap();
    }
}

fn spsc() {
    let q = SegQueue::new();

    crossbeam::scope(|scope| {
        scope.spawn(|_| {
            for i in 0..MESSAGES {
                q.push(message::new(i));
            }
        });

        for _ in 0..MESSAGES {
            loop {
                if q.pop().is_none() {
                    thread::yield_now();
                } else {
                    break;
                }
            }
        }
    })
    .unwrap();
}

fn sync() {
    let q1 = SegQueue::new();
    let q2 = SegQueue::new();

    crossbeam::scope(|scope| {
        scope.spawn(|_| {
            assert_eq!(
                unsafe {
                    libc::sched_setscheduler(
                        0,
                        libc::SCHED_FIFO,
                        &libc::sched_param { sched_priority: 1 },
                    )
                },
                0
            );
            for i in 0..MESSAGES {
                q1.push(message::new(i));
                while q2.pop().is_none() {
                    thread::yield_now();
                }
            }
        });

        assert_eq!(
            unsafe {
                libc::sched_setscheduler(
                    0,
                    libc::SCHED_FIFO,
                    &libc::sched_param { sched_priority: 2 },
                )
            },
            0
        );
        for i in 0..MESSAGES {
            while q1.pop().is_none() {
                thread::yield_now();
            }
            q2.push(message::new(i));
        }
    })
    .unwrap();
}

fn mpsc() {
    let q = SegQueue::new();

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    q.push(message::new(i));
                }
            });
        }

        for _ in 0..MESSAGES {
            loop {
                if q.pop().is_none() {
                    thread::yield_now();
                } else {
                    break;
                }
            }
        }
    })
    .unwrap();
}

fn mpmc() {
    let q = SegQueue::new();

    crossbeam::scope(|scope| {
        for _ in 0..THREADS {
            scope.spawn(|_| {
                for i in 0..MESSAGES / THREADS {
                    q.push(message::new(i));
                }
            });
        }

        for _ in 0..THREADS {
            scope.spawn(|_| {
                for _ in 0..MESSAGES / THREADS {
                    loop {
                        if q.pop().is_none() {
                            thread::yield_now();
                        } else {
                            break;
                        }
                    }
                }
            });
        }
    })
    .unwrap();
}

fn main() {
    macro_rules! run {
        ($name:expr, $f:expr) => {
            let now = ::std::time::Instant::now();
            $f;
            let elapsed = now.elapsed();
            println!(
                "{:25} {:15} {:7.3} sec",
                $name,
                "Rust segqueue",
                elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1e9
            );
        };
    }

    //run!("unbounded_mpmc", mpmc());
    //run!("unbounded_mpsc", mpsc());
    //run!("unbounded_seq", seq());
    //run!("unbounded_spsc", spsc());
    run!("unbounded_sync", sync());
}
