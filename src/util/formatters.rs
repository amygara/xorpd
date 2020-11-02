//! Man I really want this to feed to some sort of tui...

use ffi_support::FfiStr;

pub extern "win64" fn print_reg(reg: u64, name: u64) {
    let name: String = name.to_be_bytes().iter().map(|b| *b as char).collect();

    println!(
        "{}:\t\t{:#018}\t\t{:#018X}\t\t{:#064b}",
        name, reg, reg, reg
    );
}

pub extern "win64" fn print_str(rcx: FfiStr<'_>) {
    println!("{}", rcx.as_str());
}
