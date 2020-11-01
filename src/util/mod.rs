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

#[macro_export]
macro_rules! finalize {
    ($ops:ident, $e:expr) => {{
        jit!($ops
            ; mov rax, $e
            ; add rsp, 0x20
            ; ret
        );
        $ops.finalize().expect(concat!("Could not finalize ", stringify!($ops)))
    }};
}

// Imma be real, idk why this stack frame setup works
// When I do something more rational like
//      ; sub rsp, 0x20
//      ; mov QWORD [rsp], rcx
//      ; mov QWORD [rsp + 0x8], rdx
//      ; mov QWORD [rsp + 0x10], r8
//      ; mov QWORD [rsp + 0x18], r9
// the stack in gdb looks legit but it crashes when using some other calling
// convention for printing...
// Also, idk if this is just my debugger, but it seems like using this stack
// frame setup and writing to where I am overwrites some struct being used by
// the dynasm runtime...
#[macro_export]
macro_rules! call_prologue {
    ($ops:ident) => {jit!($ops
        ; sub rsp, 0x48
        ; mov QWORD [rsp + 0x28], rcx
        ; mov QWORD [rsp + 0x30], rdx
        ; mov QWORD [rsp + 0x38], r8
        ; mov QWORD [rsp + 0x40], r9
    );};
}

#[macro_export]
macro_rules! call_epilogue {
    ($ops:ident) => { jit!($ops
        ; mov rcx, QWORD [rsp + 0x28]
        ; mov rdx, QWORD [rsp + 0x30]
        ; mov r8,  QWORD [rsp + 0x38]
        ; mov r9,  QWORD [rsp + 0x40]
        ; add rsp, 0x48
    );};
}

#[macro_export]
macro_rules! call {
    ($ops:ident, $addr:expr) => {jit!($ops
        ;; call_prologue!($ops)
        ; mov  rax, QWORD $addr as _
        ; call rax
        ;; call_epilogue!($ops)
    );};
}
