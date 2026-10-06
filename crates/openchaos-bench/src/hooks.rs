const CALLGRIND_BASE: usize = ((b'C' as usize) << 24) | ((b'T' as usize) << 16);
const DUMP_STATS_AT: usize = CALLGRIND_BASE + 3;
const ZERO_STATS: usize = CALLGRIND_BASE + 1;
const START_INSTRUMENTATION: usize = CALLGRIND_BASE + 4;
const STOP_INSTRUMENTATION: usize = CALLGRIND_BASE + 5;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[inline(always)]
fn client_request(request: usize, arg1: usize) {
    let args: [usize; 6] = [request, arg1, 0, 0, 0, 0];
    unsafe {
        std::arch::asm!(
            "rol rdi, 3",
            "rol rdi, 13",
            "rol rdi, 61",
            "rol rdi, 51",
            "xchg rbx, rbx",
            in("rax") args.as_ptr(),
            inout("rdx") 0usize => _,
        );
    }
}

#[cfg(all(target_arch = "aarch64", target_os = "linux"))]
#[inline(always)]
fn client_request(request: usize, arg1: usize) {
    let args: [usize; 6] = [request, arg1, 0, 0, 0, 0];
    unsafe {
        std::arch::asm!(
            "ror x12, x12, #3",
            "ror x12, x12, #13",
            "ror x12, x12, #51",
            "ror x12, x12, #61",
            "orr x10, x10, x10",
            in("x4") args.as_ptr(),
            inout("x3") 0usize => _,
        );
    }
}

#[cfg(not(all(
    any(target_arch = "x86_64", target_arch = "aarch64"),
    target_os = "linux"
)))]
fn client_request(_request: usize, _arg1: usize) {}

/// Run `f` as the benchmark `name`.
///
/// Under callgrind, only the instructions executed inside `f` are counted and
/// they are dumped as a profile part named `name`. Outside Valgrind this just
/// calls `f`. `name` must not contain a NUL byte.
pub fn bench<R>(name: &str, f: impl FnOnce() -> R) -> R {
    let name = std::ffi::CString::new(name).expect("benchmark name contains a NUL byte");
    client_request(ZERO_STATS, 0);
    client_request(START_INSTRUMENTATION, 0);
    let out = std::hint::black_box(f());
    client_request(STOP_INSTRUMENTATION, 0);
    client_request(DUMP_STATS_AT, name.as_ptr() as usize);
    out
}
