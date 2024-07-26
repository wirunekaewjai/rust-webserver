use std::io::Write;

pub fn build_frame(fin: bool, opcode: u8, payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();

    // Byte 0: FIN, RSV1-3, Opcode
    let mut byte0 = if fin { 0b10000000 } else { 0 }; // FIN

    byte0 |= 0; // RSV1, RSV2, RSV3 are 0 (always for servers)
    byte0 |= opcode & 0x0f; // Mask opcode to 4 bits

    frame.push(byte0);

    // Byte 1: MASK (always 0 for server), Payload Length
    let payload_len = payload.len();
    let mut byte1 = 0; // MASK = 0 (server)

    if payload_len <= 125 {
        byte1 |= payload_len as u8;

        frame.push(byte1);
    } else if payload_len <= 65535 {
        byte1 |= 126;

        frame.push(byte1);
        frame.write_all(&(payload_len as u16).to_be_bytes()).unwrap(); // 2-byte extended length
    } else {
        byte1 |= 127;

        frame.push(byte1);
        frame.write_all(&(payload_len as u64).to_be_bytes()).unwrap(); // 8-byte extended length
    }

    // Payload (unmasked)
    frame.extend_from_slice(payload);
    frame
}