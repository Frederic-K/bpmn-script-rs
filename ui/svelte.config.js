// Configuration Svelte. Ce fichier sert aussi à l'extension Svelte de VS Code
// (sinon : « No Svelte configuration found »).
export default {
  // Svelte 5 avec runes uniquement : l'ancienne syntaxe (export let, $:) est
  // refusée à la compilation, pour qu'aucun composant ne passe en mode hybride.
  compilerOptions: { runes: true },
};
