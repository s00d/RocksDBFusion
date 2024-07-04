import { createApp } from "vue";
import App from "./App.vue";
import './index.css';
import Toast from "vue3-toastify";
import "vue3-toastify/dist/index.css";
import { createI18n } from 'vue-i18n';
import en from './i18n/en.json';
import ru from './i18n/ru.json';
import es from './i18n/es.json';
import zh from './i18n/zh.json';
import hi from './i18n/hi.json';
import ar from './i18n/ar.json';
import pt from './i18n/pt.json';
import bn from './i18n/bn.json';
import ja from './i18n/ja.json';
import ko from './i18n/ko.json';
import fr from './i18n/fr.json';

const i18n = createI18n({
    locale: 'en', // default locale
    messages: {
        en,
        ru,
        es,
        zh,
        hi,
        ar,
        pt,
        bn,
        ja,
        ko,
        fr
    }
});


const app = createApp(App);

app.use(Toast, {
    position: 'top-right',
    timeout: 5000,
});

createApp(App).use(i18n).mount("#app");
