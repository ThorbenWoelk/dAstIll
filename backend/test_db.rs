use libsql::{Builder, Connection, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let db = Builder::new_remote("libsql://test.turso.io".to_string(), "token".to_string())
        .build()
        .await?;
    let conn = db.connect()?;
    
    // check if it's sync or async
    let mut stmt = conn.prepare("SELECT 1")?; // wait, prepare might be async
    println!("hello");
    Ok(())
}
