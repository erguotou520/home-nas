import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'

interface ShareInfo {
    id: number
    token: string
    url: string
    file_path: string
    app_type: string
    expires_at?: string
    burn_after_read: boolean
    max_views?: number
    view_count: number
    created_at: string
}

interface CreateShareRequest {
    file_path: string
    app_type: string
    expires_in_hours?: number
    burn_after_read?: boolean
    max_views?: number
}

export function useShares() {
    return useQuery({
        queryKey: ['shares'],
        queryFn: async () => {
            const response = await apiClient.get<ShareInfo[]>('/api/shares')
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
    })
}

export function useCreateShare() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async (data: CreateShareRequest) => {
            const response = await apiClient.post<ShareInfo>('/api/shares', data)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['shares'] })
        },
    })
}

export function useDeleteShare() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async (shareId: number) => {
            const response = await apiClient.delete(`/api/shares/${shareId}`)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['shares'] })
        },
    })
}
