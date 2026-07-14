# justfile

# Set the default shell on Windows to `bash` (installed with Git).
set windows-shell := ['C:\Program Files\Git\bin\bash.exe', '-cu']

# Don't print comments.
set ignore-comments

# Allow variable redefinition.
set allow-duplicate-variables

# Be quiet.
set quiet

# New features.
set unstable
set lists

# Display functions.
done (args):= ('"' +
	GREEN + '✅ ' + style('dim') + datetime('%F %T%.3f ') + NORMAL +
	GREEN + BOLD + recipe_name() + NORMAL +
	GREEN + ITALIC + ' ' + join_list(args) + NORMAL + '\n"'
)

# Import justfiles from the `.just/` directory.
import '.just/ci.just'
import '.just/rust.just'
import '.just/cargo.just'
import '.just/github.just'


# List the commands when called without parameters.
_:
	just --list
