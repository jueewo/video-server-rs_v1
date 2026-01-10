// Update the login handler to ensure session is saved
async fn login_handler(mut session: Session) -> Result<&'static str, StatusCode> {
    session.insert("user_id", 1u32).await.map_err(|e| {
        eprintln!("Failed to insert user_id: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    session.insert("authenticated", true).await.map_err(|e| {
        eprintln!("Failed to insert authenticated: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Force save the session
    session.save().await.map_err(|e| {
        eprintln!("Failed to save session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    println!("✅ User logged in successfully");
    Ok("Logged in – you can now view private and live streams")
}
