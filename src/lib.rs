extern crate serde;
use serde::{Deserialize, Serialize};

pub const BUF_SIZE: usize = 1500;

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub msg_type: MsgType,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum MsgType {
    Register,
    List,
    File,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
