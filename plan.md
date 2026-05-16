# Fallout 4 Linux Mod Manager (Rust + Kirigami) - Detailed Implementation Plan

## 1. Problem Statement and Product Goal

Build a Linux-native mod manager for **Fallout 4** inspired by Mod Organizer 2, with:

- Reliable profile isolation
- Deterministic deploy/undeploy
- Conflict visibility and resolution
- Plugin management with LOOT sorting
- Proton/Wine runner management (detect + install)
- F4SE detection and validation

This v1 should prioritize correctness and transparency over feature breadth.

---

## 2. Confirmed Scope (v1)

### In scope

1. Fallout 4 support
2. Linux-only
3. Runner support:
   - Proton (all discovered runners)
   - WineHQ (detected/managed source)
4. Runner selection pinned **per profile**
5. Runner detection + install of **stable** releases only
6. Verification of downloaded runners (checksum/signature)
7. Steam game detection + manual path support (including GOG/third-party installs)
8. Archive mod install + FOMOD installer support
9. Global mod store + per-profile manifests
10. Full profile isolation:
    - `plugins.txt`
    - `loadorder.txt`
    - INI overrides/config files
    - enabled mod set
11. Deploy model:
    - symlink-first
    - explicit Deploy button
    - optional auto-deploy setting
12. Conflict handling:
    - MO2-style priority (higher priority wins)
    - loose vs BA2 and BA2 vs BA2 semantics
    - overwrite/unmanaged output bucket
13. Plugin validation:
    - missing masters and dependency checks
14. LOOT sort:
    - manual "Sort with LOOT" action
    - in-process Rust crate preferred
    - fallback to LOOT CLI
15. F4SE:
    - detection
    - version compatibility validation
16. UI:
    - Kirigami (KDE native)
    - Rust backend service via local IPC

### Deferred (post-v1)

1. True VFS/FUSE/overlay virtualization
2. Nexus download/API integration
3. Full external tool pipeline (xEdit/DynDOLOD/Nemesis automation)
4. Packaging focus (Flatpak/AppImage), though binary release can be done later

---

## 3. Architecture and Repository Layout

## 3.1 High-level architecture

Use a split architecture:

- **Frontend:** Kirigami/QML desktop app
- **Backend:** Rust daemon/service owning all state and filesystem operations
- **Transport:** local IPC (Unix domain socket + JSON-RPC recommended)

This enables future CLI/headless usage, cleaner error isolation, and easier scaling.

## 3.2 Suggested workspace structure

```text
mod-manager/
  Cargo.toml
  crates/
    domain-core/
    storage-sqlite/
    game-detect/
    runner-manager/
    mod-ingest/
    deploy-engine/
    plugins-engine/
    loot-engine/
    launch-engine/
    ipc-api/
    mm-daemon/
  ui/
    kirigami-app/
  docs/ (optional non-planning docs only)
```

## 3.3 Crate responsibilities

1. `domain-core`:
   - domain entities, validation rules, conflict semantics, profile invariants
2. `storage-sqlite`:
   - migrations, queries, transactional boundaries
3. `game-detect`:
   - FO4 path discovery (Steam + manual source registration)
4. `runner-manager`:
   - detect installed runners, install stable builds, verify downloads
5. `mod-ingest`:
   - archive extraction, metadata probing, FOMOD parsing/execution
6. `deploy-engine`:
   - staging plan generation, symlink graph, rollback manifests
7. `plugins-engine`:
   - parse plugin metadata, masters/dependency checks, load order persistence
8. `loot-engine`:
   - LOOT sort integration, fallback strategy, diagnostic messages
9. `launch-engine`:
   - assemble launch environment, runner invocation, F4SE validation
10. `ipc-api`:
   - request/response models, versioning, error codes
11. `mm-daemon`:
   - orchestrator wiring all crates and exposing APIs

---

## 4. Domain Model and Data Schema Draft

Design schema first; everything else depends on it.

## 4.1 Core entities

1. `games`
   - `id`, `name`, `canonical_id` (`fallout4`)
2. `instances`
   - logical install registration (path, source type: steam/manual/gog/third_party)
3. `profiles`
   - name, instance_id, pinned_runner_id, deploy settings, timestamps
4. `mods`
   - metadata, archive source, install state, global location
5. `profile_mods`
   - enabled flag, priority, per-profile state
6. `plugins`
   - plugin filename/type (esm/esp/esl), masters metadata cache
7. `profile_plugins`
   - enabled/order/index state per profile
8. `deploy_manifests`
   - immutable record of symlink plan and outputs
9. `file_index`
   - indexed mod files used for conflict calculation
10. `runner_catalog`
   - known runner records (proton/winehq, version, source, install path, verified)
11. `f4se_state`
   - detected version, compatibility result, last validation details
12. `settings`
   - global config including auto-deploy default and paths

## 4.2 Recommended invariants

1. Only one active deployment per profile at a time.
2. Deploy is atomic from user perspective (all applied or rolled back).
3. Profile isolation is absolute:
   - profile-specific files never shared by writable reference.
4. Runner assignment is explicit (no hidden fallback).
5. Failed validation blocks launch/deploy when correctness is at risk.

