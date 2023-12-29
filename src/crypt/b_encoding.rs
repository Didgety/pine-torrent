#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use serde_json;

pub trait EncodableInt: Display {}

impl EncodableInt for u8 {}
impl EncodableInt for u16 {}
impl EncodableInt for u32 {}
impl EncodableInt for u64 {}
impl EncodableInt for u128 {}
impl EncodableInt for usize{}
impl EncodableInt for i8 {}
impl EncodableInt for i16 {}
impl EncodableInt for i32 {}
impl EncodableInt for i64 {}
impl EncodableInt for i128 {}
impl EncodableInt for isize {}

pub enum BencodedData {
    Empty,
    Num(i64)
}

#[allow(dead_code)]
pub fn decode_bencoded_value(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let first = encoded_value.first().clone().unwrap();
    match first {
        b'i' => {
            return decode_integer(encoded_value)
        }
        b'l' => {
            return decode_list(encoded_value)
        }
        b'd' => {
            return decode_dict(encoded_value)
        }
        b'0'..=b'9' => {
            return decode_string(encoded_value)
        }
        _ => {
            panic!("Unhandled encoded value: {:#?}", encoded_value)
        }
    }
}

/// Decode a bencoded integer
/// i<num>e
fn decode_integer(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let end_idx = encoded_value.iter().position(|&d| d == b'e').unwrap();
    let num = std::str::from_utf8(&encoded_value[1..end_idx]).unwrap();
    let num = num.parse::<i64>().unwrap();

    (num.into(), &encoded_value[end_idx + 1..])
}

/// Decode a bencoded string
/// <len>:<string>
/// No validation of len is done
fn decode_string(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let (len, remainder) = {
        let delim_idx = encoded_value.iter().position(|&d| d == b':').unwrap();
        encoded_value.split_at(delim_idx)
    };

    let len = std::str::from_utf8(len).unwrap();
    let len = len.parse::<usize>().unwrap();
    let res = String::from_utf8_lossy(&remainder[1..len + 1]);

    (res.into(), &remainder[len + 1..])
}

/// Decoded a bencoded list
/// l<any valid bencoded values>..<><>e
fn decode_list(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let mut vals = Vec::new();
    // strip the leading 'l'
    let mut remainder = encoded_value.split_at(1).1;
    
    while !remainder.is_empty() && !remainder.starts_with(&[b'e']) {
        let (val, rest) = decode_bencoded_value(remainder);
        vals.push(val);
        remainder = rest;
    }
    return (vals.into(), &remainder[1..])
}

/// Decode a bencoded dictionary
/// d<b_string><any valid bencoded value>..<><>e
fn decode_dict(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let mut dict = serde_json::Map::new();
    // strip the leading 'd'
    let mut remainder = encoded_value.split_at(1).1;

    while !remainder.is_empty() && !remainder.starts_with(&[b'e']) {      
        let (key, val_and_rest) = decode_bencoded_value(remainder);
        let key = match key {
            serde_json::Value::String(key) => key,
            key => {
                panic!("Dict key is not a string: {key:#?}");
            }
        };
        let (val, rest) = decode_bencoded_value(val_and_rest);
        dict.insert(key, val);
        remainder = rest;
    }
    //println!("{dict:#?}");
    return(dict.into(), &remainder[1..])
}

pub fn encode_int<I: EncodableInt>(num: &I) -> Vec<u8> {
    let data = num.to_string();
    let data = data.as_bytes();
    // add 2 to len to account for the encoding delimeters 'i' and 'e'
    let mut encoded_value = vec![b'0'; data.len() + 2];

    encoded_value[0] = b'i';
    encoded_value[data.len() + 1] = b'e';
    
    let mut idx = 1;
    for d in data {
        encoded_value[idx] = *d;
        idx += 1;
    }

    encoded_value
}