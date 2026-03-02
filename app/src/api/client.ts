import { useAuthStore } from '../stores'

const API_TIMEOUT = 30000

interface ApiResponse<T> {
    data?: T
    error?: string
}

class ApiClient {
    private getHeaders(): HeadersInit {
        const token = useAuthStore.getState().token
        const headers: HeadersInit = {
            'Content-Type': 'application/json',
        }
        if (token) {
            headers['Authorization'] = `Bearer ${token}`
        }
        return headers
    }

    private getBaseUrl(): string {
        return useAuthStore.getState().serverUrl
    }

    async get<T>(endpoint: string): Promise<ApiResponse<T>> {
        try {
            const controller = new AbortController()
            const timeoutId = setTimeout(() => controller.abort(), API_TIMEOUT)

            const response = await fetch(`${this.getBaseUrl()}${endpoint}`, {
                method: 'GET',
                headers: this.getHeaders(),
                signal: controller.signal,
            })

            clearTimeout(timeoutId)

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: 'Request failed' }))
                return { error: error.error || `HTTP ${response.status}` }
            }

            const data = await response.json()
            return { data }
        } catch (error: any) {
            if (error.name === 'AbortError') {
                return { error: 'Request timeout' }
            }
            return { error: error.message || 'Network error' }
        }
    }

    async post<T>(endpoint: string, body?: any): Promise<ApiResponse<T>> {
        try {
            const response = await fetch(`${this.getBaseUrl()}${endpoint}`, {
                method: 'POST',
                headers: this.getHeaders(),
                body: body ? JSON.stringify(body) : undefined,
            })

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: 'Request failed' }))
                return { error: error.error || `HTTP ${response.status}` }
            }

            const data = await response.json()
            return { data }
        } catch (error: any) {
            return { error: error.message || 'Network error' }
        }
    }


    async postBinary<T>(endpoint: string, bytes: Uint8Array): Promise<ApiResponse<T>> {
        try {
            const token = useAuthStore.getState().token
            const headers: HeadersInit = token
                ? { Authorization: `Bearer ${token}` }
                : {}

            const response = await fetch(`${this.getBaseUrl()}${endpoint}`, {
                method: 'POST',
                headers,
                body: bytes,
            })

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: 'Request failed' }))
                return { error: error.error || `HTTP ${response.status}` }
            }

            const data = await response.json()
            return { data }
        } catch (error: any) {
            return { error: error.message || 'Network error' }
        }
    }
    async patch<T>(endpoint: string, body?: any): Promise<ApiResponse<T>> {
        try {
            const response = await fetch(`${this.getBaseUrl()}${endpoint}`, {
                method: 'PATCH',
                headers: this.getHeaders(),
                body: body ? JSON.stringify(body) : undefined,
            })

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: 'Request failed' }))
                return { error: error.error || `HTTP ${response.status}` }
            }

            const data = await response.json()
            return { data }
        } catch (error: any) {
            return { error: error.message || 'Network error' }
        }
    }

    async delete<T>(endpoint: string): Promise<ApiResponse<T>> {
        try {
            const response = await fetch(`${this.getBaseUrl()}${endpoint}`, {
                method: 'DELETE',
                headers: this.getHeaders(),
            })

            if (!response.ok) {
                const error = await response.json().catch(() => ({ error: 'Request failed' }))
                return { error: error.error || `HTTP ${response.status}` }
            }

            const data = await response.json()
            return { data }
        } catch (error: any) {
            return { error: error.message || 'Network error' }
        }
    }

    getMediaUrl(path: string): string {
        const token = useAuthStore.getState().token
        return `${this.getBaseUrl()}/api/media/stream/${encodeURIComponent(path)}?token=${token}`
    }

    getThumbnailUrl(path: string): string {
        const token = useAuthStore.getState().token
        return `${this.getBaseUrl()}/api/media/thumbnail/${encodeURIComponent(path)}?token=${token}`
    }
}

export const apiClient = new ApiClient()
