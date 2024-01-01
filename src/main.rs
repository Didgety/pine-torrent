mod crypt;
use crypt::b_encoding;

use anyhow::Context;
// use serde_json;
use clap::{ Parser, Subcommand };
use std::{ env, path::PathBuf };

use crate::crypt::b_encoding::{BEncodedData, BEStr};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Decode {
        encoded_value: String,
    },
    Encode {

    },
    Info {
        torrent: PathBuf,
    },
    
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
//        your_bittorrent.sh info <filename>.torrent
fn main() {
    let args = Args::parse();

    match args.command {
        Command::Decode { encoded_value } => {
            let (_, decoded_value) = b_encoding::decode_bencoded_value(encoded_value);
            //let (decoded_value, _) = b_encoding::decode_bencoded_value(encoded_value.as_bytes());
            println!("{}", decoded_value.to_string());
        }
        Command::Encode {} => {
            // println!("{:#?}", args[2]);
            // // let value: &i64 = &args[2].parse::<i64>().unwrap();
            // let value = &args[2];
            // // let encoded_value = b_encoding::encode_int(value);
            // let encoded_value = b_encoding::encode_str(value);
            // // for v in encoded_value.as_ref() {
            // //     println!("{:#?}", v);
            // // }
            // // println!("{}", b_encoding::decode_bencoded_value(encoded_value.as_slice()).0);
            // // println!("{}", encoded_value);
            // println!("{}", b_encoding::decode_bencoded_value(encoded_value.as_slice()).0);
        }
        Command::Info { torrent } => {                 
            let decoded_file = crypt::torrent::read_torrent(torrent);

            match decoded_file {
                BEncodedData::Dict(d) => {
                    let tracker = d.get_key_value(&BEStr::from("announce")).unwrap().1;
                    match tracker {
                        BEncodedData::ByteStr(t) => {
                            println!("Tracker: {}", t.to_string());
                        }
                        _ => {}
                    }
                    
                    let len_dict = d.get_key_value(&BEStr::from("info")).unwrap().1;
                    match len_dict {
                        BEncodedData::Dict(d_l) => {
                            let len = d_l.get_key_value(&BEStr::from("length")).unwrap().1;
                            println!("Length: {:#?}", len.to_string());
                        }
                        _ => {
                            println!("Length key in info dict missing");
                        }
                    }
                }
                _ => {
                    println!("Not a valid file");
                }
            }

            // let tracker = decoded_file.["announce"].as_str().unwrap();
            // println!("Tracker: {:#?}", tracker);
            // let len = decoded_file["info"]["length"].as_i64().unwrap();
            // println!("Length: {:#?}", len);
        }
        _ => {
            let args: Vec<String> = env::args().collect();
            let command = &args[1];
            println!("unknown command: {}", command)
        }
    }
}
