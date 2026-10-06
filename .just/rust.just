# @template/.just/rust.just

# Get the app version from `Cargo.toml`, with or without workspace.
version := trim(replace(`tomli query --filepath Cargo.toml workspace.package.version 2>/dev/null ||
	tomli query --filepath Cargo.toml package.version`, '"', ''))

# Release tag `v#.#.#`.
tag := 'v' + version

# Display "in quiet mode" for non-release profiles.
display_profile(profile) := if profile != 'release' { NORMAL + YELLOW + ' ' + profile + style('dim') + ' in quiet mode' + set_title('🦀 ' + recipe_name()) + NORMAL } else { NORMAL + GREEN + ' release ' + set_title('🦀 ' + recipe_name()) + NORMAL }

# Check the Visual Studio build tools environment variables
[no-exit-message]
[private]
[windows]
check_env:
	if ! [ -z "$(printenv VCINSTALLDIR)" ]; then \
		echo "{{ error(['The environment variable', BOLD + 'VCINSTALLDIR', 'is set']) }}"; \
		exit 1; \
	fi

# Check the Visual Studio build tools environment variables
[private]
[unix]
check_env:
	exit 0

# Benchmark
[group('Rust')]
bench profile='dev': check_env
	echo "{{ CYAN }}Run {{ BOLD }}cargo bench{{ display_profile(profile) }}"
	cargo bench --all-features --workspace \
		{{ if profile != 'release' { '--profile ' + profile + ' --quiet' } }}
	echo "{{ set_title('') + done(profile) }}"

# Check for errors
[group('Rust')]
check profile='dev': check_env
	echo "{{ CYAN }}Run {{ BOLD }}cargo check{{ display_profile(profile) }}"
	cargo check --workspace --all-features --future-incompat-report \
		{{ if profile != 'release' { '--profile ' + profile + ' --quiet' } else { '--release' } }}
	echo "{{ set_title('') + done(profile) }}"

# Check dependencies and licenses
[group('Rust')]
deny:
	echo {{ CYAN }}Run {{ BOLD }}cargo-deny{{ NORMAL }}

	# Remove corrupted cache.
	cache="${TMPDIR:-${TEMP:-/tmp}}/cargo-deny/std-replacement-data"; \
	if [ -d "$cache" ] && { [ ! -f "$cache/.git/HEAD" ] || [ ! -f "$cache/all.bin" ]; }; then \
		just delete_dir "$cache"; \
	fi

	cargo-deny --all-features --workspace --log-level error check

	echo {{ done('') }}

# Format Rust code
[group('Rust')]
format:
	echo {{ CYAN }}Run {{ BOLD }}cargo fmt{{ NORMAL }}
	cargo fmt --all
	echo {{ done('') }}
alias fmt := format

# Clippy
[group('Rust')]
clippy: check_env
	echo "{{ CYAN }}Run {{ BOLD }}cargo clippy{{ NORMAL }}"
	cargo clippy --all-targets --all-features --all -- --deny clippy::all --warn clippy::pedantic
	echo {{ done('') }}
alias lint := clippy

# Clippy with only `lint`
[env("CARGO_TERM_QUIET", "true")]
[group('Rust')]
clippy_lint lint: check_env
	echo {{ CYAN }}Run {{ BOLD }}cargo clippy{{ NORMAL + CYAN }} for the \
		{{ YELLOW + lint + CYAN }} lint{{ NORMAL }}
	cargo clippy --all-targets --all-features --all -- --warn clippy::{{ lint }}
	echo {{ done(lint) }}

# Fix easy Clippy lints (requires no uncommitted changes)
[env("CARGO_TERM_QUIET", "true")]
[group('Rust')]
clippy_fix: check_env
	echo {{ CYAN }}Fix {{ ITALIC }}easy{{ NORMAL + CYAN }} Clippy lints{{ NORMAL }}
	echo ⚠️ {{ YELLOW }}Commit all your changes first{{ NORMAL }}
	cargo clippy --fix
	echo {{ done('') }}

# Run unit and integration tests
[group('Rust')]
[no-exit-message]
test profile='dev': check_env
	echo "{{ CYAN }}Run {{ BOLD }}cargo test{{ display_profile(profile) }}"

	# Notes:
	# - `kache test-runner` version 1.00 errors with "no LC_RPATH's found".
	# - We bypass `kache` with `env -u CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER`.

	if [ "{{ profile }}" != "release" ]; then \
		if ! env -u CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER \
				cargo test --all-features --workspace --future-incompat-report --quiet \
				--profile {{ profile }} >/dev/null 2>&1; then \
			env -u CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER \
				cargo test --all-features --workspace --future-incompat-report \
				--profile {{ profile }}; \
		fi; \
	else \
		env -u CARGO_TARGET_AARCH64_APPLE_DARWIN_RUNNER \
			cargo test --all-features --workspace --future-incompat-report --release; \
	fi
	echo "{{ set_title('') + done(profile) }}"

# Clean the workspace
[group('Rust')]
clean_workspace:
	echo {{ CYAN }}Clean the workspace{{ NORMAL }}

	cargo-clean-all \
		--keep-empty-target \
		--keep-executable-same-dir \
		--yes \
		.
	just deny

	echo {{ done('') }}

# Build and install on this machine
[group('Rust')]
install: check_env
	echo {{ CYAN }}Run {{ BOLD }}cargo install{{ NORMAL }}
	cargo install --path .
	echo {{ done('') }}

# Download the crates in `Cargo.toml`
[group('Rust')]
fetch:
	echo {{ CYAN }}Run {{ BOLD }}cargo fetch{{ NORMAL }}
	cargo fetch --locked --verbose
	echo {{ done('') }}
