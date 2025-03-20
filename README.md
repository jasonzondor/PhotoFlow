# PhotoFlow

A modern photo management application built with Rust and Iced.

## Features

- Browse and view photos in a directory
- Support for various image formats including JPEG and RAW files
- Display EXIF metadata (date, camera info, exposure settings)
- Clean and modern UI built with Iced

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/PhotoFlow.git
cd PhotoFlow

# Build the project
cargo build

# Run in debug mode
cargo run
```

## Project Structure

```
.
├── src/
│   ├── main.rs      # Application entry point and state management
│   ├── photo.rs     # Photo loading and metadata handling
│   └── ui.rs        # User interface components
├── Cargo.toml       # Project dependencies and configuration
└── README.md        # Project documentation
```

## License

MIT License
