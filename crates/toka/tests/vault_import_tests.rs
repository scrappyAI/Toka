use toka::prelude::*;

#[test]
fn vault_prelude_includes_vault() {
    // Compile-time assertion: fails if `Vault` is not in scope.
    let _vault = Vault::new_memory();
} 