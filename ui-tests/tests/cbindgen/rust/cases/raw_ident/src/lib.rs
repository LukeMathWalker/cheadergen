/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[repr(u8)]
pub enum r#Enum {
    r#a,
    r#b,
}

#[repr(C)]
pub struct r#Struct {
    r#field: r#Enum,
}

#[unsafe(no_mangle)]
pub extern "C" fn r#fn(r#arg: r#Struct) {
    println!("Hello world");
}

pub mod r#mod {
    #[unsafe(no_mangle)]
    pub static r#STATIC: r#Enum = r#Enum::r#b;
}
