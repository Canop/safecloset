use {
    termimad::minimad::{Text, TextTemplate},
};

static MD: &str = r#"

# SafeCloset ${version}

SafeCloset is written by Denys Séguret. Source code and documentation can be found on *https://github.com/Canop/safecloset*

SafeCloset stores secrets in drawers. A drawer may be either top-level, or hidden in another drawer. Each drawer is protected by a passphrase.

SafeCloset leaves after 60 seconds of inactivity.

## Keyboard actions

The *^* symbol in SafeCloset means that the *control* key must be pressed.

|:-:|:-:
|key|action
|:-:|-
| *^n* | Create a drawer (inside the current drawer, if one is open)
| *^o* | Open a drawer
| *^u* | Goes up, closing the current drawer (you're back in the upper level one if you close a deep drawer)
| *^s* | Save the current drawer and all upper drawers
| *^q* | Quit without saving (with no confirmation)
| *^h* | Toggle hiding either password chars or unselected values
| *^a* | Toggle open all values
| */* | Start searching the current drawer (do *Enter* or use the down or up arrow key to freeze it)
| */* then *esc* | Remove the current filtering
| *esc* | Cancel current field edition or open a menu
| *tab* | Create a new entry or edit the value if you're already editing an entry's name
| arrow keys | Move selection, selecting either an entry name or a value
| *^↑* | Move selected line up
| *^↓* | Move selected line down
| *i* or *insert* | Start editing the selected name or value, cursor at start
| *a* | Start editing the selected name or value, cursor at end
| *d* | Remove the selected entry (with confirmation)
| *^c* | Copy the selection (or the entire field if not edited)
| *^x* | Cut the selection
| *^v* | Paste
| *Enter* | Validate the current edition
| *alt*-*Enter* or *^enter* | New line in the currently edited value
|-|-

## Guarantees

There's none. And I can do nothing for you if you forget your passphrase.


"#;

pub fn help_text() -> Text<'static> {
    let template = TextTemplate::from(MD);
    let mut expander = template.expander();
    expander.set("version", env!("CARGO_PKG_VERSION"));
    expander.expand()
}
