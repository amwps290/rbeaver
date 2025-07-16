#!/bin/bash

# RBeaver Release Script
# This script helps create releases for the RBeaver project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if we're in a git repository
check_git_repo() {
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        print_error "Not in a git repository!"
        exit 1
    fi
}

# Function to check if working directory is clean
check_clean_working_dir() {
    if ! git diff-index --quiet HEAD --; then
        print_error "Working directory is not clean. Please commit or stash your changes."
        exit 1
    fi
}

# Function to validate version format
validate_version() {
    local version=$1
    if [[ ! $version =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9]+)?$ ]]; then
        print_error "Invalid version format. Use semantic versioning (e.g., v1.0.0, v1.0.0-beta)"
        exit 1
    fi
}

# Function to check if tag already exists
check_tag_exists() {
    local tag=$1
    if git tag -l | grep -q "^$tag$"; then
        print_error "Tag $tag already exists!"
        exit 1
    fi
}

# Function to update version in Cargo.toml
update_cargo_version() {
    local version=$1
    # Remove 'v' prefix for Cargo.toml
    local cargo_version=${version#v}
    
    print_info "Updating version in Cargo.toml to $cargo_version"
    
    # Use sed to update the version
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        sed -i '' "s/^version = \".*\"/version = \"$cargo_version\"/" Cargo.toml
    else
        # Linux
        sed -i "s/^version = \".*\"/version = \"$cargo_version\"/" Cargo.toml
    fi
    
    # Verify the change
    if grep -q "version = \"$cargo_version\"" Cargo.toml; then
        print_success "Version updated in Cargo.toml"
    else
        print_error "Failed to update version in Cargo.toml"
        exit 1
    fi
}

# Function to create and push tag
create_and_push_tag() {
    local version=$1
    local message=$2
    
    print_info "Creating tag $version"
    git tag -a "$version" -m "$message"
    
    print_info "Pushing tag to origin"
    git push origin "$version"
    
    print_success "Tag $version created and pushed successfully!"
}

# Function to show usage
show_usage() {
    echo "Usage: $0 [OPTIONS] <version>"
    echo ""
    echo "Create a release for RBeaver"
    echo ""
    echo "Arguments:"
    echo "  version     Version to release (e.g., v1.0.0, v1.0.0-beta)"
    echo ""
    echo "Options:"
    echo "  -h, --help     Show this help message"
    echo "  -n, --dry-run  Show what would be done without making changes"
    echo "  -m, --message  Custom release message"
    echo ""
    echo "Examples:"
    echo "  $0 v1.0.0"
    echo "  $0 v1.0.0-beta"
    echo "  $0 -m \"Major release with new features\" v2.0.0"
    echo "  $0 --dry-run v1.0.0"
}

# Parse command line arguments
DRY_RUN=false
CUSTOM_MESSAGE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_usage
            exit 0
            ;;
        -n|--dry-run)
            DRY_RUN=true
            shift
            ;;
        -m|--message)
            CUSTOM_MESSAGE="$2"
            shift 2
            ;;
        -*)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
        *)
            VERSION="$1"
            shift
            ;;
    esac
done

# Check if version is provided
if [[ -z "$VERSION" ]]; then
    print_error "Version is required!"
    show_usage
    exit 1
fi

# Main execution
print_info "Starting release process for RBeaver $VERSION"

# Validate inputs
validate_version "$VERSION"
check_git_repo
check_clean_working_dir
check_tag_exists "$VERSION"

# Set default message if not provided
if [[ -z "$CUSTOM_MESSAGE" ]]; then
    CUSTOM_MESSAGE="Release $VERSION"
fi

if [[ "$DRY_RUN" == "true" ]]; then
    print_warning "DRY RUN MODE - No changes will be made"
    print_info "Would update Cargo.toml version to ${VERSION#v}"
    print_info "Would create tag: $VERSION"
    print_info "Would push tag to origin"
    print_info "Would trigger GitHub Actions CI/CD pipeline"
    exit 0
fi

# Confirm with user
print_warning "This will:"
print_warning "  1. Update version in Cargo.toml to ${VERSION#v}"
print_warning "  2. Commit the version change"
print_warning "  3. Create and push tag $VERSION"
print_warning "  4. Trigger GitHub Actions to build and create release"
echo ""
read -p "Continue? (y/N): " -n 1 -r
echo ""

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    print_info "Release cancelled by user"
    exit 0
fi

# Execute release steps
print_info "Updating Cargo.toml version..."
update_cargo_version "$VERSION"

print_info "Committing version change..."
git add Cargo.toml
git commit -m "Bump version to $VERSION"

print_info "Pushing version commit..."
git push origin main

print_info "Creating and pushing release tag..."
create_and_push_tag "$VERSION" "$CUSTOM_MESSAGE"

print_success "Release process completed!"
print_info "GitHub Actions will now build the release automatically."
print_info "Check the Actions tab in your GitHub repository for progress."
print_info "Release will be available at: https://github.com/$(git config --get remote.origin.url | sed 's/.*github.com[:/]\([^.]*\).*/\1/')/releases"
