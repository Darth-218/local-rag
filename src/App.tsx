import { useState, useEffect, useRef } from 'react'
import { invoke } from '@tauri-apps/api/core'
import './App.css'

interface DocumentMetadata {
  id: string
  chat_id: string
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
  created_at: string
  updated_at: string
}

interface Message {
  id: string
  chat_id: string
  role: 'user' | 'assistant'
  content: string
  created_at: string
}

interface ContextMenu {
  x: number
  y: number
  chatId: string
}

function App() {
  const [documents, setDocuments] = useState<DocumentMetadata[]>([])
  const [chats, setChats] = useState<Chat[]>([])
  const [activeChat, setActiveChat] = useState<string | null>(null)
  const [messages, setMessages] = useState<Message[]>([])
  const [inputValue, setInputValue] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null)
  const [editingChatId, setEditingChatId] = useState<string | null>(null)
  const [editingTitle, setEditingTitle] = useState('')
  const messagesEndRef = useRef<HTMLDivElement>(null)

  const loadDocuments = async (chatId: string) => {
    try {
      const docs = await invoke<DocumentMetadata[]>('get_chat_documents', { chatId })
      setDocuments(docs)
    } catch (err) {
      console.error('Failed to load documents:', err)
    }
  }

  const loadChats = async () => {
    try {
      const loadedChats = await invoke<Chat[]>('get_chats')
      setChats(loadedChats)
      if (loadedChats.length > 0 && !activeChat) {
        setActiveChat(loadedChats[0].id)
      }
    } catch (err) {
      console.error('Failed to load chats:', err)
    }
  }

  const loadMessages = async (chatId: string) => {
    try {
      const loadedMessages = await invoke<Message[]>('get_chat_messages', { chatId })
      setMessages(loadedMessages)
    } catch (err) {
      console.error('Failed to load messages:', err)
    }
  }

  useEffect(() => {
    loadChats()
  }, [])

  useEffect(() => {
    if (activeChat) {
      loadDocuments(activeChat)
      loadMessages(activeChat)
    } else {
      setDocuments([])
      setMessages([])
    }
  }, [activeChat])

  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  useEffect(() => {
    const handleClick = () => setContextMenu(null)
    document.addEventListener('click', handleClick)
    return () => document.removeEventListener('click', handleClick)
  }, [])

  const handleCreateChat = async () => {
    try {
      const newChat = await invoke<Chat>('create_chat', { title: null })
      setChats(prev => [newChat, ...prev])
      setActiveChat(newChat.id)
    } catch (err) {
      setError(String(err))
    }
  }

  const handleDeleteChat = async (chatId: string) => {
    try {
      await invoke('delete_chat', { chatId })
      setChats(prev => prev.filter(c => c.id !== chatId))
      if (activeChat === chatId) {
        const remaining = chats.filter(c => c.id !== chatId)
        setActiveChat(remaining.length > 0 ? remaining[0].id : null)
      }
    } catch (err) {
      setError(String(err))
    }
  }

  const handleStartRename = (chat: Chat) => {
    setEditingChatId(chat.id)
    setEditingTitle(chat.title)
    setContextMenu(null)
  }

  const handleRenameChat = async () => {
    if (!editingChatId || !editingTitle.trim()) return
    try {
      const updated = await invoke<Chat>('rename_chat', { 
        chatId: editingChatId, 
        title: editingTitle.trim() 
      })
      setChats(prev => prev.map(c => c.id === updated.id ? updated : c))
      setEditingChatId(null)
      setEditingTitle('')
    } catch (err) {
      setError(String(err))
    }
  }

  const handleContextMenu = (e: React.MouseEvent, chatId: string) => {
    e.preventDefault()
    setContextMenu({ x: e.clientX, y: e.clientY, chatId })
  }

  const handlePickFile = async () => {
    if (!activeChat) {
      setError('Please create or select a chat first')
      return
    }
    
    setLoading(true)
    setError(null)
    
    try {
      const filePath = await invoke<string | null>('pick_file')
      if (filePath) {
        const result = await invoke<ProcessingResult>('process_document', { 
          filePath, 
          chatId: activeChat 
        })
        console.log('Document processed:', result.metadata.name)
        await loadDocuments(activeChat)
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
      if (activeChat) {
        await loadDocuments(activeChat)
      }
    } catch (err) {
      setError(String(err))
    }
  }

  const handleSendMessage = async () => {
    if (!inputValue.trim() || !activeChat || loading) return
    
    const userMessage = inputValue.trim()
    setInputValue('')
    setLoading(true)
    
    try {
      await invoke<Message>('send_message', { 
        chatId: activeChat, 
        content: userMessage 
      })
      
      const response = await invoke<string>('ask_question', { 
        chatId: activeChat,
        query: userMessage,
        model: null
      })
      
      await invoke<Message>('send_message', {
        chatId: activeChat,
        content: response
      })
      
      await loadMessages(activeChat)
      await loadChats()
    } catch (err) {
      setError(String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSendMessage()
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
          <button className="new-chat-btn" onClick={handleCreateChat}>
            <span className="icon">+</span>
            New Chat
          </button>
        </div>
        <nav className="chat-list">
          {chats.length === 0 ? (
            <div className="empty-chats">
              <p>No chats yet</p>
              <p className="hint">Click "New Chat" to start</p>
            </div>
          ) : (
            chats.map((chat) => (
              <div 
                key={chat.id} 
                className={`chat-item ${activeChat === chat.id ? 'active' : ''}`}
                onContextMenu={(e) => handleContextMenu(e, chat.id)}
              >
                {editingChatId === chat.id ? (
                  <input
                    className="chat-title-input"
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.target.value)}
                    onBlur={handleRenameChat}
                    onKeyDown={(e) => e.key === 'Enter' && handleRenameChat()}
                    autoFocus
                  />
                ) : (
                  <button 
                    className="chat-title-btn"
                    onClick={() => setActiveChat(chat.id)}
                  >
                    {chat.title}
                  </button>
                )}
              </div>
            ))
          )}
        </nav>
      </aside>

      {/* Context Menu */}
      {contextMenu && (
        <div 
          className="context-menu"
          style={{ top: contextMenu.y, left: contextMenu.x }}
        >
          <button 
            className="context-menu-item"
            onClick={() => {
              const chat = chats.find(c => c.id === contextMenu.chatId)
              if (chat) handleStartRename(chat)
            }}
          >
            Rename
          </button>
          <button 
            className="context-menu-item danger"
            onClick={() => handleDeleteChat(contextMenu.chatId)}
          >
            Delete
          </button>
        </div>
      )}

      {/* Main Content Area */}
      <main className="main-content">
        <div className="chat-area">
          {activeChat ? (
            <>
              <div className="chat-messages">
                {messages.length === 0 ? (
                  <div className="welcome-message">
                    <h2>local-rag</h2>
                    <p>Ask questions about your documents</p>
                  </div>
                ) : (
                  messages.map((msg) => (
                    <div key={msg.id} className={`message ${msg.role}`}>
                      <div className="message-role">{msg.role === 'user' ? 'You' : 'Assistant'}</div>
                      <div className="message-content">{msg.content}</div>
                    </div>
                  ))
                )}
                <div ref={messagesEndRef} />
              </div>
              <div className="chat-input-area">
                <textarea
                  className="chat-input"
                  placeholder="Type your question..."
                  value={inputValue}
                  onChange={(e) => setInputValue(e.target.value)}
                  onKeyDown={handleKeyDown}
                  disabled={loading}
                  rows={1}
                />
                <button 
                  className="add-doc-btn"
                  onClick={handlePickFile}
                  disabled={loading || !activeChat}
                  title="Add Document"
                >
                  📎
                </button>
                <button 
                  className="send-btn"
                  onClick={handleSendMessage}
                  disabled={!inputValue.trim() || loading}
                >
                  {loading ? '...' : '→'}
                </button>
              </div>
            </>
          ) : (
            <div className="welcome-message">
              <h2>local-rag</h2>
              <p>Select a chat or create a new one to get started</p>
            </div>
          )}
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
              <p className="hint">Add documents to this chat</p>
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
