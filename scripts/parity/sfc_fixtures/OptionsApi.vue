<template>
  <div>
    <h1>{{ title }}</h1>
    <p>Count: {{ count }}</p>
    <button @click="increment">+</button>
  </div>
</template>

<script lang="ts">
import { defineComponent } from 'vue'
import { formatTitle } from '../utils/format'

export default defineComponent({
  name: 'OptionsApi',
  data() {
    return {
      count: 0,
      title: 'Counter',
    }
  },
  computed: {
    doubled(): number {
      return this.count * 2
    },
  },
  methods: {
    increment(): void {
      this.count++
    },
    reset(): void {
      this.count = 0
    },
  },
  mounted(): void {
    console.log('mounted')
  },
})
</script>

<style scoped>
h1 { font-size: 1.5rem; }
</style>
