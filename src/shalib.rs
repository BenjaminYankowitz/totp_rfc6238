pub mod sha1 {
    const HASH_SIZE: usize = 20;
    #[derive(PartialEq, Clone, Copy)]
    struct Context {
        intermediate_hash: [u32; HASH_SIZE / 4], /* Message Digest  */
        length: u64,                             /* Message length in bits      */
        message_block_index: u16,                /* Index into message block array   */
        message_block: [u8; 64],                 /* 512-bit message blocks      */
    }
    fn circular_shift(bits: u8, word: u32) -> u32 {
        let op_bits = 32 - bits;
        let bot_bits = word >> op_bits;
        let top_bits = (word & ((1 << op_bits) - 1)) << bits;
        bot_bits | top_bits
    }
    impl Context {
        pub fn new() -> Self {
            let mut ret = Context {
                intermediate_hash: [0; HASH_SIZE / 4],
                length: 0,
                message_block_index: 0,
                message_block: [0; 64],
            };
            ret.reset();
            ret
        }
        pub fn reset(&mut self) {
            self.intermediate_hash = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476, 0xC3D2E1F0];
            self.length = 0;
        }
        pub fn input(&mut self, message_array: &[u8]) {
            for message in message_array.iter().cloned() {
                self.message_block[self.message_block_index as usize] = message;
                self.message_block_index += 1;
                match self.length.checked_add(8) {
                    Some(v) => self.length = v,
                    None => panic!("message over 2^64 bits"),
                };
                if self.message_block_index == 64 {
                    self.process_message_block();
                }
            }
        }

        fn pad_message(&mut self) {
            //     /*
            //      *  Check to see if the current message block is too small to hold
            //      *  the initial padding bits and length.  If so, we will pad the
            //      *  block, process it, and then continue padding into a second
            //      *  block.
            //      */
            self.message_block[self.message_block_index as usize] = 0x80;
            self.message_block_index += 1;
            if self.message_block_index > 56 {
                self.message_block
                    .split_at_mut(self.message_block_index as usize)
                    .1
                    .fill(0);
                while self.message_block_index < 64 {
                    self.message_block[self.message_block_index as usize] = 0;
                    self.message_block_index += 1;
                }
                self.process_message_block();
            }
            self.message_block
                .split_at_mut(56)
                .0
                .split_at_mut(self.message_block_index as usize)
                .1
                .fill(0);

            //     /*
            //      *  Store the message length as the last 8 octets
            //      */
            for (to, from) in std::iter::zip(
                self.message_block.split_at_mut(56).1,
                self.length.to_be_bytes(),
            ) {
                *to = from;
            }
            self.process_message_block();
        }

        fn process_message_block(&mut self) {
            let k = [0x5A827999, 0x6ED9EBA1, 0x8F1BBCDC, 0xCA62C1D6];
            let f = [
                |b: u32, c, d| (b & c) | ((!b) & d),
                |b, c, d| b ^ c ^ d,
                |b, c, d| (b & c) | (b & d) | (c & d),
                |b, c, d| b ^ c ^ d,
            ];
            let mut w = [0u32; 80];
            for (t, val) in self.message_block.chunks_exact(4).enumerate() {
                w[t] = u32::from_be_bytes(val.try_into().expect("Must be 4 because prev line"));
            }

            for t in 16..80 {
                w[t] = circular_shift(1, w[t - 3] ^ w[t - 8] ^ w[t - 14] ^ w[t - 16]);
            }

            let [mut a, mut b, mut c, mut d, mut e] = self.intermediate_hash;
            for i in 0..4 {
                for w_v in w.iter().take((i + 1) * 20).skip(i * 20) {
                    let temp = circular_shift(5, a)
                        .overflowing_add(f[i](b, c, d))
                        .0
                        .overflowing_add(e)
                        .0
                        .overflowing_add(*w_v)
                        .0
                        .overflowing_add(k[i])
                        .0;
                    e = d;
                    d = c;
                    c = circular_shift(30, b);
                    b = a;
                    a = temp;
                }
            }
            for (to, from) in std::iter::zip(self.intermediate_hash.iter_mut(), [a, b, c, d, e]) {
                *to = to.overflowing_add(from).0;
            }

            self.message_block_index = 0;
        }
        fn result(&mut self) -> [u8; HASH_SIZE] {
                self.pad_message();
                self.message_block.fill(0); /* message may be sensitive, clear it out */
                self.length = 0; /* and clear length */
            
            let mut ret = [0;20];
            for (i, v) in self
                .intermediate_hash
                .map(|v| v.to_be_bytes())
                .iter()
                .flatten()
                .cloned()
                .enumerate()
            {
                ret[i] = v;
            };
            self.reset();
            ret
        }
    }

    pub fn hmac_sha1(key: &[u8], message: &[u8]) -> [u8; 20] {
        let mut context = Context::new();
        let key_buff : [u8;20];
        let key : &[u8] = if key.len() > 64 {
            context.input(key);
            key_buff = context.result();
            &key_buff
        } else {
            key
        };
        let mut ipad = [0x36;64];
        let mut opad = [0x5c;64];
        for (i,byte) in key.iter().enumerate() {
            ipad[i]^=byte;
            opad[i]^=byte;
        }
        context.input(&ipad);
        context.input(message);
        let temp = context.result();
        context.input(&opad);
        context.input(&temp);
        context.result()
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn circularshift_tests() {
            let v1 = !0;
            assert_eq!(circular_shift(5, v1), v1);
            let v2 = 0b1;
            let a2 = 0b100000;
            assert_eq!(circular_shift(5, v2), a2);
            let v2 = 0b1;
            let a2 = 0b100000;
            assert_eq!(circular_shift(5, v2), a2);
            let v3 = 0b01001101000110011011010110000000;
            let a3 = 0b10100011001101101011000000001001;
            assert_eq!(circular_shift(5, v3), a3);
            let v4 = 0b01101110110001011000101101110010;
            let a4 = 0b00101100010110111001001101110110;
            assert_eq!(circular_shift(11, v4), a4);
        }

        use std::fmt::Write;

        #[test]
        fn overall_tests() {
            const TESTARRAY: [&[u8]; 4] = [
                b"abc",
                b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq",
                b"a",
                b"0123456701234567012345670123456701234567012345670123456701234567",
            ];
            const REPEATCOUNT: [i32; 4] = [1, 1, 1000000, 10];
            const RESULTARRAY: [&str; 4] = [
                "A9 99 3E 36 47 06 81 6A BA 3E 25 71 78 50 C2 6C 9C D0 D8 9D ",
                "84 98 3E 44 1C 3B D2 6E BA AE 4A A1 F9 51 29 E5 E5 46 70 F1 ",
                "34 AA 97 3C D4 C4 DA A4 F6 1E EB 2B DB AD 27 31 65 34 01 6F ",
                "DE A3 56 A2 CD DD 90 C7 A7 EC ED C5 EB B5 63 93 4F 46 04 52 ",
            ];
            let mut sha = Context::new();
            for j in 0..4 {
                println!(
                    "\nTest {}: {}, '{}'",
                    j + 1,
                    REPEATCOUNT[j],
                    (str::from_utf8(TESTARRAY[j])).expect("ascii string")
                );

                for _ in 0..REPEATCOUNT[j] {
                    sha.input(TESTARRAY[j]);
                }

                let ret_var = sha.result();
                let mut out_str = String::new();
                for byte in ret_var {
                    write!(&mut out_str, "{:02X} ", byte).expect("Format string valid");
                }
                assert_eq!(out_str, RESULTARRAY[j]);
            }
        }
    }
}
