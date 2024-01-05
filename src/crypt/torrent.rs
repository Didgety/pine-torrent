use anyhow::Context;

use sha1::{Sha1, Digest};

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{hash::Hashes, b_encoding::BEncodedData};

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Torrent {
    pub announce: String,
    pub info: MetaInfo,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MetaInfo {
    /// Suggested name to save the file or directory as. utf-8 encoded
    pub name: String,
    // the name of the dict in a torrent file is literally "piece length"
    /// Number of bytes each piece in the file is split into
    /// 
    /// Files are split into fixed-size pieces that are all the same length except possibly
    /// the last one which may be truncated. Piece length is almost always a power of two,
    /// most commonly 2^18 = 256 K (Prior to Bittorrent 3.2 2^20 = 1 M was default)
    #[serde(rename = "piece length")]
    pub piece_len: usize,
    /// Maps to a string whose length is a multiple of 20.
    /// Subdivided into strings of length 20, each of which is the SHA-1 hash
    /// of the piece at the corresponding index.
    /// Each entry of 'pieces' is the SHA-1 hash of the piece at the corresponding index
    pub pieces: Hashes,

    #[serde(flatten)]
    pub keys: Keys,

}

/// There is a mutually exclusive key 'length' or key 'files'
#[derive(Debug, Clone, Deserialize, Serialize)]
// no tag tells us if it's single or multi file
// the presence of mutually exclusive "length" or "files" is the only indicator
#[serde(untagged)]
pub enum Keys {
    /// If present, length maps to length of a single file in bytes
    SingleFile {
        length: usize,
    },
    /// Otherwise files are concactenated in the order they appear in the files list.
    /// The files list is the value files maps to
    MultiFile {
        files: Vec<File>,
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct File {
    /// Length of file in bytes
    pub length: usize,
    /// List of utf-8 encoded strings corresponding to subdirectory names
    /// the last of which is the file name
    /// 0 length list is an error case
    /// In the single file case - the name key is the name of a file
    /// In the multi file case - the name key is the name of a directory
    pub path: Vec<String>,
}

// pub fn read_torrent(file: PathBuf) -> serde_json::Value {
//     let contents = std::fs::read(file).expect("Failed reading file");
//     crate::b_encoding::decode_bencoded_value(&contents).0
// }

pub fn read_torrent(file: PathBuf) -> BEncodedData {
    let contents: &[u8] = &std::fs::read(file).expect("Failed reading file");
    let torr: Torrent = serde_bencode::from_bytes(&contents).context("parse torrent with serde bencode").unwrap();
    //println!("{_torr:?}");

    println!("Tracker: {}", torr.announce);
    if let Keys::SingleFile { length } = torr.info.keys {
        println!("Length: {length}");
    } else {
        todo!();
    }

    let encoded_metainfo = serde_bencode::to_bytes(&torr.info).context("re-encode info to calculate hash").unwrap();
    let mut hasher = Sha1::new();
    hasher.update(&encoded_metainfo);
    let metainfo_hash = hasher.finalize();
    println!("Info Hash: {}", hex::encode(&metainfo_hash));

    println!("Piece Length: {}", torr.info.piece_len);
    // TODO impl iterator and other convenience traits for hash
    for hash in torr.info.pieces.0 {
        println!("{}", hex::encode(&hash));
    }

    let (_, decoded_file) = crate::b_encoding::decode_bencoded_value(contents);

    decoded_file
}