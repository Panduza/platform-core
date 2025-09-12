# Engine Module Specification

## Overview

The Engine module is the core component of the Panduza platform that handles connections, events, and communication through the Zenoh protocol. It serves as the foundational layer that powers all attributes and objects within the system.

## Core Components

### Engine Struct

The `Engine` struct is the main entity that manages the platform's communication infrastructure.

**Fields:**
- `session: Session` - A Zenoh session that handles the underlying communication protocol
- `namespace: Option<String>` - An optional namespace that provides logical separation for different engine instances

**Characteristics:**
- Implements `Clone` trait for easy duplication across different contexts
- Serves as the central hub for all pub/sub operations

### Engine Methods

#### Constructor
- `new(session: Session, namespace: Option<String>) -> Self`
  - Creates a new Engine instance with the provided Zenoh session and optional namespace
  - Initializes the core communication infrastructure

#### Topic Management
- `root_topic(&self, namespace: Option<String>) -> String`
  - Generates the root topic path for the engine
  - Combines the provided namespace (or engine's namespace) with the "pza" suffix
  - Returns a properly formatted topic string for Zenoh communication
  - Handles empty namespaces gracefully by omitting the namespace prefix

#### Communication Registration
- `register_listener<A: Into<String> + 'static>(&self, topic: A, _channel_size: usize) -> Subscriber<FifoChannelHandler<Sample>>`
  - Registers a subscriber for listening to messages on a specific topic
  - Returns a Zenoh subscriber with FIFO channel handling
  - Provides asynchronous message reception capabilities

- `register_publisher<A: Into<String> + 'static>(&self, topic: A) -> Result<Publisher, Error>`
  - Registers a publisher for sending messages to a specific topic
  - Returns a Zenoh publisher wrapped in a Result for error handling
  - Enables asynchronous message publishing capabilities

## Builder Pattern

### EngineBuilder Struct

The `EngineBuilder` provides a non-async way to prepare Engine configuration before entering a Tokio runtime context.

**Purpose:**
- Allows synchronous preparation of engine configuration
- Enables use in plugin contexts where async operations are not immediately available
- Defers the actual async connection establishment until explicitly requested

**Methods:**
- `new(options: panudza::Config) -> Self`
  - Creates a new builder instance with the specified options
  - Synchronous operation suitable for plugin initialization

- `build(self) -> Engine` (async)
  - Consumes the builder and creates the actual Engine instance
  - Establishes the Zenoh connection and finalizes the engine setup
  - Must be called within an async context

## Design Principles

1. **Separation of Concerns**: The Engine focuses solely on communication infrastructure, delegating specific functionality to other modules

2. **Async-First Design**: All communication operations are asynchronous, leveraging Tokio for concurrent operations

3. **Flexible Namespacing**: Optional namespace support allows for logical separation of different engine instances or environments

4. **Error Handling**: Proper error propagation using Result types for robust error management

5. **Builder Pattern**: Provides flexibility for different initialization contexts, particularly important for plugin systems

## Dependencies

- **Zenoh**: Core communication protocol for pub/sub messaging
- **Tokio**: Async runtime for handling concurrent operations
- **panudza::Config**: Client config in module panduza to use panudza::connection::create_client_connection
