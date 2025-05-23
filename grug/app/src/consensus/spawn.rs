pub async fn spawn_node_actor() -> (NodeRef, JoinHandle<()>) {
    // After Malachite: code/crates/starknet/host/src/spawn.rs

    // Initialize a new `GrugContext`

    // Actors wiring:
    // Spawn mempool actor
    // Spawn consensus gossip actor
    // Spawn the host actor
    // Spwan the WAL actor
    // Spawn the consensus actor
    // Spawn the top-level node actor
}
