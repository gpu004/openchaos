use core::ffi::CStr;

const CALLGRIND_BASE: usize = 0x4354_0000;
const DUMP_STATS_AT: usize = CALLGRIND_BASE + 3;
const ZERO_STATS: usize = CALLGRIND_BASE + 1;
const START_INSTRUMENTATION: usize = CALLGRIND_BASE + 4;
const STOP_INSTRUMENTATION: usize = CALLGRIND_BASE + 5;

#[cfg(all(target_arch = "x86_64", target_os = "linux"))]
#[inline]
fn client_request(request: usize, payload: usize) {
    let args: [usize; 6] = [request, payload, 0, 0, 0, 0];
    #[expect(
        unsafe_code,
        reason = "Valgrind client requests are only reachable through inline asm"
    )]
    // SAFETY: the sequence is a no-op on real hardware; under Valgrind it reads `args`, which outlives the asm, and writes only the declared output register.
    unsafe {
        core::arch::asm!(
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
#[inline]
fn client_request(request: usize, payload: usize) {
    let args: [usize; 6] = [request, payload, 0, 0, 0, 0];
    #[expect(
        unsafe_code,
        reason = "Valgrind client requests are only reachable through inline asm"
    )]
    // SAFETY: the sequence is a no-op on real hardware; under Valgrind it reads `args`, which outlives the asm, and writes only the declared output register.
    unsafe {
        core::arch::asm!(
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
fn client_request(_request: usize, _payload: usize) {}

/// Run `f` as the benchmark `name`.
///
/// Under callgrind, only the instructions executed inside `f` are counted and
/// they are dumped as a profile part named `name`. Outside Valgrind this just
/// calls `f`.
pub fn bench<R>(name: &CStr, f: impl FnOnce() -> R) -> R {
    client_request(ZERO_STATS, 0);
    client_request(START_INSTRUMENTATION, 0);
    let out = core::hint::black_box(f());
    client_request(STOP_INSTRUMENTATION, 0);
    client_request(DUMP_STATS_AT, name.as_ptr().expose_provenance());
    out
}
