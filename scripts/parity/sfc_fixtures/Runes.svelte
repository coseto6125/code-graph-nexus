<script lang="ts">
  let count = $state(0)
  let doubled = $derived(count * 2)

  $effect(() => {
    console.log('count changed:', count)
  })

  function increment(): void {
    count++
  }
</script>

<button onclick={increment}>
  {count} (doubled: {doubled})
</button>
