# Apigee Forge — Roadmap de démarrage M6-Bis

*Jalon de stabilisation du GUI avant M7. M6-Bis ne constitue pas un polish visuel : il rend le mode d’utilisation explicite, persistant, testable et réellement exploitable avec ou sans compte Apigee.*

---

## 1. Objectif M6-Bis

Le GUI doit proposer deux parcours clairement séparés dès le démarrage :

```text
Mode Demo  → données locales seedées → dashboard local
Mode Cloud → configuration OAuth → Sign in with Google → organisation → dashboard Apigee
```

Le mode courant ne doit jamais être deviné depuis une erreur réseau ou l’absence de credentials.

### Mode Demo

- utilisable sans compte Google ni organisation Apigee ;
- données locales réalistes mais explicitement fictives ;
- proxies, organisations, environnements et états de déploiement fournis par un scénario local ;
- données persistées dans un fichier SQLite chiffré SQLCipher ;
- aucune requête réseau ;
- badge et libellé `Demo` visibles partout où une confusion avec Apigee réel serait possible ;
- bouton permettant de revenir au sélecteur de mode.

### Mode Cloud

- parcours initial bloquant sur une connexion Google explicite ;
- OAuth desktop réel via `OAuthDesktopAuthProvider` du `core` ;
- refresh token conservé uniquement dans le trousseau OS ;
- sélection explicite de l’organisation et de l’environnement après authentification ;
- appels Apigee via les use cases et ports du `core`, jamais via `reqwest` dans Vue ;
- aucune organisation ou donnée Demo injectée silencieusement en cas d’erreur ;
- dashboard réel en lecture seule dans M6-Bis ; le déploiement GUI reste M8.

Le nom retenu est **Demo** / **Cloud** : `Offline` décrivait le transport mais pas l’intention utilisateur, tandis que `Cloud` distingue clairement le compte Apigee réel du scénario local.

---

## 2. Décisions d’architecture

- Le mode sélectionné est un état applicatif explicite (`Demo` ou `Cloud`), pas un détail d’authentification.
- Le mode est persisté via un `LocalStateStore` ; il est restauré au démarrage mais l’utilisateur peut le changer explicitement.
- Le frontend ne choisit jamais directement une implémentation de gateway ou de stockage.
- Le composition root Tauri choisit les implémentations :
  - Demo : `DemoLocalStateStore` + `InMemoryApigeeGateway` ou gateway Demo dédiée ;
  - Cloud : `SqlCipherLocalStore` + `OAuthDesktopAuthProvider` + `ReqwestApigeeGateway`.
- Le trousseau OS contient les tokens OAuth et la clé de chiffrement SQLCipher ; la base ne contient jamais de token OAuth.
- Les templates restent des fichiers versionnables et ne sont pas stockés dans SQLCipher.
- Le frontend reçoit des DTO explicitement marqués par leur contexte (`mode: demo|cloud`) afin d’éviter de présenter une fixture comme une donnée réelle.
- Aucun compte Apigee réel n’est requis pour le développement ou les tests M6-Bis ; le compte d’évaluation sert uniquement au checkpoint manuel déjà validé côté CLI.
- Les tâches sont atomiques dans chaque catégorie, mais regroupées en étapes fonctionnelles pour garder M6-Bis lisible.

---

## 3. État initial et points à corriger

- Le shell M6 affiche actuellement `Offline preview` sans sélecteur de mode.
- Le bouton `Sign in with Google` existe mais la configuration OAuth vient seulement de variables d’environnement et l’écran ne présente pas encore un vrai choix Cloud/Demo.
- L’absence de configuration OAuth produit une erreur, mais le parcours ne distingue pas assez un Demo volontaire d’un Cloud non configuré.
- `LocalStateStore` existe comme port dans `core`, sans implémentation SQLCipher utilisable par le GUI.
- `InMemoryApigeeGateway` existe pour les tests, mais n’est pas encore une source de scénario Demo persistant.
- Les commandes Tauri M6-02 sont branchées sur un état Cloud optionnel et doivent devenir dépendantes du mode sélectionné.
- Les composants Vue sont encore principalement réunis dans `App.vue` ; M6-Bis doit extraire les écrans et les responsabilités sans commencer l’éditeur M7.

---

## 4. Étapes atomiques regroupées par catégorie

