/** All supported project types. */
export type ProjectType = 'rust' | 'native' | 'zig' | 'swift'

/** Configuration for a language's companion book (Help menu). */
export interface BookConfig {
  commandLabel: string
  toastEntity: string
  toastAlreadyLoaded: string
  menuEvent: string
  removeMenuEvent: string
  sourceTag: string
}

/** Frontend configuration for a supported language. */
export interface LanguageConfig {
  type: ProjectType
  label: string
  badge: string
  badgeClass: string
  color: string
  extensions: string[]
  hasCargoToml: boolean
  hasBuildFlags: boolean
  buildFlagLabels: string[]
  supportsLiveCheck: boolean
  needsExtension: boolean
  toolchainName: string
  runCommandDisplay: (filename: string) => string
  /** Sub-languages (e.g. C and C++ within Native) */
  subLanguages?: { id: string; label: string; ext: string }[]
  /** Mark unstable languages whose stdlib/API may break between versions */
  experimental?: boolean
  /** Companion book available via Help menu */
  book?: BookConfig
}

const LANGUAGES: Record<ProjectType, LanguageConfig> = {
  rust: {
    type: 'rust',
    label: 'Rust',
    badge: 'RS',
    badgeClass: 'badge-rust',
    color: '#ce422b',
    extensions: ['rs'],
    hasCargoToml: true,
    hasBuildFlags: false,
    buildFlagLabels: [],
    supportsLiveCheck: true,
    needsExtension: false,
    toolchainName: 'cargo',
    runCommandDisplay: (name) => `cargo run --bin ${name}`,
    book: {
      commandLabel: 'The Rust Book',
      toastEntity: 'Rust Book chapter',
      toastAlreadyLoaded: 'Rust Book chapters are already loaded.',
      menuEvent: 'menu:rust-book',
      removeMenuEvent: 'menu:remove-rust-book',
      sourceTag: 'rust_book',
    },
  },
  native: {
    type: 'native',
    label: 'C/C++',
    badge: '\u{F8FF}', // Apple logo
    badgeClass: 'badge-native',
    color: '#44aa99',
    extensions: ['c', 'cpp'],
    hasCargoToml: false,
    hasBuildFlags: true,
    buildFlagLabels: ['C flags', 'C++ flags'],
    supportsLiveCheck: false,
    needsExtension: true,
    toolchainName: 'clang',
    runCommandDisplay: (filename) => {
      const ext = filename.split('.').pop()?.toLowerCase() ?? ''
      const stem = filename.replace(/\.[^.]+$/, '')
      return ext === 'cpp' ? `clang++ ${filename} && ./${stem}` : `clang ${filename} && ./${stem}`
    },
    subLanguages: [
      { id: 'c', label: 'C', ext: 'c' },
      { id: 'cpp', label: 'C++', ext: 'cpp' },
    ],
    book: {
      commandLabel: 'The K&R C Book',
      toastEntity: 'K&R C chapter',
      toastAlreadyLoaded: 'K&R C chapters are already loaded.',
      menuEvent: 'menu:knr-book',
      removeMenuEvent: 'menu:remove-knr-book',
      sourceTag: 'knr_book',
    },
  },
  zig: {
    type: 'zig',
    label: 'Zig',
    badge: 'ZIG',
    badgeClass: 'badge-zig',
    color: '#f7a41d',
    extensions: ['zig'],
    hasCargoToml: false,
    hasBuildFlags: true,
    buildFlagLabels: ['Zig flags'],
    supportsLiveCheck: false,
    needsExtension: true,
    toolchainName: 'zig',
    runCommandDisplay: (filename) => `zig run ${filename}`,
    experimental: true,
  },
  swift: {
    type: 'swift',
    label: 'Swift',
    badge: 'SW',
    badgeClass: 'badge-swift',
    color: '#f05138',
    extensions: ['swift'],
    hasCargoToml: false,
    hasBuildFlags: true,
    buildFlagLabels: ['Swift flags'],
    supportsLiveCheck: false,
    needsExtension: true,
    toolchainName: 'swiftc',
    runCommandDisplay: (filename) => {
      const stem = filename.replace(/\.[^.]+$/, '')
      return `swiftc ${filename} && ./${stem}`
    },
    book: {
      commandLabel: 'The Swift Book',
      toastEntity: 'Swift Book chapter',
      toastAlreadyLoaded: 'Swift Book chapters are already loaded.',
      menuEvent: 'menu:swift-book',
      removeMenuEvent: 'menu:remove-swift-book',
      sourceTag: 'swift_book',
    },
  },
}

export function getLang(type: ProjectType): LanguageConfig {
  return LANGUAGES[type] ?? LANGUAGES.rust
}

export function allLanguages(): LanguageConfig[] {
  return Object.values(LANGUAGES)
}

/** Map a file extension to a badge type for the tab bar. */
export function badgeTypeFromExt(ext: string): string {
  const map: Record<string, string> = {
    rs: 'rs', c: 'c', cpp: 'cpp', zig: 'zig', swift: 'swift',
  }
  return map[ext] ?? 'rs'
}
