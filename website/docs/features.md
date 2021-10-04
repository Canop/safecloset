
# Secure design

* The closet contains several drawers, some of them automatically created with an unknown password so that nobody can determine which drawers you're able to open, or even how many
* Each drawer is separately crypted with AES-GCM-SIV, with a random one-use nonce and the password/key of your choice. This gives an inherently long to test decrypt algorithm (but you should still use long passphrases for your drawers)
* You can have one or several drawers with real content. You can be forced to open a drawer at gun point and still keep other drawers secret without any trace, either at the top level or deeper in the drawer you opened
* When you open a drawer, with its password, you can read it, search it, edit it, close it
* In an open drawer you can create new drawers, or open deeper drawers if you know their password
* SafeCloset automatically quits on inactivity
* The size of the drawer's content isn't observable
* If you edit a drawer, an attacker storing all versions of the closet wouldn't know if you edited a deeper drawer or not
* No clear file is ever created, edition is done directly in the TUI (external editors are usually the weakest point)
* No clear data is ever given to any external library, widget, etc.
* All data is viewed and edited in the TUI application
* You can compile SafeCloset yourself. Its code is small and auditable
* The code is 100% in Rust. I wouldn't trust anything else today for such a program
* The format of the closet file is described so that another application could be written to decode your closet files in the future (assuming you have the password)
* SafeCloset can't be queryied by other applications, like browsers. This is a feature.
* You may have all your secrets in one file easy to keep with you and backup
* No company can die and lose your secrets: you keep everything, with as many copies as necessary, where you want
* No company can be forced to add some secret stealing code: SafeCloset is small, open-source and repleacable
* Cross-platform because you don't know where you'll have to use your closet
* "I'm being watched" mode in which unselected values are hidden. This mode is kept per drawer, always activated when you launch SafeCloset with the `--hide` option, and toggled with <kbd>ctrl</kbd><kbd>h</kbd>


# Cross-platform

Because you don't know where you'll need your files, SafeCloset is written for

* Linux (with several variants)
* Mac
* Windows (a recent enough terminal is needed)

# Convenience

SafeCloset is designed to allow very fast sessions, adding only a few keystrokes over the passphrase typing.

See the [most typical sessions](../usage#most-typical-sessions).
