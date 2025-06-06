#!/usr/bin/env zsh
# release.zsh - Helper script for creating releases

set -e

# Enable colors
autoload -U colors && colors

echo "${fg_bold[blue]}🚀 Beamterm Release Helper${reset_color}"
echo ""

# Check if we're in the root of the project
if [[ ! -f "Cargo.toml" ]] || [[ ! -d "beamterm-renderer" ]]; then
    echo "${fg_bold[red]}❌ Error: This script must be run from the project root${reset_color}"
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
echo "${fg[cyan]}📌 Current version: ${fg_bold[white]}v$CURRENT_VERSION${reset_color}"
echo ""

# Ask for new version with zsh's vared for better editing
echo "${fg[yellow]}Enter new version (without 'v' prefix):${reset_color}"
vared -p "${fg[green]}> ${reset_color}" -c NEW_VERSION

if [[ -z "$NEW_VERSION" ]]; then
    echo "${fg_bold[red]}❌ Error: Version cannot be empty${reset_color}"
    exit 1
fi

# Validate version format (semantic versioning)
if ! [[ "$NEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$ ]]; then
    echo "${fg_bold[red]}❌ Error: Invalid version format. Please use semantic versioning (e.g., 1.2.3 or 1.2.3-beta.1)${reset_color}"
    exit 1
fi

echo ""
echo "${fg[blue]}🔄 Updating version from ${fg_bold[white]}$CURRENT_VERSION${fg[blue]} to ${fg_bold[white]}$NEW_VERSION${reset_color}..."

# Update version in all Cargo.toml files
# Using zsh glob with null_glob option to handle no matches gracefully
setopt null_glob
cargo_files=(**/Cargo.toml)

for file in $cargo_files; do
    # Cross-platform sed (works on both macOS and Linux)
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$file"
    else
        sed -i "s/^version = \".*\"/version = \"$NEW_VERSION\"/" "$file"
    fi
    echo "  ${fg[green]}✓${reset_color} Updated $file"
done

echo "${fg_bold[green]}✅ Version updated in ${#cargo_files} Cargo.toml files${reset_color}"

# Run tests
echo ""
echo "${fg[cyan]}🧪 Running tests...${reset_color}"
cargo test --workspace --exclude beamterm-renderer || {
    echo "${fg_bold[red]}❌ Tests failed!${reset_color}"
    exit 1
}

# Check formatting
echo ""
echo "${fg[cyan]}🎨 Checking formatting...${reset_color}"
cargo fmt --all -- --check || {
    echo "${fg_bold[red]}❌ Code is not formatted! Run 'cargo fmt' to fix.${reset_color}"
    exit 1
}

# Run clippy
echo ""
echo "${fg[cyan]}📎 Running clippy...${reset_color}"
cargo clippy --all-targets --all-features -- -D warnings || {
    echo "${fg_bold[red]}❌ Clippy found issues!${reset_color}"
    exit 1
}

# Build to ensure everything compiles
echo ""
echo "${fg[cyan]}🔨 Building project...${reset_color}"
cargo build --workspace || {
    echo "${fg_bold[red]}❌ Build failed!${reset_color}"
    exit 1
}

# Check WASM build
echo "${fg[cyan]}🕸️  Checking WASM build...${reset_color}"
cargo check -p beamterm-renderer --target wasm32-unknown-unknown || {
    echo "${fg_bold[red]}❌ WASM build check failed!${reset_color}"
    exit 1
}

# Show changes before committing
echo ""
echo "${fg[yellow]}📝 Changes to be committed:${reset_color}"
git diff --name-only

# Ask for confirmation
echo ""
echo "${fg[yellow]}Proceed with commit? [y/N]${reset_color}"
read -q REPLY
echo # new line after read -q

if [[ ! "$REPLY" == "y" ]]; then
    echo "${fg[yellow]}⚠️  Release cancelled. Version changes are still in working directory.${reset_color}"
    exit 0
fi

# Commit changes
echo ""
echo "${fg[cyan]}💾 Committing version bump...${reset_color}"
git add -A
git commit -m "chore: bump version to $NEW_VERSION"

# Create tag
echo ""
echo "${fg[cyan]}🏷️  Creating tag v$NEW_VERSION...${reset_color}"
git tag -a "beamterm-v$NEW_VERSION" -m "Release v$NEW_VERSION"

# Show summary
echo ""
echo "${fg_bold[green]}✨ Release preparation complete!${reset_color}"
echo ""
echo "${fg[cyan]}📝 Summary:${reset_color}"
echo "  • Version: ${fg_bold[white]}$CURRENT_VERSION → $NEW_VERSION${reset_color}"
echo "  • Tag: ${fg_bold[white]}v$NEW_VERSION${reset_color}"
echo "  • Commit: ${fg[gray]}$(git rev-parse --short HEAD)${reset_color}"
echo ""
echo "${fg[yellow]}📋 Next steps:${reset_color}"
echo "  ${fg[white]}1.${reset_color} Review the changes: ${fg[gray]}git show${reset_color}"
echo "  ${fg[white]}2.${reset_color} Push to GitHub: ${fg[gray]}git push && git push --tags${reset_color}"
echo "  ${fg[white]}3.${reset_color} The release workflow will automatically create a GitHub release"
echo ""
echo "${fg[red]}⚠️  To undo:${reset_color}"
echo "  ${fg[gray]}git reset --hard HEAD~1${reset_color}"
echo "  ${fg[gray]}git tag -d beamterm-v$NEW_VERSION${reset_color}"