### M6-Bis-00 — Baseline et contrat de stabilisation

**Architecture et documentation**

- [ ] Confirmer la fusion de M6 et de la validation Apigee réelle dans `dev`.
- [ ] Créer la branche `feature/m6-bis-gui` depuis `dev`.
- [ ] Ajouter cette roadmap dans `STRUCTURE.md` et `PROMPT.md`.
- [ ] Figer les noms utilisateur `Demo` et `Cloud`.
- [ ] Documenter ce qui reste hors M6-Bis : éditeur M7, déploiement GUI M8, polish M9.
- [ ] Vérifier la baseline `cargo test`, `cargo clippy`, `npm test` et `npm run build`.

Commit prévu :

```text
docs(m6-bis): define Demo and Cloud GUI roadmap
```

### Catégorie A — Contrat de mode et état applicatif

#### M6-Bis-01 — Modèle de domaine Demo/Cloud

- [ ] Ajouter un type de mode partagé et sérialisable : `Demo` ou `Cloud`.
- [ ] Définir les états de session : mode sélectionné, authentification, organisation, environnement et erreur.
- [ ] Interdire une session Cloud authentifiée sans organisation explicitement sélectionnée.
- [ ] Ajouter le mode aux DTO Tauri et aux états Vue.
- [ ] Tester les transitions valides et les transitions interdites.

#### M6-Bis-02 — Persistance du mode

- [ ] Définir les clés persistées du mode, de l’organisation et de l’environnement.
- [ ] Restaurer le dernier mode sans restaurer une sélection Cloud invalide.
- [ ] Prévoir un changement explicite de mode avec nettoyage de l’état de session non applicable.
- [ ] Ajouter tests de persistance avec un store fake.

### Catégorie B — Stockage local Demo

#### M6-Bis-03 — Implémentation SQLCipher du LocalStateStore

- [ ] Vérifier la disponibilité et la compatibilité Rust de `rusqlite` avec `bundled-sqlcipher` et le toolchain du projet.
- [ ] Implémenter `SqlCipherLocalStore` dans `core/infra` derrière le port `LocalStateStore`.
- [ ] Placer le fichier dans le répertoire de données applicatif Tauri, jamais dans le repository.
- [ ] Générer ou récupérer la clé SQLCipher via le trousseau OS.
- [ ] Refuser proprement le démarrage Demo si le store ne peut pas être ouvert ou déchiffré.
- [ ] Tester création, lecture, écriture, suppression, migration minimale et erreur de clé.

#### M6-Bis-04 — Dataset Demo seedé

- [ ] Définir un dataset local versionné et explicitement fictif.
- [ ] Inclure au minimum une organisation Demo, deux environnements, deux proxies et un historique de statuts.
- [ ] Seed uniquement à la première initialisation, sans écraser les modifications locales.
- [ ] Ajouter un reset Demo explicite et confirmé par l’utilisateur.
- [ ] Vérifier qu’aucun nom, token ou identifiant du compte Apigee réel ne figure dans le dataset.

### Catégorie C — Composition root et commandes Tauri

#### M6-Bis-05 — Runtime Demo/Cloud

- [ ] Remplacer l’état Cloud optionnel actuel par une composition root choisie à partir du mode explicite.
- [ ] Brancher le gateway Demo sans réseau.
- [ ] Brancher le gateway Cloud réel avec OAuth desktop et `ReqwestApigeeGateway`.
- [ ] Garder les deux branches derrière les mêmes ports/use cases.
- [ ] Empêcher toute instanciation du service account dans le GUI interactif.
- [ ] Tester les deux compositions avec des doubles et vérifier l’absence de fuite de type infrastructure vers Vue.

#### M6-Bis-06 — Commandes mode/authentification

- [ ] Ajouter `get_app_mode` et `set_app_mode`.
- [ ] Ajouter `get_session_context` avec le mode toujours présent.
- [ ] Rendre `auth_login` valide uniquement en Cloud.
- [ ] Rendre `auth_logout` et le changement de mode idempotents.
- [ ] Retourner une erreur structurée si une commande Cloud est appelée en Demo ou inversement.
- [ ] Tester les commandes avec Tauri state fake.

#### M6-Bis-07 — Sélection Cloud et données Dashboard

