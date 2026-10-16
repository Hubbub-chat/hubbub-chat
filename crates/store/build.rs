// `sqlx::migrate!` embeds the migration files at compile time; rebuild when they change.
fn main() {
    println!("cargo:rerun-if-changed=migrations");
}
