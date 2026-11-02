export PATH := "/opt/homebrew/opt/rustup/bin:" + env_var('PATH')

default:
    @just --list

fmt:
    cargo fmt --all
    typstyle -i report slides

lint-source:
    cargo run --quiet -p xtask -- lint-source

check-fast:
    cargo fmt --all --check
    cargo clippy --all-targets --locked -- -D warnings
    cargo run --quiet -p xtask -- lint-source
    cargo nextest run --locked

check: check-fast
    cargo test --doc --locked
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --locked
    typos

report:
    mkdir -p report/out
    typst compile --root . report/main.typ report/out/main.pdf

report-format-check:
    cargo run --quiet -p xtask -- report-format-check report/fixtures/format-sample.typ

report-check:
    cargo run --quiet -p xtask -- report-check report/main.typ
    just lint-text

lint-text:
    vale report/chapters

mutants:
    git diff main > target/mutants.diff
    cargo mutants --in-diff target/mutants.diff

deny:
    cargo deny check advisories sources licenses
