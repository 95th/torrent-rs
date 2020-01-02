use crate::error::*;
use crate::reader::Reader;

use std::collections::BTreeMap;
use std::fmt;
use std::io;

// TODO: See if we can work without allocation
#[derive(Debug, Clone, PartialOrd, PartialEq)]
pub enum ValueRef<'a> {
    Int(i64),
    Bytes(&'a [u8]),
    List(Vec<Self>),
    Dict(BTreeMap<&'a str, Self>),
}

impl<'a> ValueRef<'a> {
    pub fn with_int(v: i64) -> Self {
        Self::Int(v)
    }

    pub fn with_str(s: &'a str) -> Self {
        Self::Bytes(s.as_bytes())
    }

    pub fn with_list(list: Vec<Self>) -> Self {
        Self::List(list)
    }

    pub fn with_dict(dict: BTreeMap<&'a str, Self>) -> Self {
        Self::Dict(dict)
    }

    impl_is_ty! {
        is_string == Bytes,
        is_int == Int,
        is_list == List,
        is_dict == Dict,
    }

    pub fn as_int(&self) -> Option<i64> {
        inner_if!(self == Int).map(|n| *n)
    }

    pub fn as_str(&self) -> Option<&'a str> {
        inner_if!(self == Bytes).and_then(|buf| std::str::from_utf8(buf).ok())
    }

    pub fn as_bytes(&self) -> Option<&'a [u8]> {
        inner_if!(self == Bytes)
    }

    pub fn as_list(&self) -> Option<&[Self]> {
        inner_if!(self == List)
    }

    pub fn as_list_mut(&mut self) -> Option<&mut Vec<Self>> {
        inner_if!(self == List)
    }

    pub fn into_list(self) -> Option<Vec<Self>> {
        inner_if!(self == List)
    }

    pub fn as_dict(&self) -> Option<&BTreeMap<&'a str, Self>> {
        inner_if!(self == Dict)
    }

    pub fn as_dict_mut(&mut self) -> Option<&mut BTreeMap<&'a str, Self>> {
        inner_if!(self == Dict)
    }

    pub fn into_dict(self) -> Option<BTreeMap<&'a str, Self>> {
        inner_if!(self == Dict)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = vec![];
        self.encode(&mut v).unwrap();
        v
    }

    pub fn dict_find(&self, key: &str) -> Option<&Self> {
        let dict = self.as_dict()?;
        dict.get(key)
    }

    pub fn dict_find_int(&self, key: &str) -> Option<&Self> {
        let n = self.dict_find(key)?;
        if n.is_int() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_int_value(&self, key: &str) -> Option<i64> {
        self.dict_find_int(key)?.as_int()
    }

    pub fn dict_find_str(&self, key: &str) -> Option<&Self> {
        let n = self.dict_find(key)?;
        if n.is_string() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_str_value(&self, key: &str) -> Option<&'a str> {
        self.dict_find_str(key)?.as_str()
    }

    pub fn dict_find_list(&self, key: &str) -> Option<&Self> {
        let n = self.dict_find(key)?;
        if n.is_list() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_find_list_value(&self, key: &str) -> Option<&[Self]> {
        self.dict_find_list(key)?.as_list()
    }

    pub fn dict_find_dict(&self, key: &str) -> Option<&Self> {
        let n = self.dict_find(key)?;
        if n.is_dict() {
            Some(n)
        } else {
            None
        }
    }

    pub fn dict_len(&self) -> Option<usize> {
        Some(self.as_dict()?.len())
    }

    pub fn list_at(&self, index: usize) -> Option<&Self> {
        let list = self.as_list()?;
        list.get(index)
    }

    pub fn list_string_value_at(&self, index: usize) -> Option<&'a str> {
        self.list_at(index)?.as_str()
    }

    pub fn list_int_value_at(&self, index: usize) -> Option<i64> {
        self.list_at(index)?.as_int()
    }

    pub fn list_len(&self) -> Option<usize> {
        Some(self.as_list()?.len())
    }

    pub fn encode<W: io::Write>(&'a self, w: &mut W) -> io::Result<()> {
        enum Token<'a> {
            B(&'a ValueRef<'a>),
            S(&'a str),
            E,
        }

        use Token::*;
        let mut stack = vec![B(self)];
        while !stack.is_empty() {
            match stack.pop().unwrap() {
                B(v) => match v {
                    Self::Int(n) => {
                        write!(w, "i{}e", n)?;
                    }
                    Self::Bytes(v) => {
                        write!(w, "{}:", v.len())?;
                        w.write_all(&v)?;
                    }
                    Self::List(v) => {
                        write!(w, "l")?;
                        stack.push(E);
                        stack.extend(v.iter().rev().map(|e| B(e)));
                    }
                    Self::Dict(m) => {
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

    pub fn decode(bytes: &'a [u8]) -> Result<Self> {
        Self::decode_with_limits(bytes, None, None)
    }

    pub fn decode_with_limits(
        bytes: &'a [u8],
        depth_limit: Option<usize>,
        item_limit: Option<usize>,
    ) -> Result<Self> {
        enum Kind {
            Dict(usize),
            List(usize),
        }

        let mut c_stack = vec![];
        let mut v_stack = vec![];
        let mut rdr = Reader::new(bytes);
        let mut items = 0;

        loop {
            match rdr.next_byte() {
                Some(b'e') => match c_stack.pop() {
                    Some(Kind::List(len)) => {
                        let mut vec = Vec::with_capacity(v_stack.len() - len);
                        while v_stack.len() > len {
                            vec.push(v_stack.pop().unwrap());
                        }
                        vec.reverse();
                        v_stack.push(Self::List(vec));
                    }
                    Some(Kind::Dict(len)) => {
                        if (v_stack.len() - len) % 2 != 0 {
                            return Err(Error::ParseDict);
                        }
                        let mut map = BTreeMap::new();
                        while v_stack.len() > len {
                            let val = v_stack.pop().unwrap();
                            if let Some(key) = v_stack.pop().unwrap().as_str() {
                                map.insert(key, val);
                            } else {
                                return Err(Error::ParseDict);
                            }
                        }
                        v_stack.push(Self::Dict(map))
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
                            let len = rdr.read_int_until(b':')?;
                            let value = rdr.read_exact(len as usize)?;
                            v_stack.push(Self::Bytes(value));
                        }
                        b'i' => {
                            let n = rdr.read_int_until(b'e')?;
                            v_stack.push(Self::Int(n))
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

impl<'a> From<&'a [u8]> for ValueRef<'a> {
    fn from(value: &'a [u8]) -> Self {
        Self::Bytes(value)
    }
}

impl fmt::Display for ValueRef<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.to_vec();
        write!(f, "{}", String::from_utf8_lossy(&v))
    }
}
