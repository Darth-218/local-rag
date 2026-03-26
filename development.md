# Development Log

## Current State: Phase 4 Enhanced - Chat-Scoped Documents & UI Improvements

---

## Phase 1: Foundation & Scaffolding ✅

### Completed Steps

#### Project Initialization
- Initialized Git repository
- Created README.md with project overview, architecture, and roadmap
- Created Nix development environment (`flake.nix`, `nix/shell.nix`)
- Fixed GTK schema issue for file dialogs

#### Frontend Setup (Vite + React)
```bash
npm create vite@latest src -- --template react-ts
```

#### Tauri Backend Initialization
```bash
npx tauri init --app-name "local-rag" --window-title "local-rag" ...
```

#### Phase 1 Commands
```rust
#[tauri::command] pub fn get_app_info() -> AppInfo
#[tauri::command] pub fn get_app_data_dir_cmd() -> String
```

---

## Phase 2: Document Processing Pipeline ✅

### New Cargo Dependencies
```toml
lopdf = "0.34"
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

### Document Module (`document.rs`)
- PDF text extraction using lopdf
- Text/markdown file support
- Chunking with configurable size/overlap
- Document metadata extraction

### Phase 2 Commands
```rust
#[tauri::command] pub async fn pick_file() -> Option<String>
#[tauri::command] pub async fn process_document(filePath: String) -> ProcessingResult
#[tauri::command] pub async fn get_documents_metadata() -> Vec<DocumentMetadata>
#[tauri::command] pub async fn delete_document(documentId: String) -> ()
```

### Frontend UI
- Three-panel layout (chat history, chat area, documents)
- Minimalist white theme
- Add/delete document functionality

---

## Phase 3: Embedding & Vector Storage ✅

### New Cargo Dependencies
```toml
tokio = { version = "1", features = ["full"] }
reqwest = { version = "0.12", features = ["json"] }
futures = "0.3"
```

### Ollama Module (`ollama.rs`)
- HTTP client for Ollama API
- Model listing
- Embedding generation via `/api/embeddings`
- Response generation via `/api/generate`

### Embedding Module (`embedding.rs`)
- ChromaEntry data structure
- Cosine similarity calculation
- Similar document retrieval

### New Tauri Commands
```rust
#[tauri::command] pub async fn check_ollama_status() -> OllamaStatus
#[tauri::command] pub async fn embed_document(documentId: String, model: Option<String>) -> EmbeddingResult
#[tauri::command] pub async fn search_documents(query: String, topK: Option<usize>) -> SearchResult
#[tauri::command] pub async fn ask_question(query: String, model: Option<String>) -> String
```

### Data Flow
```
Document → Extract Text → Chunk → Embed (Ollama) → Store (chroma/index.json)
Query → Embed → Search (cosine similarity) → Retrieve chunks → Generate response
```

---

## Chat Functionality

### New Module (`chat.rs`)
- Chat data structures (Chat, Message, MessageRole)
- Chat CRUD operations
- Message storage

### New Tauri Commands
```rust
#[tauri::command] pub async fn get_chats() -> Vec<Chat>
#[tauri::command] pub async fn create_chat(title: Option<String>) -> Chat
#[tauri::command] pub async fn delete_chat(chatId: String) -> ()
#[tauri::command] pub async fn rename_chat(chatId: String, title: String) -> Chat
#[tauri::command] pub async fn get_chat_messages(chatId: String) -> Vec<Message>
#[tauri::command] pub async fn send_message(chatId: String, content: String) -> Message
```

### Frontend Features
- Create new chats
- Switch between chats
- Rename chats (double-click)
- Delete chats
- Send messages and receive AI responses
- Message display with user/assistant styling

### Data Storage
```
~/.local/share/com.localrag.app/
└── chats/
    └── data.json     # All chats and messages
```

---

## Phase 4 Enhancement: Chat-Scoped Documents & UI Improvements

### Changes Made

#### 1. Documents Per Chat
- `DocumentMetadata` now includes `chat_id` field
- `TextChunk` now includes `chat_id` field  
- Documents stored in `documents/{chat_id}/` directories
- Chroma index per chat: `chroma/{chat_id}.json`

#### 2. Updated Tauri Commands
```rust
#[tauri::command] pub async fn process_document(filePath: String, chatId: String) -> ProcessingResult
#[tauri::command] pub async fn get_chat_documents(chatId: String) -> Vec<DocumentMetadata>
#[tauri::command] pub async fn embed_document(documentId: String, chatId: String, model: Option<String>) -> EmbeddingResult
#[tauri::command] pub async fn search_documents(chatId: String, query: String, topK: Option<usize>) -> SearchResult
#[tauri::command] pub async fn ask_question(chatId: String, query: String, model: Option<String>) -> String
```

#### 3. Right-Click Context Menu
- Context menu on chat items with Rename and Delete options
- Properly positioned using fixed positioning

#### 4. Add Document Button
- Moved from sidebar footer to chat input area
- Positioned next to send button
- Uses paperclip emoji (📎) icon

---

## Build Status

| Check | Status |
|-------|--------|
| `cargo check` | ✅ Passing |
| `npm run build` | ✅ Passing |

---

## Project Structure

```
local-rag/
├── src/
│   ├── App.tsx           # Main UI component
│   ├── App.css           # App styles
│   ├── index.css         # Base styles
│   └── main.tsx         # Entry point
├── src-tauri/
│   ├── src/
│   │   ├── main.rs      # Entry point
│   │   ├── lib.rs       # App setup with plugins
│   │   ├── commands.rs  # Tauri commands
│   │   ├── document.rs  # Document processing
│   │   ├── embedding.rs # Vector storage
│   │   ├── ollama.rs    # Ollama API client
│   │   ├── chat.rs      # Chat management
│   │   └── config.rs    # Config module (stub)
│   ├── Cargo.toml
│   └── tauri.conf.json
├── nix/
│   └── shell.nix
├── flake.nix
├── package.json
└── README.md
```

---

## Next: Phase 4 - LLM Integration & Chat UI

- Full chat interface with message history
- Real-time streaming responses
- Source citations display
- Chat persistence

## Commands Reference

```bash
# Enter development environment
cd /path/to/local-rag
nix develop

# Check Rust compilation
cd src-tauri && cargo check

# Build frontend
npm run build

# Run Tauri dev
npm run tauri:dev

# Build Tauri
npm run tauri:build
```

## Prerequisites for Full Functionality

```bash
# Install Ollama
curl -fsSL https://ollama.com/install.sh | sh

# Pull models
ollama pull phi3.5-mini    # For chat
ollama pull nomic-embed-text  # For embeddings
```

## Environment Info

- **Platform**: Linux (x86_64)
- **Node**: v22.22.1
- **npm**: 10.9.4
- **Rust**: rustc 1.94.0
- **Cargo**: cargo 1.94.0
- **Nix**: Working with all dependencies
