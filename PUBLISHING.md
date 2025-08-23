# Publishing BWS to crates.io

## Prerequisites

1. **Create a crates.io account**: Go to https://crates.io and sign up with GitHub
2. **Get an API token**: 
   - Go to https://crates.io/me
   - Click "New Token" 
   - Give it a name like "BWS Publishing"
   - Copy the token

## Publishing Steps

### 1. Login to crates.io
```bash
cargo login <your-api-token>
```

### 2. Verify your package is ready
```bash
# Check that everything builds correctly
cargo build --release

# Run tests to ensure everything works
cargo test

# Check package contents (dry run)
cargo publish --dry-run
```

### 3. Publish to crates.io
```bash
# Publish the package
cargo publish
```

## Important Notes

### Before Publishing:
- ✅ Update email in Cargo.toml (authors field)
- ✅ Ensure README.md is comprehensive
- ✅ Add proper keywords and categories
- ✅ Test the package thoroughly
- ✅ Check all dependencies are from crates.io

### Package Requirements:
- ✅ Unique name on crates.io (check at https://crates.io/crates/bws)
- ✅ Valid license (MIT ✅)
- ✅ Description and documentation
- ✅ Repository URL
- ✅ README file

### Post-Publishing:
- Package will be available at: https://crates.io/crates/bws
- Users can install with: `cargo install bws`
- Documentation will be auto-generated at: https://docs.rs/bws

## Version Management

### Updating versions:
1. Update version in Cargo.toml (e.g., 0.1.0 → 0.1.1)
2. Commit changes
3. Create git tag: `git tag v0.1.1`
4. Push tag: `git push origin v0.1.1`
5. Publish new version: `cargo publish`

### Semantic Versioning:
- **0.1.0** → Initial release
- **0.1.1** → Bug fixes
- **0.2.0** → New features (backwards compatible)
- **1.0.0** → First stable release

## Documentation

Your package documentation will be automatically generated from:
- README.md (shown on crates.io page)
- Doc comments in your Rust code (shown on docs.rs)
- Examples in your code

## Troubleshooting

### Common Issues:
- **Name already taken**: Choose a different name in Cargo.toml
- **Missing fields**: Ensure all required fields are in Cargo.toml
- **Build failures**: Fix any compilation errors
- **Large package**: Use `exclude` in Cargo.toml to remove unnecessary files

### Package Size Limits:
- Maximum package size: 10 MB
- Use `exclude` in Cargo.toml to remove:
  - Test files
  - Build artifacts
  - CI configuration
  - Documentation that's not essential

## Next Steps After Publishing

1. **Announce your package**:
   - Reddit r/rust
   - Rust Discord/Zulip
   - Twitter/social media

2. **Add badges to README**:
   - Crates.io version
   - Downloads count
   - Documentation link

3. **Monitor usage**:
   - Check download stats on crates.io
   - Watch for issues/feedback
   - Respond to community questions
