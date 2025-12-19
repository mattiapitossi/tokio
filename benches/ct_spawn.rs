use gungraun::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;
use tokio::runtime::{self, Runtime};

fn single_rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
}

fn multi_rt() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap()
}

fn rt() -> Runtime {
    runtime::Builder::new_multi_thread()
        .worker_threads(NUM_WORKERS)
        .enable_all()
        .build()
        .unwrap()
}

async fn work() -> usize {
    let val = 1 + 1;
    tokio::task::yield_now().await;
    black_box(val)
}

fn spawn(runtime: Runtime) {
    runtime.block_on(async {
        let h = tokio::spawn(work());
        assert_eq!(h.await.unwrap(), 2);
    });
}

fn spawn100(runtime: Runtime) {
    runtime.block_on(async {
        let mut handles = Vec::with_capacity(100);
        for _ in 0..100 {
            handles.push(tokio::spawn(work()));
        }
        for handle in handles {
            assert_eq!(handle.await.unwrap(), 2);
        }
    });
}
const NUM_WORKERS: usize = 5;
const NUM_SPAWN: usize = 10_000;

fn rt_multi_spawn_many_remote_idle(rt: Runtime) {
    let mut handles = Vec::with_capacity(NUM_SPAWN);

    for _ in 0..100 {
        for _ in 0..NUM_SPAWN {
            handles.push(rt.spawn(async {}));
        }

        rt.block_on(async {
            for handle in handles.drain(..) {
                handle.await.unwrap();
            }
        });
    }
}

#[library_benchmark]
#[bench::basic(setup = single_rt)]
#[bench::threaded(setup = multi_rt)]
fn spawn_rt(runtime: Runtime) {
    black_box(spawn(runtime));
}

#[library_benchmark]
#[bench::basic(setup = single_rt)]
#[bench::threaded(setup = multi_rt)]
fn spawn_rt_100(runtime: Runtime) {
    black_box(spawn100(runtime));
}

#[library_benchmark]
#[bench::threaded(setup = rt)]
fn multi_spawn(runtime: Runtime) {
    black_box(rt_multi_spawn_many_remote_idle(runtime));
}

library_benchmark_group!(
    name = group;
    benchmarks = spawn_rt,spawn_rt_100,multi_spawn
);

main!(library_benchmark_groups = group);
