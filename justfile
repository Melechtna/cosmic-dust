# Show available tasks
default:
    @just --list

# Build the application in release mode
build:
    cargo build --release

# Install system-wide
install: build
    #!/usr/bin/env bash
    set -e

    echo "Installing binary to /usr/bin/cosmic-dust..."
    sudo install -Dm755 target/release/cosmic-dust /usr/bin/cosmic-dust

    echo "Installing icon to /usr/share/icons/hicolor/scalable/apps/io.melechtna.CosmicDust.svg..."
    sudo install -Dm644 icons/io.melechtna.CosmicDust.svg \
        /usr/share/icons/hicolor/scalable/apps/io.melechtna.CosmicDust.svg

    echo "Installing .desktop file to /usr/share/applications/io.melechtna.CosmicDust.desktop..."
    sudo install -Dm644 desktop/io.melechtna.CosmicDust.desktop \
        /usr/share/applications/io.melechtna.CosmicDust.desktop

    echo "Installing metainfo file to /usr/share/metainfo/io.melechtna.CosmicDust.metainfo.xml..."
    sudo install -Dm644 res/io.melechtna.CosmicDust.metainfo.xml \
        /usr/share/metainfo/io.melechtna.CosmicDust.metainfo.xml

    echo "Updating icon cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Installation complete."

# Uninstall system-wide
uninstall:
    #!/usr/bin/env bash
    set -e

    echo "Removing /usr/bin/cosmic-dust (if it exists)..."
    sudo rm -f /usr/bin/cosmic-dust

    echo "Removing /usr/share/applications/io.melechtna.CosmicDust.desktop (if it exists)..."
    sudo rm -f /usr/share/applications/io.melechtna.CosmicDust.desktop

    echo "Removing /usr/share/metainfo/io.melechtna.CosmicDust.metainfo.xml (if it exists)..."
    sudo rm -f /usr/share/metainfo/io.melechtna.CosmicDust.metainfo.xml

    echo "Removing icon /usr/share/icons/hicolor/scalable/apps/io.melechtna.CosmicDust.svg (if it exists)..."
    sudo rm -f /usr/share/icons/hicolor/scalable/apps/io.melechtna.CosmicDust.svg

    echo "Updating icon cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Uninstallation complete."
