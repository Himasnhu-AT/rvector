# RVector tools

This folder contains all the tools that we utilize to build the RVector system. Each tool is a separate Rust crate that can be compiled and tested independently. The tools are organized based on their functionality and purpose.

## Tools

- `logger`: Provides logging capabilities. It ensures that all operations and events within RVector are logged appropriately for debugging and monitoring purposes.

- `monitor`: Monitors the performance and health of the RVector system. It collects metrics, logs, and other data to help identify and resolve issues.

- `tests`: Contains various tests for the RVector system. It includes unit tests, integration tests, and performance tests to ensure the system's reliability and efficiency.

> [!NOTE]
> `tests` will be depreiciated in the future, instead we'll store tests in file only
> though for E2E and integration tests, we'll have separate crates.
