# MCP Client Release Process

This document outlines the release process for the MCP Client application, including version management, testing, building, and deployment.

## Release Checklist

### Pre-Release Preparation

1. **Update Dependencies**
   - [ ] Update Rust dependencies: `cargo update`
   - [ ] Update npm dependencies: `cd src-frontend && npm update`
   - [ ] Test application with updated dependencies
   - [ ] Commit dependency updates

2. **Code Freeze**
   - [ ] Announce code freeze to development team
   - [ ] Ensure all features for the release are complete
   - [ ] Create a release branch: `git checkout -b release/vX.Y.Z`

3. **Documentation Updates**
   - [ ] Update README.md with new features
   - [ ] Update CHANGELOG.md with detailed changes
   - [ ] Update API documentation if necessary
   - [ ] Verify installation instructions are current
   - [ ] Update screenshots if UI has changed

### Testing Phase

1. **Automated Testing**
   - [ ] Ensure all unit tests pass: `cargo test --lib`
   - [ ] Run integration tests: `cargo test --test '*'`
   - [ ] Run performance benchmarks: `cargo bench --features benchmarking`
   - [ ] Run end-to-end tests: `cd src-frontend && npm run test:e2e`
   - [ ] Fix any failing tests

2. **Manual Testing**
   - [ ] Test offline functionality
   - [ ] Test auto-update process
   - [ ] Test installer packages on all platforms
   - [ ] Test performance with large datasets
   - [ ] Verify memory usage is within acceptable limits

3. **Beta Release (Optional)**
   - [ ] Tag beta release: `git tag -a vX.Y.Z-beta.1 -m "MCP Client vX.Y.Z Beta 1"`
   - [ ] Push tag: `git push origin vX.Y.Z-beta.1`
   - [ ] CI system will build beta packages
   - [ ] Distribute to beta testers
   - [ ] Collect feedback and fix issues

### Release Process

1. **Version Bump**
   - [ ] Update version in Cargo.toml
   - [ ] Update version in package.json
   - [ ] Update version in src-tauri/tauri.conf.json
   - [ ] Commit version changes: `git commit -m "Bump version to vX.Y.Z"`

2. **Final Testing**
   - [ ] Run full test suite one final time
   - [ ] Build release packages locally to verify
   - [ ] Test update process from previous version

3. **Release Tagging**
   - [ ] Merge release branch to main: `git checkout main && git merge release/vX.Y.Z`
   - [ ] Tag release: `git tag -a vX.Y.Z -m "MCP Client vX.Y.Z"`
   - [ ] Push main and tag: `git push origin main && git push origin vX.Y.Z`

4. **CI/CD Release Build**
   - [ ] CI system detects tag and starts release workflow
   - [ ] Monitor build process in GitHub Actions
   - [ ] Verify all build artifacts are created
   - [ ] Check update manifests are published correctly

5. **Release Publication**
   - [ ] Verify GitHub release is created automatically
   - [ ] Edit release notes if necessary
   - [ ] Publish release

6. **Post-Release Steps**
   - [ ] Notify users through update notification
   - [ ] Publish announcement on relevant channels
   - [ ] Monitor telemetry for issues
   - [ ] Begin next development cycle on develop branch

## Version Numbering

MCP Client follows [Semantic Versioning](https://semver.org/):

- **MAJOR version (X)** for incompatible API changes
- **MINOR version (Y)** for new features in a backward-compatible manner
- **PATCH version (Z)** for backward-compatible bug fixes

Additional labels for pre-release and build metadata are available as extensions to the MAJOR.MINOR.PATCH format.

## Release Cadence

- **Patch releases (X.Y.Z)**: As needed for bug fixes, typically every 1-2 weeks
- **Minor releases (X.Y.0)**: Every 1-2 months for new features
- **Major releases (X.0.0)**: Every 6-12 months for significant changes

## Release Artifacts

The CI/CD pipeline produces the following artifacts for each platform:

### Windows
- MSI installer for 64-bit systems
- Portable EXE for 64-bit systems
- Update manifest for auto-updates

### macOS
- Universal DMG (Intel + Apple Silicon)
- Intel-specific DMG
- Apple Silicon-specific DMG
- Update manifest for auto-updates

### Linux
- DEB package for Debian/Ubuntu
- RPM package for Fedora/RHEL
- AppImage for other distributions
- Update manifest for auto-updates

## Update Server

The update server is hosted on GitHub Pages and serves the update manifests for each platform. The CI/CD pipeline automatically publishes the update manifests to the `updates` branch.

Update endpoints:
- Windows: `https://update.mcp-client.com/windows/[version]`
- macOS: `https://update.mcp-client.com/macos/[version]`
- Linux: `https://update.mcp-client.com/linux/[version]`

## Emergency Hotfix Process

In case a critical bug is discovered in a release:

1. Create a hotfix branch from the release tag: `git checkout -b hotfix/vX.Y.Z+1 vX.Y.Z`
2. Fix the issue with minimal changes
3. Run focused tests to verify the fix
4. Update version to X.Y.Z+1
5. Tag and release following the normal release process
6. Backport the fix to the develop branch

## Telemetry Analysis

After each release, analyze telemetry data to identify issues:

1. Check error rates compared to previous version
2. Monitor performance metrics for degradation
3. Track crash reports for new crash types
4. Analyze user engagement with new features
5. Look for anomalies in usage patterns

If significant issues are detected, consider rolling out a hotfix release.