# local-rag

**A privacy-first, local RAG application.**

> Your documents, your model, your machine. No data leaves your computer.

## Features

- **100% Offline** - Works without internet
- **Privacy-First** - All data stays on your machine
- **Local LLM** - Uses Ollama for inference
- **Multi-Format Support** - PDF, Markdown, plain text
- **OCR Support** - Automatic text extraction from scanned/image PDFs via Tesseract
- **Source Citations** - See exactly where answers come from
- **Open Source** - Fully transparent, auditable code

## Tech Stack

| Component | Technology |
|-----------|------------|
| UI | React + Tauri |
| LLM | Ollama (phi3.5-mini) |
| Embeddings | Ollama (nomic-embed-text) |
| Vector Store | JSON files |

## Requirements

- **OS**: Linux, macOS, Windows
- **RAM**: 4GB minimum
- **Storage**: ~500MB for app + ~4GB for model
- **GPU**: Optional (runs on CPU)

## Getting Started

### 1. Install Ollama

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

### 2. Pull Required Models

**Chat/LLM** (for generating answers):
```bash
ollama pull phi3.5-mini   # Recommended (2.2GB)
# or
ollama pull llama3.2:1b    # Lighter option (1.3GB)
```

**Embeddings** (for document/query vectorization):
```bash
ollama pull nomic-embed-text  # Required for embeddings (~275MB)
```

> **Note**: Both models are required for full functionality.

### 3. Install local-rag

```bash
git clone https://github.com/yourusername/local-rag.git
cd local-rag
cargo tauri build
```

### 4. Run

```bash
cargo tauri dev      # Development mode
# or
cargo tauri build && ./src-tauri/target/release/local-rag  # Production
```

## Usage

1. **Add Documents** - Click "Add Files" to import PDFs or text files
2. **Wait for Indexing** - Documents are chunked and embedded automatically
3. **Ask Questions** - Type your question in the chat
4. **Get Answers** - View responses with cited sources

## Architecture

```
┌─────────────────────────────────────────┐
│           Tauri Desktop App             │
│                                         │
│  ┌─────────┐  ┌──────────┐  ┌────────┐  │
│  │  React  │  │  Ollama  │  │  JSON  │  │
│  │   UI    │──│  (LLM)   │──│ Files  │  │
│  └─────────┘  └──────────┘  └────────┘  │
│                                         │
│  Data Directory: ~/.local/share/com.localrag.app/
│  ├── documents/      # Original files   │
│  ├── chroma/         # Vector index     │
│  └── chats/         # Chat history      │
└─────────────────────────────────────────┘
```

## Data Flow

```
User uploads PDF
       ↓
Text Extraction (lopdf)
       ↓ (if empty)
OCR via Tesseract
       ↓
Chunking (fixed-size with overlap)
       ↓
Embedding (nomic-embed-text via Ollama)
       ↓
Store in local JSON (vector index)
       ↓
User query → Embed → Search → Generate with LLM
```

## Project Structure

```
local-rag/
├── src/                   # React frontend
│   ├── components/        # UI components
│   ├── hooks/             # React hooks
│   ├── lib/               # Utilities
│   └── App.tsx            # Main app
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── commands.rs    # Tauri commands
│   │   ├── document.rs    # Document processing
│   │   ├── embedding.rs   # Embedding logic
│   │   └── retrieval.rs   # RAG retrieval
│   ├── Cargo.toml
│   └── tauri.conf.json
├── README.md
└── LICENSE
```

## Configuration

Edit `~/.local-rag/config.json`:

```json
{
  "model": "phi3.5-mini",
  "embedding_model": "all-MiniLM-L6-v2",
  "chunk_size": 512,
  "chunk_overlap": 50,
  "retrieval_top_k": 5
}
```

## Roadmap

### Phase 1: Foundation & Scaffolding

| Step | Task | Deliverable |
|------|------|-------------|
| 1.1 | Initialize Tauri + React project | `npm create tauri-app` |
| 1.2 | Setup Rust dependencies | ChromaDB, pdf-extract, embedding crates |
| 1.3 | Create project structure | `/src`, `/src-tauri/src` organized |
| 1.4 | Configure data directories | `~/.local-rag/` with subdirs |
| 1.5 | Add logging + error handling | Basic setup for debugging |

**Milestone**: Empty shell app that launches without errors

---

### Phase 2: Document Processing Pipeline

| Step | Task | Deliverable |
|------|------|-------------|
| 2.1 | File picker UI | Native file dialog via Tauri |
| 2.2 | Text extraction | PDF → raw text (pdf-extract crate) |
| 2.3 | Chunking logic | Fixed-size with overlap |
| 2.4 | Markdown/text support | Direct parsing |
| 2.5 | Document metadata | Filename, date, page count |

