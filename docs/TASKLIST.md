# SERAPOD 0.7.3 — bilan

## Ajustements 0.7.3

- [x] Aide des commandes placée sous la roue, dans une zone supplémentaire de la fenêtre transparente ; centre et taille de la roue conservés.
- [x] Icônes SVG de souris avec bouton gauche/droit surligné, actions localisées et touche Esc compacte.

## Ajouts 0.7.2

- [x] Pointeur en point blanc bordé de sombre dans l'overlay, lié à la position de sélection, recentré à l'ouverture et lors des changements de dossier.
- [x] Pointeur actualisé aussi à l'intérieur d'un même secteur, avec limitation de fréquence des trames.
- [x] Clic droit : dossier parent ou fermeture à la racine ; clic consommé pour ne pas être transmis au jeu. Même navigation dans la simulation.
- [x] Point et navigation confirmés fonctionnels en jeu par l'utilisateur.

## Correction 0.7.1

- [x] Détection de fenêtre corrigée lorsque GetWindowThreadProcessId renvoie zéro : recherche par threads appartenant au processus exact helldivers2.exe.
- [x] Validation dans l'application complète avec le jeu ouvert : fenêtre détectée, moteur et overlay prêts ; statut Ready confirmé par l'utilisateur.
- [x] F1 et affichage de la roue en jeu confirmés par l'utilisateur.

## Ajouts 0.7.0

- [x] Suppression de la lueur supérieure de la sidebar et du reflet sur le logo.
- [x] Navigation : Éditeur, Profils, Options.
- [x] Première utilisation : six stratagèmes à la racine et dossier Mission contenant Hellbomb, Super Earth Flag, SOS Beacon et SEAF Artillery.
- [x] Nom du profil par défaut localisé ; les noms personnalisés existants restent inchangés.
- [x] Réinitialisation complète avec confirmation : remplace les profils par la nouvelle roue et rétablit toutes les options, langue automatique comprise.
- [x] Stockage renommé `app.serapod.desktop`, avec migration automatique de l'ancien dossier et protection des données déjà présentes.

## Correction 0.6.2

- [x] Espacement uniforme entre les secteurs, vérifié aux rayons intérieur et extérieur.
- [x] Compteur centré dans l’icône du dossier, blanc comme son nom et en gras ; ancien sous-texte supprimé.

## Correction 0.6.1

- [x] Suppression du contour de focus rectangulaire des groupes SVG de la roue ; focus clavier dessiné sur le secteur.
- [ ] Confirmer dans l’exécutable que cela supprime le cadre noir signalé pendant le déplacement.

## Ajouts 0.6.0

- [x] Sélecteurs personnalisés cohérents pour profil, langue, touches et mode, avec navigation clavier et recherche par frappe.
- [x] Sidebar en verre fumé, reflets légers et navigation aux accents jaunes.
- [x] Fond pointillé affiné : faible contraste, points fins et suppression du halo jaune.

## Correction 0.5.2

- [x] Restauration du fond à petits points derrière la roue de l’éditeur, d’après la capture fournie.

## Correction 0.5.1

- [x] Suppression des ombres SVG sur les cibles de dépôt et du fond rectangulaire de l’aperçu flottant.
- [ ] Confirmation visuelle de la disparition du rectangle noir signalé dans WebView2.

## Ajouts 0.5.0

- [x] Menu au clic droit sur les éléments de la roue de l’éditeur, avec option Retirer.
- [x] Fermeture du menu par Échap, clic ailleurs, défilement ou perte de focus.
- [x] Secteur de dépôt rempli en jaune et libellé d’action directement sur la cible.
- [x] Élément déplacé atténué et survol normal neutralisé pendant le glisser-déposer.

## Ajouts 0.4.0

- [x] Déposer un stratagème du catalogue sur un stratagème de la roue le remplace, même si la roue est pleine.
- [x] Déposer un stratagème existant sur un autre intervertit leurs positions.
- [x] Les séparations entre secteurs permettent toujours d’insérer ou de réordonner.
- [x] Corbeille visible uniquement pendant le glisser-déposer : retire une entrée existante ou annule un ajout du catalogue.
- [x] Indication « Remplacer » ou « Intervertir » au survol et cible surlignée.
- [x] Tests des remplacements, échanges, suppressions/annulations et zones de dépôt.

## Livré

- [x] Renommage visible en SERAPOD (Super Earth Radial Accessibility Program for Orbital Deployment), y compris fenêtre, exports et exécutable.
- [x] Conservation des profils existants lors du changement de nom.
- [x] Nom de l’application en jaune ; logo radial inchangé.
- [x] Versions complètes SemVer, affichage depuis le manifeste et exécutable nommé automatiquement SERAPOD-<version>.exe.
- [x] Profil actif intégré à côté du titre de l’éditeur.
- [x] Compteur de secteurs déplacé à la place de « clockwise ».
- [x] Halo et cercles décoratifs derrière la roue de l’éditeur.
- [x] Effet verre activé par défaut pour la simulation et l’overlay : surfaces translucides, reflets et lueur, désactivables ; transparence réglable de 0 à 80 %.
- [x] Glisser-déposer du catalogue vers la roue ou la liste, avec séparateur lumineux.
- [x] Réordonnancement depuis la roue et dans la liste ; rangement dans un dossier et retour vers un parent par le fil de navigation.
- [x] Refus des cycles, dépassements de capacité et de profondeur avant toute modification des données.
- [x] Icône de dossier cohérente entre roue et liste.
- [x] Signature au-dessus du nom/version dans la sidebar, cœur #FF0080.
- [x] Icône Profils remplacée par une icône de couches SVG.
- [x] Statuts de connexion plus précis et bouton de redémarrage de l’overlay.

## Vérifié

- [x] Contrôle Svelte sans erreurs ni avertissements.
- [x] 15 tests JavaScript et 6 tests Rust réussis.
- [x] Glisser-déposer réel dans l’aperçu navigateur : ajout à une position choisie, roue vers dossier et réordonnancement de liste.
- [x] Nouvelle disposition dans l’application Windows et rendu de l’effet verre dans la simulation.

## Vérifications en jeu restantes

- [x] Corriger la détection de fenêtre dans l'application complète (0.7.1).
- [x] Valider F1 et l'overlay par-dessus le jeu.
- [ ] Valider la saisie des séquences, l’absence de double déclenchement et le comportement de la caméra.
- [ ] Confirmer la détection pendant les transitions entre le jeu et les autres fenêtres, ainsi qu'après un redémarrage du jeu.

L’effet verre est un rendu translucide : aucun flou réel des pixels du jeu n’est implémenté. Le test Rust de diagnostic en direct est manuel et exclu de la suite automatique.
