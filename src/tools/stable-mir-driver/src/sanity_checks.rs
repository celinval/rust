//! Module that contains sanity checks that Stable MIR APIs don't crash and that
//! their result is coherent.
//! These checks should only depend on StableMIR APIs. See other modules for tests that compare
//! the result between StableMIR and internal APIs.
use rustc_middle::ty::TyCtxt;
use rustc_smir::stable_mir;
use std::fmt::Debug;

type TestResult = Result<(), String>;

fn check_equal<T>(val: &T, expected: &T, msg: &str) -> TestResult
where
    T: Debug + PartialEq,
{
    if val != expected {
        Err(format!("{}: \n Expected: {:?}\n Found: {:?}", msg, expected, val))
    } else {
        Ok(())
    }
}

pub fn check(val: bool, msg: String) -> TestResult {
    if !val { Err(msg) } else { Ok(()) }
}

// Test that if there is an entry point, the function is part of `all_local_items`.
pub fn test_entry_fn() -> TestResult {
    let entry_fn = stable_mir::entry_fn();
    entry_fn.map_or(Ok(()), |entry_fn| {
        let all_items = stable_mir::all_local_items();
        check(all_items.contains(&entry_fn), format!("Failed to find entry_fn `{:?}`", entry_fn))
    })
}

pub fn test_all_fns() -> TestResult {
    let all_items = stable_mir::all_local_items();
    check(!all_items.is_empty(), "Failed to find any local item".to_string())
}

pub fn test_traits() -> TestResult {
    Ok(())
}

pub fn test_reachable_fns(tcx: TyCtxt<'_>) -> TestResult {
    Ok(())
}
