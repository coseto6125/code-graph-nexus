<template>
  <div>
    <span>{{ label }}</span>
    <button @click="onClick">Click</button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface ButtonProps {
  label: string
  disabled?: boolean
}

const props = defineProps<ButtonProps>()
const emit = defineEmits<{
  click: [value: string]
  reset: []
}>()
defineExpose({ reset })

const isDisabled = computed(() => props.disabled ?? false)

function onClick(): void {
  emit('click', props.label)
}

function reset(): void {
  emit('reset')
}
</script>
