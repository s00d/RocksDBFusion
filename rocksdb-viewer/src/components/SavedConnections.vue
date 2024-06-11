<template>
  <div class="w-1/3 border-r p-4 bg-white shadow-lg">
    <h2 class="text-xl mb-4">Saved Connections</h2>
    <ul>
      <li v-for="(connection, index) in savedConnections" :key="index" class="flex justify-between items-center mb-2">
        <span @click="$emit('loadConnection', connection)" class="cursor-pointer">{{ connection.name }}</span>
        <button @click="$emit('deleteConnection', index)" class="p-1 text-red-500">
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
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
