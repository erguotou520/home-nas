import { create } from 'zustand'

interface FileEntry {
    name: string
    path: string
    isDir: boolean
    size?: number
    modified?: number
    mimeType?: string
    thumbnail?: string
    metadata?: MediaMetadata
}

interface MediaMetadata {
    title?: string
    poster?: string
    fanart?: string
    year?: string
    plot?: string
}

interface FileState {
    currentApp: string
    currentPath: string
    files: FileEntry[]
    selectedFiles: string[]
    isGridView: boolean
    isLoading: boolean
    error: string | null

    setCurrentApp: (app: string) => void
    setCurrentPath: (path: string) => void
    setFiles: (files: FileEntry[]) => void
    selectFile: (path: string) => void
    deselectFile: (path: string) => void
    toggleSelection: (path: string) => void
    clearSelection: () => void
    selectAll: () => void
    setGridView: (isGrid: boolean) => void
    setLoading: (loading: boolean) => void
    setError: (error: string | null) => void
    navigateToPath: (path: string) => void
    navigateUp: () => void
}

export const useFileStore = create<FileState>((set, get) => ({
    currentApp: 'medias',
    currentPath: '',
    files: [],
    selectedFiles: [],
    isGridView: true,
    isLoading: false,
    error: null,

    setCurrentApp: (app) => set({
        currentApp: app,
        currentPath: '',
        files: [],
        selectedFiles: [],
        // Default view based on app type
        isGridView: app === 'documents' ? false : true,
    }),

    setCurrentPath: (path) => set({ currentPath: path }),

    setFiles: (files) => set({ files, error: null }),

    selectFile: (path) => set((state) => ({
        selectedFiles: [...state.selectedFiles, path],
    })),

    deselectFile: (path) => set((state) => ({
        selectedFiles: state.selectedFiles.filter((p) => p !== path),
    })),

    toggleSelection: (path) => {
        const { selectedFiles } = get()
        if (selectedFiles.includes(path)) {
            set({ selectedFiles: selectedFiles.filter((p) => p !== path) })
        } else {
            set({ selectedFiles: [...selectedFiles, path] })
        }
    },

    clearSelection: () => set({ selectedFiles: [] }),

    selectAll: () => set((state) => ({
        selectedFiles: state.files.map((f) => f.path),
    })),

    setGridView: (isGrid) => set({ isGridView: isGrid }),

    setLoading: (loading) => set({ isLoading: loading }),

    setError: (error) => set({ error }),

    navigateToPath: (path) => set({ currentPath: path, selectedFiles: [] }),

    navigateUp: () => {
        const { currentPath } = get()
        if (!currentPath) return

        const parts = currentPath.split('/').filter(Boolean)
        parts.pop()
        set({ currentPath: parts.join('/'), selectedFiles: [] })
    },
}))
