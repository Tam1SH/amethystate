fn main() {
    println!("cargo::rustc-check-cfg=cfg(backend, values(\"redb\", \"json\", \"toml\", \"ron\"))");

    let redb = std::env::var_os("CARGO_FEATURE_REDB").is_some();
    let json = std::env::var_os("CARGO_FEATURE_JSON").is_some();
    let toml = std::env::var_os("CARGO_FEATURE_TOML").is_some();
    let ron = std::env::var_os("CARGO_FEATURE_RON").is_some();

    let mut active = Vec::new();
    if redb {
        active.push("redb");
    }
    if json {
        active.push("json");
    }
    if toml {
        active.push("toml");
    }
    if ron {
        active.push("ron");
    }

    let selected = if json {
        "json"
    } else if toml {
        "toml"
    } else if ron {
        "ron"
    } else {
        "redb"
    };

    println!("cargo:rustc-cfg=backend=\"{}\"", selected);

    if active.len() > 1 {
        println!(
            "cargo:warning=Multiple storage backends enabled: {:?}. Defaulting to '{}'.",
            active, selected
        );
    }
}
