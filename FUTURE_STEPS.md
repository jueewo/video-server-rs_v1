# Future Steps - Architecture & Development Roadmap

## 🎯 Vision

Transform the current video server into a comprehensive platform with two distinct applications:

1. **Media Server** - Simple CRUD for videos and images (hosting platform)
2. **Learning Platform** - Complex interactive course system with progress tracking

## 📋 Executive Summary

**Current State:**
- Basic video streaming server with OIDC authentication
- Simple HTML-based UI
- Video and image serving capabilities

**Future State:**
- Professional media management with full CRUD operations
- Interactive learning platform with course creation and progress tracking
- Scalable architecture that can grow into microservices if needed

## 🏗️ Architecture Overview

### Recommended Approach: Modular Monolith → Microservices

```
┌─────────────────────────────────────────────────────────────────┐
│                    Single Application (Phase 1-2)               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌──────────────────┐         ┌──────────────────────────┐    │
│  │  Media Server    │         │  Learning Platform       │    │
│  │  (Module)        │         │  (Module)                │    │
│  │                  │         │                          │    │
│  │  - Video CRUD    │         │  - Course Creation       │    │
│  │  - Image CRUD    │         │  - Progress Tracking     │    │
│  │  - Metadata      │    ─────►  - Access Control        │    │
│  │  - Simple Forms  │   API   │  - Interactive UI        │    │
│  │  - Askama        │         │  - Leptos Framework      │    │
│  └──────────────────┘         └──────────────────────────┘    │
│           │                              │                      │
│           └──────────┬───────────────────┘                      │
│                      │                                          │
│            ┌─────────▼──────────┐                              │
│            │  Shared Components  │                              │
│            │  - Authentication   │                              │
│            │  - Database         │                              │
│            │  - User Management  │                              │
│            └─────────────────────┘                              │
└─────────────────────────────────────────────────────────────────┘
```

**Why Modular Monolith First:**
- ✅ Shared authentication (no duplication)
- ✅ Single deployment (simpler operations)
- ✅ No network overhead between modules
- ✅ Easy to split later if needed
- ✅ Faster development initially

**When to Split into Microservices:**
- Learning platform becomes too large (>50k lines)
- Need independent scaling
- Different deployment schedules required
- Separate teams working on each

## 📦 Project Structure

```
video-server-rs_v1/
├── Cargo.toml                          # Workspace configuration
│
├── crates/
│   ├── user-auth/                      # ✅ Existing - Shared authentication
│   │   ├── src/lib.rs
│   │   └── Cargo.toml
│   │
│   ├── video-manager/                  # 📝 TO ENHANCE - Media CRUD
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── crud.rs                 # NEW: CRUD operations
│   │   │   ├── api.rs                  # NEW: REST API
│   │   │   └── templates/              # NEW: Askama templates
│   │   │       ├── video_list.html
│   │   │       ├── video_edit.html
│   │   │       ├── video_create.html
│   │   │       └── video_delete.html
│   │   └── Cargo.toml
│   │
│   ├── image-manager/                  # 📝 TO ENHANCE - Image CRUD
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── crud.rs                 # NEW: CRUD operations
│   │   │   ├── api.rs                  # NEW: REST API
│   │   │   └── templates/              # NEW: Askama templates
│   │   │       ├── image_list.html
│   │   │       ├── image_edit.html
│   │   │       ├── image_create.html
│   │   │       └── gallery.html
│   │   └── Cargo.toml
│   │
│   ├── learning-platform/              # 🆕 NEW - Interactive learning
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── app.rs                  # Leptos application
│   │   │   ├── components/             # Leptos UI components
│   │   │   │   ├── mod.rs
│   │   │   │   ├── course_viewer.rs
│   │   │   │   ├── course_builder.rs
│   │   │   │   ├── module_list.rs
│   │   │   │   ├── video_player.rs
│   │   │   │   ├── progress_bar.rs
│   │   │   │   └── student_dashboard.rs
│   │   │   ├── api/                    # Backend API handlers
│   │   │   │   ├── mod.rs
│   │   │   │   ├── courses.rs
│   │   │   │   ├── modules.rs
│   │   │   │   ├── enrollment.rs
│   │   │   │   └── progress.rs
│   │   │   ├── models/                 # Data models
│   │   │   │   ├── mod.rs
│   │   │   │   ├── course.rs
│   │   │   │   ├── module.rs
│   │   │   │   ├── enrollment.rs
│   │   │   │   └── progress.rs
│   │   │   └── db/                     # Database operations
│   │   │       ├── mod.rs
│   │   │       ├── courses.rs
│   │   │       └── progress.rs
│   │   └── Cargo.toml
│   │
│   └── api-client/                     # 🆕 NEW - Shared API contracts
│       ├── src/
│       │   ├── lib.rs
│       │   ├── media.rs                # Media server API types
│       │   └── learning.rs             # Learning platform API types
│       └── Cargo.toml
│
├── src/
│   └── main.rs                         # Main server entry point
│
├── migrations/                         # Database migrations
│   ├── 001_initial.sql                 # ✅ Existing
│   ├── 002_courses.sql                 # NEW: Course tables
│   ├── 003_modules.sql                 # NEW: Module tables
│   ├── 004_progress.sql                # NEW: Progress tracking
│   └── 005_enrollments.sql             # NEW: Enrollment tables
│
├── docs/
│   ├── README.md
│   ├── auth/
│   ├── features/
│   │   ├── media-crud.md               # NEW: CRUD documentation
│   │   └── learning-platform.md        # NEW: Learning platform docs
│   └── architecture/
│       └── separation-strategy.md      # NEW: When/how to split
│
└── scripts/
```

