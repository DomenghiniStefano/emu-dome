//! CPU time used by this process (user + kernel), for the bench reports.

use std::time::Duration;

#[cfg(target_os = "windows")]
pub fn process_cpu_time() -> Option<Duration> {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn GetProcessTimes(
            process: *mut c_void,
            creation: *mut FileTime,
            exit: *mut FileTime,
            kernel: *mut FileTime,
            user: *mut FileTime,
        ) -> i32;
    }

    let (mut creation, mut exit, mut kernel, mut user) = Default::default();
    // SAFETY: GetCurrentProcess returns a pseudo-handle that needs no closing; the four
    // pointers are to live, writable FILETIME-shaped structs.
    let ok = unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    };
    if ok == 0 {
        return None;
    }
    // FILETIME counts 100 ns ticks.
    let ticks = |t: &FileTime| (u64::from(t.high) << 32) | u64::from(t.low);
    Some(Duration::from_nanos(
        (ticks(&kernel) + ticks(&user)).saturating_mul(100),
    ))
}

/// Linux and Android: fields 14 and 15 of `/proc/self/stat`, in clock ticks
/// (100 per second on every Linux the project targets).
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn process_cpu_time() -> Option<Duration> {
    let stat = std::fs::read_to_string("/proc/self/stat").ok()?;
    // The command name (field 2) is in parentheses and may contain spaces.
    let rest = &stat[stat.rfind(')')? + 2..];
    let fields: Vec<&str> = rest.split_whitespace().collect();
    // `rest` starts at field 3: utime is field 14, stime field 15.
    let utime: u64 = fields.get(11)?.parse().ok()?;
    let stime: u64 = fields.get(12)?.parse().ok()?;
    Some(Duration::from_millis((utime + stime) * 10))
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android")))]
pub fn process_cpu_time() -> Option<Duration> {
    None
}
