use crate::error::{Error, Result};

use std::collections::BTreeMap;
use std::fmt;
use std::io::{self, Cursor};

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    String(Vec<u8>),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

impl Value {
    pub fn with_int(v: i64) -> Value {
        Value::Int(v)
    }

    pub fn with_str(s: &str) -> Value {
        Value::String(s.as_bytes().to_vec())
    }

    pub fn with_string(s: String) -> Value {
        Value::String(s.into_bytes())
    }

    pub fn with_list(list: Vec<Value>) -> Value {
        Value::List(list)
    }

    pub fn with_dict(map: BTreeMap<String, Value>) -> Value {
        Value::Dict(map)
    }

    pub fn is_string(&self) -> bool {
        if let Value::String(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_int(&self) -> bool {
        if let Value::Int(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_list(&self) -> bool {
        if let Value::List(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_dict(&self) -> bool {
        if let Value::Dict(_) = self {
            true
        } else {
            false
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(buf) => std::str::from_utf8(buf).ok(),
            _ => None,
        }
    }

    pub fn as_str_bytes(&self) -> Option<&[u8]> {
        match self {
            Value::String(buf) => Some(buf),
            _ => None,
        }
    }

    pub fn as_list(&self) -> Option<&[Value]> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<Vec<Value>> {
        match self {
            Value::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&BTreeMap<String, Value>> {
        match self {
            Value::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn as_dict_mut(&mut self) -> Option<&mut BTreeMap<String, Value>> {
        match self {
            Value::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn into_dict(self) -> Option<BTreeMap<String, Value>> {
        match self {
            Value::Dict(dict) => Some(dict),
            _ => None,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = vec![];
        self.encode(&mut v).unwrap();
        v
    }

    fn into_string(self) -> Option<String> {
        if let Value::String(v) = self {
            String::from_utf8(v).ok()
        } else {
            None
        }
    }

    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
        enum Token<'a> {
            B(&'a Value),
            S(&'a str),
            E,
        }

        use Token::*;
        use Value::*;
        let mut stack = vec![B(self)];
        while !stack.is_empty() {
            match stack.pop().unwrap() {
                B(v) => match v {
                    Int(n) => {
                        write!(w, "i{}e", n)?;
                    }
                    String(v) => {
                        write!(w, "{}:", v.len())?;
                        w.write_all(&v)?;
                    }
                    List(v) => {
                        write!(w, "l")?;
                        stack.push(E);
                        stack.extend(v.iter().rev().map(|e| B(e)));
                    }
                    Dict(m) => {
                        write!(w, "d")?;
                        stack.push(E);
                        for (k, v) in m.iter().rev() {
                            stack.push(B(v));
                            stack.push(S(k));
                        }
                    }
                },
                S(s) => {
                    write!(w, "{}:{}", s.len(), s)?;
                }
                E => {
                    write!(w, "e")?;
                }
            }
        }
        Ok(())
    }

    pub fn decode<R: io::Read>(bytes: &mut R) -> Result<Value> {
        #[derive(Debug)]
        enum Kind {
            Dict(usize),
            List(usize),
        }

        let mut c_stack = vec![];
        let mut v_stack = vec![];
        let mut buf = [0];
        loop {
            println!("e: {:?} {:?}", c_stack, v_stack);
            match next_byte(bytes, &mut buf) {
                Ok(b'e') => match c_stack.pop() {
                    Some(Kind::List(len)) => {
                        let mut vec = Vec::with_capacity(v_stack.len() - len);
                        while v_stack.len() > len {
                            vec.push(v_stack.pop().unwrap());
                        }
                        vec.reverse();
                        v_stack.push(Value::List(vec));
                    }
                    Some(Kind::Dict(len)) => {
                        if (v_stack.len() - len) % 2 != 0 {
                            return Err(Error::ParseDict);
                        }
                        let mut map = BTreeMap::new();
                        while v_stack.len() > len {
                            let val = v_stack.pop().unwrap();
                            if let Some(key) = v_stack.pop().and_then(|v| v.into_string()) {
                                map.insert(key, val);
                            } else {
                                return Err(Error::ParseDict);
                            }
                        }
                        v_stack.push(Value::Dict(map))
                    }
                    None => return Err(Error::InvalidChar(b'e')),
                },
                Ok(v) => {
                    if c_stack.is_empty() && !v_stack.is_empty() {
                        return Err(Error::EOF);
                    }
                    match v {
                        d @ b'0'..=b'9' => {
                            let mut value = read_until(bytes, b':', &mut buf)?;
                            value.insert(0, d);
                            let len = to_int(&value)?;
                            let mut v = vec![0u8; len as usize];
                            bytes.read_exact(&mut v).map_err(|_| Error::EOF)?;
                            v_stack.push(Value::String(v));
                        }
                        b'i' => {
                            let bytes = read_until(bytes, b'e', &mut buf)?;
                            v_stack.push(Value::Int(to_int(&bytes)?))
                        }
                        b'l' => c_stack.push(Kind::List(v_stack.len())),
                        b'd' => c_stack.push(Kind::Dict(v_stack.len())),
                        c => return Err(Error::InvalidChar(c)),
                    }
                }
                Err(Error::EOF) => break,
                Err(e) => return Err(e),
            }
        }

        if c_stack.is_empty() && v_stack.len() == 1 {
            Ok(v_stack.into_iter().next().unwrap())
        } else {
            Err(Error::EOF)
        }
    }
}

impl std::str::FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Value> {
        let mut c = Cursor::new(s);
        Value::decode(&mut c)
    }
}

impl From<&[u8]> for Value {
    fn from(value: &[u8]) -> Value {
        Value::String(value.to_vec())
    }
}

impl From<Vec<u8>> for Value {
    fn from(value: Vec<u8>) -> Value {
        Value::String(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.to_vec()))
    }
}

fn next_byte<R: io::Read>(r: &mut R, buf: &mut [u8; 1]) -> Result<u8> {
    let amt = r.read(buf).map_err(|_| Error::IO)?;
    if amt == 0 {
        Err(Error::EOF)
    } else {
        Ok(buf[0])
    }
}

fn read_until<R: io::Read>(r: &mut R, stop: u8, buf: &mut [u8; 1]) -> Result<Vec<u8>> {
    let mut v = vec![];
    loop {
        let b = next_byte(r, buf)?;
        if b == stop {
            return Ok(v);
        }
        v.push(b)
    }
}

fn to_int(b: &[u8]) -> Result<i64> {
    std::str::from_utf8(b)
        .map_err(|_| Error::ParseInt)
        .and_then(|s| s.parse().map_err(|_| Error::ParseInt))
}
