# .just/rust.just

# Benchmark
[group('Rust')]
[unix]
bench:
	echo 🦀 {{CYAN}}cargo bench{{NORMAL}}
	cargo bench --all-features

# Benchmark
[group('Rust')]
[windows]
bench:
	echo 🦀 {{CYAN}}cargo bench{{NORMAL}}
	call scripts/vcvarsall.cmd {{arch}}_{{arch}} && \
	cargo bench --all-features


# Check for errors
[group('rust')]
check:
	echo 🦀 {{CYAN}}cargo check{{NORMAL}}
	cargo check --all-features


# Check dependencies and licenses
[group('rust')]
deny:
	echo 🦀 {{CYAN}}cargo-deny{{NORMAL}}
	cargo-deny --all-features --log-level error check


# Format Rust code
[group('rust')]
fmt:
	echo 🦀 {{CYAN}} cargo fmt{{NORMAL}}
	cargo fmt --all


# Clippy
[group('rust')]
lint:
	echo 🦀 {{CYAN}}cargo clippy {{YELLOW + ITALIC}}pedantic{{NORMAL}}
	cargo clippy --all-targets --all-features --all -- --deny clippy::all --warn clippy::pedantic
alias clippy := lint


# Clippy limited to the specified lint
[group('rust')]
lintonly clippy_lint:
	cargo clippy --all-targets --all-features --all -- --warn clippy::{{clippy_lint}}


# Fix easy Clippy lints (requires no uncommitted changes)
[group('rust')]
fix_clippy:
	echo 🦀 {{CYAN}}Fixing easy Clippy lints...{{NORMAL}}
	echo ⚠️ {{YELLOW}}Commit all your changes first{{NORMAL}}
	cargo clippy --fix


# Run unit and integration tests with nextest
[group('rust')]
test:
	cargo test --all-features


# Run the CI tests like the GitHub action in `rust.yaml`
[group('rust')]
[env("CARGO_TERM_QUIET", "true")]
[env("CARGO_INCREMENTAL", "0")]
[env("CARGO_PROFILE_TEST_DEBUG", "0")]
test_ci:
	echo 🦀 {{CYAN}}cargo-deny{{NORMAL}}
	cargo-deny --all-features --log-level error check
	echo 🦀 {{CYAN}}cargo fmt{{NORMAL}}
	cargo fmt --all --check
	echo 🦀 {{CYAN}}cargo test{{NORMAL}}
	cargo test --frozen --profile ci --all-features
	echo 🦀 {{CYAN}}cargo clippy{{NORMAL}}
	cargo clippy --frozen --profile ci --all-features --tests -- -D warnings
	echo ✅ {{GREEN}}test_ci{{NORMAL}}


# Update dependencies
[group('setup')]
update_deps:
	echo ⚙️ {{CYAN}} Updating dependencies in {{GREEN}}Cargo.toml{{NORMAL}}
	cargo update --verbose
	just deny
	echo ✅ {{GREEN}}Done{{NORMAL}}
