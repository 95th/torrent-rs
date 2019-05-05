use std::collections::BTreeMap;
use std::io;

#[derive(Debug)]
pub enum Error {
    IO,
    EOF,
    ParseInt,
    ParseBytes,
    ParseString,
    ParseList,
    ParseDict,
    InvalidChar(u8)
}

#[derive(Debug)]
pub enum Value {
    Int(i64),
    String(Vec<u8>),
    List(Vec<Value>),
    Dict(BTreeMap<String, Value>),
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Int(n as i64)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.bytes().collect())
    }
}

impl From<Vec<&str>> for Value {
    fn from(v: Vec<&str>) -> Self {
        Value::List(v.into_iter().map(|s| s.into()).collect())
    }
}

type Result<T> = std::result::Result<T, Error>;

pub trait TryConvert<T> {
    fn convert(self) -> Result<T>;
}

impl TryConvert<i64> for Value {
    fn convert(self) -> Result<i64> {
        match self {
            Value::Int(n) => Ok(n),
            _ => Err(Error::ParseInt)
        }
    }
}

impl TryConvert<Vec<u8>> for Value {
    fn convert(self) -> Result<Vec<u8>> {
        match self {
            Value::String(v) => Ok(v),
            _ => Err(Error::ParseBytes)
        }
    }
}

impl TryConvert<String> for Value {
    fn convert(self) -> Result<String> {
        match self {
            Value::String(v) => String::from_utf8(v).map_err(|_| Error::ParseString),
            _ => Err(Error::ParseString)
        }
    }
}

impl TryConvert<Vec<Value>> for Value {
    fn convert(self) -> Result<Vec<Value>> {
        match self {
            Value::List(v) => Ok(v),
            _ => Err(Error::ParseList)
        }
    }
}

impl TryConvert<BTreeMap<String, Value>> for Value {
    fn convert(self) -> Result<BTreeMap<String, Value>> {
        match self {
            Value::Dict(m) => Ok(m),
            _ => Err(Error::ParseDict)
        }
    }
}

enum Token<'a> {
    B(&'a Value),
    S(&'a str),
    E
}

impl Value {
    pub fn encode<W>(&self, w: &mut W) -> io::Result<()>
        where W: io::Write {
        use Token::*;
        use Value::*;
        let mut stack = vec![B(self)];
        while !stack.is_empty() {
            match stack.pop().unwrap() {
                B(v) => match v {
                    Int(n) => {
                        write!(w, "i{}e", n)?;
                    },
                    String(v) => {
                        write!(w, "{}:", v.len())?;
                        w.write_all(&v)?;
                    },
                    List(v) => {
                        write!(w, "l")?;
                        stack.push(E);
                        stack.extend(v.iter().rev().map(|e| B(e)));
                    },
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
                },
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
            List(usize)
        }

        let mut cstack = vec![];
        let mut vstack = vec![];
        loop {
            match next_byte(bytes) {
                Ok(b'e') => match cstack.pop() {
                    Some(Kind::List(len)) => {
                        let mut vec = Vec::with_capacity(vstack.len() - len);
                        while vstack.len() > len {
                            vec.push(vstack.pop().unwrap());
                        }
                        vec.reverse();
                        vstack.push(Value::List(vec));
                    },
                    Some(Kind::Dict(len)) => {
                        if (vstack.len() - len) % 2 != 0 {
                            return Err(Error::ParseDict);
                        }
                        let mut map = BTreeMap::new();
                        while vstack.len() > len {
                            let val = vstack.pop().unwrap();
                            if let Some(key) = vstack.pop().and_then(|v| v.convert().ok()) {
                                map.insert(key, val);
                            } else {
                                return Err(Error::ParseDict);
                            }
                        }
                        vstack.push(Value::Dict(map))
                    },
                    None => return Err(Error::InvalidChar(b'e'))
                },
                Ok(v) => {
                    if cstack.is_empty() && !vstack.is_empty() {
                        return Err(Error::EOF);
                    }
                    match v {
                        d @ b'0'...b'9' => {
                            let mut value = read_until(bytes, b':')?;
                            value.insert(0, d);
                            let len = value.convert()?;
                            let mut v = vec![0u8; len as usize];
                            bytes.read_exact(&mut v).map_err(|_| Error::EOF)?;
                            vstack.push(Value::String(v));
                        }
                        b'i' => vstack.push(Value::Int(read_until(bytes, b'e')?.convert()?)),
                        b'l' => cstack.push(Kind::List(vstack.len())),
                        b'd' => cstack.push(Kind::Dict(vstack.len())),
                        c => return Err(Error::InvalidChar(c))
                    }
                },
                Err(Error::EOF) => break,
                Err(e) => return Err(e),
            }
        }

        if cstack.is_empty() && vstack.len() == 1 {
            Ok(vstack.into_iter().next().unwrap())
        } else {
            Err(Error::EOF)
        }
    }
}

fn next_byte<R: io::Read>(r: &mut R) -> Result<u8> {
    let mut v = [0];
    let amt = r.read(&mut v).map_err(|_| Error::IO)?;
    if amt == 0 {
        Err(Error::EOF)
    } else {
        Ok(v[0])
    }
}

fn read_until<R: io::Read>(r: &mut R, stop: u8) -> Result<Vec<u8>> {
    let mut v = vec![];
    loop {
        let b = next_byte(r)?;
        if b == stop {
            return Ok(v)
        }
        v.push(b)
    }
}

impl TryConvert<i64> for Vec<u8> {
    fn convert(self) -> Result<i64> {
        String::from_utf8(self)
            .map_err(|_| Error::ParseString)
            .and_then(|i| i.parse().map_err(|_| Error::ParseInt))
    }
}