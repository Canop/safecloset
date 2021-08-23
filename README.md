[![Latest Version][s1]][l1] [![Chat on Miaou][s2]][l2]

[s1]: https://img.shields.io/crates/v/safecloset.svg
[l1]: https://crates.io/crates/safecloset

[s2]: https://miaou.dystroy.org/static/shields/room.svg
[l2]: https://miaou.dystroy.org/3768?rust

**SafeCloset** keeps your secrets in password protected files.

# Warning

* This is a personal project
* It hasn't been audited
* It's not even finished anyway

SafeCloset comes with **absolutely** no guarantee. If you store your crypto wallet's password with 2B€ in SafeCloset and can't read it back, I can do nothing for you.

# Overview

A closet is stored in a file that you can backup, keep with you on an USB key, etc.

A closet contains drawers, each one is found and open with its own password.

A drawer contains a list of (key, value). Values are texts in which you can store a code, a password, comments, a poem, some data, etc.

![clear drawer](doc/clear-drawer.png)

# Features

* The closet contains several drawers, some of them automatically created with an unknown password so that nobody can determine which drawers you're able to open, or even how many
* Each drawer is separately crypted with AES-GCM-SIV, with a random one-use nonce and the password/key of your choice. This gives an inherently long to test decrypt algorithm (but you should still use long passphrases for your drawers)
* You can have one or several drawers with real content. You can be forced to open a drawer at gun point and still keep other drawers secret without any trace
* When you open a drawer, with its password, you can read it, search it, edit it, close it
* Drawers are automatically closed on inactivity
* The size of the drawer's content isn't observable
* An attacker having several versions of the closet files can't determine if you've just read or made changes, and if the content grew or shrinked
* No clear file is ever created, edition is done directly in the TUI
* No clear data is ever given to any external library, widget, etc.
* All data is viewed and edited in the TUI application
* You can compile SafeCloset yourself. Its code is small and auditable and 100% in Rust
* The format of the closet file is described so that another application could be written to decode your closet files in the future (assuming you have the password)
* SafeCloset can't be queryied by other applications, like browsers. This is a feature.
* No company can die and lose your secrets: you keep everything
* Fast and convenient to use

# Non features

* SafeCloset doesn't protect you against keyloggers
* SafeCloset doesn't protect you from somebody watching your screen while a secret value is displayed
* SafeCloset doesn't try to be space efficient: Closet files are usually about 1 MB large to allow specific features

# Features not yet implemented

- search
- option to hide password on input
- option to hide non selected values
- auto-close
- multi-line values
- help page with all keyboard shortcuts
- copy-paste
- mouse selection
- closet merge & split

# Implementation details

Drawer data are serialized in JSON before being encrypted with AES-GCM-SIV.

JSON allows for the later addition of fields while keeping the compatibility with previous closet files.

The key used for this encryption is a 32 bits Argon2 hash of the password with a closet specific salt.

# Keyboard actions

* <kbd>o</kbd> : Open a drawer
* <kbd>n</kbd> : Create a drawer (when none is open) or create a drawer entry
* <kbd>esc</kbd> : Cancel current field edition
* <kbd>tab</kbd> : Create a new entry or edit the value if you're already editing an entry's name
* arrow keys: Move selection, selecting either an entry name or a value
* <kbd>i</kbd> or <kbd>insert</kbd> : Start editing the selected name or value
* <kbd>ctrl</kbd><kbd>q</kbd> : Quit without saving
* <kbd>ctrl</kbd><kbd>s</kbd> : Save
* <kbd>ctrl</kbd><kbd>x</kbd> : Save then quit
* <kbd>Enter</kbd> : Validate the current edition
