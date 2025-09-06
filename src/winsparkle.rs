// Slice 4D Shooter - the first multiplayer shooter set in 4D space
// Copyright (C) 2023-2025  Timofei Molokov

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

#[cfg(target_os = "windows")]
use libloading::{Library, Symbol};
#[cfg(target_os = "windows")]
use std::{ffi::{CString, OsStr}, os::windows::ffi::OsStrExt, sync::Once};

#[cfg(target_os = "windows")]
static INIT: Once = Once::new();
#[cfg(target_os = "windows")]
static mut LIB: Option<Library> = None;

#[cfg(target_os = "windows")]
fn wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

// This module loads WinSparkle.dll on Windows
// to integrate automatic update checking.
// During initialization it sets app details and appcast URL,
// then runs the library to show the update UI.
// The current version of the client is displayed at this address
// https://maffi44.github.io/slice-4d-shooter/appcast.xml
#[cfg(target_os = "windows")]
pub fn init() {
    unsafe {
        INIT.call_once(|| {
            let lib = Library::new("WinSparkle.dll").expect("load WinSparkle.dll");

            type SetAppDetails = unsafe extern "C" fn(*const u16, *const u16, *const u16);
            type SetAppcastUrl = unsafe extern "C" fn(*const i8);
            type VoidFn        = unsafe extern "C" fn();
            type SetShutdownCb = unsafe extern "C" fn(unsafe extern "C" fn());
            type SetCanShutCb  = unsafe extern "C" fn(unsafe extern "C" fn() -> i32);

            let set_details: Symbol<SetAppDetails> = lib.get(b"win_sparkle_set_app_details\0").unwrap();
            let set_url:     Symbol<SetAppcastUrl> = lib.get(b"win_sparkle_set_appcast_url\0").unwrap();
            let init_fn:     Symbol<VoidFn>        = lib.get(b"win_sparkle_init\0").unwrap();
            let check_ui:    Symbol<VoidFn>        = lib.get(b"win_sparkle_check_update_with_ui\0").unwrap();
            let set_shut:    Symbol<SetShutdownCb> = lib.get(b"win_sparkle_set_shutdown_request_callback\0").unwrap();
            let set_can:     Symbol<SetCanShutCb>  = lib.get(b"win_sparkle_set_can_shutdown_callback\0").unwrap();


            unsafe extern "C" fn can_shutdown() -> i32 { 1 }
            unsafe extern "C" fn do_shutdown() { std::process::exit(0); }

            set_can(can_shutdown);
            set_shut(do_shutdown);

            let version_w = wide(env!("CARGO_PKG_VERSION"));
            set_details(wide("Slice 4D Shooter").as_ptr(), wide("Slice 4D Shooter").as_ptr(), version_w.as_ptr());
            let url = CString::new("https://maffi44.github.io/slice-4d-shooter/appcast.xml").unwrap();
            set_url(url.as_ptr());

            init_fn();
            check_ui();

            LIB = Some(lib);
        });
    }
}

#[cfg(not(target_os = "windows"))]
pub fn init() {}
