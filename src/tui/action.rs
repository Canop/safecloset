use {
    crate::tui::keys::*,
    crossterm::event::KeyEvent,
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
                if key == SHIFT_QUESTION {
                    key = QUESTION;
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
    }
}

// Define the actions that can be part of the menus
make_actions! {
    Back "back" ESC,
    CloseAllValues "*F*old All unselected Values" CONTROL_F,
    CloseDeepDrawer "go to *U*pper drawer" CONTROL_U,
    CloseShallowDrawer "Close drawer" CONTROL_U,
    ConfirmEntryRemoval "Confirm Entry Removal" Y,
    Copy "*C*opy" CONTROL_C,
    Cut "*C*ut" CONTROL_X,
    EditClosetComments "Edit Closet Comments",
    Help "Help" QUESTION,
    MoveLineDown "Move Line Down" CONTROL_DOWN,
    MoveLineUp "Move Line Up" CONTROL_UP,
    NewDrawer "*N*ew Drawer" CONTROL_N,
    NewEntry "New Entry" N,
    OpenAllValues "Un*f*old All Values" CONTROL_F,
    OpenDrawer "*O*pen Drawer" CONTROL_O,
    OpenPasswordChangeDialog "Change Drawer Password",
    Paste "Paste" CONTROL_V,
    Quit "*Q*uit" CONTROL_Q,
    RemoveLine "Remove Line" D,
    SaveDrawer "*S*ave Drawer" CONTROL_S,
    Search "Search" SLASH,
    ToggleHiding "Toggle *H*iding" CONTROL_H, // hiding either pwd chars or unselected values
}

