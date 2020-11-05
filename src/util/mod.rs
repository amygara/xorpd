pub mod formatters;

/// Ensure we're working w/ an x64 architecture.
///
/// This snippet also creates several convenience aliases for accessing registers
/// used for passing variables to functions using the [win64 calling convention].
///
/// [win64 calling convention]: https://docs.microsoft.com/en-us/cpp/build/x64-software-conventions?view=msvc-160
#[macro_export]
macro_rules! jit {
    ($ops:ident $($t:tt)*) => {dynasm!($ops
        ; .arch x64
        ; .alias arg0, rcx
        ; .alias arg1, rdx
        ; .alias arg2, r8
        ; .alias arg3, r9
        $($t)*
    );};
}

/// Specify the program's entry point (_start) and store all variables passed in to the stack.
///
/// To do this, we decrement the stack pointer by 0x20 (32 bytes), reserving
/// 0x8 (8 bytes) per register on the stack. We store them on the stack [in order].
///
/// [in order]: https://docs.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-160
#[macro_export]
macro_rules! entry_point {
    ($ops:ident) => {{
        let start = $ops.offset();
        jit!($ops
            ; sub rsp, 0x20
            ; mov QWORD [rsp], rcx
            ; mov QWORD [rsp + 0x8], rdx
            ; mov QWORD [rsp + 0x10], r8
            ; mov QWORD [rsp + 0x18], r9
        );
        start
    }};
}

/// Finalize the program by adding a short prologue and return the program as a closure.
///
/// To do this, we increment the stack pointer by 0x20 (32 bytes) to clear the
/// stack space we were using to store our variables and return.
///
/// Additionally, tell dynasm-rs that we're done JITing code and get back the
/// pointer to our entry point (_start).
///
/// Lastly, cast the pointer to a function pointer, specifying the program
/// accepts 4 u64 arguments and enclose it in a closure. The closure is purely
/// aesthetic to hide the calling convention used by the inner JIT program.
#[macro_export]
macro_rules! finalize {
    ($ops:ident, $ep:expr) => {{
        jit!($ops
            ; add rsp, 0x20
            ; ret
        );
        let buf = $ops.finalize().expect(concat!("Could not finalize ", stringify!($ops)));

        let ret = move |rcx: u64, rdx: u64, r8: u64, r9: u64| -> u64 {
            let hello_fn: extern "win64" fn(rcx: u64, rdx: u64, r8: u64, r9: u64) -> u64 =
                unsafe { mem::transmute(buf.ptr($ep)) };
            hello_fn(rcx, rdx, r8, r9)
        };

        ret
    }};
}

/// Convenience macro for [setting up a call] to an external function.
///
/// This assembly snippet reserve stack space to save volatile variables
/// in case the callee clobbers them. One tricky note here, the stack frame has
/// to be 16-byte aligned, so any change to rsp **must** be a [multiple of 16].
/// Additionally, we reserve extra space for the callee to save regs like RDI/RSI.
///
/// [setting up a call]: https://docs.microsoft.com/en-us/cpp/build/prolog-and-epilog?view=msvc-160
/// [multiple of 16]: https://docs.microsoft.com/en-us/cpp/build/stack-usage?view=msvc-160#stack-allocation
#[macro_export]
macro_rules! call_prologue {
    ($ops:ident) => {jit!($ops
        // I'm debating whether or not I save/restore RAX. I think I will as this
        // is purely a runner for xorp puzzles and not really meant to be general
        // purpose. Additionally, all I need for external functions are prints
        // and those will always return void.
        ; sub rsp, 0x48
        ; mov QWORD [rsp], rcx
        ; mov QWORD [rsp + 0x8], rdx
        ; mov QWORD [rsp + 0x10], r8
        ; mov QWORD [rsp + 0x18], r9
    );};
}

/// Convenience macro for restoring registers after a call to an external function.
///
/// Basically just the inverse of the above.
#[macro_export]
macro_rules! call_epilogue {
    ($ops:ident) => { jit!($ops
        ; mov rcx, QWORD [rsp]
        ; mov rdx, QWORD [rsp + 0x8]
        ; mov r8,  QWORD [rsp + 0x10]
        ; mov r9,  QWORD [rsp + 0x18]
        ; add rsp, 0x48
    );};
}

/// Convenience macro to call external functions. Must use the win64 calling convention.
///
/// Clobbers:
///     RAX
///
#[macro_export]
macro_rules! call {
    ($ops:ident, $addr:expr) => {jit!($ops
        ;; call_prologue!($ops)
        ; mov  rax, QWORD $addr as _
        ; call rax
        ;; call_epilogue!($ops)
    );};
}

/// Convenience macro to print the value of a register.
///
/// Clobbers:
///     RAX
///
/// Lol please don't use this if '$reg' is more than 8 chars plz
/// I suppose I should probably implement this w/ an xmmm reg...
#[macro_export]
macro_rules! print_reg {
    ($ops:ident, $reg:tt) => {
        // I looked at this under a debugger out of curiosity, it's super
        // inefficient. Implementing this w/ bit math would save us ~8 calls...
        let mut bytes = [0; 8];
        stringify!($reg).bytes().enumerate().for_each(|(i, b)| {
            bytes[i] = b;
        });

        jit!($ops
            ;; call_prologue!($ops)
            ; mov rcx, $reg
            ; mov rdx, QWORD u64::from_be_bytes(bytes) as _
            ; mov  rax, QWORD crate::util::formatters::print_reg as _
            ; call rax
            ;; call_epilogue!($ops)
        );
    };
}

/// Convenience macro to print out a string.
///
/// Clobbers:
///     RAX
#[macro_export]
macro_rules! print_str {
    ($ops:ident, $reg:tt) => {jit!($ops
        ;; call_prologue!($ops)
        ; mov rcx, $reg
        ; mov  rax, QWORD crate::util::formatters::print_str as _
        ; call rax
        ;; call_epilogue!($ops)
    );};
}