- [ ] Exposer organisations et environnements via les use cases `core` existants.
- [ ] Charger les proxies uniquement après sélection explicite organisation + environnement.
- [ ] Conserver un contrat identique Demo/Cloud pour le frontend.
- [ ] Exposer le rôle et la source de données sans afficher de credential.
- [ ] Tester succès, vide, permission refusée, expiration de session et erreur réseau.

### Catégorie D — Parcours UI stable

#### M6-Bis-08 — Écran de sélection Demo/Cloud

- [ ] Afficher cet écran avant Login ou Dashboard lorsqu’aucun mode n’est persisté.
- [ ] Présenter Demo et Cloud avec leurs conséquences compréhensibles.
- [ ] Ne jamais lancer OAuth automatiquement.
- [ ] Permettre de changer de mode depuis les préférences ou la barre de contexte.
- [ ] Tester navigation clavier, focus, libellés et états de sélection.

#### M6-Bis-09 — Écran Cloud Login réel

- [ ] En mode Cloud, n’afficher que l’écran de connexion tant que la session n’est pas authentifiée.
- [ ] Afficher la configuration manquante sans exposer de secret.
- [ ] Lancer le flux Google OAuth desktop via la commande Tauri.
- [ ] Afficher loading, succès, annulation navigateur, trousseau indisponible et erreur réseau.
- [ ] Après connexion, afficher uniquement l’écran de sélection d’organisation.
- [ ] Tester le parcours avec un provider OAuth fake ; le test réel Google reste manuel.

#### M6-Bis-10 — Écran Demo et Dashboard partagé

- [ ] Afficher un badge `Demo` persistant et non ambigu.
- [ ] Charger le dataset Demo sans appel réseau.
- [ ] Afficher les cartes organisation/environnement/proxies et le rôle Demo.
- [ ] Afficher le badge `Cloud` et l’identité Google en mode Cloud.
- [ ] Partager les composants de liste et d’état entre Demo et Cloud.
- [ ] Préserver les états loading, empty, error et success.

### Catégorie E — Tests et critères d’acceptation

#### M6-Bis-11 — Tests automatisés du parcours

- [ ] Ajouter tests Vitest du sélecteur Demo/Cloud.
- [ ] Ajouter tests du démarrage Demo sans réseau ni OAuth.
- [ ] Ajouter tests du démarrage Cloud bloqué sur Login.
- [ ] Ajouter tests Cloud login success/error avec provider Tauri fake.
- [ ] Ajouter tests de changement de mode et nettoyage de session.
- [ ] Ajouter tests Tauri/Rust du store SQLCipher et des commandes mode.
- [ ] Vérifier absence de secrets dans fixtures, logs et sorties de test.

#### M6-Bis-12 — Checkpoint de stabilité GUI

- [ ] Exécuter tests workspace, Clippy, tests frontend et build Tauri.
- [ ] Vérifier manuellement Demo : lancement, seed, navigation, reset et retour au sélecteur.
- [ ] Vérifier manuellement Cloud sans réseau réel : écran Login et erreurs contrôlées.
- [ ] Vérifier manuellement Cloud avec le compte Apigee déjà provisionné : login Google, sélection org/env, dashboard et proxies.
- [ ] Vérifier qu’aucun changement ne nécessite de refaire M3 ou la validation CLI M4.
- [ ] Marquer M6-Bis terminé dans `ROADMAP.md`.

Commit prévu :

```text
docs(m6-bis): record Demo Cloud GUI stability checkpoint
```

---

## 5. Critères d’acceptation M6-Bis

M6-Bis est terminé lorsque :

1. le premier écran permet de choisir explicitement Demo ou Cloud ;
2. Demo démarre sans compte, sans réseau et avec des données locales persistées ;
3. Cloud ne montre pas le dashboard avant une authentification Google réussie ;
4. la configuration OAuth et les erreurs sont compréhensibles sans exposer de secret ;
5. les deux modes utilisent les mêmes ports/use cases côté `core` ;
6. l’organisation et l’environnement Cloud sont toujours sélectionnés explicitement ;
7. le changement de mode ne mélange jamais les données Demo et Cloud ;
8. le fichier SQLCipher est local, chiffré et absent du repository ;
9. le parcours est couvert par tests avec doubles, sans dépendance réseau CI ;
10. le GUI est une base stable pour M7, sans commencer l’éditeur visuel ni le déploiement GUI.
