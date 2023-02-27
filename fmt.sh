# As some fmt rules are still only available in nightly, cargo fmt
# can't be ran from stable
rustup default nightly
cargo fmt
rustup default stable
