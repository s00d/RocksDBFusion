<template>
  <div class="flex-1 flex justify-center items-center">
    <div class="w-1/2 border p-4 bg-white shadow-lg">
      <h2 class="text-2xl mb-4">Connect to Server</h2>
      <div>
        <input v-model="name" placeholder="Server Name" class="w-full p-2 mb-4 border" />
        <input v-model="address" placeholder="Server Address" class="w-full p-2 mb-4 border" />
        <input v-if="usePassword" type="password" v-model="token" placeholder="Password" class="w-full p-2 mb-4 border" />
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="usePassword" class="mr-2" />
          Use Password
        </label>
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="useTunnel" class="mr-2" />
          Use SSH Tunnel
        </label>
        <div v-if="useTunnel" class="pl-4 border-l">
          <input v-model="sshHost" placeholder="SSH Host" class="w-full p-2 mb-4 border" />
          <input v-model="sshUser" placeholder="SSH User" class="w-full p-2 mb-4 border" />
          <input type="password" v-model="sshPassword" placeholder="SSH Password" class="w-full p-2 mb-4 border" />
          <input type="number" v-model.number="sshPort" placeholder="SSH Port" class="w-full p-2 mb-4 border" />
          <label class="flex items-center mb-4">
            <input type="checkbox" v-model="useSSL" class="mr-2" />
            Use SSL Certificate
          </label>

          <input v-if="certificate" type="text" class="block w-full p-1 ps-1 text-sm text-gray-900 bg-gray-50 focus:ring-blue-500" v-model="certificate" disabled />
          <button v-if="useSSL" class="p-2 bg-blue-500 text-white mr-4 mb-4" @click="selectCertificate">Select SSL Certificate</button>
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
    connectionError: String
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
      connecting: false
    };
  },
  methods: {
    async connect() {
      console.log(5555);
      this.connecting = true;
      const ssh_info = this.useTunnel ? [this.sshHost, this.sshUser, this.sshPassword, this.sshPort] : null;
      this.$emit('connect', { address: this.address, token: this.usePassword ? this.token : null, ssh_info });
      this.connecting = false;
    },
    async selectCertificate() {
      this.certificate = await open({
        directory: false,
        multiple: false,
      });
    },
    saveConnection() {
      const newConnection = {
        name: this.name,
        address: this.address,
        token: this.token,
        sshHost: this.sshHost,
        sshUser: this.sshUser,
        sshPassword: this.sshPassword,
        sshPort: this.sshPort,
        usePassword: this.usePassword,
        useTunnel: this.useTunnel,
        useSSL: this.useSSL,
        certificate: this.certificate ? this.certificate.name : null,
      };
      this.$emit('saveConnection', newConnection);
    }
  }
};
</script>
