pub const HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Kitbash Viewer</title>
  <style>
    body {
      margin: 0;
      padding: 0;
      overflow: hidden;
      background-color: #2a2a2a;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }
    #canvas-container {
      width: 100vw;
      height: 100vh;
    }
    #file-list-overlay {
      position: absolute;
      top: 20px;
      right: 20px;
      background-color: rgba(0, 0, 0, 0.8);
      color: #ffffff;
      padding: 15px;
      border-radius: 8px;
      min-width: 200px;
      max-width: 300px;
      font-size: 14px;
      opacity: 0.9;
    }
    #file-list-overlay.hidden {
      display: none;
    }
    #file-list-header {
      font-weight: bold;
      margin-bottom: 10px;
      padding-bottom: 8px;
      border-bottom: 1px solid #555;
      font-size: 12px;
      color: #aaa;
    }
    #file-list-content {
      font-family: monospace;
    }
    .file-list-item {
      padding: 4px 8px;
      margin: 2px 0;
      border-radius: 4px;
      cursor: pointer;
      pointer-events: auto;
      transition: background-color 0.15s;
    }
    .file-list-item:hover {
      background-color: rgba(80, 80, 80, 0.5);
    }
    .file-list-item.selected {
      background-color: rgba(100, 150, 255, 0.3);
      color: #8ac6ff;
    }
    .file-list-item.selected:hover {
      background-color: rgba(100, 150, 255, 0.4);
    }
    .file-list-item.hidden {
      opacity: 0.4;
      font-style: italic;
    }
    .file-list-item.failed {
      color: #ff6666;
    }
    .file-list-item.failed .visibility-icon {
      color: #ff4444;
    }
    .file-list-item .visibility-icon {
      display: inline-block;
      width: 16px;
      margin-right: 4px;
      font-style: normal;
    }
  </style>
