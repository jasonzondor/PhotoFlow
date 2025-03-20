# PhotoFlow TODO List

## High Priority

- [ ] Improve RAW file support
  - [ ] Fix Fuji X-T3 RAF support
    - Current `rawloader` crate doesn't support X-T3
    - Options:
      1. Contribute X-T3 support to `rawloader`
      2. Switch to alternative RAW processing library
      3. Use external tools (dcraw/libraw) as fallback
  - [ ] Add progress indicator for RAW file loading
  - [ ] Implement RAW file caching for faster loading

- [ ] Enhance Photo Management
  - [ ] Add photo grid view
  - [ ] Implement photo sorting (by date, name, size)
  - [ ] Add basic file operations (delete, move, rename)

## Medium Priority

- [ ] Image Viewing Features
  - [ ] Add zoom controls
  - [ ] Implement pan/scroll for large images
  - [ ] Add fullscreen mode
  - [ ] Support basic image rotation

- [ ] EXIF Data Display
  - [ ] Improve EXIF data formatting
  - [ ] Add filtering/searching of EXIF data
  - [ ] Display camera-specific metadata

## Future Enhancements

- [ ] Photo Organization
  - [ ] Add tagging system
  - [ ] Implement collections/albums
  - [ ] Add star ratings
  - [ ] Add color labels

- [ ] User Interface
  - [ ] Add dark/light theme support
  - [ ] Implement customizable keyboard shortcuts
  - [ ] Add drag-and-drop support
  - [ ] Create preferences dialog

- [ ] Performance
  - [ ] Implement image thumbnail caching
  - [ ] Add background loading for large directories
  - [ ] Optimize memory usage for large collections

## Technical Improvements

- [ ] Testing
  - [ ] Add unit tests for core functionality
  - [ ] Add integration tests
  - [ ] Set up CI/CD pipeline

- [ ] Documentation
  - [ ] Add API documentation
  - [ ] Create user guide
  - [ ] Add developer setup guide

## Completed ✓

- [x] Initial project setup
- [x] Basic image loading
- [x] Basic EXIF data display
- [x] Project structure organization
- [x] Git repository setup
