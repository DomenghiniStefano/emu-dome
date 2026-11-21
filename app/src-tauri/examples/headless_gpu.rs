//! The windowless test of `crates/emudome-core/examples/headless.rs` (same arguments, same
//! output), with the app's GPU: a core that can draw with Vulkan (Azahar) renders on the device
//! the app would give it, and its frames reach `Shared::frame` as in the app (GPU copy, then
//! readback). No window, no surface.
//!
//! cargo run --release -p emudome --example headless_gpu -- <core> <rom> [options]

#[expect(
    dead_code,
    reason = "the shared test's own `main` and helpers it does not use here"
)]
#[path = "../../../crates/emudome-core/examples/headless.rs"]
mod headless;

fn main() -> anyhow::Result<()> {
    let gpu = emudome_app::headless_setup();
    headless::run(&gpu)
}
