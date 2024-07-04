<template>
  <div class="flex h-screen">
    <SavedConnections v-if="!connected" :savedConnections="savedConnections" @loadConnection="loadConnection" @deleteConnection="deleteConnection" />
    <ConnectForm
      v-if="!connected"
      :connection.sync="connection"
      :connected="connected"
      :connectionError="connectionError"
      @connect="connect"
      @saveConnection="saveConnection"
    />
    <KeyValueManager v-else :keys="keys" :selectedKey="selectedKey" :value.sync="value" :loadingKeys="loadingKeys" :loadingValue="loadingValue" @updateSearchQuery="updateSearchQuery" @fetchKeys="fetchKeys" @selectKey="selectKey" @saveValue="saveValue" @deleteKey="deleteKey" @disconnect="disconnect" />
  </div>
</template>
<script>
import { invoke } from '@tauri-apps/api/tauri';
import { toast } from 'vue3-toastify';
import SavedConnections from './components/SavedConnections.vue';
import ConnectForm from './components/ConnectForm.vue';
import KeyValueManager from './components/KeyValueManager.vue';

export default {
  components: {
    SavedConnections,
    ConnectForm,
    KeyValueManager
  },
  data() {
    return {
      connection: {
        name: 'Default',
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
    };
  },
  methods: {
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
        toast.success("Connected to server successfully");
      } catch (e) {
        this.connectionError = 'Failed to connect to server: ' + e.message;
        toast.error(this.connectionError);
      } finally {
        this.connecting = false;
      }
    },
    async disconnect() {
      this.connected = false;
      this.connection = {
        name: 'Default',
        host: '',
        port: '',
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
      toast.info("Disconnected from server");
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
        console.error('Failed to fetch keys:', e);
        toast.error('Failed to fetch keys: ' + e.message);
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
          console.error('Failed to get value:', e);
          toast.error('Failed to get value: ' + e.message);
        } finally {
          this.loadingValue = false;
        }
      }
    },
    async saveValue() {
      if (this.connected && this.selectedKey) {
        try {
          await invoke('put_value', { key: this.selectedKey, value: this.value });
          toast.success('Value saved successfully');
        } catch (e) {
          console.error('Failed to save value:', e);
          toast.error('Failed to save value: ' + e.message);
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
          toast.success('Key deleted successfully');
        } catch (e) {
          console.error('Failed to delete key:', e);
          toast.error('Failed to delete key: ' + e.message);
        }
      }
    },
    saveConnection(newConnection) {
      this.savedConnections.push(newConnection);
      localStorage.setItem('savedConnections', JSON.stringify(this.savedConnections));
      toast.success('Connection saved successfully');
    },
    loadConnection(connection) {
      this.connection = { ...connection };
    },
    deleteConnection(index) {
      this.savedConnections.splice(index, 1);
      localStorage.setItem('savedConnections', JSON.stringify(this.savedConnections));
      toast.info('Connection deleted successfully');
    },
  },
};
</script>

