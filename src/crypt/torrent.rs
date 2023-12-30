use std::path::PathBuf;

use serde::Deserialize;

use super::hash::Hashes;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Torrent {
    announce: String,
    info: Info,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct Info {
    length: usize,
    /// Suggested name to save the file or directory as
    name: String,
    // the name of the dict in a torrent file is literally "piece length"
    /// Number of bytes each piece in the file is split into
    /// 
    /// Files are split into fixed-size pieces that are all the same length except possibly
    /// the last one which may be truncated. Piece length is almost always a power of two,
    /// most commonly 2^18 = 256 K (Prior to Bittorrent 3.2 2^20 = 1 M was default)
    #[serde(rename = "piece length")]
    piece_len: usize,
    /// Maps to a string whose length is a multiple of 20.
    /// Subdivided into strings of length 20, each of which is the SHA-1 hash
    /// of the piece at the corresponding index.
    /// Each entry of 'pieces' is the SHA-1 hash of the piece at the corresponding index
    pieces: Hashes,

    #[serde(flatten)]
    keys: Keys,

}

/// There is a mutually exclusive key 'length' or key 'files'
#[derive(Debug, Clone, Deserialize)]
// no tag tells us if it's single or multi file
#[serde(untagged)]
enum Keys {
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

#[derive(Debug, Clone, Deserialize)]
struct File {
    /// Length of file in bytes
    length: usize,
    /// List of utf-8 encoded strings corresponding to subdirectory names
    /// the last of which is the file name
    /// In the single file case - the name key is the name of a file
    /// In the multi file case - the name key is the name of a directory
    path: Vec<String>,
}

pub fn read_torrent(file: PathBuf) -> serde_json::Value {
    let contents = std::fs::read(file).expect("Failed reading file");
    crate::b_encoding::decode_bencoded_value(&contents).0
}