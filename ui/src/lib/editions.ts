// Edition configuration — controls which languages, themes, and UI elements
// are available. Set via VITE_EDITION env var at build time.
// Default: "power" (all languages, full UI).

export interface EditionConfig {
  id: string
  displayName: string
  tagline: string
  languages: string[] | null   // null = all languages (Power Edition)
  defaultTheme: string
  defaultProjectType: string
  isSingleLanguage: boolean
}

const EDITIONS: Record<string, EditionConfig> = {
  power: {
    id: 'power',
    displayName: 'Rustic Playground',
    tagline: 'All languages. All features.',
    languages: null,
    defaultTheme: 'system',
    defaultProjectType: 'rust',
    isSingleLanguage: false,
  },
  rust: {
    id: 'rust',
    displayName: 'Rustic Playground — The Rust Edition',
    tagline: 'A focused playground for learning Rust.',
    languages: ['rust'],
    defaultTheme: 'system',
    defaultProjectType: 'rust',
    isSingleLanguage: true,
  },
  clang: {
    id: 'clang',
    displayName: 'Rustic Playground — The C Edition',
    tagline: 'A focused playground for C and C++.',
    languages: ['clang'],
    defaultTheme: 'system',
    defaultProjectType: 'clang',
    isSingleLanguage: true,
  },
  zig: {
    id: 'zig',
    displayName: 'Rustic Playground — The Zig Edition',
    tagline: 'A focused playground for Zig.',
    languages: ['zig'],
    defaultTheme: 'zig',
    defaultProjectType: 'zig',
    isSingleLanguage: true,
  },
  swift: {
    id: 'swift',
    displayName: 'Rustic Playground — The Swift Edition',
    tagline: 'A focused playground for Swift.',
    languages: ['swift'],
    defaultTheme: 'swift',
    defaultProjectType: 'swift',
    isSingleLanguage: true,
  },
}

export function currentEdition(): EditionConfig {
  const id = (import.meta.env.VITE_EDITION as string) ?? 'power'
  return EDITIONS[id] ?? EDITIONS.power
}
