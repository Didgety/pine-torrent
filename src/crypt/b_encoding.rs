use core::fmt;
#[cfg(not(feature = "std"))]
use core::fmt::Display;
#[cfg(feature = "std")]
use std::fmt::Display;

use std::collections::BTreeMap;
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

impl BEStr {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn from_slice(val: &[u8]) -> Self {
        let vec = Vec::from(val);
        BEStr(vec)
    }

    fn from_vec(val: Vec<u8>) -> Self {
        BEStr(val)
    }

    fn from_str(val: &str) -> Self {
        BEStr(val.as_bytes().to_vec())
    }
}

/// Convert to String from BEncoded String
impl From<&BEStr> for String {
    fn from(val: &BEStr) -> Self {
        String::from_utf8_lossy(&val.0).to_string()
    }
}

impl fmt::Display for BEStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", String::from(self))
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
        let (_, decoded) = d_b_v(val);
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
                    .map(|(k, v)| format!("{}: {}", k, v))
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

pub fn d_b_v<V: AsRef<[u8]> + std::fmt::Debug>(encoded_value: V) -> (usize, BEncodedData) {
    let first = encoded_value.as_ref()[0];
    // println!("first {}", first.to_string());
    match first {
        b'i' => {
            // println!("int {}", first.to_string());
            // let(idx, val) = d_i(&encoded_value);
            // println!("int end idx {} val {}", idx, val);
            return d_i(encoded_value)
        }
        b'l' => {
            // println!("list {}", first.to_string());
            // let(idx, val) = d_l(&encoded_value);
            // println!("list end idx {} val {}", idx, val);
            return d_l(encoded_value)
        }
        b'd' => {
            // println!("dict {}", first.to_string());
            // let(idx, val) = d_d(&encoded_value);
            // println!("dict end idx {} val {}", idx, val);
            let (val, _) = decode_dict(encoded_value.as_ref());
            println!("{val}");
            return d_d(encoded_value)
        }
        b'0'..=b'9' => {
            // println!("str {}", first.to_string());
            // let(idx, val) = d_s(&encoded_value);
            // println!("str end idx {} val {}", idx, val);
            return d_s(encoded_value)
        }
        _ => {
            println!("{:#?}", encoded_value.as_ref()[0].to_be_bytes());
            panic!("Unhandled encoded value: {:#?}", encoded_value)
        }
    }
}

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
            println!("{:#?}", encoded_value[0].to_be_bytes());
            panic!("Unhandled encoded value: {:#?}", encoded_value)
        }
    }
}

fn d_i<I: AsRef<[u8]>>(encoded_num: I) -> (usize, BEncodedData) {
    let encoded_num = encoded_num.as_ref();
    let end_idx = encoded_num.iter().position(|&d| d == b'e').unwrap();
    
    let num = std::str::from_utf8(&encoded_num[1..end_idx]).unwrap();
    let num = num.parse::<i64>().unwrap();

    //println!("{:#?}", end_idx);
    // point to the last character to avoid index out of bounds
    // ie, i52e3:abc
    //        ^
    //        |
    (end_idx, BEncodedData::Num(num))
}

/// Decode a BEncoded integer
/// i<num>e
fn decode_integer(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    let end_idx = encoded_value.iter().position(|&d| d == b'e').unwrap();
    let num = std::str::from_utf8(&encoded_value[1..end_idx]).unwrap();
    let num = num.parse::<i64>().unwrap();

    (num.into(), &encoded_value[end_idx + 1..])
}

fn d_s<S: AsRef<[u8]>>(encoded_value: S) -> (usize, BEncodedData) {
    let encoded_value = encoded_value.as_ref();
    let delim_idx = encoded_value.iter().position(|&d| d == b':').unwrap();
    let (len, remainder) = {
        encoded_value.split_at(delim_idx)
    };

    let len = std::str::from_utf8(len).unwrap();
    println!("str len {}", len);
    let len = len.parse::<usize>().unwrap();

    println!("str {}, len {}, delim idx {}", BEStr::from_slice(encoded_value), len, delim_idx);
    
    //println!("{:#?}", delim_idx + len);

    // point to the last character to avoid index out of bounds
    // ie, 3:abci52e
    //         ^
    //         |
    (delim_idx + len, BEncodedData::ByteStr(BEStr::from_slice(&remainder[delim_idx.. delim_idx + len])))
}

/// Decode a BEncoded string
/// <len>:<string>
/// No validation of len is done
fn decode_string(encoded_value: &[u8]) -> (serde_json::Value, &[u8]) {
    //println!("{:#?}", encoded_value);
    let (len, remainder) = {
        let delim_idx = encoded_value.iter().position(|&d| d == b':').unwrap();
        encoded_value.split_at(delim_idx)
    };

    let len = std::str::from_utf8(len).unwrap();
    let len = len.parse::<usize>().unwrap();
    let res = String::from_utf8_lossy(&remainder[1..len + 1]);

    (res.into(), &remainder[len + 1..])
}

fn d_l<L: AsRef<[u8]>>(encoded_list: L) -> (usize, BEncodedData) {
    let encoded_list = encoded_list.as_ref();
    let mut vals = Vec::new();
    // strip the leading 'l'
    let mut remainder = encoded_list.split_at(1).1;
    let mut end_idx: usize = 0;
    
    while !remainder.is_empty() && !remainder.starts_with(&[b'e']) {
        let (itr_idx, decoded_value) = d_b_v(remainder);
        vals.push(decoded_value);
        remainder = &remainder[itr_idx + 1..];
        //println!("list ops remainder: {:#?}", remainder);
        end_idx += itr_idx + 1;
    }

    println!("evaluated list {:#?}", String::from_utf8_lossy(encoded_list));
    // println!("list vals: {:#?}", vals);
    println!("vals len: {}, end_idx {}", vals.len(), end_idx);

    return (end_idx + 1, BEncodedData::List(vals))
}

/// Decoded a BEncoded list
/// l<any valid BEncoded values>..<><>e
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

fn d_d<D: AsRef<[u8]>>(encoded_dict: D) -> (usize, BEncodedData) {
    // Strip leading 'd'
    let mut remainder = encoded_dict.as_ref().split_at(1).1;
    let mut dict = BEDict::new();
    let mut end_idx: usize = 0;

    while !remainder.is_empty() && !remainder.starts_with(&[b'e']) {
        println!("init remainder: {:?}", String::from_utf8_lossy(remainder));
        let (first_idx, decoded_value) = d_b_v(remainder);
        let key: BEStr;
        match decoded_value {
            BEncodedData::ByteStr(s) => {
                key = s;
            },
            data => {
                panic!("Dict key is not a string: {data:#?}");
            }
        }
        remainder = remainder.split_at(first_idx + 1).1;
        println!("split 1 remainder: {:?}", String::from_utf8_lossy(remainder));

        let (second_idx, value) = d_b_v(remainder);
        remainder = remainder.split_at(second_idx + 1).1;
        println!("split 2 remainder: {:?}", String::from_utf8_lossy(remainder));

        dict.insert(key, value);
        
        end_idx += first_idx + second_idx;

    }
    
    (end_idx + 1, BEncodedData::Dict(dict))
}

/// Decode a BEncoded dictionary
/// d<b_string><any valid BEncoded value>..<><>e
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