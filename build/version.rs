// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use rustc_version::{version, Version};
use std::io::{self, Write};
use std::process::exit;

/// MIN_VERSION should be changed when there's a new minimum version of rustc required
/// to build the project.
static MIN_VERSION: &str = "1.43.0";

fn main() {
    let ver = version().unwrap();
    let min = Version::parse(MIN_VERSION).unwrap();
    if ver < min {
        writeln!(
            &mut io::stderr(),
            "Mentat requires rustc {} or higher, you were using version {}.",
            MIN_VERSION,
            ver
        )
        .unwrap();
        exit(1);
    }
}
