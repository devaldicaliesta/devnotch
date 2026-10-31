use embed_manifest::manifest::{ActiveCodePage, DpiAwareness, ExecutionLevel, MaxVersionTested, Setting};
use embed_manifest::{embed_manifest, new_manifest};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let manifest = new_manifest("Notchi")
            .requested_execution_level(ExecutionLevel::AsInvoker)
            .dpi_awareness(DpiAwareness::PerMonitorV2Only)
            .long_path_aware(Setting::Enabled)
            .active_code_page(ActiveCodePage::Utf8)
            .max_version_tested(MaxVersionTested::Windows11Version22H2);
        embed_manifest(manifest).expect("gagal menanam manifest");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
