# Collaborative Editing Research Spike (F-30)

This document presents a comprehensive research spike comparing **Conflict-free Replicated Data Types (CRDTs)**, **Operational Transformation (OT)**, and **Simple WebRTC State Sync** for implementing concurrent multi-user collaborative editing in the ASCII Canvas Editor.

---

## 1. Executive Summary & Recommendation

| Synchronization Model | Eventual Consistency | Offline Support | Implementation Complexity | Network Overhead | Server Requirements | Recommendation |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **CRDT (Yjs / Automerge)** | **Excellent (Guaranteed)** | **Excellent (Built-in)** | **Medium (Using TS libraries)** | Low (Deltas only) | None to Low (Signaling only) | **Recommended (Hybrid TS/WASM)** |
| **Operational Transformation** | Good (Server-mediated) | Poor (Complex merging) | High ("OT Hell" edge-cases) | Extremely Low | High (Central Sequence Server) | **Defer (High infrastructure cost)** |
| **Simple WebRTC State Sync** | Poor (Divergence risk) | None | Low | High (Snapshot bloat) | None (Signaling only) | **Defer (Lacks consistency)** |

### Core Recommendation

We recommend a **TypeScript-Centric Hybrid CRDT approach utilizing Yjs**:
1. **TypeScript Orchestration**: The collaborative state engine runs in the TypeScript layer using the mature, optimized **Yjs** library. This keeps the compiled Rust WebAssembly binary small (~150KB size budget) and leverages standard JS-WebRTC / JS-WebSocket bindings.
2. **Sparse Coordinate Map**: The canvas state is represented in the CRDT as a shared `Y.Map<string, CellState>` mapping coordinate keys (e.g., `"x,y"`) to cell data (character, color, layer ID).
3. **WASM-Driven Local Rendering**: As remote updates arrive via Yjs, TypeScript applies them to the Rust/WASM `AsciiEditor` instance using targeted API calls. The editor's high-performance dirty-rect rendering path instantly repaints only the modified bounding boxes, maintaining 60 FPS.
4. **Signaling Server & Production Setup**: For serverless peer-to-peer editing, we utilize a signaling coordinator. For persistence-backed team collaboration, a standard Node-based WebSocket relay (e.g., `y-websocket`) is deployed.
5. **A Note on `y-webrtc`**: The popular `y-webrtc` package is legacy and has not been actively maintained. In production, we recommend using a custom robust WebRTC signaling coordinator (e.g., built on standard WebSockets) or leveraging standard WebSocket connections (`y-websocket`) to a centralized node/room relay.

---

## 2. Theoretical Comparison of Sync Paradigms

### A. Conflict-free Replicated Data Types (CRDTs)
CRDTs are data structures that can be independently updated on multiple nodes without central coordination. If nodes receive the same set of updates (even out of order), they are mathematically guaranteed to converge to the exact same state.

*   **How it applies to ASCII Canvas**:
    *   The canvas is represented as a sparse 2D map: `Map<CoordinateString, Cell>`.
    *   Adding or drawing characters performs an update on specific coordinate keys.
    *   Moving a selection deletes from a set of coordinate keys and writes to a new set.
    *   Layers can be represented as a `Y.Array` of layer metadata containing nested `Y.Map` layers.
*   **Conflict Resolution**: Last-Write-Wins (LWW) Register per coordinate cell using unique client IDs and lamport timestamps.
*   **Eventual Consistency**: Guaranteed mathematically. No central sequencer required.
*   **Offline Experience**: Exceptional. A user can make offline edits; when they reconnect, Yjs automatically merges the historical changes using state vectors, preserving as much user intent as possible without breaking the document structure.

### B. Operational Transformation (OT)
OT is the classic technology behind Google Docs. It relies on transmitting mutation operations (e.g., `insert(index, char)`, `delete(index)`) and transforming the parameters of those operations when concurrent edits are detected, ensuring consistent state across all clients.

*   **How it applies to ASCII Canvas**:
    *   The operations align with the editor's existing command pattern (e.g., `DrawCommand`, `EraseCommand`).
    *   When User A draws a line concurrently with User B moving a selection, the server must sequence the events.
    *   If User A's line was drawn on the old selection coordinates, the server or client must transform User A's coordinates to align with the new selection offset.
*   **Conflict Resolution**: Handled by custom Transformation Functions ($T(O_1, O_2)$) that must satisfy mathematical proofs ($T(A, B) \cdot B = T(B, A) \cdot A$).
*   **Operational Complexity**: Extremely high. Writing correct transformation functions for 2D sparse layout transformations, layer deletion/merging, and multi-cell drawing tools is notorious for introducing edge-case "desynchronization loops" where clients diverge permanently.
*   **Infrastructure**: Strongly requires a central sequencer (e.g., ShareDB, Redis) to establish a definitive global order of operations. P2P serverless operation is practically impossible.

### C. Simple WebRTC State Sync
This is a peer-to-peer state replication approach where users transmit raw snapshots or lightweight delta frames directly to each other using WebRTC data channels, bypassing formal CRDT or OT protocols.

