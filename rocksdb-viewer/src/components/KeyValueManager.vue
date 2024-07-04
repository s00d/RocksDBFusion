<template>
  <div class="w-full flex bg-gray-100">
    <div class="w-1/4 border-r bg-white shadow-lg flex flex-col">
      <!-- Search bar and add button fixed at the top -->
      <div class="p-2 border-b">
        <div class="flex items-center">
          <input type="text" v-model="searchQuery" @input="handleSearchQuery" :placeholder="$t('search')" class="w-full p-1 border rounded" />
          <button @click="openAddModal" class="p-1 pl-3 pr-3 bg-green-500 text-white ml-2 rounded">
            +
          </button>
        </div>
      </div>
      <!-- Scrollable list of keys -->
      <div class="flex-1 overflow-y-auto" @scroll="handleScroll">
        <ul>
          <li v-for="key in keys" :key="key" class="mb-1">
            <button @click="selectKey(key)" class="w-full flex justify-between items-center p-1 pl-2 pr-3 bg-gray-100 hover:bg-gray-200 rounded">
              <span>{{ key }}</span>
              <svg @click.stop="deleteKey(key)" class="w-4 h-4 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12"></path>
              </svg>
            </button>
          </li>
        </ul>
      </div>
      <!-- Disconnect button fixed at the bottom -->
      <div class="p-2 border-t">
        <button @click="$emit('disconnect')" class="w-full p-2 bg-red-500 text-white rounded-none">{{ $t('disconnect') }}</button>
      </div>
    </div>
    <div class="w-3/4 p-6">
      <div v-if="selectedKey && editableValue !== null" class="bg-white p-6 shadow-lg rounded">
        <h2 class="text-2xl mb-4">{{ selectedKey }}</h2>
        <textarea v-model="editableValue" class="w-full p-2 border mb-4 rounded" rows="10"></textarea>
        <button @click="saveValue" class="p-2 bg-blue-500 text-white rounded">{{ $t('save') }}</button>
      </div>
    </div>

    <!-- Loader for fetching keys -->
    <div v-if="loadingKeys" class="fixed inset-0 flex items-center justify-center bg-white bg-opacity-75">
      <div class="loader"></div>
    </div>

    <!-- Loader for fetching value -->
    <div v-if="loadingValue" class="fixed inset-0 flex items-center justify-center bg-white bg-opacity-75">
      <div class="loader"></div>
    </div>

    <!-- Modal for adding a new key-value pair -->
    <div v-if="showAddModal" class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50">
      <div class="bg-white p-6 rounded-lg shadow-2xl w-1/3">
        <h2 class="text-xl mb-4">{{ $t('addNewKeyValuePair') }}</h2>
        <input v-model="newKey" :placeholder="$t('key')" class="w-full p-2 mb-4 border rounded" />
        <textarea v-model="newValue" :placeholder="$t('value')" class="w-full p-2 mb-4 border rounded" rows="5"></textarea>
        <div class="flex justify-end">
          <button @click="closeAddModal" class="p-2 bg-gray-500 text-white mr-2 rounded">{{ $t('cancel') }}</button>
          <button @click="addKeyValuePair" class="p-2 bg-blue-500 text-white rounded">{{ $t('save') }}</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from 'vue3-toastify';

export default {
  props: {
    keys: Array,
    selectedKey: String,
    value: String,
    loadingKeys: Boolean,
    loadingValue: Boolean
  },
  data() {
    return {
      searchQuery: '',
      showAddModal: false,
      newKey: '',
      newValue: '',
      hasMore: true
    };
  },
  computed: {
    editableValue: {
      get() {
        return this.value;
      },
      set(newValue) {
        this.$emit('update:value', newValue);
      }
    }
  },
  methods: {
    fetchKeys(reset = false) {
      this.$emit('fetchKeys', reset);
    },
    selectKey(key) {
      this.$emit('selectKey', key);
    },
    saveValue() {
      this.$emit('saveValue');
    },
    deleteKey(key) {
      this.$emit('deleteKey', key);
    },
    handleScroll(event) {
      const bottom = event.target.scrollHeight - event.target.scrollTop === event.target.clientHeight;
      if (bottom && this.hasMore) {
        this.fetchKeys();
      }
    },
    handleSearchQuery() {
      this.$emit('updateSearchQuery', this.searchQuery);
      this.fetchKeys(true);
    },
    openAddModal() {
      this.showAddModal = true;
    },
    closeAddModal() {
      this.showAddModal = false;
      this.newKey = '';
      this.newValue = '';
    },
    async addKeyValuePair() {
      if (this.newKey && this.newValue) {
        try {
          await invoke('put_value', { key: this.newKey, value: this.newValue });
          this.$emit('fetchKeys', true);
          toast.success(this.$t('keyAdded'));
          this.closeAddModal();
        } catch (e) {
          console.error(this.$t('failedToAddKeyValuePair') + ':', e);
          toast.error(this.$t('failedToAddKeyValuePair') + ': ' + e.message);
        }
      }
    }
  }
};
</script>

<style scoped>
.loader {
  border: 16px solid #f3f3f3;
  border-radius: 50%;
  border-top: 16px solid #3498db;
  width: 120px;
  height: 120px;
  -webkit-animation: spin 2s linear infinite;
  animation: spin 2s linear infinite;
}

@-webkit-keyframes spin {
  0% { -webkit-transform: rotate(0deg); }
  100% { -webkit-transform: rotate(360deg); }
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>
