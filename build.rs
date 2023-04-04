fn main() {
    // This allow using #[cfg(sqlite)] instead of #[cfg(feature = "sqlite")], which helps when trying to add them through macros
    #[cfg(feature = "sqlite")]
    println!("cargo:rustc-cfg=sqlite");
    #[cfg(feature = "mysql")]
    println!("cargo:rustc-cfg=mysql");
    #[cfg(feature = "postgresql")]
    println!("cargo:rustc-cfg=postgresql");
    #[cfg(feature = "query_logger")]
    println!("cargo:rustc-cfg=query_logger");

    #[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgresql")))]
    compile_error!(
        "You need to enable one DB backend. To build with previous defaults do: cargo build --features sqlite"
    );
}
