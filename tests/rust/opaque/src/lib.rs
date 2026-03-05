/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

/// Fast hash map used internally.
type FastHashMap<K, V> =
    std::collections::HashMap<K, V, std::hash::BuildHasherDefault<std::collections::hash_map::DefaultHasher>>;

pub type Foo = FastHashMap<i32, i32>;

pub type Bar = Result<Foo, ()>;

#[no_mangle]
pub extern "C" fn root(a: &Foo, b: &Bar) {}
