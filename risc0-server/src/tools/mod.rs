pub fn parse_elf_from_str(elf_str: &str) -> Vec<u8> {
    let mut elf_cont: Vec<u8> = Vec::new();
    let vec_u8: Result<Vec<u8>, _> = elf_str
        .split(",")
        .into_iter()
        .map(|s| s.trim().parse::<u8>())
        .collect();
    match vec_u8 {
        Ok(v) => elf_cont = v,
        Err(e) => println!("elf parse error: {}", e),
    }
    elf_cont
}
