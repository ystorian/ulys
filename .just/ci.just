# .just/ci.just


# Run the CI tests like in `.github/workflows/ci-rust.yaml`
[group('CI')]
[env("CARGO_TERM_QUIET", "true")]
[env("CARGO_INCREMENTAL", "0")]
[env("CARGO_PROFILE_TEST_DEBUG", "0")]
ci:
	echo {{CYAN}}Running {{GREEN}}CI checks{{CYAN}} in silent mode{{NORMAL}}

	echo {{YELLOW}}cargo fmt{{NORMAL}}
	cargo fmt --all

	echo {{YELLOW}}cargo-deny{{NORMAL}}
	cargo-deny --all-features --workspace --log-level error check

	echo {{YELLOW}}cargo test{{NORMAL}}
	cargo test --frozen --profile ci --all-features --workspace

	echo {{YELLOW}}cargo clippy{{NORMAL}}
	cargo clippy --frozen --profile ci --all-features --workspace --tests -- -D warnings

	echo {{done('')}}
