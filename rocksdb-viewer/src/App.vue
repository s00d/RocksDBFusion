<template>
  <div class="flex h-screen bg-gray-200">
    <div class="absolute top-2 right-2 twilight-language-switcher">
      <form class="max-w-sm mx-auto">

        <div class="inline-block relative w-25">
          <select v-model="currentLanguage" @change="changeLanguage" class="block appearance-none w-full bg-white border border-gray-400 hover:border-gray-500 px-4 py-2 pr-8 rounded shadow leading-tight focus:outline-none focus:shadow-outline">
              <option value="en">English</option>
              <option value="ru">Русский</option>
              <option value="es">Español</option>
              <option value="zh">中文</option>
              <option value="hi">हिन्दी</option>
              <option value="ar">العربية</option>
              <option value="pt">Português</option>
              <option value="bn">বাংলা</option>
              <option value="ja">日本語</option>
              <option value="ko">한국어</option>
              <option value="fr">Français</option>
          </select>
          <div class="pointer-events-none absolute inset-y-0 right-0 flex items-center px-2 text-gray-700">
            <svg class="fill-current h-4 w-4" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20 20"><path d="M9.293 12.95l.707.707L15.657 8l-1.414-1.414L10 10.828 5.757 6.586 4.343 8z"/></svg>
          </div>
        </div>
      </form>
    </div>
    <!-- Saved Connections Sidebar -->
    <SavedConnections
      v-if="!connected"
      :savedConnections="savedConnections"
      @loadConnection="loadConnection"
      @deleteConnection="deleteConnection"
    />
    <!-- Connection Form -->
    <ConnectForm
      v-if="!connected"
      :connection.sync="connection"
      :connected="connected"
      :connectionError="connectionError"
      @connect="connect"
      @saveConnection="saveConnection"
    />
    <!-- Key-Value Manager -->
    <KeyValueManager
      v-else
      :keys="keys"
      :selectedKey="selectedKey"
      :value.sync="value"
      :loadingKeys="loadingKeys"
      :loadingValue="loadingValue"
      @updateSearchQuery="updateSearchQuery"
      @fetchKeys="fetchKeys"
      @selectKey="selectKey"
      @saveValue="saveValue"
      @deleteKey="deleteKey"
      @disconnect="disconnect"
    />
  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from 'vue3-toastify';
import SavedConnections from './components/SavedConnections.vue';
import ConnectForm from './components/ConnectForm.vue';
import KeyValueManager from './components/KeyValueManager.vue';
import { useI18n } from 'vue-i18n';

