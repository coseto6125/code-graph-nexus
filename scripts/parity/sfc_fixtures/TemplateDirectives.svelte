<script lang="ts">
  let items: string[] = ['apple', 'banana', 'cherry']
  let show: boolean = true

  async function loadData(): Promise<string[]> {
    return ['a', 'b']
  }

  const promise = loadData()
</script>

{#if show}
  <ul>
    {#each items as item}
      <li>{item}</li>
    {/each}
  </ul>
{/if}

{#await promise}
  <p>Loading...</p>
{:then data}
  <p>{data}</p>
{/await}
