# Scaffold Project
## Rust Based Scaffolder
- [ ] Project Scaffold
    - [ ] Symlink / Copy Model Source Folder
    - [ ] Make Textures Folder
    - [ ] Flatten Textures and Generate Alpha Maps

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
- [ ] Import Files via VB/IB
    - [ ] Textures
        - [ ] Use Generated Textures or Fail
- [ ] Exporter
    - [ ] Generates Mod Files

# Automated Mod Texture Builder + Linker
## Rust Based Listener
- Await File Changes in Mod Output / Textures
- Texture Join + Convert
- Rebuild Mod
- Symlink Mod
