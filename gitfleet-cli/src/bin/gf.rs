// `gf` is a shorthand binary that shares the CLI implementation with
// `gitfleet`. Coverage builds execute the primary binary's tests, so compiling
// the full include here would add an unexecuted duplicate of every CLI module
// to the coverage denominator.
#[cfg(not(coverage))]
include!("../main.rs");

#[cfg(coverage)]
fn main() {}
