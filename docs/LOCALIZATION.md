# Localisation

L’interface est traduite via `src/locales/en.json` et `src/locales/fr.json`. Aucune bibliothèque distante ni aucun téléchargement n’est nécessaire au démarrage.

## Sélection de la langue

- Par défaut, `settings.language` vaut `auto`.
- Dans l’application Windows, `system_locale` utilise `GetUserPreferredUILanguages(MUI_LANGUAGE_NAME)` pour obtenir la langue d’interface préférée de l’utilisateur Windows. Elle est relue à chaque lancement.
- Dans l’aperçu navigateur, `navigator.language` remplit ce rôle.
- La langue régionale est normalisée : `fr-FR`, `fr-CA` et `fr_CA` sélectionnent `fr`.
- Si la langue n’est pas prise en charge, l’interface utilise l’anglais.
- Le choix explicite dans **Options → Langue** est sauvegardé avec les profils, persiste au redémarrage et prend le pas sur la détection système.
- L’overlay reçoit la langue résolue avec sa configuration. Les erreurs natives utilisent des clés traduites côté interface.

## Ajouter une langue

1. Copier `src/locales/en.json` dans un nouveau fichier, par exemple `de.json`.
2. Traduire les valeurs sans modifier les clés ni les paramètres `{name}`, `{count}`, `{locale}`.
3. Importer ce fichier dans `src/lib/i18n.js`, puis l’ajouter à `messages` et à `languages` avec son nom natif (`de: 'Deutsch'`). Le sélecteur, la validation et la détection automatique le prennent alors en compte.
4. Étendre le test de parité des traductions si nécessaire, puis lancer `pnpm test` et `pnpm check`.

Une clé absente revient à sa valeur anglaise. Les traductions se mettent à jour immédiatement sans recharger la page. Les catégories, l’accessibilité des séquences et les messages sont traduits ; les noms de stratagèmes conservent les libellés du wiki source.

Les noms de profils et de sous-menus saisis par l’utilisateur restent inchangés. Les nouveaux profils par défaut utilisent `nameKey` pour leurs libellés automatiques ; renommer un élément retire cette clé. Les anciens noms sauvegardés sont conservés.
