# RVector

`RVector` is a MongoDB-inspired vector database. It is a document-based application capable of storing and retrieving vectors of any dimension. It is designed for RAG-based (Retrieval-Augmented Generation) applications, where vectors are used to represent data.

> ![Important]
> This project is currently under development. Please check back later for updates.
> Expected release date: 5th August 2024

## Features

- Store and retrieve vectors of any dimension
- Fast retrieval of vectors
- Easy to use API
- Document-based storage
- Thread-safe

## Getting Started

1. Clone the repository:

```bash
git clone https://github.com/himasnhu-at/rvector.git
```

2. Navigate to the project directory:

```bash
cd rvector
```

3. Run the CLI:

```bash
cargo run --bin cli # add --release, if you want to run the optimized version
```

4. Lint the code:

```bash
cargo clippy # lint the code
cargo fix # fix the code
```

5. Start the documentation server:

```bash
cargo doc --open
```

5. Run the tests:

```bash
cargo test
```

6. Build the project:

```bash
cargo build # add --release, if you want to run the optimized version
```

## Workspace Structure

### cli

The `cli` package provides the command-line interface for interacting with RVector. It allows users to perform various operations such as inserting, querying, and managing the vector data.

### packages/storage

The `packages/storage` module handles the low-level storage mechanisms. It is responsible for efficiently storing and retrieving vector documents from disk or other storage backends.

### packages/vector

The `packages/vector` module is the core component that manages vector operations. It includes functionality for vector arithmetic, similarity calculations, and other vector-related computations.

### packages/embeddings

The `packages/embeddings` module integrates with various embeddings models. It allows for the conversion of data into vector representations that can be stored and queried in RVector.

### packages/rvector_core

The `packages/rvector_core` module contains the core logic and algorithms of RVector. It orchestrates the interactions between storage, vector operations, and embeddings to provide a seamless user experience.

### packages/request_handler

The `packages/request_handler` module is responsible for handling API requests. It parses incoming requests, invokes the appropriate operations, and returns the results to the client.

### tools/logger

The `tools/logger` module provides logging capabilities. It ensures that all operations and events within RVector are logged appropriately for debugging and monitoring purposes.

### tools/monitor

The `tools/monitor` module provides monitoring and metrics collection. It helps track the performance and health of the RVector system, providing insights into usage patterns and potential bottlenecks.

### tools/tests

The `tools/tests` module includes test cases for the various components of RVector. It ensures that all functionalities are working as expected and helps maintain the quality and reliability of the codebase.

## Contributing

We welcome contributions from the community! Please read our [contributing guidelines](CONTRIBUTING.md) to get started.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contact

For any questions or suggestions, feel free to open an issue or contact us at [hyattherate2005@gmail.com](mailto:hyattherate2005@gmail.com).
