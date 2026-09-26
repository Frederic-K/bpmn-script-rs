import { mount } from "svelte";
import App from "./App.svelte";
import "./style.css";

// Thème : dernier choix enregistré, sinon celui du système. Appliqué avant le
// premier affichage ; App.svelte le bascule ensuite.
const themeEnregistre = localStorage.getItem("theme");
const systemeSombre = matchMedia("(prefers-color-scheme: dark)").matches;
document.documentElement.dataset.theme = themeEnregistre ?? (systemeSombre ? "dark" : "light");

mount(App, { target: document.getElementById("application") });
