#!/bin/bash
set -e

echo "🔨 Building Tydust for Windows..."

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

echo "📦 Packaging distribution..."

# Create distribution folder
rm -rf dist/tydust-windows
mkdir -p dist/tydust-windows

# Copy executable
cp target/x86_64-pc-windows-gnu/release/tydust.exe dist/tydust-windows/

# Copy assets
cp -r assets dist/tydust-windows/

# Copy audio if it exists
if [ -d "audio" ]; then
	mkdir -p dist/tydust-windows/assets/audio
	cp audio/*.wav dist/tydust-windows/assets/audio/ 2>/dev/null || true
fi

if [ -d "audio_ai" ]; then
	mkdir -p dist/tydust-windows/assets/audio_ai
	cp audio_ai/*.wav dist/tydust-windows/assets/audio_ai/ 2>/dev/null || true
fi

# Copy music
cp level-1-music-full.wav dist/tydust-windows/assets/ 2>/dev/null || true
cp phase*.wav dist/tydust-windows/assets/ 2>/dev/null || true

echo "📝 Creating README..."

cat > dist/tydust-windows/README.txt << 'EOF'
TYDUST - Vertical Shooter
=========================

How to Run:
-----------
Double-click tydust.exe

Controls:
---------
Arrow Keys - Move ship
Space - Fire primary weapon
Click X (top-right) - Exit game

Requirements:
-------------
Windows 10/11
DirectX compatible graphics

Credits:
--------
Built with Bevy Engine
Generated with Claude Code
EOF

# Create zip
cd dist
zip -r tydust-windows.zip tydust-windows/
cd ..

echo ""
echo "✓ Build complete!"
echo ""
echo "Distribution folder: dist/tydust-windows/"
echo "Zip file: dist/tydust-windows.zip"
echo ""
echo "Contents:"
ls -lh dist/tydust-windows/
