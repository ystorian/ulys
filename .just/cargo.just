# .just/cargo.just


# Variables.
# cargo binstall.
need_binstall := if which("cargo-binstall") == [] {'true'} else {'false'}
binstall_options := '--locked --no-confirm --only-signed --disable-telemetry --no-discover-github-token'
native_target := if os() == 'windows' {arch() + '-pc-windows-msvc'} else if os() == 'macos' {arch() + '-apple-darwin'}

# Architecture.
arch := if arch() == "x86_64" { "x64" } else if arch() == "aarch64" { "arm64" } else { arch() }


# Install cargo tools
[group('Cargo')]
install_cargo:
	echo {{CYAN}}Install cargo tools{{NORMAL}}

	if {{need_binstall}} ; then \
		echo {{BLUE}}Install {{GREEN}}cargo-binstall{{NORMAL}} && \
		cargo install --locked --targets {{native_target}} cargo-binstall; fi

	echo {{CYAN}}Install the required cargo tools{{NORMAL}}
	cargo-binstall \
		{{binstall_options}} \
		--targets {{native_target}} \
		cargo-update \
		just \
		cargo-clean-all \
		cargo-deny \
		zizmor

	echo {{CYAN}}Update all cargo tools{{NORMAL}}
	cargo install-update --all
	echo {{done('')}}


# Update all
[group('Cargo')]
update_all:
	echo {{BLUE}}Update all{{NORMAL}}
	just update_rust
	just update_cargo
	just bump_deps
	echo {{done('')}}


# Update Rust
[group('Cargo')]
update_rust:
	echo {{CYAN}}Update Rust{{NORMAL}}
	rustup self update
	rustup update stable
	rustup component add rust-analyzer
	echo {{done('')}}


# Update cargo tools
[group('Cargo')]
update_cargo:
	echo {{CYAN}}Update cargo tools{{NORMAL}}
	cargo install-update --all --git --locked --cooldown 7d
	echo {{done('')}}


# Update dependencies
[group('Cargo')]
bump_deps:
	echo {{CYAN}}Update dependencies in {{GREEN}}Cargo.toml{{NORMAL}}
	cargo update --verbose
	just deny
	just update_actions
	echo {{done('')}}


# List cargo tools installed with the correct target arch
[group('Cargo')]
[unix]
cargo_native:
	echo {{CYAN}}List native {{GREEN + arch + CYAN}} cargo tools{{NORMAL}}
	find $CARGO_HOME/bin -type f | xargs file | grep -v "for architecture" | grep "{{arch}}"
	echo {{done('')}}


# List cargo tools that must be reinstalled with the native architecture
[group('Cargo')]
[unix]
cargo_non_native:
	echo {{CYAN}}List {{RED}}non-{{arch + CYAN}} cargo tools{{NORMAL}}
	-find $CARGO_HOME/bin -type f | xargs file | grep -v "for architecture" | grep -v "{{arch}}"
	echo {{YELLOW}}Reinstall them with {{ITALIC}}just cargo_reinstall{{NORMAL}}
	echo {{done('')}}


# Force-reinstall a cargo tool with the native architecture
[group('Cargo')]
cargo_reinstall tool:
	echo {{CYAN}}Force-reinstall {{YELLOW + tool + CYAN}} for {{GREEN + arch + NORMAL}}
	cargo binstall {{binstall_options}} --targets {{native_target}} --force {{tool}}
	echo {{done(tool)}}
