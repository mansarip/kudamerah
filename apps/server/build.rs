fn main() {
    if std::env::var_os("CARGO_FEATURE_EMBED_WEB").is_some() {
        // Re-run include_dir when assets are added or removed, not only when an existing
        // file changes. This matters for Vite's content-hashed filenames.
        println!("cargo::rerun-if-changed=../web");
    }
}
