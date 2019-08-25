use crate::detail;
use bencode::Value;
use std::collections::BTreeMap;
use std::io::{self, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

type NodeId = Vec<u8>;
type NodeIds = Vec<(IpAddr, NodeId)>;

#[derive(Default, Debug)]
pub struct DhtState {
    pub nids: NodeIds,
    pub nodes: Vec<SocketAddr>,
    pub nodes6: Vec<SocketAddr>,
}

impl DhtState {
    pub fn read(value: &Value) -> io::Result<DhtState> {
        let mut state = DhtState::default();
        if let Value::Dict(dict) = value {
            state.nids = extract_node_ids(value, "node-id")?;

            if let Some(Value::List(list)) = dict.get("nodes") {
                state.nodes = read_endpoint_list(list)?;
            }

            if let Some(Value::List(list)) = dict.get("nodes6") {
                state.nodes6 = read_endpoint_list(list)?;
            }
        }
        Ok(state)
    }

    pub fn save(&self) -> Value {
        let mut dict = BTreeMap::new();

        if !self.nids.is_empty() {
            let mut list = vec![];
            for (addr, id) in &self.nids {
                let mut buf = vec![];
                let mut c = io::Cursor::new(&mut buf);
                c.write_all(id).unwrap();
                detail::write_address(&mut c, addr).unwrap();
                list.push(Value::String(buf));
            }
            dict.insert("node-id".to_owned(), Value::with_list(list));
        }

        if !self.nodes.is_empty() {
            dict.insert("nodes".to_owned(), save_nodes(&self.nodes).unwrap());
        }

        if !self.nodes6.is_empty() {
            dict.insert("nodes6".to_owned(), save_nodes(&self.nodes6).unwrap());
        }

        Value::with_map(dict)
    }

    pub fn clear(&mut self) {
        self.nids.clear();
        self.nids.shrink_to_fit();

        self.nodes.clear();
        self.nodes.shrink_to_fit();

        self.nodes6.clear();
        self.nodes6.shrink_to_fit();
    }
}

pub fn extract_node_ids(value: &Value, key: &str) -> io::Result<NodeIds> {
    let mut ids = NodeIds::new();
    let dict = match value {
        Value::Dict(d) => d,
        _ => return Ok(ids),
    };

    if let Some(v) = dict.get(key) {
        if let Ok(old_nid) = v.as_str_bytes() {
            if old_nid.len() == 20 {
                ids.push((IpAddr::V4(Ipv4Addr::LOCALHOST), old_nid.to_vec()));
                return Ok(ids);
            }
        }

        if let Value::List(list) = v {
            for nid in list {
                match nid.as_str_bytes() {
                    Ok(s) if s.len() == 24 || s.len() == 36 => {
                        let (id, addr) = s.split_at(20);
                        let mut c = io::Cursor::new(addr);
                        let addr = if addr.len() == 4 {
                            detail::read_v4_address(&mut c)?
                        } else {
                            detail::read_v6_address(&mut c)?
                        };
                        ids.push((addr, id.to_vec()));
                    }
                    _ => continue,
                };
            }
        }
    }

    Ok(ids)
}

fn read_endpoint_list(values: &[Value]) -> io::Result<Vec<SocketAddr>> {
    let mut list = vec![];
    for v in values {
        match v.as_str_bytes() {
            Ok(s) if s.len() == 6 || s.len() == 18 => {
                let mut c = io::Cursor::new(s);
                let addr = if s.len() == 6 {
                    detail::read_v4_socket_address(&mut c)?
                } else {
                    detail::read_v6_socket_address(&mut c)?
                };
                list.push(addr);
            }
            _ => {}
        }
    }
    Ok(list)
}

fn save_nodes(nodes: &[SocketAddr]) -> io::Result<Value> {
    let mut list = vec![];

    for node in nodes {
        let mut v = vec![];
        let mut c = io::Cursor::new(&mut v);
        detail::write_socket_addr(&mut c, node)?;
        list.push(Value::String(v));
    }

    Ok(Value::with_list(list))
}