---

## 5. Behavior Rules (Must be Implemented Exactly)

## 5.1 Conflict rules

1. Mod priority determines winner for identical loose-file paths.
2. Higher priority mod wins.
3. Loose files override BA2 resources.
4. BA2 vs BA2 conflict resolution follows plugin/load order semantics.
5. Hidden/masked files are excluded from deployment.

## 5.2 Overwrite behavior

1. Any unmanaged/generated file lands in an overwrite bucket.
2. Overwrite bucket is visible in UI.
3. Users can convert overwrite content into a named mod.

## 5.3 Deploy rules

1. Deploy plan is previewable.
2. Apply symlink-first strategy.
3. If unsupported, fail with explicit guidance (or controlled fallback path if enabled).
4. Every deploy writes a manifest for rollback and diagnostics.

---

## 6. IPC/API Contract Plan

Define API upfront with versioning.

## 6.1 API categories

1. Game/instance APIs:
   - detect, register manual path, validate path
2. Runner APIs:
   - list detected, install stable, verify installed, set profile runner
3. Profile APIs:
   - create/switch/clone/delete profile
4. Mod APIs:
   - install archive/FOMOD, enable/disable, reorder priority
5. Plugin APIs:
   - list, enable/disable, reorder, validate masters, sort via LOOT
6. Deploy APIs:
   - preview, deploy, undeploy, rollback
7. Launch APIs:
   - preflight validation, launch FO4/F4SE, collect diagnostics
8. Settings APIs:
   - auto-deploy toggle and global paths

## 6.2 Error model

Use typed errors with actionable messages:

- `ValidationError`
- `DependencyError`
- `FilesystemCapabilityError`
- `RunnerVerificationError`
- `LootIntegrationError`
- `LaunchContextError`

No silent failures.

---

## 7. Detailed Milestones with Guiding Steps

## M0 - Bootstrap and technical baseline

### Deliverables

- Rust workspace and crate scaffolding
- Kirigami app shell with IPC client stub
- Structured logging and config loading

### Steps

1. Initialize workspace and crate boundaries.
2. Define coding conventions and error handling strategy.
3. Set up migration framework.
4. Implement daemon lifecycle and Unix socket bootstrap.
5. Build minimal UI navigation shell and connect health-check API.

### Exit criteria

- UI can query daemon health/version.
- Migrations run successfully.

---

## M1 - Database and domain invariants

### Deliverables

- Full initial schema + migrations
- Domain validation layer
- Transaction helpers

### Steps

1. Implement schema for entities listed in section 4.
2. Add constraints and indexes for profile/mod/plugin joins.
3. Add repository/query abstractions.
4. Write invariant checks (runner/profile consistency, unique priorities, etc.).

### Exit criteria

- CRUD for games/instances/profiles/mod state works end to end.

---

## M2 - Game install and runner management

### Deliverables

- FO4 auto-detection (Steam)
- Manual game path registration (GOG/third-party)
- Proton/Wine detection + stable installer + verification

### Steps

1. Implement Steam library scanning.
2. Implement manual registration flow with path validation.
3. Enumerate Proton runners from known install roots.
4. Enumerate WineHQ runners.
5. Add runner install pipeline:
   - fetch metadata
   - download artifact
   - checksum/signature verification
   - install into managed directory
6. Expose runner assignment per profile.

### Exit criteria

- User can detect/install runner and pin it to a profile.

---

## M3 - Profile isolation

### Deliverables

- Profile creation/switching with full isolated state
- Isolated INI/plugin/loadorder files

### Steps

1. Define profile directory layout.
2. Implement profile file materialization.
3. Ensure all writes route through active profile context.
4. Add integrity checks to prevent cross-profile writes.

### Exit criteria

- Switching profiles changes runtime state without leakage.

---

## M4 - Mod ingestion (archive + FOMOD)

### Deliverables

- Archive install flow
- FOMOD parser/execution engine
- Global mod store with per-profile enablement

### Steps

1. Implement archive detection/extraction.
2. Normalize metadata (name/version/source/hash).
3. Parse FOMOD directives and options.
4. Execute selected FOMOD install path.
5. Index installed files in `file_index`.

### Exit criteria

- Complex FOMOD mod installs and appears in mod list with files indexed.

---

## M5 - Conflict engine + deploy system

### Deliverables

- Conflict graph and winning-file computation
- Deploy preview and apply
- Rollback/undeploy
- Overwrite bucket support

### Steps

1. Build deterministic resolver using priority + archive semantics.
2. Generate deploy plan from winner set.
3. Create symlink operations with transactional staging.
4. Write deploy manifest and rollback artifacts.
5. Implement overwrite detection and UI exposure.

### Exit criteria

- Deploy/undeploy is repeatable and reversible.

---

## M6 - Plugin engine + LOOT sorting

### Deliverables

- Plugin list management
- Missing masters validation
- Manual LOOT sort action

### Steps

1. Parse plugin metadata and dependency graph.
2. Implement enable/disable and order persistence.
3. Integrate Rust LOOT crate.
4. Add fallback to LOOT CLI if crate path fails.
5. Provide sort report and warnings.

