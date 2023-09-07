use serde::{Deserialize, Serialize};

mod de;
mod error;
mod ser;

pub use de::from_bytes;
pub use error::Error;
pub use ser::to_bytes;

#[derive(Deserialize)]
pub struct DeVecLen<L, T> {
    #[allow(dead_code)]
    len: L,
    pub items: Vec<T>,
}

#[derive(Serialize)]
pub struct SerVecLen<'a, L, T> {
    pub len: L,
    pub items: &'a Vec<T>,
}

pub trait ServerPacket {
    const TYPE: u8;
}
