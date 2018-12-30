#[macro_use]
extern crate bencher;
extern crate chrono;
extern crate ulid;

use bencher::Bencher;
use chrono::Utc;
use ulid::Ulid;

fn bench_new(b: &mut Bencher) {
    b.iter(|| Ulid::new());
}

fn bench_from_time(b: &mut Bencher) {
    let time = Utc::now();
    b.iter(|| Ulid::from_datetime(time));
}

fn bench_to_string(b: &mut Bencher) {
    let ulid = Ulid::new();
    b.iter(|| ulid.to_string());
}

fn bench_from_string(b: &mut Bencher) {
    let s = Ulid::new().to_string();
    b.iter(|| Ulid::from_string(&s).unwrap());
}

benchmark_group!(
    ulid_perf,
    bench_new,
    bench_from_time,
    bench_to_string,
    bench_from_string
);

benchmark_main!(ulid_perf);