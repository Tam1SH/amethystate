fn main() {
    let backends = ["json", "toml", "ron", "sqlite", "redb"];

    println!(
        "cargo::rustc-check-cfg=cfg(backend, values(\"redb\", \"json\", \"toml\", \"ron\", \"sqlite\"))"
    );

    let active: Vec<&str> = backends
        .iter()
        .copied()
        .filter(|&b| std::env::var_os(format!("CARGO_FEATURE_{}", b.to_uppercase())).is_some())
        .collect();

    let selected = active.first().copied().unwrap_or("redb");
    println!("cargo:rustc-cfg=backend=\"{selected}\"");

    if active.len() > 1 {
        println!(
            "cargo:warning=Multiple storage backends enabled: {active:?}. Defaulting to '{selected}'."
        );
    }
}
