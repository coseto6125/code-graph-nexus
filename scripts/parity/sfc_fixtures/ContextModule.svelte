<script context="module" lang="ts">
  import { readable } from 'svelte/store'

  export const globalCount = readable(0)

  export function formatCount(n: number): string {
    return `Count: ${n}`
  }
</script>

<script lang="ts">
  import { onMount } from 'svelte'

  let name: string = 'world'

  function greet(): string {
    return `Hello, ${name}`
  }
</script>

<p>{greet()}</p>

<style>
  p { color: navy; }
</style>
