// use anyhow::Context;
use core::fmt;

#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use std::{collections::BTreeMap, usize};
use hex::decode;
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

pub type BEDict = BTreeMap<BEStr, BEncodedData>;
pub type BEList = Vec<BEncodedData>;



#[derive(Debug, PartialEq, Hash, Eq, PartialOrd, Ord, Clone)]
pub struct BEStr ( Vec<u8> );

// TODO replace with type impl for Type::from(value)
impl BEStr {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Convert to BEStr from slice
impl From<&[u8]> for BEStr {
    fn from(val: &[u8]) -> Self {
        let vec = Vec::from(val);
        BEStr(vec)
    }
}

/// Convert to BEStr from a Vec of u8's
impl From<Vec<u8>> for BEStr {
    fn from(val: Vec<u8>) -> Self {
        BEStr(val)
    }
}

/// Convert to BEStr from a str reference
impl From<&str> for BEStr {
    fn from(val: &str) -> Self {
        BEStr(val.as_bytes().to_vec())
    }
}

/// Convert to String from BEncoded String
impl From<&BEStr> for String {
    fn from(val: &BEStr) -> Self {
        String::from_utf8_lossy(&val.0).to_string()
    }
}

/// Convert to Byte Array from BEncoded String
impl From<&BEStr> for Vec<u8> {
    fn from(val: &BEStr) -> Self {
        val.0.clone()
    }
}

impl From<&BEStr> for serde_json::Value {
    fn from(val: &BEStr) -> Self {
        match val.0.is_ascii() {
            true => serde_json::Value::String(String::from(val)),
            false => serde_json::Value::Array(
                val.0.iter()
                    .map(|&c| serde_json::Value::Number(c.into()))
                    .collect(),
            ),
        }
    }
}

impl fmt::Display for BEStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", String::from_utf8_lossy(&self.0))
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Clone, Debug)]
pub enum BEncodedData {
    Empty,
    // TODO case this for numbers larger or smaller than i64
    Num(i64),
    ByteStr(BEStr),
    List(BEList),
    Dict(BEDict)
}

impl From<&[u8]> for BEncodedData {
    fn from(val: &[u8]) -> Self {
        let (_, decoded) = decode_bencoded_value(val);
        decoded
    }
}

impl fmt::Display for BEncodedData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BEncodedData::Num(n) => {
                write!(f, "{}", n)
            }
            BEncodedData::ByteStr(s) => {
                write!(f, "{}", s)
            }
            BEncodedData::List(l) => {
                let elem: Vec<String> = l.iter()
                    .map(|e| format!("{}", e))
                    .collect();
                write!(f, "[{}]", elem.join(","))
            }
            BEncodedData::Dict(d) => {
                let elem: Vec<String> = d.iter()
                    .map(|(k, v)| format!("{}:{}", k, v))
                    .collect();
                write!(f, "{{{}}}", elem.join(","))
                // write!(f, "[{}]", d)
            }
            BEncodedData::Empty => {
                write!(f, "Empty")
            }
        }
    }
}

pub fn decode_bencoded_value<V: AsRef<[u8]> + std::fmt::Debug>(encoded_value: V) 
-> (usize, BEncodedData) {
    let first = encoded_value.as_ref()[0];

    match first {
        b'i' => {
            return decode_int(encoded_value)
        }
        b'l' => {
            return decode_list(encoded_value)
        }
        b'd' => {
            return decode_dict(encoded_value)
        }
        b'0'..=b'9' => {
            return decode_str(encoded_value)
        }
        _ => {
            println!("{:#?}", encoded_value.as_ref()[0].to_be_bytes());
            panic!("Unhandled encoded value: {:#?}", encoded_value)
        }
    }
}

// TODO handle big numbers - max size is not specified
/// Decode a BEncoded integer
/// i<num>e
fn decode_int<I: AsRef<[u8]>>(encoded_num: I) -> (usize, BEncodedData) {
    let encoded_num = encoded_num.as_ref();
    let end_idx = encoded_num.iter().position(|&d| d == b'e').unwrap();
    
    let num = std::str::from_utf8(&encoded_num[1..end_idx]).unwrap()
        .parse::<i64>().unwrap();
    //println!("{:#?}", end_idx);
    // point to the last character to avoid index out of bounds
    // ie, i52e3:abc
    //        ^
    //        |
    (end_idx + 1, BEncodedData::Num(num))
}

