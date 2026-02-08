pub fn parse_resp(input: &str) -> Vec<String> {

    let mut lines = input.split("\r\n");

    let mut result = Vec::new();

    while let Some(line) = lines.next() {

        if line.starts_with('*') {
            continue;
        }

        if line.starts_with('$') {
            if let Some(value) = lines.next() {
                result.push(value.to_string());
            }
        }
    }

    result
}
