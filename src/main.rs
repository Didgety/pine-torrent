mod crypt;
use crypt::b_encoding;
// use serde_json;
use std::env;

// Usage: your_bittorrent.sh decode "<encoded_value>"
//        your_bittorrent.sh info <filename>.torrent
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_ref() {
        "decode" => {
            let encoded_value: &[u8] = &args[2].as_bytes();
            let (decoded_value, _) = b_encoding::decode_bencoded_value(encoded_value);
            println!("{}", decoded_value.to_string());
        }
        "encode" => {
            let value: &i64 = &args[2].parse::<i64>().unwrap();
            let encoded_value = b_encoding::encode_int(value);
            // for v in encoded_value.as_ref() {
            //     println!("{:#?}", v);
            // }
            println!("{}", b_encoding::decode_bencoded_value(encoded_value.as_slice()).0);
        }
        "info" => {
            let fname = &args[2];
            let decoded_file = crypt::torrent::read_torrent(fname);
            let tracker = decoded_file["announce"].as_str().unwrap();
            println!("Tracker: {:#?}", tracker);
            let len = decoded_file["info"]["length"].as_i64().unwrap();
            println!("Length: {:#?}", len);
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}
