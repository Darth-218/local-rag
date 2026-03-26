import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

interface DocumentMetadata {
  id: string
  name: string
  file_path: string
  file_type: string
  size: number
  page_count: number | null
  word_count: number
  created_at: string
  modified_at: string
}

interface ProcessingResult {
  metadata: DocumentMetadata
  chunks: unknown[]
}

interface Chat {
  id: string
  title: string
  createdAt: string
}

function App() {
  const [documents, setDocuments] = useState<DocumentMetadata[]>([])
  const [chats] = useState<Chat[]>([
    { id: '1', title: 'New Chat', createdAt: new Date().toISOString() }
  ])
  const [activeChat, setActiveChat] = useState<string>('1')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const loadDocuments = async () => {
    try {
      const docs = await invoke<DocumentMetadata[]>('get_documents_metadata')
      setDocuments(docs)
    } catch (err) {
      console.error('Failed to load documents:', err)
    }
  }

  useEffect(() => {
    loadDocuments()
  }, [])

  const handlePickFile = async () => {
    setLoading(true)
    setError(null)
    
    try {
      const filePath = await invoke<string | null>('pick_file')
      
      if (filePath) {
        await invoke<ProcessingResult>('process_document', { filePath })
        await loadDocuments()
      }
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleDeleteDocument = async (id: string) => {
    try {
      await invoke('delete_document', { documentId: id })
      await loadDocuments()
    } catch (err) {
      setError(String(err))
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  return (
    <div className="app">
      {/* Left Sidebar - Chat History */}
      <aside className="sidebar sidebar-left">
        <div className="sidebar-header">
          <button className="new-chat-btn">
            <span className="icon">+</span>
            New Chat
          </button>
        </div>
        <nav className="chat-list">
          {chats.map((chat) => (
            <button
              key={chat.id}
              className={`chat-item ${activeChat === chat.id ? 'active' : ''}`}
              onClick={() => setActiveChat(chat.id)}
            >
              {chat.title}
            </button>
          ))}
        </nav>
      </aside>

      {/* Main Content Area */}
      <main className="main-content">
        <div className="chat-area">
          <div className="chat-messages">
            <div className="welcome-message">
              <h2>local-rag</h2>
              <p>Select a document from the right panel and ask questions about it.</p>
            </div>
          </div>
        </div>

        {/* Bottom Command Bar */}
        <div className="command-bar">
          <button 
            className="command-btn"
            onClick={handlePickFile}
            disabled={loading}
          >
            {loading ? (
              <span className="loading">Processing...</span>
            ) : (
              <>
                <span className="icon">+</span>
                Add Document
              </>
            )}
          </button>
        </div>
      </main>

      {/* Right Sidebar - Documents */}
      <aside className="sidebar sidebar-right">
        <div className="sidebar-header">
          <h3>Documents</h3>
          <span className="doc-count">{documents.length}</span>
        </div>
        <nav className="doc-list">
          {documents.length === 0 ? (
            <div className="empty-docs">
              <p>No documents</p>
              <p className="hint">Add a document to get started</p>
            </div>
          ) : (
            documents.map((doc) => (
              <div key={doc.id} className="doc-item">
                <div className="doc-icon">
                  {doc.file_type === 'pdf' ? '📄' : '📝'}
                </div>
                <div className="doc-info">
                  <span className="doc-name" title={doc.name}>{doc.name}</span>
                  <span className="doc-meta">
                    {formatFileSize(doc.size)}
                    {doc.page_count && ` • ${doc.page_count}p`}
                  </span>
                </div>
                <button 
                  className="doc-delete"
                  onClick={() => handleDeleteDocument(doc.id)}
                  title="Delete"
                >
                  ×
                </button>
              </div>
            ))
          )}
        </nav>
      </aside>

      {/* Error Toast */}
      {error && (
        <div className="error-toast">
          <span>{error}</span>
          <button onClick={() => setError(null)}>×</button>
        </div>
      )}
    </div>
  )
}

export default App
