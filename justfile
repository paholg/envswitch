run *args: 
    cargo run -- {{args}}

test *args:
    cargo nextest run --no-fail-fast {{args}}

up:
    nix flake update
    cargo upgrade -i

fix:
    cargo clippy --fix --allow-staged

lint: fmt-check clippy

fmt-check:
    cargo fmt --all -- --check

clippy:
    cargo clippy -- -D warnings

release version:
    git diff --exit-code
    cargo set-version {{version}}
    nix flake check
    git add Cargo.toml Cargo.lock
    git commit -m "Version {{version}}"
    git tag v{{version}}
    git push
    git push --tags
    
