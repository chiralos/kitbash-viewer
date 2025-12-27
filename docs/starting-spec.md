# Kitbash Simple 3D Visualisation Tool

I want a simple 3D visualisation tool for use with programmatically
authoring simple 3D meshes and scenes.

The workflow it would support is that I write Haskell code that
generates object / scene descriptions a various levels of abstraction.
At the bottom it would output a very simple and standard 3D mesh
format. At this stage we don't care about texturing, just simple
geometry.

I'd work with a text editor and GHCi, incrementally developing
combinators for generating geometry, and periodically generate mesh
files on disk. The visualisation tool would sit beside that in a
window or on another monitor.

(The Haskell authoring side is not currently part of the project,
we may move on to that later)

The visualisation tools would watch an output file directory and
refresh the visualisation when something changed. This could work
by:

- At it crudest it could be manually triggered by a keypress
- it could poll the directory every 1/10th of a second
- it could use OS features for

It would look for compatible files in the target directory and load
them all into the scene.

The the features I'd want, in rough order or priority.
- centre view on bounding box centre of whole scene
- rotate and zoom view; pan view target point
- move view target to mesh object from file
- hide / show objects

Target platforms (at least one of):
- MacOS application
- cross platform (Linux, MacOS application)
- browser / Javascript / NodeJS
- VS Code plugin

## Questions

1. Does this already exist ? Must be lightweight, simple-to-install.
2. Is this doable in a JS / browser ? Dependencies must be minimal:
mature standards, no pulling in large tree of NPM dependencies.
3. How would you make work as a JS plugin ? What language(s) does
that support ?
4. How would it be done in a Rust application ? What standard
libraries would have the necessary 3D rendering / file format
support ?

