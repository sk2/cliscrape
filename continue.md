# Continuation Plan: The Network Compiler

This document outlines the remaining high-level tasks to fully realize the Network Compiler vision for `cliscrape`.

## Phase 17: The Universal Ledger Library
**Goal**: A "Plug-and-Play" experience where disparate vendors emit identical data structures.

- [ ] **Task 17.1: Define Standard Schemas**
  - Create a `common_schemas/` directory.
  - Define JSON Schema files for the top 5 network operational states:
    - `interface.schema.json`
    - `bgp_neighbor.schema.json`
    - `lldp_neighbor.schema.json`
    - `version.schema.json`
    - `route.schema.json`

- [ ] **Task 17.2: Update Core Templates**
  - Update all relevant embedded YAML templates (`cisco_ios_*`, `arista_eos_*`, etc.) with the appropriate `common_schema` mappings to conform to the new Universal Ledger.

- [ ] **Task 17.3: Implement Schema Compliance Validation**
  - Create a new test suite (`tests/schema_compliance.rs`).
  - This test will parse all embedded templates, check if they claim a `common_schema`, and if so, validate that their output for fixture data conforms to the corresponding JSON Schema. This ensures that no template can falsely claim to be part of the Universal Ledger.

## Phase 18: The Semantic Mock Server
**Goal**: Simulate device behavior without physical hardware or VMs.

- [ ] **Task 18.1: Design "State-of-the-World" Manifest**
  - Define a JSON/YAML format that maps a device hostname to a list of commands and their corresponding JSON state records. This file will be the "source of truth" for the mock server.

- [ ] **Task 18.2: Implement `cliscrape simulate`**
  - Create a new CLI command that takes a state manifest and a device name.
  - It will provide an interactive shell that mimics a real device login.
  - When a user types a command (e.g., `show version`), the simulator will find the matching state in the manifest, pass it to the isomorphic generator ($FSM^{-1}$), and print the synthesized CLI output to the user.

- [ ] **Task 18.3: (Future v3.0) SSH Protocol Integration**
  - Integrate a lightweight SSH server library (e.g., `russh`).
  - The `simulate` command will be upgraded to run as a persistent SSH daemon.
  - This allows `deviceinteraction` and other ecosystem tools to connect to the mock server as if it were a real physical device, enabling full end-to-end integration testing.
