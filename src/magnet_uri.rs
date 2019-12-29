use crate::download_priority::DownloadPriority;
use crate::error::{Error, Result};
use crate::params::TorrentParams;
use crate::str_utl;
use common::hex;

use common::sha1::Sha1Hash;
use std::fmt;
use std::str::FromStr;
use url::Url;

impl FromStr for TorrentParams {
    type Err = Error;

    fn from_str(uri: &str) -> Result<Self> {
        let mut params = TorrentParams::default();
        parse_magnet_uri(uri, &mut params)?;
        Ok(params)
    }
}

impl fmt::Display for TorrentParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "magnet:?xt=urn:btih:")?;
        write!(f, "{}", hex::to_hex(&self.info_hash))?;

        Ok(())
    }
}

pub fn parse_magnet_uri(uri: &str, p: &mut TorrentParams) -> Result<()> {
    let url = Url::parse(uri)?;
    if url.scheme() != "magnet" {
        return Err(Error::UnsupportedUrlProtocol);
    }

    let mut tier = 0;
    let mut has_ih = false;
    for (key, value) in url.query_pairs() {
        match &*key {
            /* Display Name */
            "dn" => {
                p.name = value.to_string();
            }
            /* Tracker */
            "tr" => {
                if p.tracker_tiers.len() != p.trackers.len() {
                    p.tracker_tiers.resize(p.trackers.len(), 0);
                }
                p.trackers.push(value.to_string());
                p.tracker_tiers.push(tier);
                tier += 1;
            }
            /* Web Seed */
            "ws" => {
                p.url_seeds.push(value.to_string());
            }
            "xt" => {
                if !value.starts_with("urn:btih:") {
                    continue;
                }

                let value = &value[9..];
                let mut s = Sha1Hash::new();
                match value.len() {
                    40 => {
                        hex::from_hex(value.as_bytes(), &mut s);
                    }
                    32 => {
                        let ih = str_utl::base32_decode(value.as_bytes());
                        if ih.len() != 20 {
                            return Err(Error::InvalidInfoHash);
                        }
                        s.copy_from_slice(&ih);
                    }
                    _ => return Err(Error::InvalidInfoHash),
                }
                p.info_hash = s;
                has_ih = true;
            }
            /* Select-Only (files) */
            "so" => {
                if value
                    .chars()
                    .any(|c| !c.is_digit(10) && c != '-' && c != ',')
                {
                    continue;
                }

                let mut value = value.as_bytes();
                while !value.is_empty() {
                    let (token, rest) = str_utl::split_string(value, b',');
                    if token.is_empty() {
                        continue;
                    }

                    // TODO: What's the right number here?
                    let max_index = 10_000; // Can't risk out of memory

                    let idx1;
                    let idx2;
                    if let Some(divider) = token.iter().position(|&c| c == b'-') {
                        // it's a range

                        if divider == 0 {
                            // No start index
                            continue;
                        }

                        if divider == token.len() - 1 {
                            // No end index
                            continue;
                        }

                        idx1 = str_utl::parse_int(&token[..divider])?;
                        if idx1 < 0 || idx1 > max_index {
                            // Invalid Index
                            continue;
                        }

                        idx2 = str_utl::parse_int(&token[divider + 1..])?;
                        if idx2 < 0 || idx2 > max_index {
                            // Invalid Index
                            continue;
                        }

                        if idx1 > idx2 {
                            // Wrong range limits
                            continue;
                        }
                    } else {
                        // it's an index
                        idx1 = str_utl::parse_int(token)?;
                        if idx1 < 0 || idx1 > max_index {
                            // Invalid index
                            continue;
                        }
                        idx2 = idx1;
                    }

                    if p.file_priorities.len() <= idx2 as usize {
                        p.file_priorities
                            .resize(idx2 as usize + 1, DownloadPriority::DontDownload);
                    }

                    for i in idx1..=idx2 {
                        p.file_priorities[i as usize] = DownloadPriority::DefaultPriority;
                    }

                    value = rest;
                }
            }
            "x.pe" => {
                p.peers.push(value.parse()?);
            }
            "dht" => {
                let value = value.as_bytes();
                let divider = value.iter().rev().position(|&c| c == b':');
                if let Some(n) = divider {
                    let n = value.len() - 1 - n;
                    let host = String::from_utf8(value[..n].to_vec()).unwrap();
                    let port = std::str::from_utf8(&value[n + 1..]).unwrap();
                    p.dht_nodes.push((host, port.parse()?))
                }
            }
            _ => {}
        }
    }

    if !has_ih {
        return Err(Error::MissingInfoHash);
    }

    Ok(())
}
