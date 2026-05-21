<template>
  <div>{{ message }}</div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'MultiBlock',
})

function sharedHelper(): string {
  return 'shared'
}
</script>

<script setup lang="ts">
import { ref } from 'vue'

const message = ref<string>('hello')

function greet(name: string): string {
  return `Hello, ${name}`
}
</script>
