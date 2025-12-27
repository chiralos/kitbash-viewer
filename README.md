# Simple Mesh Viewer

All this does is watch a directory for .obj file additions,
deletions and changes, and manage a WebGL browser page (with
basic view controls) that shows the meshes.

## Implementation

This is a Rust local http server that does the directory watching
and has an embedded JS page for the actual viewer.
Made with Claude Code (starting in Dec 2025).
 
Usage :
```
  kitbash-viewer --help
```