## 🎯 Development Roadmap

### Phase 1: Media Server - Simple CRUD (Weeks 1-6)

**Objective:** Add full CRUD capabilities for videos and images with clean, simple forms.

#### Week 1-2: Setup & Video CRUD Foundation

**Tasks:**
- [ ] Add Askama template engine to video-manager
- [ ] Create base templates (layout, navigation)
- [ ] Design database schema updates for metadata

**Dependencies:**
```toml
# crates/video-manager/Cargo.toml
[dependencies]
askama = "0.12"
askama_axum = "0.4"
serde = { version = "1.0", features = ["derive"] }
```

**Database Changes:**
```sql
-- Add metadata columns to videos table
ALTER TABLE videos ADD COLUMN description TEXT;
ALTER TABLE videos ADD COLUMN tags TEXT;
ALTER TABLE videos ADD COLUMN duration INTEGER;
ALTER TABLE videos ADD COLUMN thumbnail_url TEXT;
ALTER TABLE videos ADD COLUMN upload_date DATETIME;
ALTER TABLE videos ADD COLUMN last_modified DATETIME;
```

**Routes to Add:**
```rust
GET    /videos                    → Video list page
GET    /videos/new                → Create video form
POST   /videos/create             → Handle video creation
GET    /videos/:id/edit           → Edit video form
POST   /videos/:id/update         → Handle video update
POST   /videos/:id/delete         → Delete video (with confirmation)
GET    /videos/:id                → View video details
```

#### Week 3-4: Video Templates & Forms

**Templates to Create:**

1. **video_list.html** - List all videos with filters
```html
{% extends "base.html" %}
{% block content %}
<div class="video-grid">
  {% for video in videos %}
    <div class="video-card">
      <img src="{{ video.thumbnail_url }}" />
      <h3>{{ video.title }}</h3>
      <p>{{ video.description }}</p>
      <div class="actions">
        <a href="/videos/{{ video.id }}/edit">Edit</a>
        <a href="/videos/{{ video.id }}/delete">Delete</a>
      </div>
    </div>
  {% endfor %}
</div>
{% endblock %}
```

2. **video_edit.html** - Edit form with all fields
3. **video_create.html** - Creation form
4. **video_detail.html** - View video with metadata

**Features:**
- File upload for videos
- Thumbnail upload/generation
- Tag management
- Description editor
- Visibility toggle (public/private)
- Form validation

#### Week 5-6: Image CRUD

**Similar implementation for images:**
- [ ] Image list page with gallery view
- [ ] Image upload with preview
- [ ] Bulk operations (delete multiple)
- [ ] Image metadata editing
- [ ] Album/collection organization

**Routes:**
```rust
GET    /images                    → Image gallery
GET    /images/new                → Upload form
POST   /images/create             → Handle upload
GET    /images/:id/edit           → Edit metadata
POST   /images/:id/update         → Update metadata
POST   /images/:id/delete         → Delete image
```

**Deliverables:**
- ✅ Functional CRUD for videos
- ✅ Functional CRUD for images
- ✅ Clean, simple forms (Askama)
- ✅ Basic validation
- ✅ Documentation

---

### Phase 2: API Layer (Weeks 7-9)

**Objective:** Create clean REST API for learning platform to consume.

#### Week 7-8: REST API Design & Implementation

