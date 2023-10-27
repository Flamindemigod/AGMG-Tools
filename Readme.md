# GBT - GIMI Build Tool

**GBT (GIMI Build Tool)** is a command-line utility written in Rust that draws inspiration from npm. GBT is designed to enhance your productivity when working with GIMI and asset management. Whether you need to scaffold a new project, rebuild assets, or manage mod-related tasks, GBT has you covered. This README will guide you through the installation, setup, and usage of GBT.

## Features
GBT offers a range of features to simplify the development and management of GIMI mods:

1. **Pulling Dumps**: GBT allows you to pull data from Github Repos or from local files, providing a convenient way to integrate external resources into your project.

2. **Basic Project Scaffolding**: Quickly create a basic project structure for your mod with GBT's scaffolding feature. It sets up the necessary directories and files to get you started.

3. **Asset Conversion**: GBT can split RGBA DDS files into RGB PNG and a black-and-white PNG alpha mask, making it easier to work with textures.

4. **Rebuilding Files**: Keep your project up to date by automatically rebuilding asset files when texture changes are detected, ensuring that your mods are always current.

5. **Rebuilding Mod**: When changes are made to your mod, GBT can rebuild it, so you can test and distribute the latest version with ease.

6. **Mod Linking**: Easily link your mod into the mod folder, simplifying the integration of your modifications into the game.

7. **Scripts via the Config**: Customize your workflow by defining scripts in the configuration file to automate tasks or run custom commands.

8. **Project Archiving**: Create a project archive by zipping your project, providing a convenient way to back up or share your work.

9. **Project Unzipping**: Unzip a project archive with GBT, making it simple to restore or import projects.

10. **Mod Export**: Export your mod for distribution, ensuring that it's ready to share with others.

11. **Texture Mod INI Generation**: Automatically generate texture mod INI files to properly configure your mod's textures.

12. **Self-Updater**: GBT includes a self-updater, ensuring that you always have the latest version of the tool.

## Installation

To install GBT, head over to the Release Page and grab the latest Execuatable for your platform:
And Grab the [blender plugin](https://github.com/Flamindemigod/AGMG-Tools/blob/master/Blender/blender_3dmigoto_plugin.py). This plugin conflicts with the Original Made by SilentNightSound. So make sure to remove the original.
## Getting Started
> Use `gbt --help` to see available commands and their descriptions.

1. **Initialize a New Project**: Create a new project by running the following command and following the prompts:

   ```bash
   gbt init ./Furrina
   ```
   This will run you through the basic steps of getting you setup for modding.

2. **Configuration**: Customize GBT by editing the `Config.yml` configuration file to define your scripts, mod settings, and more. 

3. **Running GBT**: Execute GBT commands to perform various tasks such as building textures, generating texture mod inis, linking mods, and more. Use `gbt run watch` to start up the watcher. 

4. **Exporting Mods** Most of the stuff in blender will remain the same as the guide made by Silent. However during the export. Navigate to the `Source/Model` Folder and save the with the name of the original object. in this case, `Furina.vb` and turn off `use foldername when exporting`. And Voila, the project should export and rebuild all your textures.

5. **Self-Update**: Keep GBT up to date by running:

   ```bash
   gbt update
   ```

## Feedback and Contributions

GBT is an open-source project, and we welcome contributions and feedback from the community. If you encounter issues, have feature requests, or would like to contribute, please visit our [GitHub repository](https://github.com/Flamindemigod/AGMG-Tools) to submit issues, create pull requests, or join discussions.

Thank you for using GBT, and we hope it helps streamline your game modding workflow!