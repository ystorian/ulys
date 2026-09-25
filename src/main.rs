// src/main.rs

//! The `ulys` command-line interface.

use std::io::{self, Write};
use std::process::ExitCode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clap::{Args, Parser, Subcommand};
use ulys::Ulys;
use uuid::Uuid;

/// Exit code when a checksum or a time is not valid.
const INVALID: u8 = 1;
/// Exit code when a value is not a ULYS or a UUID, or for an internal problem.
const ERROR: u8 = 2;

#[derive(Parser)]
#[command(name = "ulys", version, about = "Generate and validate ULYS tokens")]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
	#[command(subcommand)]
	command: Option<Command>,

	#[command(flatten)]
	generate: GenerateArgs,
}

#[derive(Subcommand)]
enum Command {
	/// Generate ULYS (default command)
	Generate(GenerateArgs),

	/// Validate ULYS and show their details
	Validate {
		/// ULYS or UUID values
		#[arg(required = true, value_name = "ID")]
		values: Vec<String>,
	},
}

#[derive(Args)]
struct GenerateArgs {
	/// Number of ULYS to generate
	#[arg(short = 'n', long, default_value_t = 1, value_name = "COUNT")]
	count: usize,

	/// Show the ULYS as UUID
	#[arg(short, long)]
	uuid: bool,
}

fn main() -> ExitCode {
	let cli = Cli::parse();
	let mut out = io::stdout().lock();

	let result = match cli.command {
		Some(Command::Generate(args)) => generate(&args, &mut out),
		Some(Command::Validate { values }) => validate(&values, &mut out),
		None => generate(&cli.generate, &mut out),
	};

	match result.and_then(|code| out.flush().map(|()| code)) {
		Ok(code) => ExitCode::from(code),
		// The reader closed the pipe, for `ulys -n 1000 | head`.
		Err(error) if error.kind() == io::ErrorKind::BrokenPipe => ExitCode::SUCCESS,
		Err(error) => {
			eprintln!("ulys: error: {error}");
			ExitCode::from(ERROR)
		}
	}
}

/// Writes `args.count` new ULYS, one on each line.
fn generate(args: &GenerateArgs, out: &mut impl Write) -> io::Result<u8> {
	for _ in 0..args.count {
		let ulys = Ulys::new();
		if args.uuid {
			writeln!(out, "{}", Uuid::from(ulys))?;
		} else {
			writeln!(out, "{ulys}")?;
		}
	}
	Ok(0)
}

/// Writes the details of each value and returns the highest exit code.
fn validate(values: &[String], out: &mut impl Write) -> io::Result<u8> {
	let mut code = 0;
	let mut first = true;
	for value in values {
		let Some(ulys) = parse(value) else {
			eprintln!("ulys: error: not a ULYS or a UUID: {value}");
			code = code.max(ERROR);
			continue;
		};
		if !first {
			writeln!(out)?;
		}
		first = false;
		let problems = problems(ulys);
		if !problems.is_empty() {
			code = code.max(INVALID);
		}
		writeln!(out, "ULYS:   {ulys}")?;
		writeln!(out, "UUID:   {}", Uuid::from(ulys))?;
		if problems.is_empty() {
			writeln!(out, "Valid:  yes")?;
		} else {
			writeln!(out, "Valid:  no ({})", problems.join(", "))?;
		}
		writeln!(out, "UUIDv8: {}", yes_no(ulys.is_uuidv8()))?;
		writeln!(out, "Time:   {}", format_utc(ulys.datetime()))?;
	}
	Ok(code)
}

/// Returns the reasons why the ULYS is not valid.
fn problems(ulys: Ulys) -> Vec<&'static str> {
	let mut problems = Vec::new();
	if !ulys.is_valid() {
		problems.push("bad checksum");
	}
	if !ulys.is_valid_time() {
		if ulys.datetime() < UNIX_EPOCH + Duration::from_millis(Ulys::MIN_TIME_MS) {
			problems.push("time before 2020");
		} else {
			problems.push("time after 2100");
		}
	}
	problems
}

/// Parses a ULYS string, or a UUID in one of the formats of the `uuid` crate.
fn parse(value: &str) -> Option<Ulys> {
	Ulys::from_string(value)
		.ok()
		.or_else(|| Uuid::parse_str(value).ok().map(Ulys::from))
}

