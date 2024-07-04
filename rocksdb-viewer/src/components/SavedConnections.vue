<template>
  <div class="w-1/4 border-r p-2 bg-white shadow-2xl rounded-l-lg">
    <h2 class="text-xl mb-4">{{ $t('savedConnections') }}</h2>
    <ul>
      <li v-for="(connection, index) in savedConnections" :key="index" class="mb-2">
        <button @click="$emit('loadConnection', connection)" class="w-full flex justify-between items-center p-2 bg-gray-100 hover:bg-gray-200 rounded">
          <span>{{ connection.name }}</span>
          <svg @click.stop="$emit('deleteConnection', index)" class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
          </svg>
        </button>
      </li>
    </ul>
  </div>
</template>

<script>
export default {
  props: {
    savedConnections: Array
  }
}
</script>

<style scoped>
/* Add any additional styles if necessary */
</style>
