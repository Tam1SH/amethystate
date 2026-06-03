fn main() {
    let backends = ["redb", "json"];

    let active_backends: Vec<_> = backends
        .iter()
        .filter(|&f| {
            let env_var = format!("CARGO_FEATURE_{}", f.to_uppercase().replace('-', "_"));
            std::env::var(env_var).is_ok()
        })
        .collect();

    if active_backends.is_empty() {
        panic!(
            "\n\n[Build Error] You must enable exactly one backend feature from: {:?}\n\n",
            backends
        );
    }

    if active_backends.len() > 1 {
        panic!(
            "\n\n[Build Error] Multiple backend features selected: {:?}. Only one is allowed.\n\n",
            active_backends
        );
    }
}
