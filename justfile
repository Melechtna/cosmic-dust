# Default recipe to show available tasks
default:
    @just --list

# Build the application in release mode
build:
    cargo build --release

# Install the application, .desktop file, and icon (system-wide)
install: build
    #!/usr/bin/env bash
    set -e

    # Define installation paths
    BIN_DIR="/usr/bin"
    DESKTOP_DIR="/usr/share/applications"
    METAINFO_DIR="/usr/share/metainfo"
    ICON_BASE_DIR="/usr/share/icons/hicolor"
    BINARY="target/release/cosmic-dust"
    DESKTOP_FILE="io.melechtna.CosmicDust.desktop"
    DESKTOP_SRC="desktop/io.melechtna.CosmicDust.desktop"
    METAINFO_FILE="io.melechtna.CosmicDust.metainfo.xml"
    METAINFO_SRC="res/io.melechtna.CosmicDust.metainfo.xml"
    ICON_SRC="icons/io.melechtna.CosmicDust.svg"
    ICON_NAME="io.melechtna.CosmicDust.svg"

    # Validate the icon exists
    if [ ! -f "$ICON_SRC" ]; then
        echo "Error: Icon $ICON_SRC does not exist. Aborting."
        exit 1
    fi

    # Validate the .desktop source file exists
    if [ ! -f "$DESKTOP_SRC" ]; then
        echo "Error: .desktop file $DESKTOP_SRC does not exist. Aborting."
        exit 1
    fi

    # Validate the metainfo source file exists
    if [ ! -f "$METAINFO_SRC" ]; then
        echo "Error: Metainfo file $METAINFO_SRC does not exist. Aborting."
        exit 1
    fi

    # Run all installation commands under a single sudo session
    sudo bash -c '
        set -e
        echo "Installing binary to '"$BIN_DIR"'/cosmic-dust..."
        install -Dm0755 '"$BINARY"' '"$BIN_DIR"'/cosmic-dust

        # Install the SVG icon in multiple sizes
        for size in 16 32 48 64 128 256; do
            ICON_DIR="'"$ICON_BASE_DIR"'/${size}x${size}/apps"
            echo "Installing icon (${size}x${size}) to $ICON_DIR/'"$ICON_NAME"'..."
            mkdir -p "$ICON_DIR"
            install -Dm0644 '"$ICON_SRC"' "$ICON_DIR/'"$ICON_NAME"'"
        done

        echo "Installing .desktop file to '"$DESKTOP_DIR"'/'"$DESKTOP_FILE"'..."
        sed "s|Exec=cosmic-dust|Exec='"$BIN_DIR"'/cosmic-dust|; s|Icon=cosmic-dust|Icon=io.melechtna.cosmic-dust|" '"$DESKTOP_SRC"' > '"$DESKTOP_DIR"'/'"$DESKTOP_FILE"'

        echo "Installing metainfo file to '"$METAINFO_DIR"'/'"$METAINFO_FILE"'..."
        install -Dm0644 '"$METAINFO_SRC"' "$METAINFO_DIR/'"$METAINFO_FILE"'"
    '

    # Update the icon theme cache
    echo "Updating icon theme cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Installation complete! You can now launch Cosmic Dust from your application menu."

# Uninstall the application, .desktop file, and icon (system-wide)
uninstall:
    #!/usr/bin/env bash
    set -e

    # Define paths
    BIN_DIR="/usr/bin"
    DESKTOP_DIR="/usr/share/applications"
    METAINFO_DIR="/usr/share/metainfo"
    ICON_BASE_DIR="/usr/share/icons/hicolor"
    DESKTOP_FILE="cosmic-dust.desktop"
    METAINFO_FILE="iio.melechtna.CosmicDust.metainfo.xml"
    ICON_NAME="io.melechtna.CosmicDust.svg"

    # Run all uninstallation commands under a single sudo session
    sudo bash -c '
        set -e
        if [ -f '"$BIN_DIR"'/cosmic-dust ]; then
            echo "Removing binary from '"$BIN_DIR"'/cosmic-dust..."
            rm '"$BIN_DIR"'/cosmic-dust
        fi

        if [ -f '"$DESKTOP_DIR"'/'"$DESKTOP_FILE"' ]; then
            echo "Removing .desktop file from '"$DESKTOP_DIR"'/'"$DESKTOP_FILE"'..."
            rm '"$DESKTOP_DIR"'/'"$DESKTOP_FILE"'
        fi

        if [ -f '"$METAINFO_DIR"'/'"$METAINFO_FILE"' ]; then
            echo "Removing metainfo file from '"$METAINFO_DIR"'/'"$METAINFO_FILE"'..."
            rm '"$METAINFO_DIR"'/'"$METAINFO_FILE"'
        fi

        # Remove the SVG icon from all sizes
        for size in 16 32 48 64 128 256; do
            ICON_DIR="'"$ICON_BASE_DIR"'/${size}x${size}/apps"
            if [ -f "$ICON_DIR/'"$ICON_NAME"'" ]; then
                echo "Removing icon (${size}x${size}) from $ICON_DIR/'"$ICON_NAME"'..."
                rm "$ICON_DIR/'"$ICON_NAME"'"
            fi
        done
    '

    # Update the icon theme cache
    echo "Updating icon theme cache..."
    sudo gtk-update-icon-cache -f /usr/share/icons/hicolor

    echo "Uninstallation complete!"
