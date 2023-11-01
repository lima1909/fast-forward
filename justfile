# default, list all just Recipe
default: 
  @just -q --list

alias t := test
alias l := clippy

set export

# cargo get: https://crates.io/crates/cargo-get
ff_version := `cargo get --entry="./fast_forward/Cargo.toml" package.version --pretty`
ff_version_msg := "New release of fast-forward version " + ff_version


# run all tests with all-features
test filter="":
  @cargo test --all-features {{filter}}

first:
  cargo test -- ui trybuild=ui_first.rs

# cargo watch for test with given filter
watch filter="":
  @cargo watch -q -c -x 'test {{filter}}'

# run cargo test and clippy
clippy: test
  @cargo clippy --tests --workspace -- -D warnings

# run cargo doc --no-deps
doc:
  @cargo doc --no-deps

# tag:
#  @git tag -a v0.0.3 -m "New release of fast-forward version 0.0.3"
#  @git push origin --tags
  

release_new_version:
  @git tag -a $ff_version -m $ff_version_msg
  @git push origin --tags
  @just --justfile ./fast_forward/justfile publish
