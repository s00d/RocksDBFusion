<template>
  <div class="flex h-screen">
    <SavedConnections v-if="!connected" :savedConnections="savedConnections" @loadConnection="loadConnection" @deleteConnection="deleteConnection" />
    <ConnectForm v-if="!connected" :connected="connected" :connectionError="connectionError" @connect="connect" @saveConnection="saveConnection" />
    <KeyValueManager v-else :keys="keys" :selectedKey="selectedKey" :value.sync="value" :loadingKeys="loadingKeys" :loadingValue="loadingValue" @updateSearchQuery="updateSearchQuery" @fetchKeys="fetchKeys" @selectKey="selectKey" @saveValue="saveValue" @deleteKey="deleteKey" @disconnect="disconnect" />


  </div>
</template>

<script>
import { invoke } from '@tauri-apps/api/tauri';
import { open } from '@tauri-apps/api/dialog';
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
      name: 'Default',
      address: '127.0.0.1:12345',
      token: '',
      sshHost: '',
      sshUser: '',
      sshPassword: '',
      sshPort: 22,
      usePassword: false,
      useTunnel: false,
      useSSL: false,
      certificate: null,
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
        const ssh_info = this.useTunnel ? [this.sshHost, this.sshUser, this.sshPassword, this.sshPort] : null;
        await invoke('connect_to_server', {
          address: this.address,
          token: this.usePassword ? this.token : null,
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
      this.address = '';
      this.token = '';
      this.sshHost = '';
      this.sshUser = '';
      this.sshPassword = '';
      this.sshPort = 22;
      this.usePassword = false;
      this.useTunnel = false;
      this.useSSL = false;
      this.certificate = null;
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
          this.value = await invoke('get_value', {key});
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
          await invoke('put_value', {key: this.selectedKey, value: this.value});
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
          await invoke('delete_value', {key});
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
      this.address = connection.address;
      this.token = connection.token;
      this.sshHost = connection.sshHost;
      this.sshUser = connection.sshUser;
      this.sshPassword = connection.sshPassword;
      this.sshPort = connection.sshPort;
      this.usePassword = connection.usePassword;
      this.useTunnel = connection.useTunnel;
      this.useSSL = connection.useSSL;
      this.certificate = connection.certificate ? {name: connection.certificate} : null;
    },
    deleteConnection(index) {
      this.savedConnections.splice(index, 1);
      localStorage.setItem('savedConnections', JSON.stringify(this.savedConnections));
      toast.info('Connection deleted successfully');
    },
  },
};
</script>
