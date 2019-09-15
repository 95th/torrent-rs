use crate::error::{Error, Result};
use crate::hex;
use crate::str_utl;
use crate::torrent_params::TorrentParams;

use common::sha1::Sha1Hash;
use std::borrow::Cow;

pub fn parse_magnet_uri(uri: &str, p: &mut TorrentParams) -> Result<()> {
    if !uri.starts_with("magnet:?") {
        return Err(Error::UnsupportedUrlProtocol);
    }
    let uri = &uri.as_bytes()[8..];

    let mut display_name = String::new();
    let mut tier = 0;
    while !uri.is_empty() {
        let (mut name, rest) = str_utl::split_string(uri, b'=');
        let (mut value, rest) = str_utl::split_string(rest, b'&');

        let (stripped_name, number) = str_utl::split_string(name, b'.');

        if number.iter().all(|&c| c >= b'0' && c <= b'9') {
            name = stripped_name;
        }

        match name {
            b"dn" /* Display Name */ => {
                display_name = str_utl::unescape_string(value)?;
            }
            b"tr" /* Tracker */ => {
                if p.tracker_tiers.len() != p.trackers.len() {
                    p.tracker_tiers.resize(p.trackers.len(), 0);
                }
                let tracker = str_utl::unescape_string(value)?;
                p.trackers.push(tracker);
                p.tracker_tiers.push(tier);
                tier += 1;
            },
            b"ws" /* Web Seed */ => {
                let weebseed = str_utl::unescape_string(value)?;
                p.url_seeds.push(weebseed);
            },
            b"xt" => {
                let mut value = Cow::Borrowed(value);
                if value.iter().any(|&c| c == b'%') {
                    value = Cow::Owned(str_utl::unescape_bytes(&value)?);
                }

                if !value.starts_with(b"urn:btih:") {
                    continue;
                }

                let value = &value[9..];
                    
                let mut s = Sha1Hash::new();
                match value.len() {
                    40 => {hex::from_hex(value, s.data_mut());},
                 32 => {
                    let ih = str_utl::base32_decode(value);
                    if ih.len() != 20 {
                        return Err(Error::InvalidInfoHash);
                    }
                    s.data_mut().copy_from_slice(&ih);
                }
                _ =>
                    return Err(Error::InvalidInfoHash),
                }
            },
            _ => {}
        }
    }

    Ok(())
}
