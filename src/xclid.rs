use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

const TXID_EPOCH_SECONDS: i64 = 1_682_924_400;

fn script_url(k: &str, v: &str) -> String {
    format!("https://abs.twimg.com/responsive-web/client-web/{k}.{v}.js")
}

pub fn get_scripts_list(text: &str) -> Result<Vec<String>, String> {
    let mut hash_map = BTreeMap::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i + 10 > bytes.len() || &bytes[i..i + 2] != b":\"" {
            continue;
        }
        i += 2;
        let value_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        let key = &text[start..start + (value_start - start - 2)];
        let value = &text[value_start..i];
        if value.len() == 7
            && value
                .bytes()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
        {
            hash_map.insert(key.to_owned(), value.to_owned());
        }
        i += 1;
    }

    if hash_map.is_empty() {
        return Err("Failed to parse scripts".to_owned());
    }

    let mut name_map = BTreeMap::new();
    let mut i = 0;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i + 2 > bytes.len() || &bytes[i..i + 2] != b":\"" {
            continue;
        }
        i += 2;
        let value_start = i;
        while i < bytes.len() && bytes[i] != b'"' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let key = &text[start..start + (value_start - start - 2)];
        let value = &text[value_start..i];
        let is_hash = value.len() == 7
            && value
                .bytes()
                .all(|ch| ch.is_ascii_digit() || (b'a'..=b'f').contains(&ch));
        if !is_hash {
            name_map.insert(key.to_owned(), value.to_owned());
        }
        i += 1;
    }

    Ok(hash_map
        .into_iter()
        .map(|(chunk_id, hash_val)| {
            let name = name_map.get(&chunk_id).unwrap_or(&chunk_id);
            script_url(name, &(hash_val + "a"))
        })
        .collect())
}

pub fn parse_vk_bytes_from_content(content: &str) -> Result<Vec<u8>, String> {
    STANDARD
        .decode(content.as_bytes())
        .map_err(|_| "Couldn't get XClientTxId key bytes".to_owned())
}

pub fn parse_anim_idx_from_script(text: &str) -> Result<Vec<usize>, String> {
    let bytes = text.as_bytes();
    let mut items = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != b'(' {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        if j >= bytes.len() || !bytes[j].is_ascii_alphanumeric() && bytes[j] != b'_' {
            i += 1;
            continue;
        }
        j += 1;
        if j >= bytes.len() || bytes[j] != b'[' {
            i += 1;
            continue;
        }
        j += 1;
        let num_start = j;
        while j < bytes.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        if num_start == j || j >= bytes.len() || bytes[j] != b']' {
            i += 1;
            continue;
        }
        let idx = text[num_start..j]
            .parse::<usize>()
            .map_err(|err| err.to_string())?;
        j += 1;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b',' {
            i += 1;
            continue;
        }
        j += 1;
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        if j + 3 <= bytes.len() && &bytes[j..j + 3] == b"16)" {
            items.push(idx);
            i = j + 3;
        } else {
            i += 1;
        }
    }

    if items.is_empty() {
        Err("Couldn't get XClientTxId indices".to_owned())
    } else {
        Ok(items)
    }
}

struct Cubic {
    curves: Vec<f64>,
}

impl Cubic {
    fn get_value(&self, time: f64) -> f64 {
        let mut start_gradient = 0.0;
        let mut end_gradient = 0.0;
        let mut start = 0.0;
        let mut mid = 0.0;
        let mut end = 1.0;

        if time <= 0.0 {
            if self.curves[0] > 0.0 {
                start_gradient = self.curves[1] / self.curves[0];
            } else if self.curves[1] == 0.0 && self.curves[2] > 0.0 {
                start_gradient = self.curves[3] / self.curves[2];
            }
            return start_gradient * time;
        }

        if time >= 1.0 {
            if self.curves[2] < 1.0 {
                end_gradient = (self.curves[3] - 1.0) / (self.curves[2] - 1.0);
            } else if self.curves[2] == 1.0 && self.curves[0] < 1.0 {
                end_gradient = (self.curves[1] - 1.0) / (self.curves[0] - 1.0);
            }
            return 1.0 + end_gradient * (time - 1.0);
        }

        while start < end {
            mid = (start + end) / 2.0;
            let x_est = Self::calculate(self.curves[0], self.curves[2], mid);
            if (time - x_est).abs() < 0.00001 {
                return Self::calculate(self.curves[1], self.curves[3], mid);
            }
            if x_est < time {
                start = mid;
            } else {
                end = mid;
            }
        }
        Self::calculate(self.curves[1], self.curves[3], mid)
    }

