pub mod formatters;

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

#[macro_export]
macro_rules! call_prologue {
    ($ops:ident) => {jit!($ops
        ; sub rsp, 0x48
        ; mov QWORD [rsp], rcx
        ; mov QWORD [rsp + 0x8], rdx
        ; mov QWORD [rsp + 0x10], r8
        ; mov QWORD [rsp + 0x18], r9
    );};
}

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

#[macro_export]
macro_rules! call {
    ($ops:ident, $addr:expr) => {jit!($ops
        ;; call_prologue!($ops)
        ; mov  rax, QWORD $addr as _
        ; call rax
        ;; call_epilogue!($ops)
    );};
}

/// Lol please don't use this if '$reg' is more than 4 chars plz
/// I suppose I should probably implement this w/ an xmmm reg...
#[macro_export]
macro_rules! print_reg {
    ($ops:ident, $reg:tt) => {
        let mut bytes = [0; 4];
        stringify!($reg).bytes().enumerate().for_each(|(i, b)| {
            bytes[i] = b;
        });

        jit!($ops
            ;; call_prologue!($ops)
            ; mov rcx, $reg
            ; mov rdx, u32::from_be_bytes(bytes) as _
            ; mov  rax, QWORD crate::util::formatters::print_reg as _
            ; call rax
            ;; call_epilogue!($ops)
        );
    };
}

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
