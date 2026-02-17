#!/bin/bash

# Deactivate Legacy Managers Script
# This script begins the process of deactivating image-manager and document-manager
# in favor of the unified media-manager and media-hub system.

set -e

echo "🔧 Legacy Manager Deactivation Script"
echo "======================================"
echo ""
echo "This script will:"
echo "1. Comment out legacy manager dependencies in Cargo.toml"
echo "2. Comment out legacy manager usage in main.rs"
echo "3. Add redirect routes for backward compatibility"
echo "4. Update navigation links to use unified routes"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo ""
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled"
    exit 1
fi

# Backup files
echo "📦 Creating backups..."
cp Cargo.toml Cargo.toml.backup
cp src/main.rs src/main.rs.backup
echo "✅ Backups created (Cargo.toml.backup, src/main.rs.backup)"

# Comment out in Cargo.toml
echo ""
echo "🔧 Updating Cargo.toml..."
sed -i.bak 's/^image-manager = /# image-manager = # DEACTIVATED - use media-manager\n# image-manager = /' Cargo.toml
sed -i.bak 's/^document-manager = /# document-manager = # DEACTIVATED - use media-manager\n# document-manager = /' Cargo.toml
rm Cargo.toml.bak
echo "✅ Cargo.toml updated"

# Comment out imports in main.rs
echo ""
echo "🔧 Updating main.rs imports..."
sed -i.bak 's/^use image_manager::/\/\/ DEACTIVATED: use image_manager::/' src/main.rs
sed -i.bak 's/^use document_manager::/\/\/ DEACTIVATED: use document_manager::/' src/main.rs
rm src/main.rs.bak

echo "✅ main.rs imports commented out"

# Create redirect routes file
echo ""
echo "📝 Creating redirect routes..."
cat > src/legacy_redirects.rs << 'EOF'
//! Legacy route redirects for backward compatibility
//!
//! These redirects ensure that old links to /images and /documents
//! continue to work by redirecting to the new unified /media routes.

use axum::{
    extract::Path,
    response::Redirect,
    Router,
    routing::get,
};

pub fn legacy_redirect_routes() -> Router {
    Router::new()
        // Image routes
        .route("/images", get(redirect_images_list))
        .route("/images/view/:slug", get(redirect_image_view))

        // Document routes
        .route("/documents", get(redirect_documents_list))
        .route("/documents/:slug", get(redirect_document_view))
}

async fn redirect_images_list() -> Redirect {
    Redirect::permanent("/media?type=image")
}

async fn redirect_image_view(Path(slug): Path<String>) -> Redirect {
    Redirect::permanent(&format!("/images/{}", slug))
}

async fn redirect_documents_list() -> Redirect {
    Redirect::permanent("/media?type=document")
}

async fn redirect_document_view(Path(slug): Path<String>) -> Redirect {
    // For markdown files, redirect to viewer
    // For other documents, redirect to media detail
    Redirect::permanent(&format!("/media/{}", slug))
}
EOF
echo "✅ Created src/legacy_redirects.rs"

# Update navbar template
echo ""
echo "🔧 Updating navbar links..."
if [ -f "templates/components/navbar.html" ]; then
    # Update Images link
    sed -i.bak 's|href="/images"|href="/media?type=image" title="View all images"|' templates/components/navbar.html

    # Update Documents link
    sed -i.bak 's|href="/documents"|href="/media?type=document" title="View all documents"|' templates/components/navbar.html

    rm templates/components/navbar.html.bak 2>/dev/null || true
    echo "✅ Navbar links updated"
else
    echo "⚠️  navbar.html not found, skipping"
fi

echo ""
echo "📋 Next Steps:"
echo ""
echo "1. Add to src/main.rs (near other mod declarations):"
echo "   mod legacy_redirects;"
echo ""
echo "2. Add to src/main.rs (in the Router chain, BEFORE other routes):"
echo "   .merge(legacy_redirects::legacy_redirect_routes())"
echo ""
echo "3. Comment out these lines in src/main.rs:"
echo "   - let image_state = Arc::new(ImageManagerState::new(...));"
echo "   - let document_state = Arc::new(DocumentManagerState::new(...));"
echo "   - .merge(image_routes().with_state(image_state))"
echo "   - .merge(document_routes().with_state(document_state))"
echo ""
echo "4. Test the application:"
echo "   cargo build"
echo "   cargo run"
echo ""
echo "5. Verify redirects work:"
echo "   curl -I http://localhost:8080/images"
echo "   curl -I http://localhost:8080/documents"
echo ""
echo "6. If everything works, commit changes:"
echo "   git add -A"
echo "   git commit -m 'Deactivate legacy image-manager and document-manager'"
echo ""
echo "7. Monitor for issues over 1-2 weeks before full removal"
echo ""
echo "✅ Initial deactivation complete!"
echo ""
echo "⚠️  IMPORTANT: Review the changes before building/running"
echo "    Backups saved as: Cargo.toml.backup, src/main.rs.backup"
