# default, list all just Recipe
default: 
  @just -q --list

alias t := test
alias l := clippy

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

tag:
  @git tag -a v0.0.2 -m "second release of fast-forward with version 0.0.2"
  @git push origin --tags