use {
    super::*,
    serde::{Deserialize, Serialize},
};


#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DrawerId {
    bytes: Box<[u8]>,
}

impl DrawerId {
    pub fn new() -> Self {
        DrawerId {
            bytes: random_bytes(20)
        }
    }
}

pub trait Identified {
    fn get_id(&self) -> &DrawerId;

    fn has_same_id<I: Identified>(&self, other: &I) -> bool {
        self.get_id() == other.get_id()
    }

}
