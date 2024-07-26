pub fn read_string_buffer(buffer: &[u8]) -> String {
    let text = String::from_utf8_lossy(buffer);
    let text = text.trim().to_string();

    // println!("> {}", text);

    text
}
