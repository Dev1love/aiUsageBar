export const themes = {
  hacker: {
    bg: '#000000',
    'bg-secondary': '#0a0a0a',
    text: '#00ff41',
    'text-dim': 'rgba(0, 255, 65, 0.4)',
    accent: '#00ff41',
    'accent-glow': 'rgba(0, 255, 65, 0.3)',
    warning: '#ffb800',
    danger: '#ff3333',
    border: 'rgba(0, 255, 65, 0.1)',
    track: 'rgba(0, 255, 65, 0.08)',
    'btn-hover': 'rgba(0, 255, 65, 0.08)',
    'net-down': '#00ff41',
    'net-up': '#00ccff',
  },
  midnight: {
    bg: '#0f0f1a',
    'bg-secondary': '#16162a',
    text: '#e2e2ea',
    'text-dim': 'rgba(255, 255, 255, 0.4)',
    accent: '#34d399',
    'accent-glow': 'rgba(52, 211, 153, 0.3)',
    warning: '#f59e0b',
    danger: '#ef4444',
    border: 'rgba(255, 255, 255, 0.06)',
    track: 'rgba(255, 255, 255, 0.06)',
    'btn-hover': 'rgba(255, 255, 255, 0.06)',
    'net-down': '#34d399',
    'net-up': '#60a5fa',
  },
  cyberpunk: {
    bg: '#0d0011',
    'bg-secondary': '#150020',
    text: '#e0d0ff',
    'text-dim': 'rgba(224, 208, 255, 0.4)',
    accent: '#bf5af2',
    'accent-glow': 'rgba(191, 90, 242, 0.3)',
    warning: '#ff9f0a',
    danger: '#ff375f',
    border: 'rgba(191, 90, 242, 0.12)',
    track: 'rgba(191, 90, 242, 0.08)',
    'btn-hover': 'rgba(191, 90, 242, 0.08)',
    'net-down': '#bf5af2',
    'net-up': '#ff375f',
  },
  frost: {
    bg: '#f0f4f8',
    'bg-secondary': '#e2e8f0',
    text: '#1a202c',
    'text-dim': 'rgba(26, 32, 44, 0.45)',
    accent: '#3182ce',
    'accent-glow': 'rgba(49, 130, 206, 0.2)',
    warning: '#dd6b20',
    danger: '#e53e3e',
    border: 'rgba(0, 0, 0, 0.08)',
    track: 'rgba(0, 0, 0, 0.06)',
    'btn-hover': 'rgba(0, 0, 0, 0.04)',
    'net-down': '#38a169',
    'net-up': '#3182ce',
  },
} as const;

export type ThemeName = keyof typeof themes;

export function applyTheme(name: ThemeName) {
  const theme = themes[name] || themes.hacker;
  const root = document.documentElement;
  for (const [key, value] of Object.entries(theme)) {
    root.style.setProperty(`--${key}`, value);
  }
}
