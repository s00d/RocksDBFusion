<template>
  <div class="flex-1 flex justify-center items-center">
    <div class="w-1/2 border p-4 bg-white shadow-lg">
      <h2 class="text-2xl mb-4">Connect to Server</h2>
      <div>
        <input v-model="connection.name" placeholder="Server Name" class="w-full p-2 mb-4 border" />
        <div class="flex mb-4">
          <input v-model="connection.host" placeholder="Server Host" class="w-3/4 p-2 border" />
          <input type="number" v-model.number="connection.port" placeholder="Server Port" class="w-1/4 p-2 border" />
        </div>
        <input v-if="connection.usePassword" type="password" v-model="connection.token" placeholder="Password" class="w-full p-2 mb-4 border" />
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="connection.usePassword" class="mr-2" />
          Use Password
        </label>
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="connection.useTunnel" class="mr-2" />
          Use SSH Tunnel
        </label>
        <div v-if="connection.useTunnel" class="pl-4 border-l">
          <input v-model="connection.sshHost" placeholder="SSH Host" class="w-full p-2 mb-4 border" />
          <input v-model="connection.sshUser" placeholder="SSH User" class="w-full p-2 mb-4 border" />
          <input type="password" v-model="connection.sshPassword" placeholder="SSH Password" class="w-full p-2 mb-4 border" />
          <input type="number" v-model.number="connection.sshPort" placeholder="SSH Port" class="w-full p-2 mb-4 border" />
          <label class="flex items-center mb-4">
            <input type="checkbox" v-model="connection.useSSL" class="mr-2" />
            Use SSL Certificate
          </label>
          <input v-if="connection.certificate" type="text" class="block w-full p-1 ps-1 text-sm text-gray-900 bg-gray-50 focus:ring-blue-500" v-model="connection.certificate" disabled />
          <button v-if="connection.useSSL" class="p-2 bg-blue-500 text-white mr-4 mb-4" @click="selectCertificate">Select SSL Certificate</button>
        </div>
        <div class="flex justify-between">
          <button @click="connect" class="w-1/2 p-2 bg-blue-500 text-white mr-2">
            <span v-if="connecting">Connecting...</span>
            <span v-else>Connect</span>
          </button>
          <button @click="saveConnection" class="w-1/2 p-2 bg-green-500 text-white">
            Save
          </button>
        </div>
        <div v-if="connectionError" class="mt-4 text-red-500">{{ connectionError }}</div>
      </div>
    </div>
  </div>
</template>

<script>
import { open } from '@tauri-apps/api/dialog';
import { toast } from 'vue3-toastify';

export default {
  props: {
    connected: Boolean,
    connectionError: String,
    connection: Object
  },
  data() {
    return {
      connecting: false,
    };
  },
  methods: {
    async connect() {
      this.connecting = true;
      this.$emit('connect', this.connection);
      this.connecting = false;
    },
    async selectCertificate() {
      const certificate = await open({
        directory: false,
        multiple: false,
      });
      this.$emit('update:certificate', certificate);
    },
    saveConnection() {
      this.$emit('saveConnection', this.connection);
    }
  }
};
</script>

