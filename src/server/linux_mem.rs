use std::fs;

pub fn rss_bytes() -> Option<u64> {
    let s = fs::read_to_string("/proc/self/statm").ok()?;
    let mut it = s.split_whitespace();

    let _size_pages: u64 = it.next()?.parse().ok()?;
    let resident_pages: u64 = it.next()?.parse().ok()?;

    let page_size = unsafe { libc::sysconf(libc::_SC_PAGESIZE) };
    if page_size <= 0 {
        return None;
    }

    Some(resident_pages * page_size as u64)
}
