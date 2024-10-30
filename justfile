default:
    @just --list

# Auto-format the source tree
fmt:
    treefmt

# Run 'cargo run' on the project
run *ARGS:
    cargo run {{ARGS}}

# Run 'cargo watch' to run the project (auto-recompiles)
watch *ARGS:
    cargo watch -x "run -- {{ARGS}}"

lint:
  cargo clippy --release -p torus-renderer --all-targets --all-features -- --deny warnings

bundle:
  nix bundle --bundler github:NixOS/bundlers#toDEB .#default
