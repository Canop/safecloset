use {
    crokey::*,
    crossterm::event::KeyEvent,
    std::fmt,
};

macro_rules! make_actions {
    {
        $( $variant:ident $label:literal $($key:expr)* , )*
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Action {
            $( $variant, )*
        }
        impl Action {
            pub fn label(self) -> &'static str {
                match self {
                    $( Action::$variant => $label, )*
                }
            }
            #[allow(unreachable_code)]
            pub fn key(self) -> Option<KeyEvent> {
                match self {
                    $( Action::$variant => {
                        $(
                            return Some($key);
                        )*
                        return None;
                    })*
                }
            }
            pub fn for_key(mut key: KeyEvent) -> Option<Self> {
                // small hack because on Windows/Azerty I seem
                // to receive 'shift-?' for '?' from crossterm
                if key == key!(shift-'?') {
                    key = key!('?');
                }
                $(
                    $(
                        if key == $key {
                            return Some(Action::$variant);
                        }
                    )*
                )*
                return None;
            }
        }
        impl fmt::Display for Action {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.label())
            }
        }
    }
}

// Define the actions that can be part of the menus
make_actions! {
    Back "back" key!(esc),
    CloseAllValues "*F*old All unselected Values" key!(ctrl-f),
    CloseDeepDrawer "go to *U*pper drawer" key!(ctrl-U),
    CloseShallowDrawer "Close drawer" key!(ctrl-U),
    ConfirmEntryRemoval "Confirm Entry Removal" key!(y),
    Copy "*C*opy" key!(ctrl-C),
    Cut "*C*ut" key!(ctrl-X),
    EditClosetComments "Edit Closet Comments",
    Help "Help" key!('?'),
    Import "Import",
    SwapLineDown "Swap Line Down" key!(ctrl-down),
    SwapLineUp "Swap Line Up" key!(ctrl-up),
    NewDrawer "*N*ew Drawer" key!(ctrl-N),
    NewEntry "New Entry" key!(n),
    NewEntryAfterCurrent "New Entry After Current" key!(shift-n),
    OpenAllValues "Un*f*old All Values" key!(ctrl-F),
    OpenDrawer "*O*pen Drawer" key!(ctrl-O),
    OpenPasswordChangeDialog "Change Drawer Password",
    Paste "Paste" key!(ctrl-V),
    Quit "*Q*uit" key!(ctrl-Q),
    RemoveLine "Remove Line" key!(d),
    SaveDrawer "*S*ave Drawer" key!(ctrl-S),
    Search "Search" key!('/'),
    ToggleHiding "Toggle *H*iding" key!(ctrl-H), // hiding either pwd chars or unselected values
    ToggleMarkdown "Toggle Markdown",
}
