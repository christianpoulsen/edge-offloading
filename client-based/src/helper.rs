

pub fn socket_addr(ipv4: &str, port: i32) -> String {
    let mut addr = String::from(ipv4);
    addr.push_str(port.to_string().trim());
    return addr;
}