### Exit criteria

- User can click "Sort with LOOT" and persist sorted order.

---

## M7 - Launch flow + F4SE validation

### Deliverables

- Preflight checks for game/runner/F4SE compatibility
- FO4/F4SE launch actions through selected runner

### Steps

1. Detect F4SE presence/version.
2. Validate against FO4 executable version.
3. Build launch command and environment from profile.
4. Emit clear actionable diagnostics on failures.

### Exit criteria

- Launch succeeds when preflight passes; blocked with clear reason when not.

---

## M8 - Kirigami UX completion

### Deliverables

- End-to-end UI flows for all v1 features
- Deploy settings (explicit + optional auto-deploy)
- Conflict and diagnostics views

### Steps

1. Implement Library/Instances/Profiles pages.
2. Implement Mods + priority reorder UI.
3. Implement Conflicts view with winner details.
4. Implement Plugins view and LOOT sort action.
5. Implement Runner manager UI and install progress.
6. Implement Deploy + Launch pages with preflight display.

### Exit criteria

- No critical operations require CLI/manual DB edits.

---

## M9 - Hardening and release candidate

### Deliverables

- Stability fixes, migration safety, final QA matrix
- Binary release path documented for later execution

### Steps

1. Add migration downgrade/upgrade tests.
2. Stress-test large mod lists and conflict resolution.
3. Validate unsupported FS cases and user messaging.
4. Tighten logs and crash diagnostics.

### Exit criteria

- End-to-end usage is stable for realistic FO4 modding workflows.

---

## 8. UI Plan (Kirigami)

## 8.1 Core pages

1. Dashboard
2. Game Instances
3. Profiles
4. Mods (priority list)
5. Conflicts
6. Plugins (sort/validate)
7. Runners
8. Deploy/Launch
9. Logs/Diagnostics
10. Settings

## 8.2 UX principles

1. Always show current profile and pinned runner.
2. Show dirty state when deploy is needed.
3. Preflight warnings must be explicit and actionable.
4. Destructive actions require clear confirmation.

---

## 9. Security, Integrity, and Reliability

1. Verify all downloaded runner artifacts before install.
2. Use atomic file operations for deploy changes where possible.
3. Lock profile operations to avoid concurrent corruption.
4. Store audit trail for runner installs and deploy operations.
5. Expose meaningful error messages with remediation hints.

---

## 10. Filesystem Compatibility Strategy (Linux)

1. Run capability detection at startup/first setup:
   - symlink support
   - write permission
   - cross-device boundary checks
2. Use symlink-first deployment as default.
3. If capability checks fail, provide controlled fallback strategy (config-gated), never silent fallback.
4. Record detected capabilities in diagnostics.

---

## 11. Testing Strategy

## 11.1 Unit tests

- Conflict resolver
- Plugin dependency graph checks
- FOMOD parser behaviors
- Runner metadata verification logic

## 11.2 Integration tests

- Full install -> enable -> deploy -> launch-preflight flow
- Profile switching with isolated files
- LOOT sort apply and persistence
- Rollback after simulated deploy failure

## 11.3 System tests

- Steam-detected FO4 install path
- Manual GOG/third-party path
- Proton and WineHQ runner combinations
- F4SE compatible/incompatible scenarios

## 11.4 Regression tests

- Known edge cases from real modpacks
- Large file count and conflict-heavy load orders

---

## 12. Implementation Order (Practical Execution)

Follow this strict order to reduce rework:

1. Schema + invariants
2. Instance/profile lifecycle
3. Runner detect/install/verify
4. Mod ingest + file indexing
5. Conflict/deploy engine
6. Plugin engine + LOOT
7. F4SE validation + launch
8. UI completeness and diagnostics
9. Hardening/performance

---

## 13. Risks and Mitigations

1. **LOOT crate compatibility risk**
   - Mitigation: keep CLI fallback adapter from day one.
2. **FOMOD complexity variance**
   - Mitigation: implement robust parser with explicit unsupported-rule reporting.
3. **Filesystem edge cases**
   - Mitigation: preflight capability checks and explicit failure messages.
4. **Runner ecosystem drift**
   - Mitigation: versioned metadata and strict verification pipeline.
5. **Profile corruption risk**
   - Mitigation: transactional writes and immutable deploy manifests.

---

## 14. Definition of Done (v1)

v1 is done when all are true:

1. User can detect/register FO4 install and create profile.
2. User can detect/install and pin Proton/Wine runner per profile.
3. User can install archive/FOMOD mods and manage priorities.
4. User can inspect conflicts and deploy deterministically.
5. User can manage plugins, validate masters, and manually LOOT-sort.
6. User can detect/validate F4SE and launch through selected runner.
7. Profile isolation is complete and verified.
8. Errors are actionable; no silent failures in core flows.

---

## 15. Immediate Next Build Tasks

1. Scaffold workspace/crates and daemon IPC contract.
2. Finalize SQLite migration v1 and repository interfaces.
3. Build game/runner detection prototype.
4. Build profile isolation file layout.
5. Stand up Kirigami shell connected to daemon health endpoint.