    fn calculate(a: f64, b: f64, m: f64) -> f64 {
        3.0 * a * (1.0 - m) * (1.0 - m) * m + 3.0 * b * (1.0 - m) * m * m + m * m * m
    }
}

fn interpolate(from_list: &[f64], to_list: &[f64], f: f64) -> Result<Vec<f64>, String> {
    if from_list.len() != to_list.len() {
        return Err(format!(
            "Mismatched interpolation args {from_list:?}: {to_list:?}"
        ));
    }
    Ok(from_list
        .iter()
        .zip(to_list)
        .map(|(a, b)| a * (1.0 - f) + b * f)
        .collect())
}

fn get_rotation_matrix(rotation: f64) -> [f64; 4] {
    let rad = rotation.to_radians();
    [rad.cos(), -rad.sin(), rad.sin(), rad.cos()]
}

fn py_round(value: f64, ndigits: i32) -> f64 {
    let scale = 10_f64.powi(ndigits);
    let value = value * scale;
    let rounded = if value.fract().abs() == 0.5 {
        let floor = value.floor();
        if floor as i64 % 2 == 0 {
            floor
        } else {
            floor + 1.0
        }
    } else {
        value.round()
    };
    rounded / scale
}

fn py_round_int(value: f64) -> i64 {
    if value.fract().abs() == 0.5 {
        let floor = value.floor() as i64;
        if floor % 2 == 0 { floor } else { floor + 1 }
    } else {
        value.round() as i64
    }
}

fn solve(value: f64, min_val: f64, max_val: f64, rounding: bool) -> f64 {
    let result = value * (max_val - min_val) / 255.0 + min_val;
    if rounding {
        result.floor()
    } else {
        py_round(result, 2)
    }
}

fn float_to_hex(mut x: f64) -> String {
    let mut result = Vec::new();
    let mut quotient = x as i64;
    let mut fraction = x - quotient as f64;

    while quotient > 0 {
        quotient = (x / 16.0) as i64;
        let remainder = (x - (quotient as f64 * 16.0)) as u8;
        result.insert(
            0,
            if remainder > 9 {
                (remainder + 55) as char
            } else {
                (b'0' + remainder) as char
            },
        );
        x = quotient as f64;
    }

    if fraction == 0.0 {
        return result.into_iter().collect();
    }

    result.push('.');
    while fraction > 0.0 {
        fraction *= 16.0;
        let integer = fraction as u8;
        fraction -= integer as f64;
        result.push(if integer > 9 {
            (integer + 55) as char
        } else {
            (b'0' + integer) as char
        });
    }
    result.into_iter().collect()
}

pub fn cacl_anim_key(frames: Vec<f64>, target_time: f64) -> Result<String, String> {
    if frames.len() < 11 {
        return Err("frames must contain at least 11 values".to_owned());
    }

    let from_color = [frames[0], frames[1], frames[2], 1.0];
    let to_color = [frames[3], frames[4], frames[5], 1.0];
    let from_rotation = [0.0];
    let to_rotation = [solve(frames[6], 60.0, 360.0, true)];
    let curves = frames[7..]
        .iter()
        .enumerate()
        .map(|(i, value)| solve(*value, if i % 2 == 0 { 0.0 } else { -1.0 }, 1.0, false))
        .collect::<Vec<_>>();
    let val = Cubic { curves }.get_value(target_time);

    let color = interpolate(&from_color, &to_color, val)?
        .into_iter()
        .map(|value| value.clamp(0.0, 255.0))
        .collect::<Vec<_>>();
    let rotation = interpolate(&from_rotation, &to_rotation, val)?;

    let matrix = get_rotation_matrix(rotation[0]);
    let mut str_arr = color[..3]
        .iter()
        .map(|value| format!("{:x}", py_round_int(*value)))
        .collect::<Vec<_>>();

    for value in matrix {
        let mut rounded = py_round(value, 2);
        if rounded < 0.0 {
            rounded = -rounded;
        }
        let hex_value = float_to_hex(rounded);
        let item = if hex_value.starts_with('.') {
            format!("0{hex_value}").to_lowercase()
        } else if !hex_value.is_empty() {
            hex_value
        } else {
            "0".to_owned()
        };
        str_arr.push(item);
    }

    str_arr.extend(["0".to_owned(), "0".to_owned()]);
    Ok(str_arr.concat().replace(['.', '-'], ""))
}

