import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'

interface FileEntry {
    name: string
    path: string
    is_dir: boolean
    size?: number
    modified?: number
    mime_type?: string
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

interface ListResponse {
    entries: FileEntry[]
    total: number
    path: string
    app: string
}

export function useFiles(app: string, path: string = '') {
    return useQuery({
        queryKey: ['files', app, path],
        queryFn: async () => {
            const endpoint = path
                ? `/api/files/${app}/${encodeURIComponent(path)}`
                : `/api/files/${app}`

            const response = await apiClient.get<ListResponse>(endpoint)
            if (response.error) {
                throw new Error(response.error)
            }

            // Normalize snake_case to camelCase
            return {
                ...response.data!,
                entries: response.data!.entries.map(e => ({
                    name: e.name,
                    path: e.path,
                    isDir: e.is_dir,
                    size: e.size,
                    modified: e.modified,
                    mimeType: e.mime_type,
                    thumbnail: e.thumbnail,
                    metadata: e.metadata,
                })),
            }
        },
        staleTime: 30000, // 30 seconds
    })
}

export function useCopyFile() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async ({ app, path, destination }: { app: string; path: string; destination: string }) => {
            const response = await apiClient.patch(`/api/files/${app}/${encodeURIComponent(path)}`, {
                operation: 'copy',
                destination,
            })
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data
        },
        onSuccess: (_, { app }) => {
            queryClient.invalidateQueries({ queryKey: ['files', app] })
        },
    })
}

export function useMoveFile() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async ({ app, path, destination }: { app: string; path: string; destination: string }) => {
            const response = await apiClient.patch(`/api/files/${app}/${encodeURIComponent(path)}`, {
                operation: 'move',
                destination,
            })
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data
        },
        onSuccess: (_, { app }) => {
            queryClient.invalidateQueries({ queryKey: ['files', app] })
        },
    })
}

export function useDeleteFile() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async ({ app, path }: { app: string; path: string }) => {
            const response = await apiClient.delete(`/api/files/${app}/${encodeURIComponent(path)}`)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data
        },
        onSuccess: (_, { app }) => {
            queryClient.invalidateQueries({ queryKey: ['files', app] })
        },
    })
}


export function useUploadFile() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async ({
            app,
            path,
            filename,
            bytes,
            overwrite = false,
        }: {
            app: string
            path: string
            filename: string
            bytes: Uint8Array
            overwrite?: boolean
        }) => {
            const encodedPath = path
                ? path.split('/').filter(Boolean).map(encodeURIComponent).join('/')
                : ''
            const targetPath = encodedPath ? `/${encodedPath}` : ''
            const endpoint = `/api/files/${app}/upload${targetPath}?filename=${encodeURIComponent(filename)}&overwrite=${overwrite}`
            const response = await apiClient.postBinary<{ success: boolean; path: string }>(endpoint, bytes)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data
        },
        onSuccess: (_, { app }) => {
            queryClient.invalidateQueries({ queryKey: ['files', app] })
        },
    })
}
