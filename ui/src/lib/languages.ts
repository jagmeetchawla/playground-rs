/** All supported project types. */
export type ProjectType = 'rust' | 'clang' | 'zig' | 'swift'

/** Configuration for a language's companion book (Help menu). */
export interface BookConfig {
  commandLabel: string
  toastEntity: string
  toastAlreadyLoaded: string
  menuEvent: string
  removeMenuEvent: string
  sourceTag: string
  /** Base URL for the online book (omit trailing slash) */
  bookUrl?: string
  /** Map of project name → chapter URL path (appended to bookUrl) */
  chapterUrls?: Record<string, string>
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
  /** Sub-languages (e.g. C and C++ within Clang) */
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
      commandLabel: 'The Rust Programming Language (2024 Edition)',
      toastEntity: 'Rust Book chapter',
      toastAlreadyLoaded: 'Rust Book chapters are already loaded.',
      menuEvent: 'menu:rust-book',
      removeMenuEvent: 'menu:remove-rust-book',
      sourceTag: 'rust_book',
      bookUrl: 'https://doc.rust-lang.org/book',
      chapterUrls: {
        rust_ch01_getting_started: '/ch01-00-getting-started.html',
        rust_ch02_guessing_game: '/ch02-00-guessing-game-tutorial.html',
        rust_ch03_concepts: '/ch03-00-common-programming-concepts.html',
        rust_ch04_ownership: '/ch04-00-understanding-ownership.html',
        rust_ch05_structs: '/ch05-00-structs.html',
        rust_ch06_enums: '/ch06-00-enums.html',
        rust_ch07_modules: '/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html',
        rust_ch08_collections: '/ch08-00-common-collections.html',
        rust_ch09_errors: '/ch09-00-error-handling.html',
        rust_ch10_generics: '/ch10-00-generics.html',
        rust_ch11_testing: '/ch11-00-testing.html',
        rust_ch12_minigrep: '/ch12-00-an-io-project.html',
        rust_ch13_closures: '/ch13-00-functional-features.html',
        rust_ch14_cargo: '/ch14-00-more-about-cargo.html',
        rust_ch15_smart_pointers: '/ch15-00-smart-pointers.html',
        rust_ch16_concurrency: '/ch16-00-concurrency.html',
        rust_ch17_async: '/ch17-00-async-await.html',
        rust_ch18_oop: '/ch18-00-oop.html',
        rust_ch19_patterns: '/ch19-00-patterns.html',
        rust_ch20_advanced: '/ch20-00-advanced-features.html',
        rust_ch21_web_server: '/ch21-00-final-project-a-web-server.html',
      },
    },
  },
  clang: {
    type: 'clang',
    label: 'C/C++',
    badge: 'CLG',
    badgeClass: 'badge-clang',
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
      commandLabel: 'The C Programming Language (K&R, 2nd Ed.)',
      toastEntity: 'K&R chapter',
      toastAlreadyLoaded: 'K&R chapters are already loaded.',
      menuEvent: 'menu:knr-book',
      removeMenuEvent: 'menu:remove-knr-book',
      sourceTag: 'knr_book',
    },
  },
  zig: {
    type: 'zig',
    label: 'Zig (v0.15)',
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
      commandLabel: 'The Swift Programming Language (Swift 6.1)',
      toastEntity: 'Swift Book chapter',
      toastAlreadyLoaded: 'Swift Book chapters are already loaded.',
      menuEvent: 'menu:swift-book',
      removeMenuEvent: 'menu:remove-swift-book',
      sourceTag: 'swift_book',
      bookUrl: 'https://docs.swift.org/swift-book/documentation/the-swift-programming-language',
      chapterUrls: {
        swift_ch01_basics: '/thebasics',
        swift_ch02_control_flow: '/controlflow',
        swift_ch03_functions: '/functions',
        swift_ch04_collections: '/collectiontypes',
        swift_ch05_structs_classes: '/classesandstructures',
        swift_ch06_enums: '/enumerations',
        swift_ch07_protocols: '/protocols',
        swift_ch08_error_handling: '/errorhandling',
      },
    },
  },
}

export function getLang(type: ProjectType): LanguageConfig {
  return LANGUAGES[type] ?? LANGUAGES.rust
}

export function allLanguages(): LanguageConfig[] {
  return Object.values(LANGUAGES)
}

export function enabledLanguageConfigs(enabled: string[]): LanguageConfig[] {
  return allLanguages().filter(l => enabled.includes(l.type))
}

/** Map a file extension to a badge type for the tab bar. */
export function badgeTypeFromExt(ext: string): string {
  const map: Record<string, string> = {
    rs: 'rs', c: 'c', cpp: 'cpp', zig: 'zig', swift: 'swift',
  }
  return map[ext] ?? 'rs'
}
