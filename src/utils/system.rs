use std::path::Path;

/// Check if the current user is root
pub fn is_root_user() -> bool {
    #[cfg(unix)]
    {
        unsafe { libc::geteuid() == 0 }
    }
    #[cfg(not(unix))]
    {
        false
    }
}

/// Get owner and group information for a file
pub fn get_owner_group(path: &Path) -> (Option<String>, Option<String>, Option<u32>, Option<u32>) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;

        if let Ok(metadata) = path.metadata() {
            let uid = metadata.uid();
            let gid = metadata.gid();

            // Get username from uid
            let owner = unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() {
                    let name = std::ffi::CStr::from_ptr((*pw).pw_name);
                    name.to_string_lossy().to_string()
                } else {
                    uid.to_string()
                }
            };

            // Get group name from gid
            let group = unsafe {
                let gr = libc::getgrgid(gid);
                if !gr.is_null() {
                    let name = std::ffi::CStr::from_ptr((*gr).gr_name);
                    name.to_string_lossy().to_string()
                } else {
                    gid.to_string()
                }
            };

            return (Some(owner), Some(group), Some(uid), Some(gid));
        }
    }

    (None, None, None, None)
}
