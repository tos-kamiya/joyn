use std::fs::File;

#[cfg(target_os = "linux")]
use { std::mem, std::os::unix::io::AsRawFd };

#[cfg(target_os = "linux")]
pub fn get_pipe_buffer_size(inp: &File) -> Option<usize> {
    unsafe {
        let fd = inp.as_raw_fd();
        let mut s: libc::stat = mem::zeroed();
        libc::fstat(fd, &mut s);
        if (s.st_mode & libc::S_IFMT) == libc::S_IFIFO {
            let pipe_size = libc::fcntl(fd, libc::F_GETPIPE_SZ);
            if pipe_size >= 0 {
                return Some(pipe_size as usize);
            }
        }
    }

    None
}

#[cfg(not(target_os = "linux"))]
pub fn get_pipe_buffer_size(inp: &File) -> Option<usize> {
    None
}

