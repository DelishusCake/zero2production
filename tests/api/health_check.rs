use sqlx::PgPool;

use crate::helpers::TestApp;

#[sqlx::test]
async fn is_present(pool: PgPool) -> sqlx::Result<()> {
    let app = TestApp::spawn(&pool).await;

    let res = app.health_check().await.expect("Failed to execute request");

    assert!(res.status().is_success());

    Ok(())
}
