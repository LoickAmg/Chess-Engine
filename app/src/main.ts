import { createApp } from "vue";
import { createPinia } from "pinia";
import "@fontsource/fredoka/latin-500.css";
import "@fontsource/fredoka/latin-600.css";
import "@fontsource/fredoka/latin-700.css";
import "@fontsource/nunito/latin-400.css";
import "@fontsource/nunito/latin-600.css";
import "@fontsource/nunito/latin-700.css";
import "@fontsource/nunito/latin-800.css";
import App from "./App.vue";
import "./style.css";

createApp(App).use(createPinia()).mount("#app");
