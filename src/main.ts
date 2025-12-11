import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import PrimeVue from "primevue/config";
import Lara from "@primevue/themes/lara";
import "primeicons/primeicons.css";
import "primeflex/primeflex.css";

import Button from "primevue/button";
import { TabPanel, TabView } from "primevue";
import Tooltip from "primevue/tooltip";
import Toast from "primevue/toast";
import ToastService from "primevue/toastservice";
import InputNumber from "primevue/inputnumber";
import FloatLabel from "primevue/floatlabel";
import SelectButton from "primevue/selectbutton";

const app = createApp(App);
const pinia = createPinia();

app.use(PrimeVue, {
  theme: {
    preset: Lara,
    options: {
      prefix: "p",
      darkModeSelector: "none", // Disable dark mode completely
      cssLayer: false,
    },
  },
});

app.use(pinia);
app.use(ToastService);

app.component("Button", Button);
app.component("TabView", TabView);
app.component("TabPanel", TabPanel);
app.component("Toast", Toast);
app.directive("tooltip", Tooltip);
app.component("InputNumber", InputNumber);
app.component("FloatLabel", FloatLabel);
app.component("SelectButton", SelectButton);

app.mount("#app");
