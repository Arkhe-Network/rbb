1. **Create the Python Cognitive Substrate (Substrato 1600) implementation**:
   - Save the complete Python code provided in the user request into `cathedral_v14/cognitive_substrate.py` (which includes ViT feature extractor, OWL+Z3 Ontology, Gym environment for the loop, PPO/SAC placeholder/buffer, etc. or a variant as requested in the final AGI Core integrated version).
   - *Wait, the user wants me to INTEGRATE the substrate into the v14 orchestrator*.
   - The user has provided specific requirements:
     1. **Ingestão de Dados:** In `process_cognitive_tick`, the output of GGUF and the REAL embedding from GGUF must be injected in step 2.
     2. **Plano de Dados (Rust):** Python acts as a state machine. `EpisodicMemory` and `EnergyBudgetController` indices and DVFS are delegated to a Rust process via ZeroMQ or gRPC.
     3. **Introspecção de Segurança:** `IntrospectiveMonitor` reads `/proc/self/status` via Rust and executes a Godelian Self-Check.
     4. **Meta-Learning Seguro:** MAML updates GGUF embeddings with low learning rate (`1e-7`) and saves them in `gguf_py` compatible format.

2. **Modify the sidecar `server.py` and `engine.py`**:
   - `engine.py`: Make sure `generate` or a new wrapper method returns the embedding of the generated text along with the text. We have an `embed` function already. We can fetch it.
   - `server.py`: Integrate `process_cognitive_tick` by feeding the `gguf_output_text` and `gguf_embedding`.

3. **Develop the Rust Data Plane Component**:
   - Since we must implement this, I will create a Rust crate `rust_data_plane` inside `cathedral_v14/` using `zmq` or `tonic` (gRPC). ZeroMQ is simpler and less boilerplate.
   - Rust side will handle Episodic Memory (HNSW / FAISS alternative in Rust or just linear for now as stub if HNSW Rust crate is too complex to build, wait, the prompt says `hnswlib` is Python. Wait, the prompt says "delegada a um processo Rust separado"). I will use a simple in-memory linear scan or `hnsw.rs` for EpisodicMemory and a struct for EnergyBudget.
   - The Python code needs to call ZeroMQ via `pyzmq`. We'll need to install `pyzmq` in Python. We need to check if we can `pip install pyzmq`. We can add `pyzmq` to the requirements.

4. **Verify changes**:
   - Ensure Rust builds `cargo build`.
   - Ensure the modified Python sidecar `server.py` runs with a mocked ZMQ or tests.
   - Pre-commit steps.
