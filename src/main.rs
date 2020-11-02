mod util;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};

use std::mem;

fn main() {
    let program = program("Hello World!");
    let ret = program(1, 2, 3, 4);
    println!("Program returned: {}", ret);
}

/// As Dynasm-rs requires the "win64" calling convention be used, his function
/// returns a closure wrapping the raw function type:
///
/// `extern "win64" fn (rcx: u64, rdx: u64, r8: u64, r9: u64) -> /* RAX */ u64`
///
/// As such, several aliases are provided for convenience:
///
///     ; .alias arg0, rcx
///     ; .alias arg1, rdx
///     ; .alias arg2, r8
///     ; .alias arg3, r9
///
fn program(message: &str) -> impl Fn(u64, u64, u64, u64) -> u64 {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    jit!(ops
        ; ->h:
        ; .bytes message.as_bytes()
        ; .byte 0
    );

    // This macro also generates a prologue for our JIT'd function and can be
    // seen in ./util/mod.rs. In summary, it stores all 4 args passed in to the
    // functon on the stack for convieniance at RSP + 0x{0,8,10,18}. As these
    // are all volatile regs we want to keep them safe in case of potential future
    // mutation.
    let start = entry_point!(ops);

    jit!(ops
        ;; print_reg!(ops, r9)
        ; lea arg0, [->h]
        ;; print_str!(ops, arg0)
        ; mov arg0, message.len() as _
        ;; call!(ops, example)
        ;; print_reg!(ops, rax)
        ; mov rax, [rsp + 0x18]
    );

    finalize!(ops, start)
}

extern "win64" fn example(arg0: u64) -> u64 {
    println!("message length: {}", arg0);
    42
}
