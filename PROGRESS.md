## Development Milestones

### Milestone 1: MVP (Minimum Viable Product)
- [x] Rust HTTP server serving embedded HTML/JS
- [x] Three.js scene with basic lighting and camera controls
- [x] Load single OBJ file and display
- [x] File watching with manual refresh
- [x] Basic keyboard controls (camera reset, reload)

### Milestone 2: Core Features
- [x] Multiple OBJ file loading
- [x] Automatic reload on file change (WebSocket)
- [x] Object selection (click to select)
- [x] File list overlay
- [x] Object visibility toggle
- [x] Navigation between objects ([/] keys)

### Milestone 3: Polish
- [x] Standard view angles (1-6 keys)
- [x] View framing (f/F keys)
- [x] Wireframe overlay mode
- [x] Grid/ground plane
- [x] CLI arguments
- [x] Auto-open browser option
- [x] Selection highlighting (emissive glow)

### Milestone 4: Robustness
- [x] Error handling for malformed OBJ
- [ ] WebSocket reconnection
- [ ] Cross-platform testing
- [ ] Documentation and usage examples
- [ ] Config file
- [ ] Additional settings (see below)

### Additional Settings

 Viewer Settings:
  - --background <COLOR> - Background color as hex (default: 0x2a2a2a)
  - --grid / --no-grid - Show/hide grid on startup (default: shown)
  - --grid-size <SIZE> - Grid size in units (default: 20)
  - --grid-divisions <N> - Number of grid divisions (default: 20)

  Camera Settings:
  - --camera-x <X> - Initial camera X position (default: 5)
  - --camera-y <Y> - Initial camera Y position (default: 5)
  - --camera-z <Z> - Initial camera Z position (default: 5)
  - --fov <DEGREES> - Camera field of view (default: 75)

  Optional/Advanced:
  - --config <FILE> - Load settings from config file (TOML/JSON)
  - --wireframe-mode <0|1|2> - Initial wireframe mode (default: 0)
