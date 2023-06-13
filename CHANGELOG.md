### next
- sort, accessible through the menu - Fix #31

<a name="v1.2.3"></a>
### v1.2.3 - 2023-04-06
Fix a crash on some combinations of scroll+search

<a name="v1.2.2"></a>
### v1.2.2 - 2023-04-03
Better clipboard support on Mac - Fix #30

<a name="v1.2.1"></a>
### v1.2.1 - 2023-03-10
Values aren't rendered as markdown by default anymore. Markdown rendering is now opt-in, through the drawer menu - Fix #27,#28

<a name="v1.1.0"></a>
### v1.1.0 - 2023-02-28
Import feature: lets you import from another drawer, from a drawer of another closet file, or from a CSV file.

<a name="v1.0.0"></a>
### v1.0.0 - 2023-01-13
There's no reason to wait more for a 1.0.0: SafeCloset is stable and complete.

<a name="v0.7.0"></a>
### v0.7.0 - 2022-10-18
- shift-n key combination for adding an entry immediately after the selected one

<a name="v0.6.4"></a>
### v0.6.4 - 2022-09-20
- fix backslash character ('\') not rendered in values - Fix #22
- colorize bold so that it's visible in terminal not rendering bold as bold
- I don't provide the Raspberry precompiled version anymore due to a difficulty with cargo cross

<a name="v0.6.3"></a>
### v0.6.3 - 2022-08-30
- increase delay before auto closing to 120 seconds
- fix tab key active behind open menu - Fix #21

<a name="v0.6.2"></a>
### v0.6.2 - 2022-05-05
- update termimad to solve a potential crash

<a name="v0.6.1"></a>
### v0.6.1 - 2022-01-16
- more relevant contextual hints - Fix #20

<a name="v0.6.0"></a>
### v0.6.0 - 2021-12-03
- support for more mouse interactions
- clear comments at start of closet file (example purpose is holding the name of the soft to find it if you have as bad a memory as me)
- closet clear comments editor

<a name="v0.5.3"></a>
### v0.5.3 - 2021-11-28
- Fix crash on Windows Terminal on some mouse operations - Fix #17
- better support of wide characters - Fix #18
- update status when waiting for long tasks

<a name="v0.5.2"></a>
### v0.5.2 - 2021-11-19
- fix rendering problems on sides on Windows (eg duplicate status line) - Fix #14

<a name="v0.5.1"></a>
### v0.5.1 - 2021-11-17
- fix documentation on values folding in help screen

<a name="v0.5.0"></a>
### v0.5.0 - 2021-11-17
- change drawer password - Fix #5

<a name="v0.4.0"></a>
### v0.4.0 - 2021-11-11
- the esc key opens a menu displaying relevant commands and their keys
- ctrl-A toggles having all values always open (choice is kept in drawer settings) - Fix #8
- various improvements of ergonomics

<a name="v0.3.0"></a>
### v0.3.0 - 2021-10-30
- ctrl-x no longer saves and quits
- clipboard feature now default
- support of selection in inputs (with shift arrow keys or mouse drag)
- ctrl-x, ctrl-c, ctrl-v are shortcuts for cutting, copying, pasting in inputs
- more mouse support (for example mouse wheel in inputs)

<a name="v0.2.6"></a>
### v0.2.6 - 2021-10-22
- various improvements on focusing and unfocusing the search input
- when editing a multiline value, ctrl-down and ctrl-up swap lines
- improve suggestions in status bar

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
