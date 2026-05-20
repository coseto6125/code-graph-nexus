<script lang="ts">
  import { onMount } from 'svelte';

  let count = $state(0);

  function increment(): void {
    count++;
  }

  onMount(() => {
    console.log('Counter mounted');
  });
</script>

<button on:click={increment}>{count}</button>

<style>
  button { font-size: 1.5rem; }
</style>
