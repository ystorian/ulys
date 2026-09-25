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
setup:
	# Make it fast.
	just no_index

	# Git pre-commit hook.
	just install_git_hooks

	# Rust.
	just update_rust
	just update_cargo_tools

# Bump all dependancies
bump:
	# Just
	just lint_just

	# Rust.
	just bump_cargo
	just fetch

	# GitHub Actions.
	just update_actions
	just audit_actions

# Clean
clean:
	# Rust.
	just clean_workspace
