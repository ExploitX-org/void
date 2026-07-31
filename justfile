version := `grep '^version' "$(bash __dirname)Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/'`

default: run

build: _write-version
  cargo build

run: _write-version
  cargo generate-lockfile && nix run

develop:
  nix develop

clippy:
  cargo clippy --all-targets

_write-version:
  printf '%s' {{version}} > "$(bash __dirname)version"
