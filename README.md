# Cosmic Dust

Cosmic Dust is a disk usage visualizer designed as a modern alternative to Filelight. It provides a grid-based visualization of file and directory sizes, with intuitive navigation and a sleek design that aligns with the COSMIC aesthetic by System76. Each file or directory is represented as a squircle, color-coded based on its size, making it easy to identify large files and folders at a glance.

# Features
- **Grid-Based Visualization**: Displays files and directories as squircles in a grid, sorted by size.
- **Color-Coded Sizes**: Uses a 5-color scale (Lapis, Jade, Gold, Purple, Crimson) to represent file sizes, ranging from 0 to 1 TB.
- **Hover Information**: Hover over a file or directory to see its name and size.
- **COSMIC Aesthetic**: Designed to fit seamlessly into the COSMIC desktop environment with rounded squircles and a modern look.

# Installation
Cosmic Dust can be installed system-wide using the provided justfile. The installation process builds the application, installs the binary, .desktop file, metainfo file, and icons, and updates the icon cache.

# Prerequisites
- **Rust**: Ensure you have the Rust toolchain installed. You can install it using rustup or your native packagemanagers version, whichever is available on your distro.
- **Just**: The just command-line tool ensures proper installation, it should be available in your systems repositories.


**Clone the Repository**:

```
git clone https://github.com/melechtna/cosmic-dust.git
cd cosmic-dust
```
**Build and Install**: Run the following command to build and install Cosmic Dust
``` 
just install
```
This will:

- Build the application in release mode.
- Install the binary to /usr/bin/cosmic-dust.
- Install the .desktop file to /usr/share/applications/io.melechtna.CosmicDust.desktop.
- Install the metainfo file to /usr/share/metainfo/io.melechtna.CosmicDust.metainfo.xml.
- Install the application icon in various sizes (16x16 to 256x256) to /usr/share/icons/hicolor.
- Update the icon theme cache.
**Launch the Application**: After installation, you can launch Cosmic Dust from your application menu or by running:
```
cosmic-dust
```
To enable verbose output for debugging, use:
```
cosmic-dust --verbose
```
# Uninstallation
To uninstall Cosmic Dust and remove all associated files, run:
```just uninstall```

This will:

- Remove the binary from /usr/bin/cosmic-dust.
- Remove the .desktop file from /usr/share/applications.
- Remove the metainfo file from /usr/share/metainfo.
- Remove the icons from /usr/share/icons/hicolor.
- Update the icon theme cache.
# Packaging
Cosmic Dust is not officially packaged for various distributions. While contributions to create packages (e.g., .deb, .rpm, or Flatpak) are welcome, the maintainers do not take responsibility for packaging or maintaining these packages. If you’re interested in packaging Cosmic Dust for your distribution, feel free to do so yourself.

License
Cosmic Dust is licensed under the MIT License. See the  file for details.

# Contributions
Contributions are welcome! There are strict style guidelines (namely the ones set by COSMIC) that will be enforced and adhere'd to when accepting pull requests.

# Adoption by System76/COSMIC
System76 and the COSMIC team are welcome to adopt this repository as an official project. If the COSMIC team is interested in integrating Cosmic Dust into the COSMIC ecosystem, please reach out to the maintainer to discuss the process.

# Acknowledgements
- **System76**: For creating the COSMIC desktop environment, which Cosmic Dust follows stylistically as closely as possible.
**Iced**: The Rust GUI framework used to build the application.

# Contact
For questions, bug reports, or feature requests, please open an issue on the GitHub repository. 

However try to keep in mind that this project is designed to be simplistic, so feature requests are unlikely to be accepted. Example being, I will not, under any circumstances, accept responsibility for adding a delete function within this application.

This is to avoid accidental file deletions, as you can use the UI to find what you want to delete, and go delete it yourself from your file browser.
