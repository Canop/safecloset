
The current version of SafeCloset works on linux, Mac, and Windows.

# From source

You'll need to have the [Rust development environment](https://www.rustup.rs) installed and up to date.

Fetch the [Canop/safecloset](https://github.com/Canop/safecloset) repository, move to the safecloset directory, then run

```bash
cargo install --path .
```

If you want to enable the clipboard feature (to copy to SafeCloset), do

```bash
cargo install --features clipboard --path .
```

# From precompiled binaries

Binaries are made available at every release on [GitHub](https://github.com/Canop/safecloset/releases).

When you download executable files, you'll have to ensure the shell can find them. An easy solution is to put them in `/usr/local/bin`. You may also have to set them executable using `chmod +x safecloset`.

# From crates.io

You'll need to have the [Rust development environment](https://www.rustup.rs) installed and up to date.

Once it's installed, use cargo to install safecloset:

    cargo install safecloset

If you want to enable the clipboard feature (to copy to SafeCloset), do

    cargo install safecloset --features clipboard
