# Manual Release Scripts

## manual-release.sh

Creates a single-platform npm package for testing or manual publishing.

### Usage

```bash
./scripts/manual-release.sh
```

This script will:
1. Detect your current platform (macOS/Linux/Windows, x64/ARM64)
2. Install the Rust target if needed
3. Build the release binary
4. Create the dist structure
5. Zip the binary
6. Install npm dependencies
7. Create a .tgz package

### Testing Locally

After running the script:

```bash
# Install globally to test
npm install -g npx-cli/dev-manager-mcp-*.tgz

# Run it
dev-manager-mcp --help

# Uninstall when done
npm uninstall -g dev-manager-mcp
```

### Publishing Manually

```bash
cd npx-cli
npm login
npm publish dev-manager-mcp-*.tgz --access public
```

### Important Notes

- This creates a **single-platform** package (only your current platform)
- For production releases with **all platforms**, use GitHub Actions
- The manual script is useful for:
  - Local testing before setting up CI/CD
  - Quick iterations during development
  - Publishing platform-specific test versions