**Milestone**: Upload a PDF, see it parsed into chunks

---

### Phase 3: Embedding & Vector Storage

| Step | Task | Deliverable |
|------|------|-------------|
| 3.1 | Embedding model setup | all-MiniLM-L6-v2 via candle/transformers |
| 3.2 | ChromaDB integration | Collection creation, CRUD |
| 3.3 | Batch embedding | Process chunks efficiently |
| 3.4 | Document management | List/delete indexed docs |

**Milestone**: Index documents, query vectors stored in ChromaDB

---

### Phase 4: LLM Integration

| Step | Task | Deliverable |
|------|------|-------------|
| 4.1 | Ollama API client | Connection, health check |
| 4.2 | Prompt templating | RAG prompt with context |
| 4.3 | Retrieval → Generation | Connect chunks to LLM |
| 4.4 | Streaming responses | Real-time token output |

**Milestone**: Ask a question, get a streamed answer

---

### Phase 5: UI/UX

| Step | Task | Deliverable |
|------|------|-------------|
| 5.1 | Document library view | List indexed files |
| 5.2 | Chat interface | Input, messages, citations |
| 5.3 | Upload flow | Progress indicator, feedback |
| 5.4 | Settings panel | Model selection, chunk config |
| 5.5 | Source citations | Clickable excerpts in response |

**Milestone**: Functional chat app with citations

---

### Phase 6: Polish & Edge Cases

| Step | Task | Deliverable |
|------|------|-------------|
| 6.1 | Error handling | Network failures, corrupt files |
| 6.2 | Empty states | No docs uploaded yet |
| 6.3 | Large file handling | Progress + chunking |
| 6.4 | Config persistence | Save/load from JSON |

**Milestone**: Robust, no-crash app

---

### Phase 7: Distribution

| Step | Task | Deliverable |
|------|------|-------------|
| 7.1 | Build configuration | Icons, app name, version |
| 7.2 | Cross-platform builds | Linux .AppImage/.deb, macOS .dmg |
| 7.3 | Installer experience | First-run setup wizard |
| 7.4 | Documentation | README, contributing guide |

**Milestone**: Release-ready binary

---

## Later Features (Post-Release)

- Chat history / conversation memory
- Quick-start tutorial / onboarding
- PDF with tables and figures support
- Web URL scraping
- Shared class knowledge base mode
- Reranking with cross-encoders
- Multi-tenant / shared corpus option

---

## Development Setup with Nix

This project includes a Nix development environment with all dependencies pre-configured.

### Option 1: Flakes (Recommended)

```bash
# Enter the dev environment
nix develop

# Or with direnv for auto-activation
echo "use flake" > .envrc
direnv allow
```

### Option 2: Traditional shell.nix

```bash
nix-shell
# or
nix-shell nix/shell.nix
```

### What's Included

- Rust toolchain (rustc, cargo, clippy, rustfmt)
- Node.js 22 + pnpm
- All Tauri system dependencies (webkitgtk, gtk3, etc.)
- Tesseract OCR + English language data
- Build tools (gcc, make, patchelf)

### Manual Setup (Without Nix)

If not using Nix, install dependencies manually:

**Ubuntu/Debian:**
```bash
sudo apt install pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.0-dev
```

**Rust:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Node.js:**
```bash
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install nodejs
npm install -g pnpm
```

---

## TODO List

### Phase 1: Foundation
- [x] Initialize Tauri + React project
- [x] Setup Rust dependencies
- [x] Create project structure
- [x] Configure data directories (`~/.local-rag/`)
- [x] Add logging + error handling

### Phase 2: Document Processing
- [x] File picker UI
- [x] PDF text extraction
- [x] OCR support (Tesseract for scanned/image PDFs)
- [x] Chunking logic (fixed-size + overlap)
- [x] Markdown/text support
- [x] Document metadata

### Phase 3: Embedding & Storage
- [x] Ollama API client
- [x] Embedding generation
- [x] Cosine similarity search
- [x] Document embedding storage
- [x] RAG retrieval pipeline

### Phase 4: LLM Integration
- [x] Ollama API client
- [x] Prompt templating
- [x] Retrieval → Generation pipeline
- [ ] Streaming responses

### Phase 5: UI/UX
- [x] Document library view
- [x] Chat interface layout
- [ ] Upload flow with progress
- [ ] Settings panel
- [ ] Source citations display

### Phase 6: Polish
- [ ] Error handling
- [ ] Empty states
- [ ] Large file handling
- [ ] Config persistence

### Phase 7: Distribution
- [ ] Build configuration
- [ ] Cross-platform builds
- [ ] Installer experience
- [ ] Documentation

## License

MIT

## Contributing

Contributions welcome! Please read the code and follow existing patterns.