*   **How it applies to ASCII Canvas**:
    *   When a user draws, the changed cells are serialized to a binary packet and broadcasted directly to all connected peers via WebRTC.
    *   Upon receipt, peers overwrite their local grid at those coordinates.
*   **Conflict Resolution**: Basic timestamp-based Last-Write-Wins (LWW) or cursor-based ownership locks.
*   **Eventual Consistency**: Extremely weak. Network latency, out-of-order packets, and system clock drift make state divergence highly likely. For example, if User A and User B type different characters on the exact same cell simultaneously, different peers may receive the packets in different orders, leaving peers in permanently mismatched states.
*   **Scale Limits**: Restricted to full-mesh topologies (usually < 8 peers). If a peer drops out or reconnects, syncing the full history/document state requires complex and fragile peer negotiation.

---

## 3. Recommended Collaborative Architecture

The recommended architecture utilizes a **TypeScript-centric CRDT orchestrator** communicating with the **Rust/WASM editor core**.

```
                           +-------------------------------------+
                           |         Web Browser Clients         |
                           +-------------------------------------+
                                       |              |
                        (WebRTC Signaling Channel)    (y-websocket)
                                       |              |
                                       v              v
+------------------+       +-------------------------------------+
| Signaling Server | <---> |         Yjs CRDT Engine             |  (TypeScript layer)
|  (P2P Discovery) |       |  (Shared Map/Array, LWW-Registers)  |
+------------------+       +-------------------------------------+
                                       ^              |
                   (Local draw events) |              | (Apply remote deltas)
                                       |              v
                           +-------------------------------------+
                           |         WASM AsciiEditor            |  (Rust Core layer)
                           |  - High Performance Grid            |
                           |  - Dirty-Rect Render Queue          |
                           |  - Vector Font Engine               |
                           +-------------------------------------+
```

### State Mapping Design

To minimize memory overhead and keep synchronization snappy, we represent the canvas with three distinct Yjs shared types:

1.  **Canvas Metadata (`Y.Map`)**:
    *   `width`: integer (e.g., 80)
    *   `height`: integer (e.g., 40)
    *   `schemaVersion`: integer (currently 1)
2.  **Layer Hierarchy (`Y.Array<Y.Map>`)**:
    *   Each item in the array represents a layer.
    *   Properties: `id` (UUID), `name` (string), `visible` (boolean), `locked` (boolean).
    *   The actual cells for each layer are stored in a nested `Y.Map` under the key `cells`.
3.  **Cell Storage (`Y.Map<string, string>`)**:
    *   Key: `"x,y"` (e.g., `"24,15"`).
    *   Value: Single character representation or a short encoded string containing attributes (e.g., `"A|bold"`).
    *   *Rationale*: Storing cells as key-value entries in a flat `Y.Map` allows Yjs to synchronize cell-level edits with extreme granularity. If two users edit different parts of the canvas, their operations never conflict.

### Ephemeral State: Cursors & Carets

Mouse cursor coordinates, typing carets, and active selection boxes should **never** be persisted in the permanent CRDT state to avoid bloating the historical update logs.

*   Instead, we use **Yjs Awareness (y-protocols/awareness)**.
*   This protocol transmits ephemeral user states (name, color, cursor X, cursor Y, active tool) as lightweight peer-to-peer broadcasts over WebRTC/WebSockets.
*   Clients listen to awareness updates and render semi-transparent, colored remote cursors and selection bounding boxes as overlays on top of the ASCII canvas, mimicking Figma's multiplayer design.

---

## 4. Deep-Dive Integration Strategy with Rust/WASM

A major engineering consideration is how the existing WASM-based editor core (`AsciiEditor`) integrates with the TypeScript CRDT.

### The TS-to-WASM Mutation Loop

We avoid compiling the heavy CRDT engine into the WASM binary. Instead, we establish a clean unidirectional communication loop:

```
[Local User Interaction]
  1. User draws a shape (e.g., dragging Rectangle Tool).
  2. Rust `AsciiEditor` generates visual preview ops locally.
  3. On Mouse Up / Key Down: Rust commits the shape to its local Grid.
  4. Rust triggers an event callback `onGridModified(changed_cells: Vec<CellUpdate>)`.
  5. TypeScript intercepts the callback and writes the updates to the local Yjs Map:
     `yCells.set("${cell.x},${cell.y}", cell.char)`.

[Remote Update Propagation]
  1. Remote peer receives the Yjs sync update.
  2. Yjs triggers an event listener `yCells.observe(event => { ... })`.
  3. TypeScript extracts the changed coordinate keys and characters.
  4. TypeScript updates the local WASM Grid directly:
     `editor.applyRemoteMutations(updates: Array<{x, y, char}>)`.
  5. Rust core updates its internal state, tracks the modified bounding boxes via its
     `dirty_tracker`, and triggers a high-speed redraw on the HTML Canvas.
```

### Implementing Selective Undo/Redo

