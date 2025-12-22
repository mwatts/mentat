# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Mentat is a persistent, embedded knowledge base built in Rust. It implements a Datalog-based query system over SQLite, inspired by Datomic and DataScript. The project provides a flexible relational store with schema evolution, entity-attribute-value storage, and optional sync capabilities.

## Build Commands

```bash
# Build all crates
cargo build

# Run all tests
cargo test --all

# Run tests for a specific crate (use crate name, not folder)
cargo test -p mentat_query_algebrizer -- --nocapture

# Build/test specific crate
cargo build -p mentat_db
cargo test -p mentat_core

# Build the CLI
cargo build -p mentat_cli

# Check outdated dependencies across all crates
make outdated
```

## Features

The main crate supports these Cargo features:
- `bundled_sqlite3` (default): Bundles SQLite
- `sqlcipher`: Enables SQLite encryption
- `syncable` (default): Enables Tolstoy sync functionality

## Architecture

The project is a Cargo workspace with ~20 crates organized in layers:

### Core Data Types (bottom layer)
- `edn/` - EDN parser (Clojure-like data format) using rust-peg
- `core-traits/` - Fundamental types: `ValueType`, `TypedValue`, `Entid`, `Attribute`
- `core/` - Core utilities, `Schema`, `Keyword`, SQL-type mappings

### Query Processing Pipeline
- `query-algebrizer/` - Converts parsed Datalog queries + schema → `AlgebraicQuery` (the largest query component)
- `query-projector/` - Handles projection of query variables into output data structures
- `query-pull/` - Implements pull expressions for entity traversal
- `query-sql/` - Abstract SQL representation bridging Mentat types to SQL concepts
- `sql/` - Converts abstract SQL to actual SQL text and bindings

### Data Layer
- `db/` - Core storage logic: bootstrapping, transactions, attribute cache, schema management
- `transaction/` - Transaction handling, entity builder, query execution

### Sync (optional)
- `tolstoy/` - Proof-of-concept sync implementation with timeline support

### Public Interface
- `src/` (main crate) - Assembles components: `Store`, `Conn`, `InProgress`, vocabulary management
- `tools/cli/` - Command-line interface (`mentat_cli`)
- `ffi/` - Foreign function interface for non-Rust consumers

### Trait Crates
Each major component has a companion `-traits` crate (`db-traits`, `sql-traits`, `query-*-traits`, etc.) to break dependency cycles and improve compile times.

## Key Types

- `Store` - Main entry point wrapping database connection + metadata
- `Conn` - Database connection with schema cache
- `InProgress` - In-progress transaction handle
- `Schema` - In-memory representation of the database schema
- `TypedValue` - Mentat's value type (Long, Double, String, Keyword, Ref, etc.)

## Macros

The main crate exports convenience macros:
- `var!(?name)` - Create a query variable
- `kw!(:namespace/name)` - Create a keyword

## SQLite Requirements

Requires SQLite 3.8.0+ for partial indices. Uses FTS4 for full-text search. By default bundles SQLite; can link against system SQLite by disabling `bundled_sqlite3` feature.

## Issue Tracking with Beads

This project uses **bd** (beads) for issue tracking. Run `bd onboard` to get started.

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --status in_progress  # Claim work
bd close <id>         # Complete work
bd sync               # Sync with git
```

## Session Completion Workflow

When ending a work session, complete ALL steps:

1. File issues for remaining work with `bd create`
2. Run quality gates if code changed (tests, build)
3. Update issue status - close finished work
4. Push to remote:
   ```bash
   git pull --rebase
   bd sync
   git push
   ```
5. Verify `git status` shows "up to date with origin"

Work is NOT complete until `git push` succeeds.