**API Structure:**
```rust
// API v1 Routes
GET    /api/v1/videos              → List videos (with filters)
GET    /api/v1/videos/:id          → Get video details
GET    /api/v1/videos/:id/stream   → Get streaming URL
POST   /api/v1/videos/:id/view     → Track view (analytics)

GET    /api/v1/images              → List images
GET    /api/v1/images/:id          → Get image details
GET    /api/v1/images/:id/url      → Get image URL
```

**API Response Format:**
```rust
// api-client/src/media.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub duration: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub stream_url: String,
    pub visibility: String,
    pub tags: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageResponse {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub url: String,
    pub thumbnail_url: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub created_at: String,
}
```

**Authentication:**
```rust
// API requires valid session or API key
async fn api_auth_middleware(
    session: Session,
    headers: HeaderMap,
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Check session OR API key
    if is_authenticated(&session).await {
        return Ok(next.run(request).await);
    }
    
    if let Some(api_key) = headers.get("X-API-Key") {
        if validate_api_key(api_key).await? {
            return Ok(next.run(request).await);
        }
    }
    
    Err(StatusCode::UNAUTHORIZED)
}
```

#### Week 9: API Documentation & Testing

**Tasks:**
- [ ] Generate OpenAPI/Swagger documentation
- [ ] Create API test suite
- [ ] Write API usage guide
- [ ] Add rate limiting
- [ ] Add API versioning

**Tools:**
```toml
[dependencies]
utoipa = "4.0"          # OpenAPI generation
utoipa-swagger-ui = "4.0"
```

**Deliverables:**
- ✅ Complete REST API
- ✅ API documentation
- ✅ Test coverage
- ✅ Rate limiting

---

### Phase 3: Learning Platform Foundation (Weeks 10-15)

**Objective:** Set up Leptos and create basic course structure.

#### Week 10-12: Leptos Setup & Database Schema

**Add Leptos:**
```toml
# crates/learning-platform/Cargo.toml
[dependencies]
leptos = { version = "0.6", features = ["csr"] }
leptos_axum = "0.6"
leptos_router = "0.6"
leptos_meta = "0.6"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

**Database Schema:**
```sql
-- migrations/002_courses.sql
CREATE TABLE courses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    description TEXT,
    instructor_id INTEGER NOT NULL,
    thumbnail_url TEXT,
    visibility TEXT NOT NULL DEFAULT 'private', -- public, private, unlisted
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (instructor_id) REFERENCES users(id)
);

-- migrations/003_modules.sql
CREATE TABLE modules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    course_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    order_index INTEGER NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (course_id) REFERENCES courses(id) ON DELETE CASCADE
);

-- Module can contain multiple videos
CREATE TABLE module_videos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id INTEGER NOT NULL,
    video_id INTEGER NOT NULL,
    order_index INTEGER NOT NULL,
    title TEXT,                        -- Override video title for this module
    FOREIGN KEY (module_id) REFERENCES modules(id) ON DELETE CASCADE,
    FOREIGN KEY (video_id) REFERENCES videos(id) ON DELETE CASCADE,
    UNIQUE(module_id, video_id)
);

-- migrations/004_progress.sql
CREATE TABLE user_progress (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    course_id INTEGER NOT NULL,
    module_id INTEGER NOT NULL,
    video_id INTEGER NOT NULL,
    completed BOOLEAN NOT NULL DEFAULT FALSE,
    last_position REAL NOT NULL DEFAULT 0.0,  -- Seconds
    last_watched_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (course_id) REFERENCES courses(id),
    FOREIGN KEY (module_id) REFERENCES modules(id),
    FOREIGN KEY (video_id) REFERENCES videos(id),
    UNIQUE(user_id, course_id, video_id)
);

-- migrations/005_enrollments.sql
CREATE TABLE enrollments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    course_id INTEGER NOT NULL,
    enrolled_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    status TEXT NOT NULL DEFAULT 'active',  -- active, completed, dropped
    progress_percent REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (course_id) REFERENCES courses(id),
    UNIQUE(user_id, course_id)
);
```

**Data Models:**
```rust
// crates/learning-platform/src/models/course.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub instructor_id: i64,
    pub thumbnail_url: Option<String>,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub course_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub order_index: i32,
    pub videos: Vec<ModuleVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleVideo {
    pub id: i64,
    pub module_id: i64,
    pub video_id: i64,
    pub order_index: i32,
    pub title: Option<String>,
    // Video details from media server API
    pub video: VideoResponse,
}
```

#### Week 13-15: Basic Leptos UI

**Create Core Components:**

1. **Course List Component:**
```rust
// crates/learning-platform/src/components/course_list.rs
use leptos::*;

