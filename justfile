run *args: 
    cargo run -- {{args}}

test *args:
    cargo nextest run {{args}}

release version:
    git diff --exit-code
    cargo set-version {{version}}
    cargo test
    git add Cargo.toml Cargo.lock
    git commit -m "Version {{version}}"
    git tag v{{version}}
    git push
    git push --tags
    