The existing editor has a robust Ring Buffer History (`History`) built with the Command Pattern. In a collaborative environment, a standard global undo is destructive: if User A presses `Ctrl+Z`, it must not undo User B's latest independent edits.

*   **Solution**: We delegate collaborative Undo/Redo to **Yjs's `Y.UndoManager`**.
*   The `Y.UndoManager` automatically tracks only local modifications made by the active client ID, ignoring concurrent remote mutations.
*   When a user invokes Undo, Yjs rolls back the local CRDT state, which in turn triggers the "Remote Update Propagation" path, updating the WASM grid state and rendering the canvas perfectly in sync.
*   We bypass or disable the internal Rust `AsciiEditor` undo history stack when collaborative mode is active, letting Yjs act as the definitive history controller.

---

## 5. Performance, Latency, and Network Overhead Analysis

*Note: The performance, snapshot sizing, and latency figures below represent estimated back-of-the-napkin calculations based on typical ASCII layout dimensions, derived from general Yjs performance evaluations.*

### A. Network Packet Size Estimates
ASCII Canvas is inherently lightweight compared to high-resolution bitmap editors.

*   **Full Grid Snapshot (80 × 40 = 3200 cells)**:
    *   Raw cells (assuming ~50% filled): ~1.6KB characters.
    *   Yjs encoded document update snapshot (estimated): **~5KB to 8KB** (highly compact).
    *   Initial loading synchronization over WebRTC takes **< 15ms** (estimated) on standard broadband.
*   **Incremental Deltas (Drawing a line or shape)**:
    *   Drawing a 10-character line changes 10 coordinates.
    *   Yjs binary update packet size (estimated): **~150 to 250 bytes**.
    *   Transmitted instantly in a single UDP/WebRTC frame, ensuring real-time responsiveness (< 30ms estimated latency peer-to-peer).

For official large-scale performance benchmarks and internal optimization reports, refer to the [Yjs Benchmarks and Performance Documentation](https://github.com/dopt/cr-benchmarks).

### B. Frame-Rate and CPU Overhead
*   **WASM Optimization**: The project uses **dirty-rect pixel buffer rendering** (ADR-028). When a collaborative update alters 5 cells, only those 5 cell areas are cleared and redrawn on the HTML Canvas. This keeps CPU usage close to 0% and guarantees a smooth 60 FPS even with 10+ users drawing simultaneously.
*   **Debouncing Outbound Previews**: During active mouse drags (e.g., drawing a large rectangle), we render the "preview shape" strictly on the local canvas. We only push the final committed shape to the Yjs CRDT on mouse release. This prevents flooding the peer-to-peer network with intermediate, discarded preview states.

---

## 6. Actionable Implementation Path (Future Phase)

If collaborative editing is prioritized for development in a future release, the following modular steps should be taken:

### Step 1: Expose WASM Boundary APIs for Remote Mutations
Enhance `src/wasm/helpers.rs` to expose high-performance batch mutations. Below is a conceptual, syntactically correct Rust/WASM implementation for applying batch updates to the canvas grid without triggering recursive history records:

```rust
use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct RemoteCellUpdate {
    pub x: i32,
    pub y: i32,
    pub char_str: String,
}

#[wasm_bindgen]
impl AsciiEditor {
    /// Applies batch remote mutations directly to the active layer of the grid.
    /// Remote mutations are written in place and marked as dirty, bypassing the local command undo history.
    #[wasm_bindgen(js_name = applyRemoteMutations)]
    pub fn apply_remote_mutations(&mut self, serialized_updates: &JsValue) -> Result<(), JsValue> {
        let updates: Vec<RemoteCellUpdate> = serde_wasm_bindgen::from_value(serialized_updates.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to deserialize updates: {:?}", e)))?;

        for update in updates {
            let ch = update.char_str.chars().next().unwrap_or(' ');

            // Assuming self.grid resides in the active layer of the model
            if self.grid.is_within_bounds(update.x, update.y) {
                // Mutate grid cell directly
                self.grid.set_char(update.x, update.y, ch);

                // Mark cell as dirty in the dirty rect tracker to trigger incremental repainting
                self.dirty_tracker.mark_dirty(update.x, update.y);
            }
        }

        // Request a repaint for the dirty bounding boxes
        self.request_render();
        Ok(())
    }
}
```

### Step 2: Integrate Yjs Core & Providers
Install the lightweight Yjs packages in `web/package.json`:
```json
"dependencies": {
  "yjs": "^13.6.0",
  "y-websocket": "^2.0.0"
}
```

### Step 3: Implement TS Orchestrator Layer (`web/collab.ts`)
Create a new frontend feature module `web/collab.ts` to manage:
*   Y.Doc initialization and room joining.
*   Event observation (`yMap.observe`) mapping to WASM updates.
*   Yjs Awareness synchronization to render remote user cursors (`.caret` style indicators).

### Step 4: Add Multiplayer UI Elements
Add lightweight collaborative UI:
*   A "Share Board" button to generate unique Room hashes.
*   Peer presence indicators in the top toolbar (colored avatars representing active collaborators).