#[component]
pub fn CourseList(cx: Scope) -> impl IntoView {
    let (courses, set_courses) = create_signal(cx, vec![]);
    let (loading, set_loading) = create_signal(cx, true);

    create_effect(cx, move |_| {
        spawn_local(async move {
            let result = fetch_courses().await;
            set_courses.set(result);
            set_loading.set(false);
        });
    });

    view! { cx,
        <div class="course-list">
            <h1>"Available Courses"</h1>
            {move || {
                if loading.get() {
                    view! { cx, <p>"Loading courses..."</p> }.into_view(cx)
                } else {
                    courses.get().into_iter()
                        .map(|course| view! { cx,
                            <CourseCard course=course />
                        })
                        .collect_view(cx)
                }
            }}
        </div>
    }
}
```

2. **Course Viewer Component:**
```rust
// crates/learning-platform/src/components/course_viewer.rs
use leptos::*;
use leptos_router::*;

#[component]
pub fn CourseViewer(cx: Scope) -> impl IntoView {
    let params = use_params_map(cx);
    let course_id = move || {
        params.with(|p| p.get("id").and_then(|id| id.parse::<i64>().ok()))
    };

    let (course, set_course) = create_signal(cx, None);
    let (current_module, set_current_module) = create_signal(cx, None);
    let (current_video, set_current_video) = create_signal(cx, None);

    view! { cx,
        <div class="course-viewer">
            <div class="sidebar">
                <ModuleList 
                    course=course
                    on_select=move |video| set_current_video.set(Some(video))
                />
            </div>
            
            <div class="main-content">
                <VideoPlayer video=current_video />
                <VideoProgress video=current_video />
            </div>
        </div>
    }
}
```

**Routes:**
```rust
// crates/learning-platform/src/app.rs
use leptos::*;
use leptos_router::*;

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <Router>
            <Routes>
                <Route path="/courses" view=|cx| view! { cx, <CourseList/> }/>
                <Route path="/courses/:id" view=|cx| view! { cx, <CourseViewer/> }/>
                <Route path="/dashboard" view=|cx| view! { cx, <StudentDashboard/> }/>
                <Route path="/admin/courses" view=|cx| view! { cx, <CourseBuilder/> }/>
            </Routes>
        </Router>
    }
}
```

**Deliverables:**
- ✅ Database schema for courses
- ✅ Basic Leptos setup
- ✅ Course list page
- ✅ Course viewer page
- ✅ Module navigation

---

### Phase 4: Learning Platform - Advanced Features (Weeks 16-24)

**Objective:** Add interactive features, progress tracking, and access control.

#### Week 16-18: Progress Tracking

**Features:**
- [ ] Track video watch time
- [ ] Mark videos as completed
- [ ] Calculate course progress percentage
- [ ] Resume from last position
- [ ] Progress dashboard

**Implementation:**
```rust
// crates/learning-platform/src/components/video_player.rs
#[component]
pub fn VideoPlayer(cx: Scope, video: ReadSignal<Option<VideoResponse>>) -> impl IntoView {
    let (progress, set_progress) = create_signal(cx, 0.0);
    let (completed, set_completed) = create_signal(cx, false);
    
    // Load saved progress
    create_effect(cx, move |_| {
        if let Some(v) = video.get() {
            spawn_local(async move {
                let saved = fetch_progress(v.id).await;
                set_progress.set(saved.last_position);
            });
        }
    });

    // Save progress periodically
    create_effect(cx, move |_| {
        // Every 5 seconds, save current position
        set_interval(
            move || {
                if let Some(v) = video.get() {
                    spawn_local(async move {
                        save_progress(v.id, progress.get()).await;
                    });
                }
            },
            Duration::from_secs(5),
        );
    });

    view! { cx,
        <div class="video-player-container">
            <video 
                src=move || video.get().map(|v| v.stream_url)
                on:timeupdate=move |ev| {
                    let time = event_target_value(&ev).parse::<f64>().unwrap_or(0.0);
                    set_progress.set(time);
                    
                    // Mark as completed if watched 90%
                    if let Some(v) = video.get() {
                        if time / v.duration as f64 > 0.9 {
                            set_completed.set(true);
                        }
                    }
                }
            />
            <ProgressBar progress=progress />
        </div>
    }
}
```

#### Week 19-21: Course Builder (Instructor Interface)

**Features:**
- [ ] Create/edit courses
- [ ] Add/remove modules
- [ ] Organize videos into modules
- [ ] Drag-and-drop reordering
- [ ] Preview course as student

**UI Components:**
```rust
// Interactive course builder with drag-and-drop
#[component]
pub fn CourseBuilder(cx: Scope) -> impl IntoView {
    let (modules, set_modules) = create_signal(cx, vec![]);
    let (available_videos, set_available_videos) = create_signal(cx, vec![]);

    view! { cx,
        <div class="course-builder">
            <div class="course-info">
                <input type="text" placeholder="Course Title" />
                <textarea placeholder="Course Description" />
            </div>
            
            <div class="modules-section">
                <h2>"Course Modules"</h2>
                <ModuleBuilder 
                    modules=modules
                    on_reorder=move |new_order| set_modules.set(new_order)
                />
            </div>
            
            <div class="video-library">
                <h2>"Video Library"</h2>
                <VideoLibrary 
                    videos=available_videos
                    on_add_to_module=move |video, module_id| {
                        // Add video to module
                    }
                />
            </div>
            
            <button on:click=|_| save_course()>"Save Course"</button>
            <button on:click=|_| preview_course()>"Preview"</button>
        </div>
    }
}
```

#### Week 22-24: Access Control & Enrollment

**Features:**
- [ ] Course enrollment system
- [ ] Public vs private courses
- [ ] Payment integration (optional)
- [ ] Certificate generation
- [ ] Student management

**Access Control:**
```rust
// Check if user has access to course
async fn check_course_access(
    user_id: i64,
    course_id: i64,
) -> Result<bool, Error> {
    let course = get_course(course_id).await?;
    
    // Public courses: everyone can access
    if course.visibility == "public" {
        return Ok(true);
    }
    
    // Private courses: must be enrolled
    if course.visibility == "private" {
        return check_enrollment(user_id, course_id).await;
    }
    
    // Instructors can always access their courses
    if course.instructor_id == user_id {
        return Ok(true);
    }
    
    Ok(false)
}
```

**Enrollment System:**
```rust
// API endpoints for enrollment
POST   /api/v1/courses/:id/enroll       → Enroll in course
DELETE /api/v1/courses/:id/enroll       → Unenroll
GET    /api/v1/enrollments              → User's enrollments
GET    /api/v1/courses/:id/students     → Course students (instructor only)
```

**Deliverables:**
- ✅ Full progress tracking
- ✅ Interactive course builder
- ✅ Access control system
- ✅ Enrollment management
- ✅ Student dashboard

---

## 🔄 Migration Strategy: Monolith → Microservices

### When to Consider Splitting

**Triggers for separation:**
1. Learning platform exceeds 50,000 lines of code
2. Need independent scaling (different load patterns)
3. Separate deployment schedules required
4. Different teams working on each service
5. Performance bottlenecks from shared resources

### How to Split

**Phase 1: Prepare for Separation**
```
Week 1-2: API Contract Definition
- Define clear API boundaries
- Version all APIs (v1, v2, etc.)
- Document all endpoints
- Create API client library

