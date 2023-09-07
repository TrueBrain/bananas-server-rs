use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use super::super::wire;
use super::VecLen;

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

#[derive(Deserialize, Debug)]
pub struct ClientInfoListBranch {
    pub branch: String,
    pub version: String,
}

#[derive(Deserialize, Debug)]
pub struct ClientInfoIdContentInfo {
    pub content_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct ClientInfoExtIdContentInfo {
    pub content_type: ContentType,
    pub unique_id: u32,
}

#[derive(Deserialize, Debug)]
pub struct ClientInfoExtIdMd5ContentInfo {
    pub content_type: ContentType,
    pub unique_id: u32,
    pub md5: [u8; 16],
}

#[derive(Serialize, Debug)]
pub struct ServerInfo {
    pub content_type: ContentType,
    pub content_id: u32,
    pub filesize: u32,
    pub name: String,
    pub version: String,
    pub url: String,
    pub description: String,
    pub unique_id: u32,
    pub md5: [u8; 16],
    pub dependencies: VecLen<u8, u32>,
    pub tags: VecLen<u8, String>,
}
impl wire::ServerPacket for ServerInfo {
    const TYPE: u8 = 4;
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub enum ClientPacket {
    #[serde(rename = "0")]
    ClientInfoList {
        content_type: ContentType,
        openttd_version: u32,
        branches: Option<VecLen<u8, ClientInfoListBranch>>,
    },
    #[serde(rename = "1")]
    ClientInfoId {
        content_infos: VecLen<u16, ClientInfoIdContentInfo>,
    },
    #[serde(rename = "2")]
    ClientInfoExtId {
        content_infos: VecLen<u8, ClientInfoExtIdContentInfo>,
    },
    #[serde(rename = "3")]
    ClientInfoExtIdMd5 {
        content_infos: VecLen<u8, ClientInfoExtIdMd5ContentInfo>,
    },
    // 4 is a server-packet
    #[serde(rename = "5")]
    ClientContent {
        content_infos: VecLen<u16, ClientInfoIdContentInfo>,
    },
    // 6 is a server-packet
}
