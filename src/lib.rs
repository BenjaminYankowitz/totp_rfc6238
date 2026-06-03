use std::{
    fmt,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
mod inplace_veclib;

pub struct Key {
    data: Vec<u8>,
}

impl Key {
    pub fn from_base32(key: &str) -> Option<Self> {
        let key: String = key
            .chars()
            .filter(|c| c.is_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .collect();
        let mut key_u8 = Vec::with_capacity(key.len().div_ceil(8) * 5);
        for slice in key.as_bytes().chunks(8) {
            let mut val: u64 = 0;
            for c in slice.iter().cloned() {
                val *= 32;
                val += if c.is_ascii_uppercase() {
                    c - b'A'
                } else if (b'2'..=b'7').contains(&c) {
                    c - b'2' + 26
                } else {
                    return None;
                } as u64;
            }
            val <<= 24;
            for byte in val.to_be_bytes().iter().take(5).cloned() {
                key_u8.push(byte);
            }
        }
        Some(Key { data: key_u8 })
    }
}

#[derive(Clone, Copy)]
pub struct Config {
    t0: SystemTime,
    time_step: Duration,
    num_digits: u8,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            t0: UNIX_EPOCH,
            time_step: Duration::from_secs(30),
            num_digits: 6,
        }
    }
}

#[derive(Clone, Copy)]
pub struct OneTimeCode {
    code: u32,
    num_digits: u8,
    start_valid: SystemTime,
    end_valid: SystemTime,
}

impl OneTimeCode {
    pub fn new(code: u32, index: u64, config: Config) -> Self {
        let start_valid =
            config.t0 + Duration::from_nanos_u128(config.time_step.as_nanos() * index as u128);
        let end_valid = start_valid + config.time_step;
        OneTimeCode {
            code,
            num_digits: config.num_digits,
            start_valid,
            end_valid,
        }
    }
    pub fn code(&self) -> u32 {
        self.code % ((10_u32).pow(self.num_digits as u32))
    }
    pub fn is_valid(&self) -> bool {
        let time = SystemTime::now();
        time >= self.start_valid && time < self.end_valid
    }
}

impl fmt::Display for OneTimeCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:0width$}",
            self.code(),
            width = self.num_digits as usize
        )
    }
}

fn four_bytes(data: &[u8; 20]) -> u32 {
    let offset_val = (data[19] % 16) as usize;
    let val = u32::from_be_bytes(
        data.as_slice()
            .split_at(offset_val)
            .1
            .split_at(4)
            .0
            .try_into()
            .expect("we split off 4 bytes"),
    );
    val & (!(1 << 31))
}

mod shalib;

pub use crate::shalib::sha1;

pub fn hotp_raw(key: &Key, step: u64) -> u32 {
    let hash = sha1::hmac_sha1(key.data.as_slice(), &step.to_be_bytes());
    four_bytes(&hash)
}

pub fn time_to_index(time: SystemTime, config: Config) -> Option<u64> {
    let time_passed = time.duration_since(config.t0).ok()?;
    u64::try_from(time_passed.as_nanos() / config.time_step.as_nanos()).ok()
}

pub fn totp_time(key: &Key, config: Config, time: SystemTime) -> Option<OneTimeCode> {
    let time_index = time_to_index(time, config)?;
    Some(OneTimeCode::new(
        hotp_raw(key, time_index),
        time_index,
        config,
    ))
}

pub fn totp_now(key: &Key, config: Config) -> Option<OneTimeCode> {
    totp_time(key, config, SystemTime::now())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_check() {
        let key = vec![
            0, 134, 93, 112, 160, 120, 57, 160, 28, 223, 122, 248, 131, 152, 11, 215, 32, 144, 181,
            156,
        ];
        let config = Config::default();
        let time = SystemTime::UNIX_EPOCH + Duration::from_nanos_u128(1780439950593396161);
        let v1 = totp_time(&Key { data: key }, config, time).unwrap();
        assert_eq!(v1.code(), 491933);
    }
}
