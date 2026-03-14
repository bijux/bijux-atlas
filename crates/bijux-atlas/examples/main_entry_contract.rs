fn main() {
    let entry: fn() -> std::process::ExitCode = bijux_atlas::main_entry;
    let _ = entry;
    println!("cli-entrypoint=main_entry");
}
