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

pub fn cpu_seconds_total() -> Option<f64> {
    let s = fs::read_to_string("/proc/self/stat").ok()?;

    let rparen = s.rfind(')')?;
    let after = s.get(rparen + 2..)?;

    let fields: Vec<&str> = after.split_whitespace().collect();

    let utime_ticks: u64 = fields.get(11)?.parse().ok()?;
    let stime_ticks: u64 = fields.get(12)?.parse().ok()?;

    let clk_tck = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if clk_tck <= 0 {
        return None;
    }

    let total_ticks = utime_ticks + stime_ticks;
    Some(total_ticks as f64 / clk_tck as f64)
}
