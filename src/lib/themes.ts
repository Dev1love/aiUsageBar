export const themes = {
  glass: {
    bg: 'rgba(30, 30, 30, 0.55)',
    'bg-secondary': 'rgba(255, 255, 255, 0.06)',
    text: 'rgba(255, 255, 255, 0.9)',
    'text-dim': 'rgba(255, 255, 255, 0.35)',
    accent: 'rgba(255, 255, 255, 0.85)',
    'accent-glow': 'rgba(255, 255, 255, 0.15)',
    warning: '#ffcc00',
    danger: '#ff453a',
    border: 'rgba(255, 255, 255, 0.12)',
    track: 'rgba(255, 255, 255, 0.08)',
    'btn-hover': 'rgba(255, 255, 255, 0.06)',
    'net-down': 'rgba(48, 209, 88, 0.9)',
    'net-up': 'rgba(100, 210, 255, 0.9)',
    backdrop: 'saturate(180%) blur(20px)',
  },
  'glass-blue': {
    bg: 'rgba(40, 100, 180, 0.25)',
    'bg-secondary': 'rgba(100, 170, 255, 0.08)',
    text: 'rgba(220, 230, 255, 0.92)',
    'text-dim': 'rgba(180, 200, 255, 0.4)',
    accent: 'rgba(120, 180, 255, 0.9)',
    'accent-glow': 'rgba(80, 140, 255, 0.2)',
    warning: '#ffcc00',
    danger: '#ff453a',
    border: 'rgba(100, 160, 255, 0.12)',
    track: 'rgba(100, 160, 255, 0.08)',
    'btn-hover': 'rgba(100, 160, 255, 0.08)',
    'net-down': 'rgba(48, 209, 88, 0.9)',
    'net-up': 'rgba(140, 200, 255, 0.95)',
    backdrop: 'saturate(180%) blur(20px)',
  },
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

// All CSS variable keys used by themes — ensures cleanup when switching
const allKeys = ['bg', 'bg-secondary', 'text', 'text-dim', 'accent', 'accent-glow',
  'warning', 'danger', 'border', 'track', 'btn-hover', 'net-down', 'net-up', 'backdrop'];

export function applyTheme(name: ThemeName) {
  const theme = themes[name] || themes.glass;
  const root = document.documentElement;
  const entries = theme as Record<string, string>;
  for (const key of allKeys) {
    if (entries[key]) {
      root.style.setProperty(`--${key}`, entries[key]);
    } else {
      root.style.removeProperty(`--${key}`);
    }
  }
}
