git := env_var_or_default("GIT", "git")
rustc := env_var_or_default("RUSTC", "rustc")
cargo := env_var_or_default("CARGO", "cargo")
cargo_watch := env_var_or_default("CARGO_WATCH", "cargo-watch")

just := env_var_or_default("JUST", just_executable())

root_dir := invocation_directory()
package := env_var_or_default("PKG", "unset")

version := `cargo get version | head --bytes=-1`
sha := `git rev-parse --short HEAD`

_default:
  {{just}} --list

#############
# Utilities #
#############

# Print the current version
print-version:
    @echo -n "{{version}}"

# Print the current SHA
print-sha:
    @echo -n "{{sha}}"

# Ensure a binary is present
ensure-binary bin env_name:
    #!/usr/bin/env -S bash -euo pipefail
    if [ -z "$(command -v {{bin}})" ]; then
      echo "Missing binary [{{bin}}], make sure it is installed & on your PATH (or override it's location with {{env_name}})";
      echo "(if the binary is not on your system, you may need to install the package via cargo)";
      exit 1;
    fi

###########
# Project #
###########

# Set up the project
setup:
    @{{just}} ensure-binary rustc RUSTC
    @{{just}} ensure-binary cargo CARGO

# Format
fmt:
    {{cargo}} fmt

# Lint
lint:
    {{cargo}} clippy

# Lint the project
lint-watch:
    @{{just}} ensure-binary cargo-watch CARGO_WATCH
    @{{cargo}} watch --watch=src --shell 'just lint'

# Build
build:
    {{cargo}} build

# Build continuously (development mode)
build-watch:
    @{{just}} ensure-binary cargo-watch CARGO_WATCH
    {{cargo}} watch -x build

# Build the release version of the binary
build-release:
    @{{cargo}} build --release

# Check the project
check:
    @{{cargo}} check

# Ensure that the # of commits is what we expect
check-commit-count now before count:
    @export COUNT=$(($(git rev-list --count {{now}} --no-merges) - $(git rev-list --count {{before}}))) && \
    if [ "$COUNT" != "1" ]; then \
      echo -e "[error] number of commits ($COUNT) is *not* {{count}} -- please squash commits"; \
      exit 1; \
    fi

########
# Test #
########

test_focus := env_var_or_default("TEST", "")

test: test-unit test-int test-examples

# Run unit tests
test-unit:
    @{{just}} ensure-binary cargo-nextest CARGO_NEXTEST
    @{{cargo}} nextest run -E 'kind(lib)'
    @{{cargo}} nextest run -F async-std -E 'kind(lib)'

# Run unit tests continuously
test-unit-watch:
    @{{just}} ensure-binary cargo-watch CARGO_WATCH
    @{{just}} ensure-binary cargo-nextest CARGO_NEXTEST
    @{{cargo}} watch -- {{cargo}} nextest run {{test_focus}}

test-int:
    @{{just}} ensure-binary cargo-nextest CARGO_NEXTEST
    @{{cargo}} nextest run -E 'kind(test)'

test-examples:
    @{{cargo}} run --example async-drop-simple --features=tokio
    @{{cargo}} run --example async-drop-async-std --features=async-std

######################
# Release Management #
######################

publish_crate := env_var_or_default("PUBLISH_CRATE", "no")
changelog_file_path := env_var_or_default("CHANGELOG_FILE_PATH", "CHANGELOG")

# Generate the changelog
changelog:
  {{git}} cliff --unreleased --tag={{version}} --prepend={{changelog_file_path}}

# Generic release automation
release version:
    @if [ "{{package}}" == "unset" ]; then \
      echo "[error] Cannot release all packages at once, ENV var PKG must be set"; \
      exit 1; \
    fi
    cd ./crates/{{package}} && {{just}} release {{version}}
