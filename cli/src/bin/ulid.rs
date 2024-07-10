extern crate structopt;

use std::io::{self, Write};
use ulys::{Generator, Ulys};

use std::{thread, time::Duration};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    /// Number of ULYSes to generate
    #[structopt(short = "n", long = "count", default_value = "1")]
    count: u32,
    #[structopt(short = "m", long = "monotonic")]
    monotonic: bool,
    /// ULYSes for inspection
    #[structopt(conflicts_with = "count")]
    ulyses: Vec<String>,
}

fn main() {
    let opt = Opt::from_args();

    if !opt.ulyses.is_empty() {
        inspect(&opt.ulyses);
    } else {
        generate(opt.count, opt.monotonic);
    }
}

fn generate(count: u32, monotonic: bool) {
    let stdout = io::stdout();
    let stderr = io::stderr();
    let mut locked = stdout.lock();
    let mut err_locked = stderr.lock();
    if monotonic {
        let mut gen = Generator::new();
        let mut i = 0;
        while i < count {
            match gen.generate() {
                Ok(ulys) => {
                    writeln!(&mut locked, "{}", ulys).unwrap();
                    i += 1;
                }
                Err(_) => {
                    writeln!(
                        &mut err_locked,
                        "Failed to create new ulys due to overflow, sleeping 1 ms"
                    )
                    .unwrap();
                    thread::sleep(Duration::from_millis(1));
                    // do not increment i
                }
            }
        }
    } else {
        for _ in 0..count {
            writeln!(&mut locked, "{}", Ulys::new()).unwrap();
        }
    }
}

fn inspect(values: &[String]) {
    for val in values {
        let ulys = Ulys::from_string(val);
        match ulys {
            Ok(ulys) => {
                let upper_hex = format!("{:X}", ulys.0);
                println!(
                    "
REPRESENTATION:

  String: {}
     Raw: {}

COMPONENTS:

       Time: {}
  Timestamp: {}
    Payload: {}
",
                    ulys.to_string(),
                    upper_hex,
                    time::OffsetDateTime::from(ulys.datetime()),
                    ulys.timestamp_ms(),
                    upper_hex.chars().skip(6).collect::<String>()
                );
            }
            Err(e) => {
                println!("{} is not a valid ULYS: {}", val, e);
            }
        }
    }
}
