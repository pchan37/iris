mod cipher;
mod errors;
mod files;
mod iris_stream;
mod iris_tcp_stream;
mod room_mapping;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum IrisMessage {
    Acknowledge,
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
