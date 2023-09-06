use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::helpers::VecLen;

#[derive(Serialize_repr, Deserialize_repr, Debug)]
#[repr(u8)]
pub enum ContentType {
    BaseGraphics = 1,
    NewGRF = 2,
    AI = 3,
    AILibrary = 4,
    Scenario = 5,
    Heightmap = 6,
    BaseSounds = 7,
    BaseMusic = 8,
    Game = 9,
    GameLibrary = 10,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfoListBranch {
    branch: String,
    version: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfoIdContentInfo {
    content_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfoExtIdContentInfo {
    content_type: ContentType,
    unique_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClientInfoExtIdMd5ContentInfo {
    content_type: ContentType,
    unique_id: u32,
    md5: [u8; 16],
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Packet {
    ClientInfoList {
        content_type: ContentType,
        openttd_version: u32,
        branches: Option<VecLen<u8, ClientInfoListBranch>>,
    },
    ClientInfoId {
        content_infos: VecLen<u16, ClientInfoIdContentInfo>,
    },
    ClientInfoExtId {
        content_infos: VecLen<u8, ClientInfoExtIdContentInfo>,
    },
    ClientInfoExtIdMd5 {
        content_infos: VecLen<u8, ClientInfoExtIdMd5ContentInfo>,
    },
    ClientContent {
        content_infos: VecLen<u16, ClientInfoIdContentInfo>,
    },
}
