# Frequently Asked Questions


## Why multiline text pasting in entry cells doesn't work?

First, you should not paste multiline text in left cells. They are the names of entries and are designed to contain short single line name describing the entry. The right cells are the values, where you can paste multiline text.

You could use <kbd>ctrl+v</kbd> to paste multiline text. SafeCloset has special handing for it. It should work out of box on Linux and Linux based systems (macOS, Termux etc.). For Windows users, you have to pay attention to these points.
* If you are using `cmd.exe` or the native PowerShell command line, remember to use <kbd>ctrl+v</kbd>. Don't use the shortcut <kbd>alt+space e p</kbd> or the system menu (`Edit -> Paste`) from the window's top-left icon.
* If you are a Windows Terminal user, you need to change its built-in pasting shortcut. Newer versions of Windows Terminal uses <kbd>ctrl+v</kbd> for pasting. You would find a line similar to `{ "command": "paste", "keys": "ctrl+v" },` in its configuration file. This will interfere with SafeCloset's handing of <kbd>ctrl+v</kbd>. You have to change Windows Terminal's pasting shortcut to something else (not <kbd>ctrl+v</kbd>) to make SafeCloset's multiline pasting work. For example, you can use the setting `{ "command": "paste", "keys": "ctrl+shift+v" },` to use <kbd>ctrl+shift+v</kbd> for pasting in Windows Terminal. After that, <kbd>ctrl+v</kbd> should work for multiline text pasting in SafeCloset. 