Week 3-4: Separate Databases
- Create separate database for learning platform
- Migrate course/progress tables
- Update connection strings
- Test data migration
```

**Phase 2: Service Extraction**
```
Week 5-6: Deploy Learning Platform Separately
- Create separate deployment pipeline
- Configure reverse proxy (Caddy/Nginx)
- Set up service discovery
- Configure health checks

Week 7-8: Testing & Monitoring
- End-to-end testing
- Load testing
- Set up monitoring (Prometheus/Grafana)
- Configure alerting
```

**Microservices Architecture:**
```
┌──────────────────────────────────────────────────────┐
│                  API Gateway / Reverse Proxy          │
│                     (Caddy / Nginx)                   │
└────────────┬─────────────────────────┬────────────────┘
             │                         │
             ▼                         ▼
┌────────────────────────┐   ┌────────────────────────┐
│   Media Server         │   │  Learning Platform     │
│   (Port 3000)          │   │  (Port 4000)           │
├────────────────────────┤   ├────────────────────────┤
│ - Video CRUD           │   │ - Leptos UI            │
│ - Image CRUD           │   │ - Course Management    │
│ - Streaming            │   │ - Progress Tracking    │
│ - Simple HTML          │   │ - Interactive Features │
│                        │   │                        │
│ SQLite / PostgreSQL    │   │ PostgreSQL             │
└────────────┬───────────┘   └───────────┬────────────┘
             │                           │
             └────────────┬──────────────┘
                          │
                  ┌───────▼────────┐
                  │  Auth Service  │
                  │  (Casdoor)     │
                  └────────────────┘
