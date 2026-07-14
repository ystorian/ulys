#  .just/github.just

# Requirements:
# zizmor
# - `cargo install --locked zizmor`
# pinact:
# - macOS: `brew install pinact`
# - Windows: `winget install suzuki-shunsuke.pinact`


# Audit GitHub Actions with `zizmor`
[group('GitHub Actions')]
audit_actions:
	echo {{CYAN}}Audit actions in {{GREEN}}.github/workflows/*.yaml{{NORMAL}}
	zizmor --version
	zizmor --persona=auditor .
	echo {{done('')}}


# Update and pin GitHub Actions versions with `pinact` with cooldown
[group('GitHub Actions')]
update_actions:
	echo {{CYAN}}Update and pin actions in {{YELLOW}}.github/workflows/*.yaml{{NORMAL}}
	pinact --version
	pinact run --update --verify-comment --min-age 7
	echo {{done('')}}


# Update and pin GitHub Actions versions with `pinact`
[group('GitHub Actions')]
[private]
update_actions_latest:
	echo {{CYAN}}Update and pin actions in {{YELLOW}}.github/workflows/*.yaml{{NORMAL}}
	pinact --version
	pinact run --update --verify-comment
	echo {{done('')}}


# Check GitHub Actions versions with `pinact`
[group('GitHub Actions')]
check_actions:
	echo {{CYAN}}Check actions in {{YELLOW}}.github/workflows/*.yaml{{NORMAL}}
	pinact --version
	pinact run --verify-comment
	echo {{done('')}}
