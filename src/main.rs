mod util;

use dynasmrt::{dynasm, DynasmApi, DynasmLabelApi};
use ffi_support::FfiStr;

use std::{mem, process::Command};

// I bet this could be macro'd away...
unsafe extern "win64" fn print_str(rcx: FfiStr) {
    println!("{}", rcx.as_str());
}

fn main() {
    // rust-gdb hates breaking on main
    foof();
}

fn foof() {
    let mut ops = dynasmrt::x64::Assembler::new().unwrap();

    jit!(ops
        ; ->h:
        ; .bytes Command::new("fortune").output().unwrap().stdout
        ; .byte 0
    );

    let start = entry_point!(ops);

    jit!(ops
        ; lea arg0, [->h]
        ;; call!(ops, print_str)
    );

    let buf = finalize!(ops, 0);

    let hello_fn: extern "win64" fn() -> u64 = unsafe { mem::transmute(buf.ptr(start)) };

    assert_eq!(hello_fn(), 0);
}
