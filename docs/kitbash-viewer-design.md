# Kitbash 3D Mesh Viewer - Design Document

## Project Overview

A lightweight 3D mesh visualization tool for iterative procedural geometry authoring. The tool watches a directory for OBJ mesh files and provides real-time 3D visualization with minimal UI and keyboard-driven interaction.

### Primary Use Case

Developer writes Haskell code that generates 3D geometry, periodically outputting OBJ mesh files to a watched directory. The viewer sits in a separate window/monitor, automatically refreshing the 3D view as files are created or modified, enabling rapid visual feedback during development.

### Core Requirements

- **Lightweight**: Single native binary, no runtime dependencies, works completely offline
- **Simple**: No complex UI - mouse manipulation and keyboard shortcuts only
- **Fast**: Immediate visual feedback on file changes
- **Portable**: Cross-platform (macOS, Linux, Windows)
- **Focused**: Supports OBJ format only, no texturing, basic geometry visualization

### Technology Stack

- **Backend**: Rust native binary
  - HTTP server for serving viewer and mesh files
  - File system watching for auto-reload
  - WebSocket for push notifications
- **Frontend**: Browser-based viewer
  - Three.js for 3D rendering
  - Single HTML page with embedded JavaScript
  - All assets embedded in Rust binary for offline operation

---

## Interaction & Interface Specification

### Mouse Controls

- **Left drag**: Orbit/rotate camera around target point
- **Right drag** (or Ctrl+Left drag): Pan view (move target point in screen space)
- **Scroll wheel**: Zoom in/out
- **Click on mesh**: Select object and set as view target

### Keyboard Controls

#### Standard View Angles (1-6)
Set camera to axis-aligned orthographic-style views, framing all visible objects:

- `1`: Front view (+Z looking at -Z)
- `2`: Back view (-Z looking at +Z)
- `3`: Right view (+X looking at -X)
- `4`: Left view (-X looking at +X)
- `5`: Top view (+Y looking at -Y)
- `6`: Bottom view (-Y looking at +Y)

#### Object Navigation
- `[`: Previous object in file list (by modification time, newest first)
- `]`: Next object in file list

#### View Framing
- `f`: Frame current selected object (fit in view)
- `F`: Frame all visible objects (reset to scene bounding box)
- `Space`: Reset camera to default 3/4 isometric view

#### Visibility
- `h`: Toggle hide/show for current selected object
- `H`: Toggle hide all / show all

#### Display Modes
- `l`: Toggle file list overlay
- `w`: Toggle wireframe view
- `e`: Toggle wireframe overlay (edges on top of solid shading)
- `g`: Toggle grid/ground plane (10x10 unit grid at y=0)

#### Utility
- `r`: Force reload all files from disk
- `'`: Quit (tell server to shut down)

### Visual Design