```

**Communication:**
```rust
// Media server provides API
GET /api/v1/videos/:id

// Learning platform consumes it
let video = reqwest::get(
    format!("{}/api/v1/videos/{}", MEDIA_SERVER_URL, id)
)
.await?
.json::<VideoResponse>()
.await?;
```

---

## 📊 Technology Stack Summary

### Media Server (Simple)
- **Framework:** Axum
- **Templates:** Askama
- **Database:** SQLite → PostgreSQL (optional)
- **Deployment:** Single binary

### Learning Platform (Complex)
- **Framework:** Axum + Leptos
- **Frontend:** Leptos (Rust → WASM)
- **Database:** PostgreSQL (recommended for complex queries)
- **Caching:** Redis (for progress tracking)
- **Deployment:** Binary + static assets (WASM)

### Shared
- **Authentication:** Casdoor (OIDC)
- **Storage:** File system / S3
- **Monitoring:** Prometheus + Grafana
- **Logging:** tracing / log
- **CI/CD:** GitHub Actions

---

## 📈 Success Metrics

### Media Server
- [ ] All CRUD operations functional
- [ ] < 100ms response time for forms
- [ ] Clean, intuitive UI
- [ ] Full test coverage (>80%)

### Learning Platform
- [ ] Course creation in < 5 minutes
- [ ] Progress tracking real-time updates
- [ ] < 2s page load time
- [ ] Mobile responsive UI
- [ ] >90% uptime

---

## 🎓 Learning Resources

### Askama (Templates)
- Documentation: https://djc.github.io/askama/
- Examples: https://github.com/djc/askama/tree/main/testing

### Leptos (Interactive UI)
- Book: https://leptos-rs.github.io/leptos/
- Examples: https://github.com/leptos-rs/leptos/tree/main/examples
- Video Tutorial: https://www.youtube.com/playlist?list=PL7r-PXl6ZPcCIOFaL7nVHXZvBmHNhrh_Q

### Microservices Patterns
- API Gateway Pattern
- Service Discovery
- Circuit Breaker
- Event-Driven Architecture

---

## 🚀 Getting Started

### Immediate Next Steps

1. **Week 1: Add Askama to video-manager**
```bash
cd crates/video-manager
cargo add askama askama_axum
```

2. **Week 1: Create first template**
```bash
mkdir -p crates/video-manager/src/templates
touch crates/video-manager/src/templates/video_list.html
```

3. **Week 2-3: Implement video CRUD**
- Create forms
- Add routes
- Test functionality

4. **Week 4+: Continue with roadmap**

---

## 📝 Notes & Considerations

### Database Choice
- **SQLite:** Good for development and small deployments
- **PostgreSQL:** Recommended for production (better concurrency, complex queries)
- **Migration:** Plan for SQLite → PostgreSQL migration path

### State Management
- Media server: Simple, stateless
- Learning platform: Complex state (use Leptos signals)

### Caching Strategy
- Video metadata: Cache with Redis
- Progress tracking: Write-through cache
- Course data: Cache invalidation on updates

### Scalability
- Media server: Stateless, easy to scale horizontally
- Learning platform: More complex, consider session affinity

### Security
- API authentication: JWT or API keys
- Rate limiting: Per user/IP
- CSRF protection: Built into Axum/Leptos
- SQL injection: Use parameterized queries (sqlx)

---

## 🎯 Summary

This roadmap provides a clear path from simple CRUD operations to a full-featured learning platform:

1. **Phase 1 (6 weeks):** Add simple CRUD with Askama
2. **Phase 2 (3 weeks):** Create REST API
3. **Phase 3 (6 weeks):** Build Leptos foundation
4. **Phase 4 (9 weeks):** Advanced features

**Total timeline:** ~24 weeks (6 months)

**Key decisions:**
- ✅ Start with modular monolith
- ✅ Use Askama for simple CRUD
- ✅ Use Leptos for complex learning UI
- ✅ Split into microservices only when needed

**This architecture:**
- Separates concerns appropriately
- Uses the right tool for each job
- Allows for future growth
- Maintains simplicity where possible
- Adds complexity only where it provides value

---

**Last Updated:** January 2024  
**Status:** Planning Phase  
**Next Review:** After Phase 1 completion