
pub fn read_frame(buffer: &mut Vec<u8>) -> Option<(u8, Option<Vec<u8>>)> {
    if buffer.len() > 2 {
        // TODO: implement continuous message using fin 1/0
        // let fin = (buffer[0] & 0b10000000) != 0;

        let opcode = buffer[0] & 0x0f;

        if opcode == 8 {
            return Some((opcode, None));
        }

        if opcode == 1 || opcode == 2 {
            let mask = (buffer[1] & 0b10000000) != 0;

            let mut payload_len = (buffer[1] & 0b01111111) as usize;
            let mut offset = 2;

            if payload_len == 126 {
                if buffer.len() < offset + 2 {
                    return None;
                }

                let mut len_buf = [0; 2];
                len_buf.copy_from_slice(&buffer[offset..offset + 2]);

                payload_len = u16::from_be_bytes(len_buf) as usize;
                offset += 2;
            } else if payload_len == 127 {
                if buffer.len() < offset + 8 {
                    return None;
                }

                let mut len_buf = [0; 8];
                len_buf.copy_from_slice(&buffer[offset..offset + 8]);

                payload_len = u64::from_be_bytes(len_buf) as usize;
                offset += 8;
            }

            if buffer.len() < offset + 4 {
                return None;
            }

            let mut masking_key = [0; 4];

            if mask {
                masking_key.copy_from_slice(&buffer[offset..offset + 4]);
                offset += 4;
            }

            if buffer.len() < offset + payload_len {
                return None;
            }

            let mut payload = vec![0; payload_len];

            payload.copy_from_slice(&buffer[offset..offset + payload_len]);

            if mask {
                for (i, byte) in payload.iter_mut().enumerate() {
                    *byte ^= masking_key[i % 4];  // XOR with corresponding masking key byte
                }
            }

            buffer.clear();

            return Some((opcode, Some(payload)));
        }
    }

    None
}
