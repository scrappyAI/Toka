use toka_cli::{CliApp, Commands};
use serde_json::Value;

#[tokio::test]
async fn mint_happy_path() {
    let app = CliApp::new();
    let output = app
        .execute(Commands::Mint {
            asset: 1,
            to: 42,
            amount: 1000,
        })
        .await
        .expect("mint should succeed");

    let v: Value = serde_json::from_str(&output).expect("valid JSON");
    assert!(v.get("AssetMinted").is_some());
    let minted = &v["AssetMinted"];
    assert_eq!(minted["to"].as_u64(), Some(42));
    assert_eq!(minted["amount"].as_u64(), Some(1000));
}

#[tokio::test]
async fn transfer_without_balance_fails() {
    let app = CliApp::new();
    let res = app
        .execute(Commands::Transfer {
            from: 1,
            to: 2,
            amount: 50,
        })
        .await;
    assert!(res.is_err(), "transfer should fail due to insufficient balance");
    let err = format!("{:#?}", res.unwrap_err());
    assert!(err.to_lowercase().contains("insufficient"));
}

#[tokio::test]
async fn balance_query_after_mint() {
    let app = CliApp::new();
    app.execute(Commands::Mint {
        asset: 1,
        to: 99,
        amount: 200,
    })
    .await
    .expect("mint should succeed");

    let out = app
        .execute(Commands::Balance { entity: 99 })
        .await
        .expect("balance query should succeed");

    let v: Value = serde_json::from_str(&out).expect("valid JSON");
    assert_eq!(v["balance"].as_u64(), Some(200));
}