//! How long a path this process can create (Trello #94): the numbers the shell checks a core's
//! deepest files against before the core loads (`emudome_core::save_path`).
//!
//! Windows: 260 characters (`MAX_PATH`) unless long paths are on for the process, which takes
//! both the executable's manifest (`longPathAware`, `app/src-tauri/windows-app-manifest.xml`)
//! and the system policy (`LongPathsEnabled` under
//! `HKLM\SYSTEM\CurrentControlSet\Control\FileSystem`, which EmuDome only reads, never writes).
//! Which of the two decides is asked of the system itself (`RtlAreLongPathsEnabled`, what the
//! loader read at the process start), so a test executable without the manifest gets 260 too.
//! Linux and Android: `PATH_MAX`, 4096 bytes. macOS: 1024 bytes.

/// The longest paths this process can create, without the terminating NUL (in UTF-16 units
/// when `utf16`, else in bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathLimits {
    pub dir: usize,
    pub file: usize,
    pub utf16: bool,
}

/// Whether paths over 260 characters work for this process (Windows; always true elsewhere,
/// where no such limit exists).
pub fn long_paths_enabled() -> bool {
    imp::long_paths_enabled()
}

/// The system's long-path policy as set (Windows: the `LongPathsEnabled` registry value, read
/// only); `None` where there is no such policy or it cannot be read (absent means off).
pub fn long_paths_policy() -> Option<bool> {
    imp::long_paths_policy()
}

/// The limits of this process now ([`PathLimits`]).
pub fn path_limits() -> PathLimits {
    imp::path_limits()
}

#[cfg(target_os = "windows")]
mod imp {
    use std::ffi::c_void;

    use super::PathLimits;

    /// `MAX_PATH` (260) less the NUL; a folder keeps 12 more for an 8.3 file name
    /// (`CreateDirectory`'s documented limit: 248 with the NUL).
    const LEGACY: PathLimits = PathLimits {
        dir: 247,
        file: 259,
        utf16: true,
    };

    /// With long paths: 32 767 UTF-16 units, the system's `UNICODE_STRING` limit, less the NUL
    /// and the `\\?\` prefix the system adds.
    const LONG: PathLimits = PathLimits {
        dir: 32_762,
        file: 32_762,
        utf16: true,
    };

    #[link(name = "kernel32")]
    extern "system" {
        fn GetModuleHandleW(name: *const u16) -> *mut c_void;
        fn GetProcAddress(module: *mut c_void, name: *const u8) -> *mut c_void;
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn RegGetValueW(
            key: *mut c_void,
            sub_key: *const u16,
            value: *const u16,
            flags: u32,
            kind: *mut u32,
            data: *mut c_void,
            len: *mut u32,
        ) -> i32;
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    pub fn long_paths_enabled() -> bool {
        type Query = unsafe extern "system" fn() -> u8;
        let ntdll = wide("ntdll.dll");
        // SAFETY: ntdll is mapped in every Windows process; the name strings are NUL-terminated.
        // `RtlAreLongPathsEnabled` (Windows 10 1607 and later) takes nothing and returns a
        // BOOLEAN; looked up by name so an older system that lacks it reads as «off».
        unsafe {
            let module = GetModuleHandleW(ntdll.as_ptr());
            if module.is_null() {
                return false;
            }
            let f = GetProcAddress(module, c"RtlAreLongPathsEnabled".as_ptr().cast());
            if f.is_null() {
                return false;
            }
            let query = std::mem::transmute::<*mut c_void, Query>(f);
            query() != 0
        }
    }

    pub fn long_paths_policy() -> Option<bool> {
        // `HKEY_LOCAL_MACHINE`: the predefined handle, sign-extended as the headers define it.
        let hklm = 0x8000_0002_u32 as i32 as isize as *mut c_void;
        const RRF_RT_REG_DWORD: u32 = 0x0000_0010;
        let key = wide(r"SYSTEM\CurrentControlSet\Control\FileSystem");
        let name = wide("LongPathsEnabled");
        let mut value: u32 = 0;
        let mut len = std::mem::size_of::<u32>() as u32;
        // SAFETY: the strings are NUL-terminated; `value` is a writable DWORD and `len` its size.
        // Read only: no key is opened for writing.
        let status = unsafe {
            RegGetValueW(
                hklm,
                key.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut value as *mut u32).cast(),
                &mut len,
            )
        };
        (status == 0).then_some(value != 0)
    }

    pub fn path_limits() -> PathLimits {
        if long_paths_enabled() {
            LONG
        } else {
            LEGACY
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
mod imp {
    use super::PathLimits;

    pub fn long_paths_enabled() -> bool {
        true
    }

    pub fn long_paths_policy() -> Option<bool> {
        None
    }

    /// `PATH_MAX` (4096 bytes with the NUL).
    pub fn path_limits() -> PathLimits {
        PathLimits {
            dir: 4095,
            file: 4095,
            utf16: false,
        }
    }
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "android")))]
mod imp {
    use super::PathLimits;

    pub fn long_paths_enabled() -> bool {
        true
    }

    pub fn long_paths_policy() -> Option<bool> {
        None
    }

    /// macOS: `PATH_MAX` (1024 bytes with the NUL).
    pub fn path_limits() -> PathLimits {
        PathLimits {
            dir: 1023,
            file: 1023,
            utf16: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{long_paths_enabled, long_paths_policy, path_limits};

    /// Whatever this machine says, the limits follow it, and a test executable (no long-path
    /// manifest) never gets long paths on Windows. Printed for the record (`--nocapture`).
    #[test]
    fn the_limits_follow_the_long_path_answer() {
        let limits = path_limits();
        println!(
            "long paths: policy {:?}, this process {}, limits {limits:?}",
            long_paths_policy(),
            long_paths_enabled()
        );
        assert!(limits.dir <= limits.file);
        if cfg!(target_os = "windows") {
            assert!(!long_paths_enabled(), "a test binary has no manifest");
            assert_eq!((limits.dir, limits.file), (247, 259));
        } else {
            assert!(limits.file >= 1023);
        }
    }
}
