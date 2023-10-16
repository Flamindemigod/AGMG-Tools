# Scaffold Project
## Rust Based Scaffolder
- [x] Project Scaffold
    - [x] Symlink / Copy Model Source Folder
    - [x] Make Textures Folder
    - [x] Flatten Textures and Generate Alpha Maps

## Folder Structure

    /
    ├─ Output/
    │  ├─ *.ini
    │  ├─ ...Diffuse.dds
    │  ├─ ...Lightmap.dds
    │  ├─ ...NormalMap.dds
    ├─ Source/
    │  ├─ ...Diffuse.dds
    │  ├─ ...ib....txt
    │  ├─ ...Lightmap.dds
    │  ├─ ...NormalMap.dds
    │  ├─ ...vb....txt
    ├─ Textures/
    │  ├─ ...DiffuseAlpha.png
    │  ├─ ...DiffuseFlat.png
    │  ├─ ...LightmapAlpha.png
    │  ├─ ...LightmapFlat.png
    │  ├─ ...NormalMap.png


# Blender Model Custom Importer 
## Python Based
- [x] Import Files via VB/IB
    - [x] Cleanup Meshes
    - [x] Textures
        - [x] Use Generated Textures or Fail
- [x] Exporter
    - [x] Generates Mod Files

# Automated Mod Texture Builder + Linker
## Rust Based Listener
- [x] Await File Changes in Mod Output / Textures
- [x] Texture Join + Convert
- [x] Rebuild Mod
- [x] Symlink Mod
