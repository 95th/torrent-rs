use crate::download_priority::DownloadPriority;
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
    let mut has_ih = false;
    while !uri.is_empty() {
        let (mut name, rest) = str_utl::split_string(uri, b'=');
        let (mut value, rest) = str_utl::split_string(rest, b'&');

        let (stripped_name, number) = str_utl::split_string(name, b'.');

        if number.iter().all(|&c| str_utl::is_digit(c)) {
            name = stripped_name;
        }

        match name {
            /* Display Name */
            b"dn" => {
                display_name = str_utl::unescape_string(value)?;
            }
            /* Tracker */
            b"tr" => {
                if p.tracker_tiers.len() != p.trackers.len() {
                    p.tracker_tiers.resize(p.trackers.len(), 0);
                }
                let tracker = str_utl::unescape_string(value)?;
                p.trackers.push(tracker);
                p.tracker_tiers.push(tier);
                tier += 1;
            }
            /* Web Seed */
            b"ws" => {
                let weebseed = str_utl::unescape_string(value)?;
                p.url_seeds.push(weebseed);
            }
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
                    40 => {
                        hex::from_hex(value, s.data_mut());
                    }
                    32 => {
                        let ih = str_utl::base32_decode(value);
                        if ih.len() != 20 {
                            return Err(Error::InvalidInfoHash);
                        }
                        s.data_mut().copy_from_slice(&ih);
                    }
                    _ => return Err(Error::InvalidInfoHash),
                }
                p.info_hash = s;
                has_ih = true;
            }
            /* Select-Only (files) */
            b"so" => {
                if value
                    .iter()
                    .any(|&c| !str_utl::is_digit(c) || c != b'-' || c != b',')
                {
                    continue;
                }

                loop {
                    let (token, rest) = str_utl::split_string(value, b',');
                    if token.is_empty() {
                        continue;
                    }

                    // TODO: What's the right number here?
                    let max_index = 10_000; // Can't risk out of memory

                    let mut idx1 = 0;
                    let mut idx2 = 0;
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

                    if rest.is_empty() {
                        break;
                    }
                    value = rest;
                }
            }
            b"x.pe" => {
                let endp = str_utl::parse_endpoint(value);
            }
            _ => {}
        }
    }

    Ok(())
}