export default {
  components: {
    SavedConnections,
    ConnectForm,
    KeyValueManager
  },
  data() {
    return {
      connection: {
        name: this.$t('defaultConnectionName'),
        host: '127.0.0.1',
        port: 12345,
        token: '',
        sshHost: '',
        sshUser: '',
        sshPassword: '',
        sshPort: 22,
        usePassword: false,
        useTunnel: false,
        useSSL: false,
        certificate: null,
      },
      connected: false,
      connecting: false,
      connectionError: null,
      savedConnections: JSON.parse(localStorage.getItem('savedConnections')) || [],
      keys: [],
      searchQuery: '',
      selectedKey: null,
      value: null,
      limit: 20,
      start: 0,
      loadingKeys: false,
      loadingValue: false,
      hasMore: true,
      currentLanguage: 'en'
    };
  },
  methods: {
    changeLanguage() {
      this.$i18n.locale = this.currentLanguage;
    },
    async connect() {
      this.connecting = true;
      this.connectionError = null;

      try {
        const { host, port, token, sshHost, sshUser, sshPassword, sshPort, usePassword, useTunnel } = this.connection;
        const ssh_info = useTunnel ? [sshHost, sshUser, sshPassword, sshPort] : null;
        await invoke('connect_to_server', {
          host,
          port,
          token: usePassword ? token : null,
          ssh_info
        });
        this.connected = true;
        this.fetchKeys();
        toast.success(this.$t('connected'));
      } catch (e) {
        this.connectionError = this.$t('failedToConnect') + ': ' + e.message;
        toast.error(this.connectionError);
      } finally {
        this.connecting = false;
      }
    },
    async disconnect() {
      this.connected = false;
      this.resetConnection();
      toast.info(this.$t('disconnected'));
    },
    resetConnection() {
      this.connection = {
        name: this.$t('defaultConnectionName'),
        host: '127.0.0.1',
        port: 12345,
        token: '',
        sshHost: '',
        sshUser: '',
        sshPassword: '',
        sshPort: 22,
        usePassword: false,
        useTunnel: false,
        useSSL: false,
        certificate: null,
      };
      this.keys = [];
      this.selectedKey = null;
      this.value = null;
      this.limit = 20;
      this.start = 0;
      this.loadingKeys = false;
      this.loadingValue = false;
      this.hasMore = true;
    },
    updateSearchQuery(value) {
      this.searchQuery = value;
    },
    async fetchKeys(reset = false) {
      if (!this.connected || this.loadingKeys) return;

      if (reset) {
        this.start = 0;
        this.keys = [];
        this.hasMore = true;
      }

      if (!this.hasMore) return;

      this.loadingKeys = true;
      try {
        const keys = await invoke('get_keys', {
          start: this.start,
          limit: this.limit,
          query: this.searchQuery || null
        });
        if (keys.length < this.limit) {
          this.hasMore = false;
        }
        this.keys = [...this.keys, ...keys];
        this.start += this.limit;
      } catch (e) {
        console.error(this.$t('failedToFetchKeys') + ':', e);
        toast.error(this.$t('failedToFetchKeys') + ': ' + e.message);
      } finally {
        this.loadingKeys = false;
      }
    },
    async selectKey(key) {
      this.selectedKey = key;
      this.value = null;
      this.loadingValue = true;
      if (this.connected) {
        try {
          this.value = await invoke('get_value', { key });
        } catch (e) {
          console.error(this.$t('failedToGetValue') + ':', e);
          toast.error(this.$t('failedToGetValue') + ': ' + e.message);
        } finally {
          this.loadingValue = false;
        }
      }
    },
    async saveValue() {
      if (this.connected && this.selectedKey) {
        try {
          await invoke('put_value', { key: this.selectedKey, value: this.value });
          toast.success(this.$t('valueSaved'));
        } catch (e) {
          console.error(this.$t('failedToSaveValue') + ':', e);
          toast.error(this.$t('failedToSaveValue') + ': ' + e.message);
        }
      }
    },
    async deleteKey(key) {
      if (this.connected) {
        try {
          await invoke('delete_value', { key });
          this.keys = this.keys.filter(k => k !== key);
          if (this.selectedKey === key) {
            this.selectedKey = null;
            this.value = null;
          }
          toast.success(this.$t('keyDeleted'));
        } catch (e) {
          console.error(this.$t('failedToDeleteKey') + ':', e);
          toast.error(this.$t('failedToDeleteKey') + ': ' + e.message);
        }
      }
    },
    saveConnection(newConnection) {
      this.savedConnections.push(newConnection);
      localStorage.setItem('savedConnections', JSON.stringify(this.savedConnections));
      toast.success(this.$t('connectionSaved'));
    },
    loadConnection(connection) {
      this.connection = { ...connection };
    },
    deleteConnection(index) {
      this.savedConnections.splice(index, 1);
      localStorage.setItem('savedConnections', JSON.stringify(this.savedConnections));
      toast.info(this.$t('connectionDeleted'));
    },
  },
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
  0% {
    -webkit-transform: rotate(0deg);
  }
  100% {
    -webkit-transform: rotate(360deg);
  }
}

@keyframes spin {
  0% {
    transform: rotate(0deg);
  }
  100% {
    transform: rotate(360deg);
  }
}
</style>
