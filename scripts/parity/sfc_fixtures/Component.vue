<template>
  <div class="counter">
    <button @click="decrement">-</button>
    <span>{{ count }}</span>
    <button @click="increment">+</button>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useStore } from '../composables/useStore'

const count = ref<number>(0)

function increment(): void {
  count.value++
}

function decrement(): void {
  count.value--
}
</script>

<style scoped>
.counter { display: flex; gap: 8px; }
</style>
