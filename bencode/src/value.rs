use crate::error::{Error, Result};
use crate::util::{to_int, Reader};

use std::collections::BTreeMap;
use std::fmt;
use std::io;

#[derive(Debug, Clone, PartialOrd, PartialEq)]
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

    pub fn dict_find_int(&self, key: &str) -> Option<&Value> {
        let dict = self.as_dict()?;
        let n = dict.get(key)?;
        if n.is_int() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_int_value(&self, key: &str) -> Option<i64> {
        self.dict_find_int(key)?.as_int()
    }

    pub fn dict_find_str(&self, key: &str) -> Option<&Value> {
        let dict = self.as_dict()?;
        let n = dict.get(key)?;
        if n.is_string() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_str_value(&self, key: &str) -> Option<&str> {
        self.dict_find_str(key)?.as_str()
    }

    pub fn dict_find_list(&self, key: &str) -> Option<&Value> {
        let dict = self.as_dict()?;
        let n = dict.get(key)?;
        if n.is_list() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_list_value(&self, key: &str) -> Option<&[Value]> {
        self.dict_find_list(key)?.as_list()
    }

    pub fn dict_find_dict(&self, key: &str) -> Option<&Value> {
        let dict = self.as_dict()?;
        let n = dict.get(key)?;
        if n.is_dict() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_len(&self) -> Option<usize> {
        Some(self.as_dict()?.len())
    }

    pub fn list_at(&self, index: usize) -> Option<&Value> {
        let list = self.as_list()?;
        list.get(index)
    }

    pub fn list_string_value_at(&self, index: usize) -> Option<&str> {
        self.list_at(index)?.as_str()
    }

    pub fn list_int_value_at(&self, index: usize) -> Option<i64> {
        self.list_at(index)?.as_int()
    }

    pub fn list_len(&self) -> Option<usize> {
        Some(self.as_list()?.len())
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

    pub fn decode(bytes: &[u8]) -> Result<Value> {
        Self::decode_with_limits(bytes, None, None)
    }

    pub fn decode_with_limits(
        bytes: &[u8],
        depth_limit: Option<usize>,
        item_limit: Option<usize>,
    ) -> Result<Value> {
        #[derive(Debug)]
        enum Kind {
            Dict(usize),
            List(usize),
        }

        let mut c_stack = vec![];
        let mut v_stack = vec![];
        let mut items = 0;
        let mut rdr = Reader::new(bytes);

        loop {
            match rdr.next_byte() {
                Some(b'e') => match c_stack.pop() {
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
                Some(v) => {
                    if c_stack.is_empty() && !v_stack.is_empty() {
                        return Err(Error::EOF);
                    }

                    match depth_limit {
                        Some(limit) if c_stack.len() > limit => return Err(Error::DepthLimit),
                        _ => {}
                    }

                    match item_limit {
                        Some(limit) if items > limit => return Err(Error::ItemLimit),
                        _ => items += 1,
                    }

                    match v {
                        _d @ b'0'..=b'9' => {
                            rdr.move_back();
                            let len = to_int(rdr.read_until(b':')?)?;
                            let value = rdr.read_exact(len as usize)?;
                            v_stack.push(Value::String(value.to_vec()));
                        }
                        b'i' => {
                            let n = to_int(rdr.read_until(b'e')?)?;
                            v_stack.push(Value::Int(n))
                        }
                        b'l' => c_stack.push(Kind::List(v_stack.len())),
                        b'd' => c_stack.push(Kind::Dict(v_stack.len())),
                        c => return Err(Error::InvalidChar(c)),
                    }
                }
                None => break,
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
        Value::decode(s.as_bytes())
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