/// Decode a BEncoded string
/// <len>:<string>
/// No validation of len is done
fn decode_str<S: AsRef<[u8]>>(encoded_value: S) -> (usize, BEncodedData) {
    let encoded_value = encoded_value.as_ref();
    println!("processing string: \n{}", BEStr::from(encoded_value));

    let delim_idx = encoded_value.iter().position(|&d| d == b':').unwrap();
    let (len, _) = {
        encoded_value.split_at(delim_idx)
    };

    let len = std::str::from_utf8(len).unwrap()
        .parse::<usize>().unwrap();
    println!("str len {}\n", len);

    let text = BEStr(
        encoded_value[delim_idx + 1..delim_idx + 1 + len].to_vec()
    );

    // println!("len {}, delim idx {}\nstr: {}\n", len, delim_idx, BEStr::from_slice(&remainder[delim_idx.. delim_idx + len]));
    // println!("str remainder: {}\n", BEStr::from_slice(&remainder[delim_idx..]));

    // point to the last character to avoid index out of bounds
    // ie, 3:abci52e
    //         ^
    //         |
    (delim_idx + len + 1, BEncodedData::ByteStr(text))
}

/// Decoded a BEncoded list
/// l<any valid BEncoded values>..<><>e
fn decode_list<L: AsRef<[u8]>>(encoded_list: L) -> (usize, BEncodedData) {
    let encoded_list = encoded_list.as_ref();
    let mut vals = Vec::new();
    // strip the leading 'l'
    let mut encoded_list = &encoded_list[1..];
    let mut end_idx: usize = 1;

    loop {
        match encoded_list.iter().next().unwrap() {
            b'e' => break,
            _ => {
                let (itr_idx, decoded_value) = decode_bencoded_value(encoded_list);
                vals.push(decoded_value);
                encoded_list = &encoded_list[itr_idx..];
                end_idx += itr_idx;
            }
        }
    }

    println!("evaluated list {:#?}", String::from_utf8_lossy(encoded_list));
    // println!("list vals: {:#?}", vals);
    println!("vals len: {}, end_idx {}", vals.len(), end_idx);

    return (end_idx + 1, BEncodedData::List(vals))
}

/// Decode a BEncoded dictionary
/// d<b_string><any valid BEncoded value>..<><>e
fn decode_dict<D: AsRef<[u8]>>(encoded_dict: D) -> (usize, BEncodedData) {
    // Strip leading 'd'
    let mut encoded_dict = encoded_dict.as_ref().split_at(1).1;
    let mut dict = BEDict::new();
    let mut end_idx: usize = 1;

    loop {
        match encoded_dict.iter().next().unwrap() {
            b'e' => break,
            _ => {
                let (key_idx, key) = decode_bencoded_value(encoded_dict);

                let key = match key {
                    BEncodedData::ByteStr(s) => s,
                    data => panic!("Dict key is not a string: {data:#?}"),
                };

                encoded_dict = &encoded_dict[key_idx..];
                end_idx += key_idx;

                let (val_idx, val) = decode_bencoded_value(encoded_dict);

                encoded_dict = &encoded_dict[val_idx..];
                end_idx += val_idx;

                dict.insert(key, val);
            }
        }
    }
    
    (end_idx + 1, BEncodedData::Dict(dict))
}

#[allow(dead_code)]
pub fn encode_int<I: EncodableInt>(num: &I) -> Vec<u8> {
    let data = num.to_string();
    // TODO test the below 3 lines of code
    // data.insert_str(0, 'i');
    // data.insert(data.len(), 'e');
    // return data.as_bytes().to_owned();

    let data = data.as_bytes();
    // add 2 to len to account for the encoding delimeters 'i' and 'e'
    let mut encoded_value = Vec::with_capacity(data.len() + 2);

    encoded_value[0] = b'i';
    encoded_value[data.len() + 1] = b'e';
    
    let mut idx = 1;
    for d in data.as_ref() {
        encoded_value[idx] = *d;
        idx += 1;
    }

    encoded_value
    //BEncodedData::Num(encoded_value)
}

#[allow(dead_code)]
pub fn encode_str(text: &str) -> Vec<u8> {
    let mut value = String::from(text);
    let prefix = text.len().to_string() + ":";
    value.insert_str(0, &prefix);

    let encoded_value = value.as_bytes().to_owned();

    encoded_value
    //BEncodedData::ByteStr(encoded_value)
}   