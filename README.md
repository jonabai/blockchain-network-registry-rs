# Blockchain Network Registry API

A Rust-based microservice for managing blockchain networks, built with Clean/Hexagonal Architecture principles.

## Overview

This service provides a REST API for managing blockchain network configurations, including:
- Network registration with chain ID, RPC URLs, and block explorer URLs
- Support for mainnet and testnet networks
- Fee and gas limit multiplier configuration
- Soft delete functionality for network deactivation

## Architecture

The project follows **Clean/Hexagonal Architecture** with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                     INFRASTRUCTURE                          │
│  ┌───────────────────────────────────────────────────────┐  │
│  │                    APPLICATION                         │  │
│  │  ┌─────────────────────────────────────────────────┐  │  │
│  │  │                    DOMAIN                        │  │  │
│  │  │                                                  │  │  │
│  │  │   Entities, Value Objects, Gateway Traits        │  │  │
│  │  │                                                  │  │  │
│  │  └─────────────────────────────────────────────────┘  │  │
│  │                                                        │  │
│  │   Use Cases, DTOs, Application Services                │  │
│  │                                                        │  │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
│   Handlers, Repositories, External APIs, Database           │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Layer Structure

| Layer | Location | Purpose |
|-------|----------|---------|
| **Domain** | `src/domain/` | Core business logic, entities, gateway traits |
| **Application** | `src/application/` | Use cases orchestrating business logic |
| **Infrastructure** | `src/infrastructure/` | External concerns (HTTP, database, etc.) |
| **Shared** | `src/shared/` | Cross-cutting utilities (errors, etc.) |

## Project Structure

```
src/
├── main.rs                          # Application entry point
├── lib.rs                           # Library exports
├── domain/
│   ├── models/
│   │   └── network.rs               # Network entity, NetworkId
│   └── gateways/
│       └── network_repository.rs    # Repository trait
├── application/
│   └── use_cases/
│       └── networks/
│           ├── create_network.rs
│           ├── get_network_by_id.rs
│           ├── get_active_networks.rs
│           ├── update_network.rs
│           ├── partial_update_network.rs
│           └── delete_network.rs
├── infrastructure/
│   ├── driven_adapters/
│   │   ├── config.rs                # Configuration management
│   │   ├── database.rs              # Database connection
│   │   └── network_repository/
│   │       └── postgres.rs          # PostgreSQL implementation
│   └── driving_adapters/
│       └── api_rest/
│           ├── dto/
│           │   └── network.rs       # Request/Response DTOs
│           ├── handlers/
│           │   └── networks.rs      # HTTP handlers
│           └── middleware/
│               └── auth.rs          # JWT authentication
└── shared/
    └── errors/
        └── mod.rs                   # Error types
```

## Prerequisites

- **Rust** 1.75 or later
- **PostgreSQL** 15 or later
- **SQLx CLI** (for migrations)

## Getting Started

### 1. Install Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install SQLx CLI for database migrations
cargo install sqlx-cli --no-default-features --features postgres
```

### 2. Set Up PostgreSQL Database

```bash
# Create database and user
psql -U postgres << EOF
CREATE USER network_registry WITH PASSWORD 'network_registry';
CREATE DATABASE network_registry OWNER network_registry;
GRANT ALL PRIVILEGES ON DATABASE network_registry TO network_registry;
EOF
```

Or using Docker:

```bash
docker run -d \
  --name network-registry-db \
  -e POSTGRES_USER=network_registry \
  -e POSTGRES_PASSWORD=network_registry \
  -e POSTGRES_DB=network_registry \
  -p 5432:5432 \
  postgres:15
```

### 3. Configure Environment

Create a `config/local.toml` file for local development (optional):

```toml
[server]
host = "127.0.0.1"
port = 3000

[database]
url = "postgres://network_registry:network_registry@localhost:5432/network_registry"
max_connections = 10
min_connections = 2

[jwt]
secret = "your-secure-secret-key-here"
expires_in_secs = 3600
```

Or use environment variables:

```bash
export APP__DATABASE__URL="postgres://network_registry:network_registry@localhost:5432/network_registry"
export APP__JWT__SECRET="your-secure-secret-key-here"
```

### 4. Run Database Migrations

```bash
sqlx migrate run
```

### 5. Run the Application

```bash
# Development mode
cargo run

# Or with hot reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run
```

The server will start at `http://127.0.0.1:3000`.

## API Endpoints

