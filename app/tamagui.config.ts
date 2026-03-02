import { createTamagui, createTokens } from '@tamagui/core'
import { config as configBase } from '@tamagui/config'

const tokens = createTokens({
  ...configBase.tokens,
  color: {
    ...configBase.tokens.color,
    // Custom colors for NAS app
    primary: '#6366f1',
    primaryDark: '#4f46e5',
    secondary: '#8b5cf6',
    background: '#0f172a',
    backgroundLight: '#1e293b',
    surface: '#1e293b',
    surfaceHover: '#334155',
    text: '#f8fafc',
    textMuted: '#94a3b8',
    border: '#334155',
    success: '#22c55e',
    error: '#ef4444',
    warning: '#f59e0b',
  },
})

const config = createTamagui({
  ...configBase,
  tokens,
  themes: {
    dark: {
      background: tokens.color.background,
      backgroundHover: tokens.color.surfaceHover,
      backgroundPress: tokens.color.surface,
      backgroundFocus: tokens.color.surface,
      color: tokens.color.text,
      colorHover: tokens.color.text,
      colorPress: tokens.color.textMuted,
      colorFocus: tokens.color.text,
      borderColor: tokens.color.border,
      borderColorHover: tokens.color.primary,
      borderColorPress: tokens.color.primary,
      borderColorFocus: tokens.color.primary,
      primary: tokens.color.primary,
      secondary: tokens.color.secondary,
    },
    light: {
      background: '#ffffff',
      backgroundHover: '#f1f5f9',
      backgroundPress: '#e2e8f0',
      backgroundFocus: '#f1f5f9',
      color: '#0f172a',
      colorHover: '#0f172a',
      colorPress: '#475569',
      colorFocus: '#0f172a',
      borderColor: '#e2e8f0',
      borderColorHover: tokens.color.primary,
      borderColorPress: tokens.color.primary,
      borderColorFocus: tokens.color.primary,
      primary: tokens.color.primary,
      secondary: tokens.color.secondary,
    },
  },
  defaultTheme: 'dark',
})

export type AppConfig = typeof config
declare module '@tamagui/core' {
  interface TamaguiCustomConfig extends AppConfig {}
}

export default config
