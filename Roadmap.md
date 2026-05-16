# Project Roadmap: Linux Native Mod Manager

This roadmap outlines the path from the current working MVP to a fully featured, Mod Organizer 2 (MO2) equivalent mod manager built natively for Linux using Rust and Kirigami/QML.

## 🏁 Milestone 1: UI Foundation & Basic Workflows
*Focus: Upgrading the QML UI to be highly interactive and matching the MO2 top-bar layout.*

- [ ] **Top Toolbar Overhaul:** 
  - Add a global "Install Mod" button.
  - Add a visual Profile selector.
  - Add a "Shortcuts" dropdown to configure and launch auxiliary tools (xEdit, WryeBash, Bodyslide, etc.).
- [ ] **Runner & Launch Options:**
  - Enhance the Runner dropdown to include a "Settings" button.
  - Allow defining custom environment variables per profile/runner (e.g., `WINEDLLOVERRIDES`, `PROTON_LOG=1`).
- [ ] **Drag-and-Drop Installation:** 
  - Implement a QML `DropArea` so users can drag `.zip` and `.7z` files directly from their file manager into the app to trigger extraction and ingestion.

## 📦 Milestone 2: Archive & Plugin Expansions
*Focus: Expanding format support and properly handling base game files.*

- [ ] **RAR Archive Support:** 
  - Integrate a Rust crate (e.g., `unrar` or `compress-tools`) into `mod-ingest/archive.rs` to support extracting `.rar` mod files.
- [ ] **Base Game Plugins Visibility:** 
  - Auto-detect `Fallout4.esm` and official DLCs.
  - Pin them to the top of the Plugin Load Order (indexes 0, 1, 2...).
  - Prevent the user from unchecking, deleting, or sorting base game files.

## 🗂️ Milestone 3: Virtual File System (VFS) Visibility
*Focus: Letting the user see exactly what the symlink deployer is doing.*

- [ ] **VFS Backend API:** 
  - Create an IPC endpoint that reads the current `deploy_manifest` and constructs a hierarchical JSON tree of deployed files.
- [ ] **Data Tab (Tree View):** 
  - Implement a `TreeView` in the right pane's "Data" tab.
  - Display the VFS tree, showing which mod provides which specific file (e.g., `Textures/weapons/gun.dds -> [Weapon Mod A]`).

## ⚡ Milestone 4: Advanced List Management (MO2 Parity)
*Focus: Visual reordering, conflict resolution, and organization.*

- [ ] **Drag-and-Drop Priority (Left Pane):** 
  - Allow users to drag rows to change mod priority dynamically.
- [ ] **Drag-and-Drop Load Order (Right Pane):** 
  - Allow users to manually drag plugins to override LOOT sorting.
- [ ] **Conflict Visualization:** 
  - Add MO2's lightning bolt icons (`+` overwrites, `-` overwritten, `+/-` mixed) to the mod list.
  - Populate the bottom-right conflict detail pane when a mod is clicked.
- [ ] **Categories & Filters:** 
  - Add a database column for categories.
  - Add a bottom search bar to filter the mod list by name or category.
- [ ] **Separators:** 
  - Allow users to right-click and create "Dummy Mods" to act as visual separators in the priority list.

## 🛠️ Milestone 5: Tools & Integration
*Focus: Built-in utilities for serious modders.*

- [ ] **INI Editor:** 
  - A dedicated tab/window to safely edit the profile-specific `fallout4.ini`, `fallout4prefs.ini`, and `fallout4custom.ini`.
- [ ] **Save Game Manager:** 
  - A tab next to Plugins/Data to view local save files.
  - Cross-reference save files with the active load order and flag saves that are missing required plugins.
- [ ] **Downloads Tab:** 
  - Manage a designated downloads directory.
  - Double-click to install downloaded archives.
- [ ] **Nexus Integration (Future):** 
  - Store Nexus Mod IDs and versions.
  - Handle `nxm://` links from the browser.
  - Display "Update Available" warnings.

## 🐧 Milestone 6: Linux Specifics & Polish
*Focus: Stability, feedback, and debugging on Proton/Wine.*

- [ ] **Proton/Wine Console Log Window:** 
  - Add a dockable window or tab that captures and streams the `stdout/stderr` of the launched game process in real-time for debugging F4SE/ENB crashes.
- [ ] **Global Progress / Status Bar:** 
  - Add a bottom status bar with a loading spinner/progress bar so the UI doesn't look frozen during heavy I/O tasks (hashing, extracting).
- [ ] **Onboarding & Instance Management:** 
  - Replace the hardcoded `instance_id` logic. 
  - Show a welcome screen for first-time users to detect or manually locate their game installation.