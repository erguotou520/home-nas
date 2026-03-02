import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'
import { useAuthStore } from '../stores'

interface LoginRequest {
    username: string
    password: string
}

interface LoginResponse {
    token: string
    user: {
        id: number
        username: string
        role: string
    }
}

interface UserInfo {
    id: number
    username: string
    role: string
}

export function useLogin() {
    const { setToken, setUser } = useAuthStore()

    return useMutation({
        mutationFn: async (data: LoginRequest) => {
            const response = await apiClient.post<LoginResponse>('/api/auth/login', data)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
        onSuccess: (data) => {
            setToken(data.token)
            setUser(data.user)
        },
    })
}

export function useCurrentUser() {
    const { token } = useAuthStore()

    return useQuery({
        queryKey: ['currentUser'],
        queryFn: async () => {
            const response = await apiClient.get<UserInfo>('/api/auth/me')
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
        enabled: !!token,
    })
}

export function useRegisterUser() {
    const queryClient = useQueryClient()

    return useMutation({
        mutationFn: async (data: { username: string; password: string; role?: string }) => {
            const response = await apiClient.post<UserInfo>('/api/auth/register', data)
            if (response.error) {
                throw new Error(response.error)
            }
            return response.data!
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['users'] })
        },
    })
}
