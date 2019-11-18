extern crate serde;
use serde::{Deserialize, Serialize};

pub const BUF_SIZE: usize = 64 * 1024 * 1024;

#[derive(Debug, Serialize)]
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
