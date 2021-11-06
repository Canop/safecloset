use {
    crate::tui::keys::*,
    crossterm::event::KeyEvent,
};

macro_rules! make_actions {
    {
        $( $variant:ident $label:literal $key:expr, )*
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Action {
            $( $variant, )*
        }
        //pub static ACTIONS: &[Action] = &[
        //    $( Action::$variant, )*
        //];
        impl Action {
            pub fn label(self) -> &'static str {
                match self {
                    $( Action::$variant => $label, )*
                }
            }
            pub fn key(self) -> KeyEvent {
                match self {
                    $( Action::$variant => $key, )*
                }
            }
            pub fn for_key(mut key: KeyEvent) -> Option<Self> {
                // small hack because on Windows/Azerty I seem
                // to receive 'shift-?' for '?' from crossterm
                if key == SHIFT_QUESTION {
                    key = QUESTION;
                }
                $(
                    if key == $key {
                        return Some(Action::$variant);
                    }
                )*
                return None;
            }
        }
    }
}

// Define the actions that can be part of the menus
make_actions! {
    Back "back" ESC,
    NewDrawer "*N*ew Drawer" CONTROL_N,
    OpenDrawer "*O*pen Drawer" CONTROL_O,
    SaveDrawer "*S*ave Drawer" CONTROL_S,
    CloseDrawer "goes to *U*pper drawer or to root" CONTROL_U,
    Help "Help" QUESTION,
    Quit "*Q*uit" CONTROL_Q,
    MoveLineUp "Move Line Up" CONTROL_UP,
    MoveLineDown "Move Line Down" CONTROL_DOWN,
    ToggleHiding "Toggle *H*idding" CONTROL_H, // hiding either pwd chars or unselected values
    Copy "*C*opy" CONTROL_C,
    Cut "*Cut" CONTROL_X,
    Paste "Paste" CONTROL_V,
}

