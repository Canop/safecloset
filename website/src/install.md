
The current version of SafeCloset works on linux, Mac, Windows, and Android (over Termux).

# From source

You'll need to have the [Rust development environment](https://www.rustup.rs) installed and up to date.

Fetch the [Canop/safecloset](https://github.com/Canop/safecloset) repository, move to the safecloset directory, then run

```bash
cargo install --locked --path .
```

# From precompiled binaries

Binaries are made available at every release on [GitHub](https://github.com/Canop/safecloset/releases).

When you download executable files, you'll have to ensure the shell can find them. An easy solution is to put them in `/usr/local/bin`.

You may also have to set them executable on linux using `chmod +x safecloset`.

# From crates.io

You'll need to have the [Rust development environment](https://www.rustup.rs) installed and up to date.

Once it's installed, use cargo to install safecloset:

    cargo install --locked safecloset

# FAQ

*(if you encountered a problem and solved it, please tell me so that we can help other users)*

## Why --locked

The `--locked` argument forces the dependencies to be in the same version than when I released.
This protects against some possible attacks on the dependency chain.

## Compilation failed

Most often, this is due to a not up to date compiler.

You should update your Rust installation.
This is usually done with `rustup update`.

## Copy-Paste problem on Windows

* If you are using `cmd.exe` or the native PowerShell command line, remember to use <kbd>ctrl</kbd><kbd>v</kbd>. Don't use the the system menu Edit/Paste from the window's top-left icon.

* If you are a Windows Terminal user, you need to change its built-in pasting shortcut. Newer versions of Windows Terminal uses <kbd>ctrl</kbd><kbd>v</kbd> for pasting. You would find a line similar to `{ "command": "paste", "keys": "ctrl+v" },` in its configuration file. This will interfere with SafeCloset's handing of <kbd>ctrl</kbd><kbd>v</kbd>. You have to change Windows Terminal's pasting shortcut to something else (not <kbd>ctrl</kbd><kbd>v</kbd>) to make SafeCloset's multiline pasting work. For example, you can use the setting `{ "command": "paste", "keys": "ctrl+shift+v" },` to use <kbd>ctrl</kbd><kbd>shift</kbd><kbd>v</kbd> for pasting in Windows Terminal. After that, <kbd>ctrl</kbd><kbd>v</kbd> should work for multiline text pasting in SafeCloset.

## Copy-Paste problem in Termux

For copy-pasting to work properly, you also need to install the [Termux:API](https://wiki.termux.com/wiki/Termux:API), which is an Android APP, just like Termux.

You have to install Termux and Termux:API from the same place. For example, if you installed Termux from F-Droid, install Termux:API from F-Droid, too. Mixing the installation from different stores will cause compatibility issues, because each download website uses a specific key for keysigning Termux and Addons.

