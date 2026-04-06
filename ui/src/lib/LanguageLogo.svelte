<script lang="ts">
  import rustLogo from './assets/logos/rust.svg'
  import cargoLogo from './assets/logos/cargo.svg'
  import cLogo from './assets/logos/c.svg'
  import cppLogo from './assets/logos/cpp.svg'
  import zigLogo from './assets/logos/zig.svg'
  import swiftLogo from './assets/logos/swift.svg'

  let {
    type,
    size = 16,
  }: {
    type: string
    size?: number
  } = $props()

  const logos: Record<string, string> = {
    rust: rustLogo,
    cargo: cargoLogo,
    c: cLogo,
    cpp: cppLogo,
    clang: cppLogo,
    zig: zigLogo,
    swift: swiftLogo,
  }

  let src = $derived(logos[type] ?? logos.rust)
</script>

<img {src} alt={type} width={size} height={size} class="lang-logo" class:invert-dark={type === 'rust'} />

<style>
  .lang-logo {
    display: inline-block;
    vertical-align: middle;
    object-fit: contain;
  }
  /* Rust logo is black — invert to white in dark themes */
  :global(:root) .invert-dark,
  :global(.theme-dark) .invert-dark {
    filter: invert(1);
  }
  :global(.theme-light) .invert-dark {
    filter: none;
  }
</style>
