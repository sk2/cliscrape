# GSD Backlog: The Network Compiler

This backlog tracks the remaining roadmap-shaped work that is not yet complete.

## Active Backlog

### Phase 17: Universal Ledger Library
- Status: Planned
- Milestone: v2.0 The Network Compiler
- Goal: Deliver a plug-and-play experience where disparate vendors emit identical data structures.
- Depends on: Phase 13
- Requirements: `LEDGER-01`, `LOG-02`

#### Backlog Items
- [ ] Create `common_schemas/` and define standard schemas for `interface`, `bgp_neighbor`, `lldp_neighbor`, `version`, and `route`.
- [ ] Update embedded YAML templates (`cisco_ios_*`, `arista_eos_*`, and peers) with `common_schema` mappings.
- [ ] Add `tests/schema_compliance.rs` to validate fixture output against the declared common schema.

## Future Backlog

### Phase 18: Semantic Mock Server
- Status: Planned
- Milestone: v3.0 Isomorphic Ecosystem
- Goal: Simulate device behavior without physical hardware or VMs.
- Depends on: Phase 15
- Requirements: `MOCK-01`, `MOCK-02`

#### Backlog Items
- [ ] Implement `cliscrape simulate` as an interactive shell for device emulation.
- [ ] Accept a device state file and synthesize command responses via isomorphic generation.
- [ ] Preserve vendor-authentic output formatting so simulated responses match real devices closely.

### Phase 19: State-of-the-World Manifests
- Status: Planned
- Milestone: v3.0 Isomorphic Ecosystem
- Goal: Define the source-of-truth format for multi-device simulation state.
- Depends on: Phase 18

#### Backlog Items
- [ ] Design a JSON/YAML manifest mapping device hostnames to commands and JSON state records.
- [ ] Define how simulated devices select command variants and fixture state from the manifest.
- [ ] Add example manifests for single-device and multi-device test environments.

### Phase 20: SSH/CLI Protocol Integration
- Status: Planned
- Milestone: v3.0 Isomorphic Ecosystem
- Goal: Expose the simulator over real protocols for end-to-end tooling integration.
- Depends on: Phase 18, Phase 19

#### Backlog Items
- [ ] Integrate a lightweight SSH server library such as `russh`.
- [ ] Run `cliscrape simulate` as a persistent SSH daemon.
- [ ] Validate that external tools can connect to the mock server as if it were a physical device.