#### Default View
- Camera starts at default 3/4 isometric position (e.g., position [1, 1, 1] looking at origin)
- Scene centered on bounding box of all loaded objects
- Simple directional + ambient lighting
- Background: Dark grey (#2a2a2a) - common in 3D tools, easy on eyes

#### Rendering
- Solid shaded geometry using Phong or Standard material
- Smooth shading if vertex normals present in OBJ, auto-generated otherwise
- Wireframe overlay mode: black edge lines rendered on top of solid surface
- Selected object indicated by subtle emissive glow

#### UI Overlay
Minimal text overlay in top-left corner (toggleable with `l` key):

```
● terrain.obj      17:42    ← filled circle visible
○ cube.obj         17:41    ← unfilled circle hidden 
● sphere.obj       17:38
● building.obj     17:35
```

Elements:
- Filename
- Modification time (HH:MM format)
- Status indicator: ● (visible), ○ (hidden)
- Selection indicated by highlighting of line
- Files ordered by modification time, newest first

---

## Architecture Design

### System Components

#### 1. Rust Backend (Native Binary)

**Module Structure**:
```
src/
  main.rs           Entry point, CLI setup
  server.rs         HTTP server + WebSocket handler
  watcher.rs        File system monitoring
  config.rs         Config file parsing
  assets.rs         Embedded HTML/JS/Three.js
```

**Dependencies**:
- `axum` - Web framework and HTTP server
- `tokio` - Async runtime
- `tower-http` - Static file serving middleware
- `notify` - Cross-platform file system watching
- `clap` - CLI argument parsing
- `serde` + `toml` - Config file serialization
- `axum::extract::ws` - WebSocket support

**Responsibilities**:
- Serve embedded HTML/JS/CSS viewer page
- Serve Three.js library (embedded in binary)
- Serve OBJ files from watched directory
- Watch directory for file changes (create, modify, delete)
- Push file change notifications via WebSocket
- Parse CLI arguments and config file

#### 2. Browser Frontend

**File Structure** (embedded in Rust binary):
```
assets/
  index.html        Main viewer page
  viewer.js         Three.js viewer implementation
  three.min.js      Three.js library (~600KB minified)
  style.css         Minimal styling (optional)
```

**Responsibilities**:
- Three.js scene setup and rendering
- OBJ file loading and parsing (using Three.js OBJLoader)
- Camera controls (OrbitControls from Three.js)
- Keyboard and mouse input handling
- WebSocket client for receiving file updates
- File list overlay rendering
- Object selection and highlighting

### Data Flow

```
┌─────────────────────────────────────────────────┐
│ Haskell Code (geometry authoring - out of scope)│
└─────────────────┬───────────────────────────────┘
                  │ writes OBJ files
                  ▼
         ┌─────────────────┐
         │  ./output/*.obj │ (watched directory)
         └────────┬────────┘
                  │ file system events
                  ▼
         ┌─────────────────┐
         │  Rust Watcher   │ (notify crate)
         └────────┬────────┘
                  │ WebSocket notification
                  ▼
         ┌─────────────────┐
         │ Browser Client  │
         └────────┬────────┘
                  │ HTTP GET /obj/filename
                  ▼
         ┌─────────────────┐
         │  Rust Server    │ reads from disk
         └────────┬────────┘
                  │ returns OBJ file content
                  ▼
         ┌─────────────────┐
         │ Three.js Loader │ parses OBJ → geometry
         └────────┬────────┘
                  │ adds to scene
                  ▼
         ┌─────────────────┐
         │  WebGL Render   │
         └─────────────────┘
```

### Communication Protocol

#### HTTP Routes (REST API)

```
GET  /                      Serve index.html (main viewer page)
GET  /three.min.js          Serve embedded Three.js library
GET  /viewer.js             Serve viewer JavaScript code
GET  /style.css             Serve CSS (if separate from HTML)
GET  /api/files             JSON list of OBJ files with metadata
GET  /obj/{filename}        Serve specific OBJ file content
WS   /ws                    WebSocket endpoint for live updates
```

#### `/api/files` Response Format

```json
{
  "files": [
    {
      "name": "terrain.obj",
      "mtime": 1703612520,
      "size": 45231
    },
    {
      "name": "cube.obj",
      "mtime": 1703612480,
      "size": 1024
    }
  ]
}
```

Files ordered by modification time (newest first).

#### WebSocket Messages

**Server → Client**:
```json
{ "type": "file_added", "filename": "cube.obj", "mtime": 1234567890 }
{ "type": "file_modified", "filename": "cube.obj", "mtime": 1234567891 }
{ "type": "file_removed", "filename": "old.obj" }
{ "type": "refresh_all" }
```

**Client → Server**:
```json
{ "type": "ping" }
```
(Optional keep-alive, if needed)

### Startup Sequence

```bash
$ cd my-geometry-project
$ kitbash-viewer --watch ./output --port 8080 --open
```

**Server initialization**:
1. Parse CLI arguments
2. Load config file (if exists): `./kitbash.toml` → `~/.config/kitbash-viewer/config.toml` → defaults
3. Merge config with CLI args (CLI takes precedence)
4. Scan watch directory for existing .obj files
5. Start HTTP server on configured port
6. Start file watcher on configured directory
7. If `--open` flag set, open browser to `http://localhost:PORT`

**Browser initialization**:
1. Load `index.html` from server
2. Load embedded JavaScript and Three.js
3. Initialize Three.js scene, camera, renderer
4. GET `/api/files` to retrieve initial file list
5. For each file, GET `/obj/{filename}` and load into scene
6. Connect WebSocket to `/ws`
7. Listen for file change notifications
8. Render loop starts

**File change handling**:
1. Haskell code writes/modifies OBJ file in watched directory
2. `notify` crate detects file system event
3. Rust server sends WebSocket message to connected clients
4. Browser receives notification
5. If new file: GET `/obj/{filename}`, parse, add to scene
6. If modified: GET `/obj/{filename}`, remove old mesh, add new mesh
7. If deleted: Remove mesh from scene
8. Update file list overlay

### Configuration

#### CLI Arguments

```
kitbash-viewer [OPTIONS]

OPTIONS:
    -w, --watch <DIR>       Directory to watch for OBJ files [default: ./output]
    -p, --port <PORT>       HTTP server port [default: 8080]
    -c, --config <FILE>     Config file path [default: ./kitbash.toml]
    -o, --open              Open browser automatically on startup
    -h, --help              Print help information
    -V, --version           Print version information
```

#### Config File Format (`kitbash.toml`)

```toml
# Server configuration
port = 8080
auto_open = true

# File watching
watch_dir = "./output"
poll_interval_ms = 100  # For polling fallback if native watching unavailable

# Viewer defaults
default_camera_position = [1.0, 1.0, 1.0]
show_grid = true
show_file_list = true
```

**Config precedence**: CLI args > `./kitbash.toml` > `~/.config/kitbash-viewer/config.toml` > built-in defaults

#### Config File Locations

1. Project-specific: `./kitbash.toml` (current working directory)
2. User global: `~/.config/kitbash-viewer/config.toml` (or platform equivalent)
3. If neither exists, use built-in defaults

### Error Handling

#### Malformed OBJ Files
- Three.js OBJLoader throws exception
- Catch error, log to browser console
- Display error indicator in file list overlay (e.g., ⚠ icon)
- Skip mesh, don't crash viewer
- On next file modification, attempt reload

#### Missing Files
- WebSocket notifies file deleted
- Remove mesh from scene
- Update file list overlay
- Log to console

#### Network Errors
- If WebSocket disconnects, attempt reconnect with exponential backoff (1s, 2s, 4s, 8s max)
- After multiple failed reconnects, show connection status warning in console
- Continue functioning with manual reload (`r` key)
- Don't spam reconnection attempts indefinitely

#### Port Already in Use
- Server fails to start
- Print clear error message with suggestion to use `--port` flag
- Exit gracefully

---

## Implementation Details

### Three.js Scene Setup

```javascript
// Basic scene structure
const scene = new THREE.Scene();
scene.background = new THREE.Color(0x2a2a2a); // Dark grey background

const camera = new THREE.PerspectiveCamera(75, aspect, 0.1, 1000);
const renderer = new THREE.WebGLRenderer({ antialias: true });

// Lighting
const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
const directionalLight = new THREE.DirectionalLight(0xffffff, 0.6);
directionalLight.position.set(1, 1, 1);

// Camera controls
const controls = new THREE.OrbitControls(camera, renderer.domElement);

// Ground grid (10x10 units, toggleable)
const gridHelper = new THREE.GridHelper(10, 10);
scene.add(gridHelper); // Can be toggled with 'g' key
```

### OBJ Loading Pattern

```javascript
const loader = new THREE.OBJLoader();

function loadMesh(filename) {
  return fetch(`/obj/${filename}`)
    .then(response => response.text())
    .then(objText => {
      const object = loader.parse(objText);
      
      // Apply materials
      object.traverse(child => {
        if (child.isMesh) {
          child.material = new THREE.MeshPhongMaterial({
            color: 0xcccccc,
            flatShading: false
          });
        }
      });
      
      scene.add(object);
      meshes.set(filename, object);
      return object;
    })
    .catch(err => {
      console.error(`Failed to load ${filename}:`, err);
      fileErrors.set(filename, err.message);
    });
}
```

### Wireframe Overlay Implementation

```javascript
function toggleWireframe(mesh, enabled) {
  if (enabled && !mesh.userData.wireframe) {
    // Create wireframe overlay
    const wireGeo = new THREE.WireframeGeometry(mesh.geometry);
    const wireMat = new THREE.LineBasicMaterial({ color: 0x000000 });
    const wireframe = new THREE.LineSegments(wireGeo, wireMat);
    
    mesh.add(wireframe);
    mesh.userData.wireframe = wireframe;
  } else if (!enabled && mesh.userData.wireframe) {
    // Remove wireframe overlay
    mesh.remove(mesh.userData.wireframe);
    mesh.userData.wireframe.geometry.dispose();
    mesh.userData.wireframe.material.dispose();
    delete mesh.userData.wireframe;
  }
}
```

### Object Selection with Raycasting

```javascript
const raycaster = new THREE.Raycaster();
const mouse = new THREE.Vector2();

renderer.domElement.addEventListener('click', (event) => {
  // Calculate mouse position in normalized device coordinates
  mouse.x = (event.clientX / window.innerWidth) * 2 - 1;
  mouse.y = -(event.clientY / window.innerHeight) * 2 + 1;
  
  raycaster.setFromCamera(mouse, camera);
  const intersects = raycaster.intersectObjects(scene.children, true);
  
  if (intersects.length > 0) {
    const object = intersects[0].object;
    selectObject(object);
  } else {
    // Clicking empty space deselects current object
    deselectObject();
  }
});

function selectObject(object) {
  // Clear previous selection
  if (selectedObject) {
    selectedObject.material.emissive.setHex(0x000000);
  }
  
  // Highlight new selection
  selectedObject = object;
  object.material.emissive.setHex(0x333333);
  
  // Update file list overlay
  updateFileListDisplay();
}

function deselectObject() {
  if (selectedObject) {
    selectedObject.material.emissive.setHex(0x000000);
    selectedObject = null;
    updateFileListDisplay();
  }
}
```

### File Watching (Rust)

```rust
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::Path;

fn watch_directory(path: &Path, tx: tokio::sync::mpsc::Sender<FileEvent>) {
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        match res {
            Ok(event) => {
                match event.kind {
                    EventKind::Create(_) => {
                        // Send file_added message
                    }
                    EventKind::Modify(_) => {
                        // Send file_modified message
                    }
                    EventKind::Remove(_) => {
                        // Send file_removed message
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Watch error: {:?}", e),
        }
    }).unwrap();
    
    watcher.watch(path, RecursiveMode::NonRecursive).unwrap();
}
```

### Standard View Positioning

```javascript
const STANDARD_VIEWS = {
  1: { position: [0, 0, 5], up: [0, 1, 0], name: 'Front' },
  2: { position: [0, 0, -5], up: [0, 1, 0], name: 'Back' },
  3: { position: [5, 0, 0], up: [0, 1, 0], name: 'Right' },
  4: { position: [-5, 0, 0], up: [0, 1, 0], name: 'Left' },
  5: { position: [0, 5, 0], up: [0, 0, -1], name: 'Top' },
  6: { position: [0, -5, 0], up: [0, 0, 1], name: 'Bottom' }
};

function setStandardView(viewNumber) {
  const view = STANDARD_VIEWS[viewNumber];
  if (!view) return;
  
  // Calculate scene bounds
  const bounds = calculateVisibleBounds();
  const center = bounds.getCenter(new THREE.Vector3());
  const size = bounds.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  
  // Position camera
  const distance = maxDim / Math.tan(camera.fov * Math.PI / 360);
  camera.position.set(...view.position).normalize().multiplyScalar(distance);
  camera.position.add(center);
  camera.up.set(...view.up);
  
  // Point at center
  controls.target.copy(center);
  controls.update();
}
```

---

## Phase 2 Features (Future Enhancements)

### Display & Debugging
- **Normal visualization**: Toggle display of vertex/face normals as colored arrows (`n` key)
  - Use `THREE.VertexNormalsHelper` or `THREE.FaceNormalsHelper`
  - Useful for debugging geometry generation
  
- **Smooth/flat shading toggle**: Switch between smooth and flat shading (`s` key)
  - `material.flatShading = true/false`

- **Edge detection modes**: Toggle between "all edges" and "feature edges only" (`e` key)
  - All edges: Show every triangle edge
  - Feature edges: Use `THREE.EdgesGeometry` with threshold angle (e.g., 15°)
  - Useful for seeing actual features vs tessellation artifacts

### Camera & View
- **Orthographic camera mode**: Toggle between perspective and orthographic projection
  - Useful for technical/CAD-style viewing
  
- **Camera bookmarks**: Save and recall custom camera positions (Ctrl+1-9 to save, 1-9 to recall if extended beyond standard views)

### File Management
- **File filtering**: Only show/load files matching a pattern
  - CLI flag: `--pattern "*.obj"` or `--exclude "temp_*.obj"`
  
- **Subdirectory support**: Recursively watch subdirectories
  - Organize objects into folders, maintain hierarchy in file list

### Materials & Appearance
- **Material library support**: Parse and apply .mtl files if present
  - OBJ files can reference .mtl for colors, textures
  
- **Per-object color override**: Assign random colors to objects for visual distinction
  - Useful when multiple similar objects in scene

- **Background color/environment**: Configurable background (solid color, gradient, skybox)

### Measurement & Analysis
- **Bounding box display**: Show axis-aligned bounding boxes for objects
  - Useful for debugging size/placement
  
- **Vertex/face count display**: Show poly count in file list or overlay
  - Performance monitoring, optimization feedback

- **Distance measurement**: Click two points to measure distance
  - Useful for verifying dimensions

### Export & Capture
- **Screenshot capture**: Save current viewport as PNG (`p` key)
  - Using `renderer.domElement.toDataURL()`
  
- **Camera state export**: Save current camera position/target to config
  - Reproducible views

### Performance
- **Geometry merging**: Combine multiple meshes into single draw call
  - Only needed for very large scenes (100+ objects)
  - `BufferGeometryUtils.mergeGeometries()`

- **LOD (Level of Detail)**: Simplified geometry when zoomed out
  - Unlikely to be needed at expected scales

### UI Enhancements
- **Configurable key bindings**: User-defined keyboard shortcuts
  - Load from config file

- **Status bar**: Show current mode, camera info, selection details
  - More detailed than file list overlay

- **Command palette**: Press `/` to search/execute commands
  - Modern app pattern (like VS Code)

---

## Performance Targets

### Expected Scale
- 5-20 OBJ files per scene
- 100-5,000 triangles per file
- Total scene: 10,000-50,000 triangles

### Performance Goals
- 60 fps rendering on modest hardware
- <100ms file reload time for typical mesh
- <50ms from file save to visual update (file watching latency)

### Optimization Strategy
- Start simple, optimize only if needed
- WebGL can handle 100K+ triangles easily at 60fps
- File I/O is fast for small files (typical OBJ <1MB)
- Geometry merging available if draw call count becomes issue

---

## Development Milestones

### Milestone 1: MVP (Minimum Viable Product)
- Rust HTTP server serving embedded HTML/JS
- Three.js scene with basic lighting and camera controls
- Load single OBJ file and display
- File watching with manual refresh
- Basic keyboard controls (camera reset, reload)

### Milestone 2: Core Features
- Multiple OBJ file loading
- Automatic reload on file change (WebSocket)
- Object selection (click to select)
- File list overlay
- Object visibility toggle
- Navigation between objects ([/] keys)

### Milestone 3: Polish
- Standard view angles (1-6 keys)
- View framing (f/F keys)
- Wireframe overlay mode
- Grid/ground plane
- CLI arguments and config file
- Auto-open browser option
- Selection highlighting (emissive glow)

### Milestone 4: Robustness
- Error handling for malformed OBJ
- WebSocket reconnection
- Cross-platform testing
- Documentation and usage examples

---

## Testing Strategy

### Manual Testing Scenarios
1. **Basic workflow**: Start viewer, create OBJ file, verify auto-load
2. **File modification**: Modify existing OBJ, verify mesh updates
3. **File deletion**: Delete OBJ file, verify mesh removed from scene
4. **Multiple files**: Load 10+ OBJ files, verify performance
5. **Malformed OBJ**: Create invalid OBJ, verify graceful failure
6. **Keyboard shortcuts**: Test all key bindings
7. **Mouse controls**: Verify orbit, pan, zoom, click-to-select
8. **Standard views**: Test all 6 standard view angles
9. **Config file**: Test CLI args, local config, global config precedence
10. **Offline mode**: Disconnect network, verify full functionality

### Edge Cases
- Empty watch directory (no OBJ files)
- Very large OBJ file (>10MB)
- Rapid file changes (save spam)
- Unicode filenames
- Deeply nested vertex/face counts
- OBJ with no normals (verify auto-generation)

### Platform Testing
- macOS (primary development platform)
- Linux (Ubuntu/Debian)
- Windows (if time permits)

---

## File Format Notes

### OBJ Format Support
The viewer supports the core OBJ specification:

**Supported elements**:
- `v x y z` - Vertex positions (required)
- `vn x y z` - Vertex normals (optional, auto-generated if missing)
- `f v1 v2 v3 ...` - Faces (triangles or polygons)
- `f v1/vt1/vn1 v2/vt2/vn2 ...` - Faces with texture coords and normals
- `o ObjectName` - Object names (for grouping)

**Not supported** (initially):
- `vt u v` - Texture coordinates (ignored)
- `mtllib` / `usemtl` - Material library (Phase 2)
- `g GroupName` - Groups (treated same as objects)
- `s on/off` - Smooth shading groups (auto-determined)

**Three.js OBJLoader** handles parsing automatically, so full spec support depends on Three.js implementation.

---

## Deployment

### Building
```bash
cargo build --release
```

Output: `target/release/kitbash-viewer` (or `.exe` on Windows)

### Distribution
- Single binary, no dependencies
- Can be copied to `/usr/local/bin` or equivalent
- No installation required
- Works completely offline

### Binary Size
Expected size: 2-5 MB
- Rust binary: ~1-2 MB
- Three.js embedded: ~600 KB
- Other assets: <100 KB

---

## Success Criteria

The project is successful when:
1. A single command starts the viewer and watches a directory
2. Saving an OBJ file from Haskell code shows the geometry within 100ms
3. All core keyboard/mouse interactions work intuitively
4. The tool works completely offline
5. The binary runs on macOS and Linux without installation
6. The viewer handles typical procedural geometry (100-5000 triangles) at 60fps
7. The interface stays out of the way (minimal/no UI chrome)

---

## Dependencies Summary

### Rust Crates
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower-http = { version = "0.5", features = ["fs", "trace"] }
notify = "6"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
toml = "0.8"
serde_json = "1"
```

### JavaScript Libraries
- Three.js r160+ (embedded, ~600KB minified)
- No other dependencies (OrbitControls, OBJLoader included in Three.js)

---

## References

- [Three.js Documentation](https://threejs.org/docs/)
- [OBJ Format Specification](http://www.martinreddy.net/gfx/3d/OBJ.spec)
- [notify crate](https://docs.rs/notify/)
- [axum framework](https://docs.rs/axum/)
