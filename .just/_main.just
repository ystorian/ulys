# .just/_main.just

# Common recipes.
import 'git.just'
import 'github.just'
import 'utils.just'

# Language recipes.
import 'cargo.just'
import 'rust.just'

# Project recipes.
import 'ci.just'

# Project variables.

# Additional Cargo tools required by this project (see `.just/cargo.just`).
additional_cargo_tools := ['zizmor']

# Setup this project (tooling, hooks)
[group('_')]
setup:
	just no_index

	# Rust.
	just update_rust
	just update_cargo_tools

# Bump all dependencies
[group('_')]
bump:
	just lint_just
	just bump_cargo
	just fetch
	just update_actions
	just audit_actions

# Clean
clean:
	just clean_workspace
