/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub mod other;

#[path = "other2.rs"]
pub mod other2;

pub mod other3 {
    #[path = "other4.rs"]
    pub mod other4;
}
