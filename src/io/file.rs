pub fn read_file(path: &str) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|_| panic!("Failed to read {path}"))
}