use error::{Error, Result};
use std::collections::BTreeMap;
use std::fmt;
use std::io;
use std::io::Cursor;

pub mod error;

#[derive(Debug)]
pub enum Value {
    Int(i64),
    String(Vec<u8>),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

impl Value {
    pub fn as_int(&self) -> Result<i64> {
        if let Value::Int(n) = self {
            Ok(*n)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::Int, Got: {}",
                self
            )))
        }
    }

    pub fn as_str(&self) -> Result<&str> {
        if let Value::String(v) = self {
            std::str::from_utf8(v).map_err(|_| Error::ParseString)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::String, Got: {}",
                self
            )))
        }
    }

    pub fn as_list(&self) -> Result<&[Value]> {
        if let Value::List(list) = self {
            Ok(list)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::List, Got: {}",
                self
            )))
        }
    }

    pub fn as_map(&self) -> Result<&BTreeMap<String, Value>> {
        if let Value::Dict(map) = self {
            Ok(map)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::Dict, Got: {}",
                self
            )))
        }
    }

    pub fn into_string(self) -> Result<String> {
        if let Value::String(v) = self {
            String::from_utf8(v).map_err(|_| Error::ParseString)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::String, Got: {}",
                self
            )))
        }
    }

    pub fn into_list(self) -> Result<Vec<Value>> {
        if let Value::List(list) = self {
            Ok(list)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::List, Got: {}",
                self
            )))
        }
    }

    pub fn into_map(self) -> Result<BTreeMap<String, Value>> {
        if let Value::Dict(map) = self {
            Ok(map)
        } else {
            Err(Error::IncorrectType(format!(
                "Expected: Value::Dict, Got: {}",
                self
            )))
        }
    }

    pub fn with_int(v: i64) -> Value {
        Value::Int(v)
    }

    pub fn with_string(s: String) -> Value {
        Value::String(s.into_bytes())
    }

    pub fn with_str(s: &str) -> Value {
        Value::String(s.as_bytes().iter().copied().collect())
    }

    pub fn with_list(list: Vec<Value>) -> Value {
        Value::List(list)
    }

    pub fn with_map(map: BTreeMap<String, Value>) -> Value {
        Value::Dict(map)
    }
}

impl std::str::FromStr for Value {
    type Err = Error;

    fn from_str(s: &str) -> Result<Value> {
        let mut c = Cursor::new(s);
        Value::decode(&mut c)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut v = vec![];
        self.encode(&mut v).unwrap();
        write!(f, "{}", std::str::from_utf8(&v).unwrap())
    }
}

enum Token<'a> {
    B(&'a Value),
    S(&'a str),
    E,
}

impl Value {
    pub fn encode<W: io::Write>(&self, w: &mut W) -> io::Result<()> {
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
                        for (k, v) in m {
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
        enum Kind {
            Dict(usize),
            List(usize),
        }

        let mut c_stack = vec![];
        let mut v_stack = vec![];
        let mut buf = [0];
        loop {
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
                            if let Some(key) = v_stack.pop().and_then(|v| v.into_string().ok()) {
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
                            let len = from_bytes(&value)?;
                            let mut v = vec![0u8; len as usize];
                            bytes.read_exact(&mut v).map_err(|_| Error::EOF)?;
                            v_stack.push(Value::String(v));
                        }
                        b'i' => {
                            let bytes = read_until(bytes, b'e', &mut buf)?;
                            v_stack.push(Value::Int(from_bytes(&bytes)?))
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

fn from_bytes(b: &[u8]) -> Result<i64> {
    std::str::from_utf8(b)
        .map_err(|_| Error::ParseInt)
        .and_then(|s| s.parse().map_err(|_| Error::ParseInt))
}
