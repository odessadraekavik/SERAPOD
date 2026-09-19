# Versions de SERAPOD

SERAPOD suit [Semantic Versioning 2.0.0](https://semver.org/lang/fr/) : `MAJEUR.MINEUR.CORRECTIF`.

Le projet est en développement initial (`0.y.z`). La première version publique, `0.8.0`, ajoute les mises à jour GitHub. Les corrections seules incrémentent le correctif ; les nouvelles fonctionnalités incrémentent le mineur. Une version publiée ne sera pas remplacée par une autre compilation distribuée avec le même numéro.

Le contrat de compatibilité concerne les profils JSON exportés (champ `version`, indépendant de la version du logiciel), les réglages sauvegardés et leur migration. La version majeure 1 sera réservée à la stabilisation de ce contrat et du fonctionnement en jeu.

Mettre à jour ensemble `package.json`, `src-tauri/Cargo.toml` et `src-tauri/tauri.conf.json`. Le script de compilation vérifie leur cohérence. L’interface lit la version depuis `package.json` ; la sortie est `portable/SERAPOD-<version>.exe`.

Le nom visible est SERAPOD — Super Earth Radial Accessibility Program for Orbital Deployment. L'identifiant Tauri est `app.serapod.desktop`. L'ancien dossier `app.stratcom.desktop` migre automatiquement au premier lancement si le nouveau n'existe pas. Le localStorage accepte l'ancienne clé `stratcom.v1` et sauvegarde sous `serapod.v1`.