</head>
<body>
  <div id="canvas-container"></div>

  <div id="file-list-overlay">
    <div id="file-list-header">Files (Tab to toggle)</div>
    <div id="file-list-content"></div>
  </div>

  <script type="importmap">
  {
    "imports": {
      "three": "https://cdn.jsdelivr.net/npm/three@0.160.0/build/three.module.js",
      "three/addons/": "https://cdn.jsdelivr.net/npm/three@0.160.0/examples/jsm/"
    }
  }
  </script>

  <script type="module">
    import * as THREE from 'three';
    import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
    import { OBJLoader } from 'three/addons/loaders/OBJLoader.js';

    // Scene setup
    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x2a2a2a);

    // Camera setup - 3/4 isometric view
    const camera = new THREE.PerspectiveCamera(
      75,
      window.innerWidth / window.innerHeight,
      0.1,
      1000
    );
    camera.position.set(5, 5, 5);
    camera.lookAt(0, 0, 0);

    // Store initial camera position and target for reset
    const initialCameraPosition = new THREE.Vector3(5, 5, 5);
    const initialCameraTarget = new THREE.Vector3(0, 0, 0);

    // Renderer setup
    const renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    document.getElementById('canvas-container').appendChild(renderer.domElement);

    // Lighting
    const ambientLight = new THREE.AmbientLight(0xffffff, 0.4);
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 0.6);
    directionalLight.position.set(1, 1, 1);
    scene.add(directionalLight);

    // Grid/ground plane
    const gridSize = 20;
    const gridDivisions = 20;
    const gridHelper = new THREE.GridHelper(gridSize, gridDivisions, 0xaaaaaa, 0x666666);
    scene.add(gridHelper);

    // Camera controls
    const controls = new OrbitControls(camera, renderer.domElement);
    // controls.enableDamping = true;
    // controls.dampingFactor = 0.05;

    // Selection
    let selectedObject = null;
    const raycaster = new THREE.Raycaster();
    const mouse = new THREE.Vector2();

    // Wireframe mode: 0 = solid, 1 = solid + wireframe, 2 = wireframe only
    let wireframeMode = 0;
    const wireframeOverlays = new Map(); // Map from object to wireframe overlay

    // OBJ Loader
    const objLoader = new OBJLoader();
    const loadedMeshes = new Map();
    const loadingFiles = new Set(); // Track files currently being loaded
    const failedFiles = new Map(); // Track files that failed to load (filename -> error)

    // Function to load and display an OBJ file
    function loadOBJ(filename) {
      // Prevent duplicate loads (race condition protection)
      if (loadingFiles.has(filename) || loadedMeshes.has(filename)) {
        console.log(`${filename} already loading or loaded, skipping`);
        return;
      }

      // Clear any previous error for this file
      failedFiles.delete(filename);

      loadingFiles.add(filename);
      console.log(`Starting load: ${filename}`);

      objLoader.load(
        `/scene/${filename}`,
        (object) => {
          // Check if the object contains any actual geometry
          let hasMeshes = false;
          object.traverse((child) => {
            if (child.isMesh && child.geometry && child.geometry.attributes.position) {
              hasMeshes = true;
            }
          });

          if (!hasMeshes) {
            // Object loaded but contains no valid geometry
            console.error(`Error loading ${filename}: No valid geometry found`);
            failedFiles.set(filename, {
              error: null,
              message: 'No valid geometry found in file',
              timestamp: new Date()
            });
            loadingFiles.delete(filename);
            updateFileList();
            return;
          }

          // Apply material to all meshes in the loaded object
          object.traverse((child) => {
            if (child.isMesh) {
              child.material = new THREE.MeshPhongMaterial({
                color: 0xcccccc,
                flatShading: false,
                side: THREE.DoubleSide
                // TODO: May remove this and require correct winding order in OBJ files
              });
            }
          });

          scene.add(object);
          loadedMeshes.set(filename, object);
          loadingFiles.delete(filename);
          applyWireframeToObject(object); // Apply current wireframe mode
          console.log(`Loaded: ${filename}`);
          updateFileList();
        },
        (xhr) => {
          console.log(`${filename}: ${(xhr.loaded / xhr.total * 100).toFixed(2)}% loaded`);
        },
        (error) => {
          console.error(`Error loading ${filename}:`, error);
          console.error(`  Error type: ${error.type || 'unknown'}`);
          console.error(`  Error message: ${error.message || error.toString()}`);

          // Store error information
          failedFiles.set(filename, {
            error: error,
            message: error.message || error.toString(),
            timestamp: new Date()
          });

          loadingFiles.delete(filename);
          updateFileList();
        }
      );
    }

    // Function to clear all loaded meshes
    function clearAllMeshes() {
      // Unhighlight selected object if any
      if (selectedObject) {
        unhighlightObject(selectedObject);
      }

      loadedMeshes.forEach((object) => {
        scene.remove(object);
      });
      loadedMeshes.clear();
      loadingFiles.clear();
      failedFiles.clear();
      selectedObject = null;
      console.log('Cleared all meshes');
      updateFileList();
    }

    // Function to load all OBJ files from the scene directory
    async function loadAllFiles() {
      try {
        const response = await fetch('/api/files');
        const data = await response.json();

        console.log(`Found ${data.files.length} OBJ file(s)`);

        for (const fileInfo of data.files) {
          loadOBJ(fileInfo.name);
        }
      } catch (error) {
        console.error('Error loading file list:', error);
      }
    }

    // Function to reload all files (clear and reload)
    async function reloadAllFiles() {
      console.log('Reloading all files...');
      clearAllMeshes();
      await loadAllFiles();
    }

    // Keyboard controls
    window.addEventListener('keydown', (event) => {
      switch(event.key) {
        case '0':
          // Reset camera to initial position
          camera.position.copy(initialCameraPosition);
          controls.target.copy(initialCameraTarget);
          controls.update();
          console.log('Camera reset to initial position');
          break;
        case 'r':
        case 'R':
          reloadAllFiles();
          break;
        case 'Tab':
          event.preventDefault();
          const overlay = document.getElementById('file-list-overlay');
          overlay.classList.toggle('hidden');
          break;
        case 'h':
        case 'H':
          if (event.shiftKey) {
            // Shift+H: Show all objects
            loadedMeshes.forEach((object) => {
              object.visible = true;
            });
            console.log('Showing all objects');
            updateFileList();
          } else if (selectedObject) {
            // H: Toggle visibility of selected object
            selectedObject.visible = !selectedObject.visible;
            const filename = getObjectFilename(selectedObject);
            console.log(`${filename} ${selectedObject.visible ? 'shown' : 'hidden'}`);
            updateFileList();
          }
          break;
        case '[':
          // Select previous object
          if (loadedMeshes.size > 0) {
            const filenames = Array.from(loadedMeshes.keys()).sort();
            const currentFilename = selectedObject ? getObjectFilename(selectedObject) : null;
            let currentIndex;

            if (!currentFilename) {
              // Nothing selected: select first object
              currentIndex = 0;
            } else {
              currentIndex = filenames.indexOf(currentFilename);
              // Move to previous, wrap around if needed
              currentIndex = (currentIndex - 1 + filenames.length) % filenames.length;
            }

            const newFilename = filenames[currentIndex];
            const newObject = loadedMeshes.get(newFilename);

            // Update highlighting
            if (selectedObject) unhighlightObject(selectedObject);
            selectedObject = newObject;
            highlightObject(selectedObject);

            console.log(`Selected: ${newFilename}`);
            updateFileList();
          }
          break;
        case ']':
          // Select next object
          if (loadedMeshes.size > 0) {
            const filenames = Array.from(loadedMeshes.keys()).sort();
            const currentFilename = selectedObject ? getObjectFilename(selectedObject) : null;
            let currentIndex;

            if (!currentFilename) {
              // Nothing selected: select last object
              currentIndex = filenames.length - 1;
            } else {
              currentIndex = filenames.indexOf(currentFilename);
              // Move to next, wrap around if needed
              currentIndex = (currentIndex + 1) % filenames.length;
            }

            const newFilename = filenames[currentIndex];
            const newObject = loadedMeshes.get(newFilename);

            // Update highlighting
            if (selectedObject) unhighlightObject(selectedObject);
            selectedObject = newObject;
            highlightObject(selectedObject);

            console.log(`Selected: ${newFilename}`);
            updateFileList();
          }
          break;
        case 'f':
        case 'F':
          if (event.shiftKey) {
            // Shift+F: Frame all visible objects
            const visibleObjects = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
            if (visibleObjects.length > 0) {
              frameObjects(visibleObjects, camera.position.clone().sub(controls.target).normalize());
              console.log('Framed all visible objects');
            }
          } else if (selectedObject) {
            // F: Frame selected object
            frameObjects([selectedObject], camera.position.clone().sub(controls.target).normalize());
            const filename = getObjectFilename(selectedObject);
            console.log(`Framed: ${filename}`);
          }
          break;
        case '1':
          // Front view
          const allObjects1 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects1.length > 0) {
            frameObjects(allObjects1, new THREE.Vector3(0, 0, 1));
            console.log('Front view');
          }
          break;
        case '2':
          // Back view
          const allObjects2 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects2.length > 0) {
            frameObjects(allObjects2, new THREE.Vector3(0, 0, -1));
            console.log('Back view');
          }
          break;
        case '3':
          // Right view
          const allObjects3 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects3.length > 0) {
            frameObjects(allObjects3, new THREE.Vector3(1, 0, 0));
            console.log('Right view');
          }
          break;
        case '4':
          // Left view
          const allObjects4 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects4.length > 0) {
            frameObjects(allObjects4, new THREE.Vector3(-1, 0, 0));
            console.log('Left view');
          }
          break;
        case '5':
          // Top view
          const allObjects5 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects5.length > 0) {
            frameObjects(allObjects5, new THREE.Vector3(0, 1, 0));
            console.log('Top view');
          }
          break;
        case '6':
          // Bottom view
          const allObjects6 = Array.from(loadedMeshes.values()).filter(obj => obj.visible);
          if (allObjects6.length > 0) {
            frameObjects(allObjects6, new THREE.Vector3(0, -1, 0));
            console.log('Bottom view');
          }
          break;
        case 'w':
        case 'W':
          // Cycle wireframe mode
          wireframeMode = (wireframeMode + 1) % 3;
          applyWireframeModeToAll();
          const modes = ['Solid', 'Solid + Wireframe', 'Wireframe'];
          console.log(`Wireframe mode: ${modes[wireframeMode]}`);
          break;
        case 'g':
        case 'G':
          // Toggle grid visibility
          gridHelper.visible = !gridHelper.visible;
          console.log(`Grid ${gridHelper.visible ? 'shown' : 'hidden'}`);
          break;
      }
    });

    // Track mouse down position to distinguish clicks from drags
    let mouseDownPos = { x: 0, y: 0 };
    renderer.domElement.addEventListener('mousedown', (event) => {
      mouseDownPos.x = event.clientX;
      mouseDownPos.y = event.clientY;
    });

    // Mouse click for object selection
    renderer.domElement.addEventListener('click', (event) => {
      // Ignore if this was a drag (mouse moved more than 5 pixels)
      const dragDistance = Math.sqrt(
        Math.pow(event.clientX - mouseDownPos.x, 2) +
        Math.pow(event.clientY - mouseDownPos.y, 2)
      );
      if (dragDistance > 5) {
        return; // This was a drag, not a click
      }

      // Calculate mouse position in normalized device coordinates (-1 to +1)
      const rect = renderer.domElement.getBoundingClientRect();
      mouse.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
      mouse.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;

      // Update raycaster with camera and mouse position
      raycaster.setFromCamera(mouse, camera);

      // Get all mesh objects from loaded files
      const meshObjects = [];
      loadedMeshes.forEach((object) => {
        object.traverse((child) => {
          if (child.isMesh) {
            meshObjects.push(child);
          }
        });
      });

      // Check for intersections
      const intersects = raycaster.intersectObjects(meshObjects, false);

      if (intersects.length > 0) {
        // Find the root object (the loaded OBJ file object)
        let rootObject = intersects[0].object;
        while (rootObject.parent && !loadedMeshes.has(getObjectFilename(rootObject))) {
          rootObject = rootObject.parent;
        }

        // Unhighlight previous selection
        if (selectedObject && selectedObject !== rootObject) {
          unhighlightObject(selectedObject);
        }

        selectedObject = rootObject;
        highlightObject(selectedObject);
        const filename = getObjectFilename(rootObject);
        console.log(`Selected: ${filename}`);
        updateFileList();
      } else {
        if (selectedObject) {
          unhighlightObject(selectedObject);
          console.log('Deselected');
        }
        selectedObject = null;
        updateFileList();
      }
    });

    // Helper function to get filename for a loaded object
    function getObjectFilename(object) {
      for (const [filename, obj] of loadedMeshes.entries()) {
        if (obj === object) {
          return filename;
        }
      }
      return null;
    }

    // Frame objects in view by positioning camera
    // objects: array of THREE.Object3D to frame
    // direction: THREE.Vector3 indicating camera direction from center
    function frameObjects(objects, direction) {
      if (objects.length === 0) return;

      // Calculate bounding box of all objects
      const box = new THREE.Box3();
      objects.forEach(obj => {
        const objBox = new THREE.Box3().setFromObject(obj);
        box.union(objBox);
      });

      // Get center and size
      const center = box.getCenter(new THREE.Vector3());
      const size = box.getSize(new THREE.Vector3());

      // Calculate camera distance to fit objects in view
      const maxDim = Math.max(size.x, size.y, size.z);
      const fov = camera.fov * (Math.PI / 180);
      const cameraDistance = Math.abs(maxDim / Math.sin(fov / 2)) * 1.25; // 1.25x margin

      // Position camera along direction vector at calculated distance
      const offset = direction.clone().normalize().multiplyScalar(cameraDistance);
      camera.position.copy(center).add(offset);

      // Point controls at center
      controls.target.copy(center);
      controls.update();
    }

    // Apply wireframe mode to a single object
    function applyWireframeToObject(object) {
      // Remove existing wireframe overlays if present
      if (wireframeOverlays.has(object)) {
        const overlays = wireframeOverlays.get(object);
        overlays.forEach(({ mesh, wireframe }) => {
          mesh.remove(wireframe);
          wireframe.geometry.dispose();
          wireframe.material.dispose();
        });
        wireframeOverlays.delete(object);
      }

      object.traverse((child) => {
        if (child.isMesh) {
          if (wireframeMode === 0) {
            // Solid only
            child.material.wireframe = false;
            // Restore original color
            child.material.color.setHex(0xcccccc);
          } else if (wireframeMode === 1) {
            // Solid + wireframe overlay
            child.material.wireframe = false;
            child.material.color.setHex(0xcccccc);
            // Create wireframe overlay
            const wireframeGeo = new THREE.EdgesGeometry(child.geometry);
            const wireframeMat = new THREE.LineBasicMaterial({ color: 0x000000, linewidth: 1 });
            const wireframeLines = new THREE.LineSegments(wireframeGeo, wireframeMat);
            child.add(wireframeLines);

            // Store reference for cleanup
            if (!wireframeOverlays.has(object)) {
              wireframeOverlays.set(object, []);
            }
            wireframeOverlays.get(object).push({ mesh: child, wireframe: wireframeLines });
          } else if (wireframeMode === 2) {
            // Wireframe only - use light uniform color
            child.material.wireframe = true;
            child.material.color.setHex(0xdddddd);
            child.material.emissive.setHex(0x333333); // Add slight glow for uniform appearance
          }
        }
      });
    }

    // Apply wireframe mode to all loaded objects
    function applyWireframeModeToAll() {
      loadedMeshes.forEach((object) => {
        applyWireframeToObject(object);
      });
    }

    // Highlight selected object with emissive glow
    function highlightObject(object) {
      if (!object) return;

      object.traverse((child) => {
        if (child.isMesh) {
          // Store original emissive for later restoration
          if (!child.userData.originalEmissive) {
            child.userData.originalEmissive = child.material.emissive.clone();
          }
          // Set highlight glow (subtle blue)
          child.material.emissive.setHex(0x224488);
        }
      });
    }

    // Remove highlight from object
    function unhighlightObject(object) {
      if (!object) return;

      object.traverse((child) => {
        if (child.isMesh && child.userData.originalEmissive) {
          // Restore original emissive color
          child.material.emissive.copy(child.userData.originalEmissive);
        }
      });
    }

    // Update the file list overlay
    function updateFileList() {
      const fileListContent = document.getElementById('file-list-content');
      fileListContent.innerHTML = '';

      // Collect all filenames (loaded and failed)
      const allFilenames = new Set([
        ...loadedMeshes.keys(),
        ...failedFiles.keys()
      ]);

      if (allFilenames.size === 0) {
        fileListContent.innerHTML = '<div style="color: #888; font-style: italic;">No files loaded</div>';
        return;
      }

      const filenames = Array.from(allFilenames).sort();
      const selectedFilename = selectedObject ? getObjectFilename(selectedObject) : null;

      filenames.forEach(filename => {
        const item = document.createElement('div');
        item.className = 'file-list-item';
        const object = loadedMeshes.get(filename);
        const failedInfo = failedFiles.get(filename);

        if (filename === selectedFilename) {
          item.classList.add('selected');
        }

        // Add visibility status or error status
        if (failedInfo) {
          item.classList.add('failed');
        } else if (object && !object.visible) {
          item.classList.add('hidden');
        }

        const icon = document.createElement('span');
        icon.className = 'visibility-icon';

        if (failedInfo) {
          icon.textContent = '✕';
          icon.title = failedInfo.message;
        } else {
          icon.textContent = (object && object.visible) ? '●' : '○';
        }

        const text = document.createTextNode(filename);

        item.appendChild(icon);
        item.appendChild(text);

        // Add click handler to select the object
        item.addEventListener('click', () => {
          if (object) {
            // Update highlighting
            if (selectedObject && selectedObject !== object) {
              unhighlightObject(selectedObject);
            }
            selectedObject = object;
            highlightObject(selectedObject);
            console.log(`Selected: ${filename}`);
            updateFileList();
          }
        });

        fileListContent.appendChild(item);
      });
    }

    // Initial load of all files
    loadAllFiles();

    // WebSocket connection for live updates
    function connectWebSocket() {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const ws = new WebSocket(`${protocol}//${window.location.host}/ws`);

      ws.onopen = () => {
        console.log('WebSocket connected - live file updates enabled');
      };

      ws.onmessage = (event) => {
        const msg = JSON.parse(event.data);
        console.log('File change event:', msg);

        switch(msg.type) {
          case 'file_added':
            console.log(`Auto-loading new file: ${msg.filename}`);
            loadOBJ(msg.filename); // loadOBJ handles duplicate checking internally
            break;
          case 'file_modified':
            console.log(`Auto-reloading modified file: ${msg.filename}`);
            // Remove old version if it exists
            if (loadedMeshes.has(msg.filename)) {
              const oldObject = loadedMeshes.get(msg.filename);
              oldObject.traverse((child) => {
                if (child.isMesh) {
                  if (child.geometry) child.geometry.dispose();
                  if (child.material) child.material.dispose();
                }
              });
              scene.remove(oldObject);
              loadedMeshes.delete(msg.filename);
            }
            loadOBJ(msg.filename); // loadOBJ handles duplicate checking
            break;
          case 'file_removed':
            console.log(`Removing deleted file: ${msg.filename}`);
            if (loadedMeshes.has(msg.filename)) {
              const object = loadedMeshes.get(msg.filename);

              // Clear selection and highlight if this object was selected
              if (selectedObject === object) {
                unhighlightObject(selectedObject);
                selectedObject = null;
              }

              // Dispose of geometries and materials
              object.traverse((child) => {
                if (child.isMesh) {
                  if (child.geometry) child.geometry.dispose();
                  if (child.material) child.material.dispose();
                }
              });

              scene.remove(object);
              loadedMeshes.delete(msg.filename);
              updateFileList();
            }
            break;
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
      };

      ws.onclose = () => {
        console.log('WebSocket disconnected - reconnecting in 2s...');
        setTimeout(connectWebSocket, 2000);
      };
    }

    connectWebSocket();

    // Handle window resize
    window.addEventListener('resize', () => {
      camera.aspect = window.innerWidth / window.innerHeight;
      camera.updateProjectionMatrix();
      renderer.setSize(window.innerWidth, window.innerHeight);
    });

    // Animation loop
    function animate() {
      requestAnimationFrame(animate);
      controls.update();
      renderer.render(scene, camera);
    }

    animate();

    console.log('Kitbash Viewer initialized');
  </script>
</body>
</html>"#;