| Method | Endpoint | Description | Auth Required |
|--------|----------|-------------|---------------|
| `POST` | `/networks` | Create a new network | Yes |
| `GET` | `/networks` | List all active networks | Yes |
| `GET` | `/networks/:id` | Get network by ID | Yes |
| `PUT` | `/networks/:id` | Full update (except active) | Yes |
| `PATCH` | `/networks/:id` | Partial update (including active) | Yes |
| `DELETE` | `/networks/:id` | Soft delete network | Yes |

### Request/Response Examples

#### Create Network

```bash
curl -X POST http://localhost:3000/networks \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <your-jwt-token>" \
  -d '{
    "chainId": 1,
    "name": "Ethereum Mainnet",
    "rpcUrl": "https://mainnet.infura.io/v3/YOUR-PROJECT-ID",
    "otherRpcUrls": ["https://eth.llamarpc.com"],
    "testNet": false,
    "blockExplorerUrl": "https://etherscan.io",
    "feeMultiplier": 1.0,
    "gasLimitMultiplier": 1.2,
    "defaultSignerAddress": "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD"
  }'
```

#### Response

```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "chainId": 1,
  "name": "Ethereum Mainnet",
  "rpcUrl": "https://mainnet.infura.io/v3/YOUR-PROJECT-ID",
  "otherRpcUrls": ["https://eth.llamarpc.com"],
  "testNet": false,
  "blockExplorerUrl": "https://etherscan.io",
  "feeMultiplier": 1.0,
  "gasLimitMultiplier": 1.2,
  "active": true,
  "defaultSignerAddress": "0x742d35Cc6634C0532925a3b844Bc9e7595f1dEaD",
  "createdAt": "2024-12-29T10:30:00Z",
  "updatedAt": "2024-12-29T10:30:00Z"
}
```

#### Error Response

```json
{
  "error": {
    "code": "CONFLICT",
    "message": "Network with chain_id 1 already exists"
  },
  "timestamp": "2024-12-29T10:30:00Z"
}
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Unit Tests Only

```bash
cargo test --lib
```

### Run Tests with Output

```bash
cargo test -- --nocapture
```

### Run Specific Test

```bash
cargo test test_network_new
```

### Test Coverage

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage report
cargo tarpaulin --out Html
```

## Development

### Code Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run clippy
cargo clippy

# Treat warnings as errors
cargo clippy -- -D warnings
```

### Build for Production

```bash
cargo build --release
```

### Generate Documentation

```bash
cargo doc --open
```

## Configuration

Configuration is loaded from multiple sources (in order of precedence):

1. Environment variables (prefix: `APP__`, separator: `__`)
2. `config/local.toml` (optional, for local overrides)
3. `config/{RUN_MODE}.toml` (based on `RUN_MODE` env var)
4. `config/default.toml` (base configuration)

### Configuration Options

| Key | Environment Variable | Description | Default |
|-----|---------------------|-------------|---------|
| `server.host` | `APP__SERVER__HOST` | Server bind address | `127.0.0.1` |
| `server.port` | `APP__SERVER__PORT` | Server port | `3000` |
| `database.url` | `APP__DATABASE__URL` | PostgreSQL connection URL | - |
| `database.max_connections` | `APP__DATABASE__MAX_CONNECTIONS` | Max pool connections | `10` |
| `database.min_connections` | `APP__DATABASE__MIN_CONNECTIONS` | Min pool connections | `2` |
| `jwt.secret` | `APP__JWT__SECRET` | JWT signing secret | - |
| `jwt.expires_in_secs` | `APP__JWT__EXPIRES_IN_SECS` | Token expiration (seconds) | `3600` |

## Tech Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | 1.75+ | Language |
| Tokio | 1.x | Async runtime |
| Axum | 0.7.x | Web framework |
| PostgreSQL | 15+ | Database |
| SQLx | 0.8.x | Database driver |
| Serde | 1.x | Serialization |
| Validator | 0.18.x | DTO validation |
| Tracing | 0.1.x | Structured logging |
| jsonwebtoken | 9.x | JWT authentication |

## Business Rules

1. **Chain ID Uniqueness**: Each network must have a unique `chainId`
2. **Soft Delete**: DELETE operations set `active=false` instead of removing records
3. **Active Networks Only**: GET `/networks` returns only networks where `active=true`
4. **PUT vs PATCH**: PUT cannot modify `active` field; PATCH can
5. **Ethereum Address Format**: Must match pattern `0x[a-fA-F0-9]{40}`
6. **URL Validation**: All URLs must include protocol (`http://` or `https://`)
7. **Multipliers**: `feeMultiplier` and `gasLimitMultiplier` must be >= 0
8. **Other RPC URLs**: Limited to 10 items maximum

## License

MIT
