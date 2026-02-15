# CI/CD Setup

## Fixed Issues
- ✅ Code formatting and linting
- ✅ Updated artifact actions to v4
- ✅ Simplified CI to focus on Linux (macOS/Windows non-blocking)
- ✅ Fixed all GitHub URLs (cliai-team → cliai)
- ✅ ARM64 builds use `cross` tool
- ✅ Documentation workflow configured

## Enable GitHub Pages
1. Go to: **Settings** → **Pages**
2. Set **Source** to: **GitHub Actions**
3. Save

Documentation will be at: `https://cliai.github.io/cliai/`

## What Runs When

### On Push to Main (CI)
- Tests on Linux (required), macOS/Windows (optional)
- Security audit (non-blocking)
- Code coverage (non-blocking)
- Documentation deploys to GitHub Pages

### On Tag Push (v0.1.0)
- Builds binaries for all platforms
- Creates GitHub Release with downloads
- Publishes to crates.io (if token set)

## Create Release
```bash
git tag v0.1.0
git push origin v0.1.0
```

## Optional Secrets
- `CARGO_REGISTRY_TOKEN` - for crates.io publishing
- `CODECOV_TOKEN` - for coverage reports
