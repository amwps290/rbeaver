# GitHub Actions CI/CD for RBeaver

This directory contains GitHub Actions workflows for automated building, testing, and releasing of the RBeaver database management tool.

## Workflows Overview

### 1. CI/CD Pipeline (`ci.yml`)

**Triggers:**
- Push to `main` branch
- Creation of version tags (e.g., `v1.0.0`)
- Pull requests to `main` branch

**Features:**
- **Test Suite**: Runs formatting checks, clippy lints, and unit tests
- **Multi-platform Build**: Compiles for Linux (x86_64) and Windows (x86_64)
- **Automatic Release**: Creates GitHub releases when version tags are pushed
- **Artifact Upload**: Stores compiled binaries for 30 days

**System Dependencies Handled:**
- Linux GUI libraries (X11, Wayland, OpenGL, EGL)
- Audio libraries for potential future audio features
- All egui/eframe dependencies

### 2. Optimized Build (`build-optimized.yml`)

**Triggers:**
- Manual workflow dispatch (can be triggered from GitHub UI)

**Features:**
- **Safe Optimization**: Uses thin LTO and conservative optimization settings
- **Binary Compression**: Uses UPX to reduce binary size (with fallback if compression fails)
- **Archive Creation**: Creates tar.gz (Linux) and zip (Windows) packages
- **Manual Release**: Option to create releases with custom tags
- **Extended Retention**: Artifacts stored for 90 days

**Optimization Profile:**
```toml
[profile.release-opt]
inherits = "release"
opt-level = 3
lto = "thin"
codegen-units = 1
panic = "abort"
strip = true
```

### 3. Simple Build (`build-simple.yml`)

**Triggers:**
- Manual workflow dispatch (backup option for compatibility issues)

**Features:**
- **Basic Optimization**: Uses only safe, widely compatible optimization flags
- **No Compression**: Avoids potential UPX compatibility issues
- **Reliable Builds**: Designed for maximum compatibility across environments
- **Quick Fallback**: Use when optimized builds encounter issues

### 4. Code Quality (`quality.yml`)

**Triggers:**
- Push to `main` or `develop` branches
- Pull requests to `main` or `develop` branches
- Weekly schedule (Sundays at 2 AM UTC)

**Features:**
- **Strict Linting**: Enhanced clippy checks with pedantic and nursery lints
- **Security Audit**: Checks for known vulnerabilities in dependencies
- **Dependency Analysis**: Detects unused and outdated dependencies
- **Test Coverage**: Generates coverage reports and uploads to Codecov
- **Performance Benchmarks**: Runs on main branch pushes

## Usage Instructions

### Creating a Release

#### Automatic Release (Recommended)
1. Create and push a version tag:
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```
2. The CI pipeline will automatically build and create a GitHub release

#### Manual Optimized Release
1. Go to GitHub Actions tab
2. Select "Optimized Build" workflow
3. Click "Run workflow"
4. Fill in the parameters:
   - Check "Create a release after successful build"
   - Enter release tag (e.g., `v1.0.0-optimized`)
5. Click "Run workflow"

### Monitoring Build Status

All workflows provide detailed logs and status badges. You can add these badges to your main README:

```markdown
![CI/CD](https://github.com/your-username/RBeaver/workflows/CI/CD%20Pipeline/badge.svg)
![Quality](https://github.com/your-username/RBeaver/workflows/Code%20Quality/badge.svg)
```

### Artifacts and Downloads

- **Regular builds**: Available in Actions tab under each workflow run
- **Releases**: Available in the Releases section with downloadable binaries
- **Coverage reports**: Automatically uploaded to Codecov (if configured)

## Configuration Requirements

### Secrets (Optional)
- `CODECOV_TOKEN`: For uploading test coverage reports to Codecov

### Repository Settings
- Ensure "Actions" are enabled in repository settings
- For automatic releases, ensure the repository has "Write" permissions for Actions

## Binary Naming Convention

- **Linux**: `rbeaver-linux-x86_64`
- **Windows**: `rbeaver-windows-x86_64.exe`
- **Optimized builds**: Include compression and are packaged in archives

## Troubleshooting

### Common Issues

1. **Linux build fails with missing libraries**
   - The workflow installs all required system dependencies
   - If issues persist, check the specific error in the build logs

2. **Windows build fails**
   - Usually related to MSVC toolchain issues
   - The workflow uses the official Microsoft-hosted runners

3. **Optimization build fails with LTO errors**
   - Error: `options -C embed-bitcode=no and -C lto are incompatible`
   - Solution: Use the "Simple Build" workflow as a fallback
   - The optimized build now uses safer "thin" LTO instead of "fat" LTO

4. **UPX compression fails**
   - UPX might fail on some binaries; this is handled gracefully
   - The uncompressed binary will still be available
   - Error is logged but doesn't fail the build

5. **Release creation fails**
   - Check that the tag follows semantic versioning (e.g., `v1.0.0`)
   - Ensure repository has proper permissions for Actions

6. **Cargo profile errors**
   - If custom profiles cause issues, the simple build uses standard release profile
   - Check Rust version compatibility for advanced profile features

### Performance Considerations

- **Cache Strategy**: All workflows use Cargo registry caching to speed up builds
- **Parallel Jobs**: Test and build jobs run in parallel when possible
- **Incremental Builds**: Leverages Rust's incremental compilation

## Customization

### Adding New Targets
To add support for additional platforms, modify the matrix in `ci.yml`:

```yaml
matrix:
  include:
    - target: x86_64-unknown-linux-gnu
      os: ubuntu-latest
      name: rbeaver-linux-x86_64
    - target: x86_64-pc-windows-msvc
      os: windows-latest
      name: rbeaver-windows-x86_64.exe
    # Add new targets here
    - target: aarch64-apple-darwin
      os: macos-latest
      name: rbeaver-macos-arm64
```

### Modifying Build Flags
Edit the `RUSTFLAGS` environment variable in the optimized build workflow to adjust compilation settings.

### Changing Quality Checks
Modify the clippy arguments in `quality.yml` to adjust linting strictness:

```yaml
- name: Run clippy (strict)
  run: |
    cargo clippy --all-targets --all-features -- \
      -D warnings \
      # Add or remove lint rules here
```
