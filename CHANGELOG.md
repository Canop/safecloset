### next
- various improvements on behaviors of focusing and unfocusing the search input

<a name="v0.2.5"></a>
### v0.2.5 - 2021-10-05
- closing a drawer (and going to the upper drawer) is now done with ctrl-u
- on some platforms, ctrl-c copies the selected cell (if safecloset is compiled with "clipbpoard" feature)

<a name="v0.2.4"></a>
### v0.2.4 - 2021-10-03
- now both ctrl-enter and alt-enter can be used to insert a new line in a value (but many terminals support only one of them)

<a name="v0.2.4"></a>
### v0.2.4 - 2021-10-03
- now both ctrl-enter and alt-enter can be used to insert a new line in a value (but many terminals support only one of them)

<a name="v0.2.3"></a>
### v0.2.3 - 2021-10-01
- fix a crash on rendering with an empty value

<a name="v0.2.2"></a>
### v0.2.2 - 2021-09-29
- `-o` option to immediately prompt for password for drawer opening
- ctrl-v pastes the content of the clipboard (if safecloset is compiled with "clipbpoard" feature)
- mouse wheel support

<a name="v0.2.1"></a>
### v0.2.1 - 2021-09-26
- help screen
- ctrl-c to close a drawer or the help screen
- 'a' key edits a field, cursor at end, while 'i' puts the cursor at start

<a name="v0.2.0"></a>
### v0.2.0 - 2021-09-19
- mouse support in inputs and for cell selection
- sub-drawers (and breaking change in closet format)

<a name="v0.1.3"></a>
### v0.1.3 - 2021-09-08
- quit on inactivity
- swap entries with ctrl-up and ctrl-down
- multi-line values

<a name="v0.1.2"></a>
### v0.1.2 - 2021-08-24
- fuzzy search

<a name="v0.1.1"></a>
### v0.1.1 - 2021-08-24
- password characters visibility toggle (hidden initially)
- unselected values visibility toggle (preference kept in drawer, and automatic hiding if launched with --hide)
- switched from JSON to MessagePack as serialization format (breaking change)
- entry removal with the 'd' key

<a name="v0.1.0"></a>
### v0.1.0 - 2021-08-23
Yes it has a version, but it doesn't mean you can use it. Wait for the 0.2 at least!