pub fn parse_anim_arr_from_paths(
    paths: Vec<String>,
    vk_bytes: Vec<u8>,
) -> Result<Vec<Vec<f64>>, String> {
    if paths.is_empty() {
        return Err("Couldn't get XClientTxId animation array".to_owned());
    }
    if vk_bytes.len() <= 5 {
        return Err("Couldn't get XClientTxId key bytes".to_owned());
    }

    let idx = vk_bytes[5] as usize % paths.len();
    let path = paths[idx].trim();
    let data = path
        .get(9..)
        .ok_or_else(|| "Couldn't get XClientTxId animation array".to_owned())?;

    let mut rows = Vec::new();
    for part in data.split('C') {
        let mut row = Vec::new();
        let mut cur = String::new();
        for ch in part.chars() {
            if ch.is_ascii_digit() {
                cur.push(ch);
            } else if !cur.is_empty() {
                row.push(cur.parse::<f64>().map_err(|err| err.to_string())?);
                cur.clear();
            }
        }
        if !cur.is_empty() {
            row.push(cur.parse::<f64>().map_err(|err| err.to_string())?);
        }
        rows.push(row);
    }
    Ok(rows)
}

pub fn calc_anim_key_from_indices(
    vk_bytes: Vec<u8>,
    anim_idx: Vec<usize>,
    anim_arr: Vec<Vec<f64>>,
) -> Result<String, String> {
    if anim_idx.is_empty() || anim_arr.is_empty() {
        return Err("Couldn't get XClientTxId animation data".to_owned());
    }

    let mut frame_time = 1_i64;
    for idx in anim_idx.iter().skip(1) {
        let byte = vk_bytes
            .get(*idx)
            .ok_or_else(|| "XClientTxId index out of range".to_owned())?;
        frame_time *= (*byte as i64) % 16;
    }
    frame_time = ((frame_time as f64 / 10.0) + 0.5).floor() as i64 * 10;

    let frame_idx_byte = vk_bytes
        .get(anim_idx[0])
        .ok_or_else(|| "XClientTxId index out of range".to_owned())?;
    let frame_idx = (*frame_idx_byte as usize) % 16;
    let frame_row = anim_arr
        .get(frame_idx)
        .ok_or_else(|| "XClientTxId frame index out of range".to_owned())?;
    cacl_anim_key(frame_row.clone(), frame_time as f64 / 4096.0)
}

pub fn xclid_calc(
    vk_bytes: &[u8],
    anim_key: &str,
    method: &str,
    path: &str,
) -> Result<String, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| err.to_string())?
        .as_millis() as i64;
    let ts = ((now - TXID_EPOCH_SECONDS * 1000) / 1000) as u32;
    let ts_bytes = (0..4)
        .map(|i| ((ts >> (i * 8)) & 0xFF) as u8)
        .collect::<Vec<_>>();

    let payload = format!(
        "{}!{path}!{ts}{}{}",
        method.to_uppercase(),
        "obfiowerehiring",
        anim_key
    );
    let digest = Sha256::digest(payload.as_bytes());

    let mut pld = Vec::with_capacity(vk_bytes.len() + 21);
    pld.extend(vk_bytes);
    pld.extend(ts_bytes);
    pld.extend(&digest[..16]);
    pld.push(3);

    let num = rand::rng().random_range(0..=255);
    let mut out = Vec::with_capacity(pld.len() + 1);
    out.push(num);
    out.extend(pld.into_iter().map(|x| x ^ num));
    Ok(STANDARD.encode(out).trim_end_matches('=').to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_list_uses_name_map_when_present() {
        let text = r#"123:"abcdef1",456:"ondemand.s",456:"123abcd""#;
        let scripts = get_scripts_list(text).unwrap();
        assert!(scripts.contains(
            &"https://abs.twimg.com/responsive-web/client-web/ondemand.s.123abcda.js".to_owned()
        ));
    }

    #[test]
    fn anim_key_is_stable_for_sample_frame() {
        let frames = vec![
            0.0, 64.0, 128.0, 255.0, 128.0, 32.0, 200.0, 1.0, 64.0, 128.0, 255.0,
        ];
        assert_eq!(
            cacl_anim_key(frames, 0.25).unwrap(),
            "354d6c07ae147ae147ae0e147ae147ae1480e147ae147ae14807ae147ae147ae00"
        );
    }
}
