pub fn read_file(path: &str) -> Vec<u8> {
    std::fs::read(path).expect(&format!("Failed to read {}", path))
}