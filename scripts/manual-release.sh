#!/bin/bash
set -e

echo "🚀 Manual Release Script for dev-manager-mcp"
echo ""

# Detect platform
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map to our platform naming
if [[ "$OS" == "darwin" ]]; then
  if [[ "$ARCH" == "arm64" ]]; then
    PLATFORM="macos-arm64"
    TARGET="aarch64-apple-darwin"
  else
    PLATFORM="macos-x64"
    TARGET="x86_64-apple-darwin"
  fi
  BINARY_NAME="dev-manager-mcp"
elif [[ "$OS" == "linux" ]]; then
  if [[ "$ARCH" == "aarch64" || "$ARCH" == "arm64" ]]; then
    PLATFORM="linux-arm64"
    TARGET="aarch64-unknown-linux-musl"
  else
    PLATFORM="linux-x64"
    TARGET="x86_64-unknown-linux-musl"
  fi
  BINARY_NAME="dev-manager-mcp"
elif [[ "$OS" =~ "mingw" || "$OS" =~ "msys" || "$OS" =~ "cygwin" ]]; then
  if [[ "$ARCH" == "aarch64" || "$ARCH" == "arm64" ]]; then
    PLATFORM="windows-arm64"
    TARGET="aarch64-pc-windows-msvc"
  else
    PLATFORM="windows-x64"
    TARGET="x86_64-pc-windows-msvc"
  fi
  BINARY_NAME="dev-manager-mcp.exe"
else
  echo "❌ Unsupported platform: $OS-$ARCH"
  exit 1
fi

echo "📦 Detected platform: $PLATFORM"
echo "🎯 Target: $TARGET"
echo ""

# Check if rustup target is installed
if ! rustup target list --installed | grep -q "$TARGET"; then
  echo "📥 Installing Rust target: $TARGET"
  rustup target add "$TARGET"
  echo ""
fi

# Build the binary
echo "🔨 Building release binary..."
cargo build --release --target "$TARGET"
echo ""

# Create npx-cli dist structure
echo "📁 Creating dist structure..."
mkdir -p npx-cli/dist/"$PLATFORM"
echo ""

# Copy and zip the binary
BINARY_PATH="target/$TARGET/release/$BINARY_NAME"
if [[ ! -f "$BINARY_PATH" ]]; then
  echo "❌ Binary not found at: $BINARY_PATH"
  exit 1
fi

echo "📦 Zipping binary..."
if command -v zip &> /dev/null; then
  zip -j npx-cli/dist/"$PLATFORM"/dev-manager-mcp.zip "$BINARY_PATH"
else
  echo "❌ 'zip' command not found. Please install zip."
  exit 1
fi
echo ""

# Install npm dependencies if needed
if [[ ! -d "npx-cli/node_modules" ]]; then
  echo "📥 Installing npm dependencies..."
  cd npx-cli
  npm install
  cd ..
  echo ""
fi

# Pack the npm package
echo "📦 Creating npm package..."
cd npx-cli
npm pack
cd ..
echo ""

# Find the created tarball
TARBALL=$(ls npx-cli/dev-manager-mcp-*.tgz | head -n1)

echo "✅ Package created successfully!"
echo ""
echo "📦 Package: $TARBALL"
echo "🏷️  Platform: $PLATFORM"
echo ""
echo "Next steps:"
echo "  1. Test locally: npm install -g $TARBALL"
echo "  2. Login to npm: cd npx-cli && npm login"
echo "  3. Publish: npm publish --access public"
echo ""
echo "⚠️  Note: This package only contains the $PLATFORM binary."
echo "    For multi-platform support, use GitHub Actions CI/CD."
