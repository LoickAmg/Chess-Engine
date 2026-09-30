import { createApp } from "vue";
import { createPinia } from "pinia";
// Polices des thèmes : Classique (Cormorant + Inter), Persona 5 (Anton), Persona 3
// (Barlow Condensed), Encre de Chine (Shippori Mincho + Yuji Syuku pour la calligraphie).
import "@fontsource/inter/latin-400.css";
import "@fontsource/inter/latin-500.css";
import "@fontsource/inter/latin-600.css";
import "@fontsource/inter/latin-700.css";
import "@fontsource/cormorant-garamond/latin-500.css";
import "@fontsource/cormorant-garamond/latin-600.css";
import "@fontsource/cormorant-garamond/latin-700.css";
import "@fontsource/cormorant-garamond/latin-500-italic.css";
import "@fontsource/anton/latin-400.css";
import "@fontsource/barlow-condensed/latin-600.css";
import "@fontsource/barlow-condensed/latin-700.css";
import "@fontsource/barlow-condensed/latin-700-italic.css";
import "@fontsource/shippori-mincho/latin-500.css";
import "@fontsource/shippori-mincho/latin-700.css";
import "@fontsource/shippori-mincho/latin-800.css";
import "@fontsource/shippori-mincho/japanese-800.css";
import "@fontsource/yuji-syuku/latin-400.css";
import "@fontsource/yuji-syuku/japanese-400.css";
import App from "./App.vue";
import "./style.css";

createApp(App).use(createPinia()).mount("#app");
