pub fn parse_resp_one(input: &[u8]) -> Result<Option<(Vec<String>, usize)>, String> {
    let mut i = 0;

    fn read_line<'a>(input: &'a [u8], i: &mut usize) -> Option<&'a [u8]> {
        let start = *i;

        while *i + 1 < input.len() {
            if input[*i] == b'\r' && input[*i + 1] == b'\n' {
                let line = &input[start..*i];
                *i += 2; // consome \r\n
                return Some(line);
            }
            *i += 1;
        }

        *i = start;
        None
    }

    // header: *<n>\r\n
    let header = match read_line(input, &mut i) {
        Some(l) => l,
        None => return Ok(None),
    };

    if header.first() != Some(&b'*') {
        return Err("expected array header *<n>".into());
    }

    let n: usize = std::str::from_utf8(&header[1..])
        .map_err(|_| "invalid utf8 in array length")?
        .parse()
        .map_err(|_| "invalid array length")?;

    let mut parts = Vec::with_capacity(n);

    for _ in 0..n {
        // $<len>\r\n
        let bulk_hdr = match read_line(input, &mut i) {
            Some(l) => l,
            None => return Ok(None),
        };

        if bulk_hdr.first() != Some(&b'$') {
            return Err("expected bulk header $<len>".into());
        }

        let len_i64: i64 = std::str::from_utf8(&bulk_hdr[1..])
            .map_err(|_| "invalid utf8 in bulk length")?
            .parse()
            .map_err(|_| "invalid bulk length")?;

        if len_i64 < -1 {
            return Err("bulk length < -1".into());
        }

        if len_i64 == -1 {
            // Null bulk (para argumentos: vamos representar como string vazia por enquanto)
            parts.push(String::new());
            continue;
        }

        let len = len_i64 as usize;

        // precisa ter len bytes + \r\n
        if i + len + 2 > input.len() {
            return Ok(None);
        }

        let data = &input[i..i + len];
        i += len;

        if input.get(i) != Some(&b'\r') || input.get(i + 1) != Some(&b'\n') {
            return Err("bulk data not terminated with \\r\\n".into());
        }
        i += 2;

        parts.push(String::from_utf8_lossy(data).to_string());
    }

    Ok(Some((parts, i))) // i = bytes consumidos
}
