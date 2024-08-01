# RVector packages

This folder contains all different packages that we utilize to build the RVector system. Each package is a separate Rust crate that can be compiled and tested independently. The packages are organized based on their functionality and purpose.

## Packages

- `configs`: Contains the configuration settings for the RVector system. And how you can load custom configurations.

- `embeddings`: Integrates with various embeddings models. It allows for the conversion of data into vector representations that can be stored and queried in RVector.

- `rvector_core`: Contains the core logic and algorithms of RVector. It orchestrates the interactions between storage, vector operations, and embeddings to provide a seamless user experience.

- `storage`: Handles the low-level storage mechanisms. It is responsible for efficiently storing and retrieving vector documents from disk or other storage backends.

- `vector`: The core component that manages vector operations. It includes functionality for vector arithmetic, similarity calculations, and other vector-related computations.

- `request_handler`: Helps `rvector_core` to handle certain functionalities like HTTP requests, etc.

> [!NOTE]
> The `request_handler` package might be deprecated in the future as we are planning to move towards a more modular architecture. Where `<project_root>/server` will be the main entry point for the server.
