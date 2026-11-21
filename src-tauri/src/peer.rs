//! Kernel-provided account identity supplements tokens; it is not model identity.
use std::os::{fd::AsRawFd, unix::net::UnixStream};

pub(crate) fn current_uid() -> libc::uid_t {
    // SAFETY: geteuid has no arguments or memory preconditions.
    unsafe { libc::geteuid() }
}

#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
))]
fn peer_uid(stream: &UnixStream) -> Result<libc::uid_t, String> {
    let (mut uid, mut gid) = (0, 0);
    // SAFETY: the live stream owns its descriptor; both output pointers are valid.
    if unsafe { libc::getpeereid(stream.as_raw_fd(), &mut uid, &mut gid) } != 0 {
        return Err("Couldn't verify the local connection's account.".into());
    }
    Ok(uid)
}

#[cfg(any(target_os = "linux", target_os = "android"))]
fn peer_uid(stream: &UnixStream) -> Result<libc::uid_t, String> {
    let mut credentials = std::mem::MaybeUninit::<libc::ucred>::uninit();
    let mut len = std::mem::size_of::<libc::ucred>() as libc::socklen_t;
    // SAFETY: the output buffer and its length have the exact SO_PEERCRED layout.
    let result = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            credentials.as_mut_ptr().cast(),
            &mut len,
        )
    };
    if result != 0 || len as usize != std::mem::size_of::<libc::ucred>() {
        return Err("Couldn't verify the local connection's account.".into());
    }
    // SAFETY: successful getsockopt initialized the complete checked structure.
    Ok(unsafe { credentials.assume_init() }.uid)
}

#[cfg(not(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly",
    target_os = "linux",
    target_os = "android"
)))]
fn peer_uid(_stream: &UnixStream) -> Result<libc::uid_t, String> {
    Err("Account verification is unavailable on this platform.".into())
}

fn require_uid(stream: &UnixStream, expected: libc::uid_t) -> Result<(), String> {
    if peer_uid(stream)? != expected {
        return Err("The local agent connection belongs to a different account.".into());
    }
    Ok(())
}

pub(crate) fn require_same_account(stream: &UnixStream) -> Result<(), String> {
    require_uid(stream, current_uid())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn kernel_identity_is_checked_in_both_directions_and_mismatch_is_rejected() {
        let (a, b) = UnixStream::pair().unwrap();
        assert_eq!(peer_uid(&a).unwrap(), current_uid());
        require_same_account(&a).unwrap();
        require_same_account(&b).unwrap();
        assert!(require_uid(&a, current_uid().wrapping_add(1)).is_err());
    }
}
