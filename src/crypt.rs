use std::str::from_utf8;

use serde_json;

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
        let (key, v_res) = decode_bencoded_value(remainder);
        let key = match key {
            serde_json::Value::String(key) => key,
            key => {
                panic!("Dict key is not a string: {key:#?}");
            }
        };
        let (val, rest) = decode_bencoded_value(v_res);
        dict.insert(key, val);
        remainder = rest;
    }
    //println!("{dict:#?}");
    return(dict.into(), &remainder[1..])
}