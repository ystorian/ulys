use bencher::{benchmark_group, benchmark_main, Bencher};
use std::time::SystemTime;
use ulys::{Generator, Ulys, ULYS_LEN};

fn bench_new(b: &mut Bencher) {
    b.iter(Ulys::new);
}

fn bench_generator_generate(b: &mut Bencher) {
    let mut gen = Generator::new();
    b.iter(|| gen.generate().unwrap());
}

fn bench_from_time(b: &mut Bencher) {
    let time = SystemTime::now();
    b.iter(|| Ulys::from_datetime(time));
}

fn bench_to_str(b: &mut Bencher) {
    let ulys = Ulys::new();
    b.iter(|| {
        let mut buffer = [0; ULYS_LEN];
        ulys.array_to_str(&mut buffer);
    });
}

fn bench_to_string(b: &mut Bencher) {
    let ulys = Ulys::new();
    b.iter(|| ulys.to_string());
}

fn bench_from_string(b: &mut Bencher) {
    let s = Ulys::new().to_string();
    b.iter(|| Ulys::from_string(&s).unwrap());
}

benchmark_group!(
    ulys_perf,
    bench_new,
    bench_generator_generate,
    bench_from_time,
    bench_to_str,
    bench_to_string,
    bench_from_string
);

benchmark_main!(ulys_perf);
