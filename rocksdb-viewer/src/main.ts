import { createApp } from "vue";
import App from "./App.vue";
import './index.css';
import Toast from "vue3-toastify";
import "vue3-toastify/dist/index.css";

const app = createApp(App);

app.use(Toast, {
    position: 'top-right',
    timeout: 5000,
});

createApp(App).mount("#app");
