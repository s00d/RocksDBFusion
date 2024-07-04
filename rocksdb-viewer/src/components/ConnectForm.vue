<template>
  <div class="flex-1 flex justify-center items-center">
    <div class="w-2/2 border p-2 bg-white shadow-2xl rounded-lg">
      <h2 class="text-2xl mb-6">{{ $t('connectToServer') }}</h2>
      <div>
        <input v-model="connection.name" :placeholder="$t('serverName')" class="w-full p-2 mb-4 border rounded" />
        <div class="flex mb-4">
          <input v-model="connection.host" :placeholder="$t('serverHost')" class="w-3/4 p-2 border rounded" />
          <input type="number" v-model.number="connection.port" :placeholder="$t('serverPort')" class="w-1/4 p-2 border rounded" />
        </div>
        <input v-if="connection.usePassword" type="password" v-model="connection.token" :placeholder="$t('password')" class="w-full p-2 mb-4 border rounded" />
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="connection.usePassword" class="mr-2" />
          {{ $t('usePassword') }}
        </label>
        <label class="flex items-center mb-4">
          <input type="checkbox" v-model="connection.useTunnel" class="mr-2" />
          {{ $t('useSSHTunnel') }}
        </label>
        <div v-if="connection.useTunnel" class="pl-4 border-l">
          <input v-model="connection.sshHost" :placeholder="$t('sshHost')" class="w-full p-2 mb-4 border rounded" />
          <input v-model="connection.sshUser" :placeholder="$t('sshUser')" class="w-full p-2 mb-4 border rounded" />
          <input type="password" v-model="connection.sshPassword" :placeholder="$t('sshPassword')" class="w-full p-2 mb-4 border rounded" />
          <input type="number" v-model.number="connection.sshPort" :placeholder="$t('sshPort')" class="w-full p-2 mb-4 border rounded" />
          <label class="flex items-center mb-4">
            <input type="checkbox" v-model="connection.useSSL" class="mr-2" />
            {{ $t('useSSLCertificate') }}
          </label>
          <input v-if="connection.certificate" type="text" class="block w-full p-2 text-sm text-gray-900 bg-gray-50 rounded focus:ring-blue-500" v-model="connection.certificate" disabled />
          <button v-if="connection.useSSL" class="p-2 bg-blue-500 text-white mr-4 mb-4 rounded" @click="selectCertificate">{{ $t('selectSSLCertificate') }}</button>
        </div>
        <div class="flex justify-between">
          <button @click="connect" class="w-1/2 p-2 bg-blue-500 text-white mr-2 rounded">
            <span v-if="connecting">{{ $t('connecting') }}</span>
            <span v-else>{{ $t('connect') }}</span>
          </button>
          <button @click="saveConnection" class="w-1/2 p-2 bg-green-500 text-white rounded">
            {{ $t('save') }}
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