fn yes_no(value: bool) -> &'static str {
	if value { "yes" } else { "no" }
}

/// Formats a time as `YYYY-MM-DD HH:MM:SS UTC`.
fn format_utc(time: SystemTime) -> String {
	let seconds = time.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
	let (year, month, day) = civil_from_days(seconds / 86_400);
	let seconds = seconds % 86_400;
	format!(
		"{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
		seconds / 3600,
		seconds % 3600 / 60,
		seconds % 60
	)
}

/// Converts days since 1970-01-01 to a Gregorian date, with the algorithm of Howard Hinnant.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
	let z = days + 719_468;
	let era = z / 146_097;
	let day_of_era = z % 146_097;
	let year_of_era =
		(day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
	let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
	let mp = (5 * day_of_year + 2) / 153;
	let day = day_of_year - (153 * mp + 2) / 5 + 1;
	let month = if mp < 10 { mp + 3 } else { mp - 9 };
	let year = era * 400 + year_of_era + u64::from(month <= 2);
	(year, month, day)
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Makes a ULYS with a correct checksum at the given time, in milliseconds since the Unix
	/// epoch.
	fn ulys_at(ms: u64) -> Ulys {
		let data = u128::from(ms) << 80;
		let hash = xxhash_rust::xxh3::xxh3_64(&data.to_be_bytes());
		Ulys(data | u128::from(hash >> 32))
	}

	fn utc(seconds: u64) -> String {
		format_utc(UNIX_EPOCH + Duration::from_secs(seconds))
	}

	#[test]
	fn test_format_utc() {
		assert_eq!(utc(0), "1970-01-01 00:00:00 UTC");
		assert_eq!(utc(951_782_400), "2000-02-29 00:00:00 UTC");
		assert_eq!(utc(1_000_000_000), "2001-09-09 01:46:40 UTC");
		assert_eq!(utc(4_107_542_399), "2100-02-28 23:59:59 UTC");
		assert_eq!(utc(4_107_542_400), "2100-03-01 00:00:00 UTC");
	}

	#[test]
	fn test_time_limits() {
		assert_eq!(utc(Ulys::MIN_TIME_MS / 1000), "2020-01-01 00:00:00 UTC");
		assert_eq!(utc(Ulys::MAX_TIME_MS / 1000), "2101-01-01 00:00:00 UTC");
	}

	#[test]
	fn test_problems() {
		let valid = Ulys::from_string("068dkwmn3a441g20mzbsmyk5b8").unwrap();
		let invalid = Ulys::from_string("068dkwmn3a441g20mzbsmy0000").unwrap();

		assert!(problems(valid).is_empty());
		assert!(problems(Ulys::new()).is_empty());
		assert_eq!(problems(invalid), ["bad checksum"]);
		assert!(problems(ulys_at(Ulys::MIN_TIME_MS)).is_empty());
		assert!(problems(ulys_at(Ulys::MAX_TIME_MS - 1)).is_empty());
		assert_eq!(
			problems(ulys_at(Ulys::MIN_TIME_MS - 1)),
			["time before 2020"]
		);
		assert_eq!(problems(ulys_at(Ulys::MAX_TIME_MS)), ["time after 2100"]);
		assert_eq!(problems(Ulys(1)), ["bad checksum", "time before 2020"]);
	}

	#[test]
	fn test_parse() {
		let ulys = Ulys::new();
		let uuid = Uuid::from(ulys);

		assert_eq!(parse(&ulys.to_string()), Some(ulys));
		assert_eq!(parse(&uuid.to_string()), Some(ulys));
		assert_eq!(parse(&uuid.simple().to_string()), Some(ulys));
		assert_eq!(parse("not-an-id"), None);
	}

	#[test]
	fn test_validate_exit_code() {
		let valid = "068dkwmn3a441g20mzbsmyk5b8".to_string();
		let invalid = "068dkwmn3a441g20mzbsmy0000".to_string();
		let mut out = Vec::new();

		assert_eq!(validate(std::slice::from_ref(&valid), &mut out).unwrap(), 0);
		assert_eq!(
			validate(&[valid.clone(), invalid], &mut out).unwrap(),
			INVALID
		);
		assert_eq!(validate(&[valid, "x".into()], &mut out).unwrap(), ERROR);
	}
}
