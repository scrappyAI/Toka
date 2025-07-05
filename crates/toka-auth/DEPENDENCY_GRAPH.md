# Toka Auth Dependency Graph

This document provides a visual representation of the dependency tree for the `toka-auth` crate.

## Overview

The `toka-auth` crate is responsible for authentication and authorization functionality within the Toka ecosystem. It leverages several key dependencies to provide secure, async-compatible authentication services.

## Main Dependencies

```mermaid
graph TD
    A[toka-auth v0.2.1] --> B[async-trait v0.1.88]
    A --> C[serde v1.0.219]
    A --> D[jsonwebtoken v9.3.1]
    A --> E[toka-types v0.2.1]
    A --> F[uuid v1.17.0]
    
    %% Core functionality
    B --> B1[proc-macro2]
    B --> B2[quote]
    B --> B3[syn]
    
    C --> C1[serde_derive]
    
    D --> D1[base64]
    D --> D2[ring]
    D --> D3[pem]
    D --> D4[serde_json]
    D --> D5[simple_asn1]
    
    E --> E1[serde]
    
    F --> F1[getrandom]
    F --> F2[serde]
    
    %% Styling
    classDef mainCrate fill:#ff6b6b,stroke:#333,stroke-width:3px,color:#fff
    classDef internalCrate fill:#4ecdc4,stroke:#333,stroke-width:2px,color:#fff
    classDef coreDep fill:#45b7d1,stroke:#333,stroke-width:2px,color:#fff
    classDef utilDep fill:#96ceb4,stroke:#333,stroke-width:1px
    classDef procMacro fill:#ffeaa7,stroke:#333,stroke-width:1px
    
    class A mainCrate
    class E internalCrate
    class B,C,D,F coreDep
    class D1,D2,D3,D4,D5,F1,F2 utilDep
    class B1,B2,B3,C1 procMacro
```

## Development Dependencies

```mermaid
graph TD
    A[toka-auth v0.2.1] --> G[chrono v0.4.41]
    A --> H[proptest v1.7.0]
    A --> I[tokio v1.46.0]
    A --> J[tokio-test v0.4.4]
    
    %% Development tools
    G --> G1[num-traits]
    G --> G2[serde]
    G --> G3[iana-time-zone]
    
    H --> H1[num-traits]
    H --> H2[rand_chacha]
    H --> H3[rusty-fork]
    H --> H4[bit-set]
    H --> H5[regex-syntax]
    
    I --> I1[mio]
    I --> I2[bytes]
    I --> I3[parking_lot]
    I --> I4[pin-project-lite]
    I --> I5[socket2]
    
    J --> J1[async-stream]
    J --> J2[futures-core]
    J --> J3[tokio-stream]
    
    %% Styling
    classDef mainCrate fill:#ff6b6b,stroke:#333,stroke-width:3px,color:#fff
    classDef devDep fill:#fd79a8,stroke:#333,stroke-width:2px,color:#fff
    classDef devUtil fill:#fdcb6e,stroke:#333,stroke-width:1px
    
    class A mainCrate
    class G,H,I,J devDep
    class G1,G2,G3,H1,H2,H3,H4,H5,I1,I2,I3,I4,I5,J1,J2,J3 devUtil
```

## Dependency Categories

### Core Authentication Dependencies
- **jsonwebtoken**: JWT token handling and validation
- **ring**: Cryptographic primitives for secure operations
- **base64**: Base64 encoding/decoding for token formats
- **pem**: PEM format handling for certificates and keys
- **simple_asn1**: ASN.1 parsing for cryptographic data structures

### Serialization & Data Handling
- **serde**: Serialization and deserialization framework
- **serde_json**: JSON serialization support
- **uuid**: UUID generation and handling

### Async Support
- **async-trait**: Enables async functions in traits
- **tokio**: Async runtime (dev dependency)
- **tokio-test**: Testing utilities for async code

### Development & Testing
- **chrono**: Date and time handling for testing
- **proptest**: Property-based testing framework
- **tokio-test**: Async testing utilities

### Internal Dependencies
- **toka-types**: Internal type definitions shared across Toka crates

## Key Features Enabled

1. **JWT Authentication**: Complete JWT token lifecycle management
2. **Async Compatibility**: Full async/await support through async-trait
3. **Secure Cryptography**: Ring-based cryptographic operations
4. **Flexible Serialization**: Serde-based data transformation
5. **Comprehensive Testing**: Property-based and async testing capabilities
6. **Type Safety**: Strong type safety through toka-types integration

## Architecture Notes

The dependency graph shows that `toka-auth` is designed as a secure, async-first authentication library with the following architectural decisions:

- **Cryptographic Security**: Uses `ring` for cryptographic operations, providing high-performance, secure implementations
- **Standard Compliance**: Supports industry-standard JWT tokens and PEM formats
- **Async-First Design**: Built with async/await patterns for non-blocking operations
- **Type Safety**: Leverages Rust's type system and internal type definitions for compile-time safety
- **Testing-Focused**: Extensive testing infrastructure with both unit and property-based tests

This dependency structure ensures that `toka-auth` can provide secure, efficient authentication services while maintaining compatibility with the broader Toka ecosystem.