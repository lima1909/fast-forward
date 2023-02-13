# default, list all just Recipe
default: 
  @just -q --list

alias t := test
alias l := clippy

# run all tests with all-features
test filter="":
  @cargo test --all-features {{filter}}

# cargo watch for test with given filter
watch filter="":
  @cargo watch -q -c -x 'test {{filter}}'

# run cargo test and clippy
clippy: test
  @cargo clippy --tests --workspace -- -D warnings