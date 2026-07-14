# .just/rust.just


# Benchmark
[group('Rust')]
bench:
	echo {{CYAN}}cargo bench{{NORMAL}}
	cargo bench --all-features --workspace
	echo {{done('')}}


# Check for errors
[group('Rust')]
check:
	echo {{CYAN}}cargo check{{NORMAL}}
	cargo check --workspace --all-features
	echo {{done('')}}


# Check dependencies and licenses
[group('Rust')]
deny:
	echo {{CYAN}}cargo-deny{{NORMAL}}
	cargo-deny --all-features --workspace --log-level error check
	echo {{done('')}}


# Format Rust code
[group('Rust')]
format:
	echo {{CYAN}}cargo fmt{{NORMAL}}
	cargo fmt --all
	echo {{done('')}}
alias fmt := format


# Clippy
[group('Rust')]
clippy:
	echo {{CYAN}}cargo clippy {{YELLOW + ITALIC}}pedantic{{NORMAL}}
	cargo clippy --all-targets --all-features --all -- --deny clippy::all --warn clippy::pedantic
	echo {{done('')}}
alias lint := clippy


# Run Clippy limited to the specified lint
[group('Rust')]
clippy_lint lint:
	cargo clippy --all-targets --all-features --all -- --warn clippy::{{lint}}
	echo {{done(lint)}}


# Fix easy Clippy lints (requires no uncommitted changes)
[group('Rust')]
clippy_fix:
	echo {{CYAN}}Fix easy Clippy lints{{NORMAL}}
	echo ⚠️ {{YELLOW}}Commit all your changes first{{NORMAL}}
	cargo clippy --fix
	echo {{done('')}}


# Run unit and integration tests
[group('Rust')]
test:
	cargo test --all-features --workspace
	echo {{done('')}}


# Clean the workspace
[group('Rust')]
clean_workspace:
	echo {{CYAN}}Clean the workspace{{NORMAL}}
	cargo-clean-all --yes .
	cargo check --all-features
	cargo-deny --all-features --log-level error check
	echo {{done('')}}


# Build in release mode and install locally
[group('Rust')]
install:
	echo {{CYAN}}cargo install{{NORMAL}}
	cargo install --path .
	echo {{done('')}}
