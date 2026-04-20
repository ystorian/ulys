# justfile

# Don't print comments.
set ignore-comments := true

# Allow variable redefinition.
set allow-duplicate-variables

# Be quiet.
set quiet

# Import justfiles from the `.just/` directory.
import '.just/rust.just'

# List the commands when called without parameters.
_:
	just --list
