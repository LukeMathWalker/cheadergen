/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#[export_name="rust_print_hello_world"]
pub extern fn say_hello() {
  println!("Hello, World!");
}

#[repr(C)]
pub struct SelfTypeTestStruct {
  times: u8,
}

impl SelfTypeTestStruct {
  #[export_name="SelfTypeTestStruct_should_exist_ref"]
  #[unsafe(no_mangle)]
  pub extern fn should_exist_ref(&self) {
    println!("should_exist_ref");
  }

  #[export_name="SelfTypeTestStruct_should_exist_ref_mut"]
  #[unsafe(no_mangle)]
  pub extern fn should_exist_ref_mut(&mut self) {
    println!("should_exist_ref_mut");
  }

  #[export_name="SelfTypeTestStruct_should_not_exist_box"]
  #[unsafe(no_mangle)]
  pub extern fn should_not_exist_box(self: Box<SelfTypeTestStruct>) {
    println!("should_not_exist_box");
  }

  #[export_name="SelfTypeTestStruct_should_not_exist_return_box"]
  #[unsafe(no_mangle)]
  pub extern fn should_not_exist_box() -> Box<Self> {
    println!("should_not_exist_box");
  }

  #[export_name="SelfTypeTestStruct_should_exist_annotated_self"]
  #[unsafe(no_mangle)]
  pub extern fn should_exist_annotated_self(self: Self) {
    println!("should_exist_annotated_self");
  }

  #[export_name="SelfTypeTestStruct_should_exist_annotated_mut_self"]
  #[unsafe(no_mangle)]
  #[allow(unused_mut)]
  pub extern fn should_exist_annotated_mut_self(mut self: Self) {
    println!("should_exist_annotated_mut_self");
  }

  #[export_name="SelfTypeTestStruct_should_exist_annotated_by_name"]
  #[unsafe(no_mangle)]
  pub extern fn should_exist_annotated_by_name(self: SelfTypeTestStruct) {
    println!("should_exist_annotated_by_name");
  }

  #[export_name="SelfTypeTestStruct_should_exist_annotated_mut_by_name"]
  #[unsafe(no_mangle)]
  #[allow(unused_mut)]
  pub extern fn should_exist_annotated_mut_by_name(mut self: SelfTypeTestStruct) {
    println!("should_exist_annotated_mut_by_name");
  }

  #[export_name="SelfTypeTestStruct_should_exist_unannotated"]
  #[unsafe(no_mangle)]
  pub extern fn should_exist_unannotated(self) {
    println!("should_exist_unannotated");
  }

  #[export_name="SelfTypeTestStruct_should_exist_mut_unannotated"]
  #[unsafe(no_mangle)]
  #[allow(unused_mut)]
  pub extern fn should_exist_mut_unannotated(mut self) {
    println!("should_exist_mut_unannotated");
  }
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern fn free_function_should_exist_ref(test_struct: &SelfTypeTestStruct) {
  println!("free_function_should_exist_ref");
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern fn free_function_should_exist_ref_mut(test_struct: &mut SelfTypeTestStruct) {
  println!("free_function_should_exist_ref_mut");
}

#[unsafe(no_mangle)]
pub extern fn unnamed_argument(_: &mut SelfTypeTestStruct) {
  println!("unnamed_argument");
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern fn free_function_should_not_exist_box(boxed: Box<SelfTypeTestStruct>) {
  println!("free_function_should_not_exist_box");
}

#[unsafe(no_mangle)]
#[allow(unused_variables)]
pub extern fn free_function_should_exist_annotated_by_name(test_struct: SelfTypeTestStruct) {
  println!("free_function_should_exist_annotated_by_name");
}

#[unsafe(no_mangle)]
#[allow(unused_mut)]
#[allow(unused_variables)]
pub extern fn free_function_should_exist_annotated_mut_by_name(mut test_struct: SelfTypeTestStruct) {
  println!("free_function_should_exist_annotated_mut_by_name");
}

struct Opaque {
  times: u8
}

#[repr(C)]
pub struct PointerToOpaque { ptr: *mut Opaque }

impl PointerToOpaque {
  #[export_name="PointerToOpaque_create"]
  pub extern fn create(times: u8) -> PointerToOpaque {
    PointerToOpaque { ptr: Box::into_raw(Box::new(Opaque { times })) }
  }

  #[export_name="PointerToOpaque_sayHello"]
  pub extern fn say_hello(self: PointerToOpaque) {
    if let Some(nonnull) = std::ptr::NonNull::new(self.ptr) {
      for _ in 0 .. unsafe { nonnull.as_ref().times } {
        println!("Hello!")
      }
    }
  }
}
