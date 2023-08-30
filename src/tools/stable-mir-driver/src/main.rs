//! Test that users are able to inspec the MIR body of functions and types

#![feature(rustc_private)]
#![feature(assert_matches)]

mod sanity_checks;

extern crate rustc_middle;
extern crate rustc_smir;

use rustc_middle::ty::TyCtxt;
use rustc_smir::{rustc_internal, stable_mir};
use std::process::ExitCode;

const CHECK_ARG: &str = "--check-smir";

/// This is a wrapper that can be used to replace rustc.
///
/// Besides all supported rustc arguments, use `--check-smir` to run all the stable-mir checks.
/// This allows us to use this tool in cargo projects to analyze the target crate only by running
/// `cargo rustc --check-smir`.
fn main() -> ExitCode {
    let mut check_smir = false;
    let args: Vec<_> = std::env::args()
        .filter(|arg| {
            let is_check_arg = arg == CHECK_ARG;
            check_smir |= is_check_arg;
            !is_check_arg
        })
        .collect();

    let result = if check_smir {
        rustc_internal::StableMir::new(args, test_stable_mir).run()
    } else {
        rustc_internal::StableMir::new(args, |_| {}).run()
    };
    if result.is_ok() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}

macro_rules! run_tests {
    ($( $test:ident($($tcx:ident)?) ),+) => {
        [$({
            let result = $test($($tcx)?);
            println!("Test {}: {}", stringify!($test), result.as_ref().err().unwrap_or(&"Ok".to_string()));
            result
        },)+]
    };
}

/// This function invoke other tests and process their results.
/// Tests should avoid panic,
fn test_stable_mir(tcx: TyCtxt<'_>) {
    use sanity_checks::*;

    let results =
        run_tests![test_entry_fn(), test_all_fns(), test_traits(), test_reachable_fns(tcx)];
    results.iter().any(Result::is_err).then(|| panic!("Tests failed."));
}
