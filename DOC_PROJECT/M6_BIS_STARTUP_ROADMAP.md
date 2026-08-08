# Apigee Forge — Roadmap de démarrage M6-Bis

*Jalon de stabilisation du GUI avant M7. M6-Bis ne constitue pas un polish visuel : il rend le mode d’utilisation explicite, persistant, testable et réellement exploitable avec ou sans compte Apigee.*

---

## 1. Objectif M6-Bis

Le GUI doit proposer deux parcours clairement séparés dès le démarrage :

```text
Mode Demo → données locales → dashboard local
Mode Live → configuration OAuth → Sign in with Google → organisation → dashboard Apigee
```

Le mode par défaut est **Live**. Le mode Demo doit toujours être sélectionné explicitement ; il ne doit jamais être activé automatiquement à cause d’une erreur réseau ou de credentials absents.

### Mode Demo

- utilisable sans compte Google ni organisation Apigee ;
- branché sur le gateway Demo et les mêmes contrats/use cases que Live ;
- données persistées dans un fichier SQLite chiffré SQLCipher ;
- aucune requête réseau ;
- badge et libellé `Demo` visibles partout où une confusion avec Apigee réel serait possible ;
- dataset réaliste et tutoriel complet reportés à la fin du MVP, après stabilisation de l’UX ;
- bouton permettant de revenir au sélecteur de mode.

### Mode Live

- parcours initial bloquant sur une connexion Google explicite ;
- OAuth desktop réel via `OAuthDesktopAuthProvider` du `core` ;
- refresh token conservé uniquement dans le trousseau OS ;
- sélection explicite de l’organisation et de l’environnement après authentification ;
- appels Apigee via les use cases et ports du `core`, jamais via `reqwest` dans Vue ;
- aucune organisation ou donnée Demo injectée silencieusement en cas d’erreur ;
- dashboard réel en lecture seule dans M6-Bis ; le déploiement GUI reste M8.

**Décision de nommage** : `Live` est le libellé utilisateur retenu pour le mode réel, y compris avec l’organisation Apigee d’évaluation. Le type Rust `AppMode::Cloud` peut rester temporairement le nom technique interne afin d’éviter une migration de contrat inutile ; l’UI et la documentation utilisent `Live`. `Offline` décrivait le transport, pas l’intention utilisateur.

---

## 2. Décisions d’architecture

- Le mode sélectionné est un état applicatif explicite (`Demo` ou `Live` côté UX, `Demo` ou `Cloud` dans le contrat technique temporaire), pas un détail d’authentification.
- Le mode par défaut est Live/Cloud ; Demo n’est jamais un fallback implicite.
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

- [x] Confirmer la fusion de M6 et de la validation Apigee réelle dans `dev`.
- [x] Créer la branche `feature/m6-bis-gui` depuis `dev`.
- [x] Ajouter cette roadmap dans `STRUCTURE.md` et `PROMPT.md`.
- [x] Figer les noms utilisateur `Demo` et `Live` (`Cloud` reste le nom technique interne temporaire).


Commit prévu :

```text
docs(m6-bis): define Demo and Cloud GUI roadmap
```

### Catégorie A — Contrat de mode et état applicatif

#### M6-Bis-01 — Modèle de domaine Demo/Cloud

- [x] Ajouter un type de mode partagé et sérialisable : `Demo` ou `Cloud`.
- [x] Définir les états de session : mode sélectionné, authentification, organisation, environnement et erreur.
- [x] Interdire l’accès au contexte Dashboard Cloud sans organisation explicitement sélectionnée.
- [x] Ajouter le mode aux DTO Tauri et aux états Vue.
- [x] Tester les transitions valides et les transitions interdites.

#### M6-Bis-02 — Persistance du mode

- [x] Définir les clés persistées du mode, de l’organisation et de l’environnement.
- [x] Restaurer le dernier mode sans restaurer une sélection Cloud invalide.
- [x] Prévoir un changement explicite de mode avec nettoyage de l’état de session non applicable.
- [x] Ajouter tests de persistance avec un store fake.

### Catégorie B — Stockage local Demo

#### M6-Bis-03 — Implémentation SQLCipher du LocalStateStore

- [x] Vérifier la disponibilité et la compatibilité Rust de `rusqlite` avec `bundled-sqlcipher` et Rust 1.85.1. Sous Windows, le build utilise OpenSSL installé avec `OPENSSL_DIR` et `OPENSSL_NO_VENDOR=1`.
- [x] Implémenter `SqlCipherLocalStore` dans `core/infra` derrière le port `LocalStateStore`.
- [x] Placer le fichier dans le répertoire de données applicatif Tauri, jamais dans le repository.
- [x] Générer ou récupérer la clé SQLCipher via le trousseau OS.
- [x] Refuser proprement le démarrage Demo si le store ne peut pas être ouvert ou déchiffré.
- [x] Tester création, lecture, écriture, suppression, migration minimale et erreur de clé.

#### M6-Bis-04 — Dataset Demo seedé — reporté post-MVP

**Décision de cadrage** : le seeding complet n’est pas un prérequis pour stabiliser le MVP. Le code du gateway Demo, du store SQLCipher et des contrats communs est préparé, mais le dataset réaliste et le tutoriel ne seront pas construits maintenant.

Le seeding sera réalisé après les fonctionnalités principales, lorsque l’UX sera maîtrisée. Il pourra alors fournir une démonstration cohérente et presque identique au comportement réel, sans gaspiller du temps sur des fixtures qui risqueraient de suivre une UI encore instable.

- [ ] Définir un dataset local versionné et explicitement fictif — reporté.
- [ ] Inclure une organisation Demo, deux environnements, deux proxies et un historique de statuts — reporté.
- [ ] Seed uniquement à la première initialisation — reporté.
- [ ] Ajouter un reset Demo explicite et confirmé par l’utilisateur — reporté.
- [ ] Vérifier qu’aucun nom, token ou identifiant du compte Apigee réel ne figure dans le dataset — à appliquer lors du seeding.

Ce report ne bloque pas les contrats Demo/Live ni la suite M6-Bis ; il déplace uniquement le contenu de démonstration à la fin du MVP.

### Catégorie C — Composition root et commandes Tauri

#### M6-Bis-05 — Runtime Demo/Cloud

- [x] Remplacer l’état Cloud optionnel actuel par une composition root choisie à partir du mode explicite.
- [x] Brancher le gateway Demo sans réseau.
- [x] Brancher le gateway Cloud réel avec OAuth desktop et `ReqwestApigeeGateway`.
- [x] Garder les deux branches derrière les mêmes ports/use cases.
- [x] Empêcher toute instanciation du service account dans le GUI interactif.
- [x] Tester la compilation des deux compositions et vérifier l’absence de fuite de type infrastructure vers Vue.

#### M6-Bis-06 — Commandes mode/authentification

- [x] Ajouter `get_app_mode` et `set_app_mode`.
- [x] Ajouter `session_status` avec le mode toujours présent.
- [x] Rendre `auth_login` valide uniquement en Cloud.
- [x] Rendre `auth_logout` et le changement de mode idempotents.
- [x] Retourner une erreur structurée si une commande Cloud est appelée en Demo ou inversement.
- [ ] Ajouter le test Tauri state fake complet ; les transitions domaine et le bridge DTO sont déjà testés.

#### M6-Bis-07 — Sélection Cloud et données Dashboard

- [x] Exposer organisations et environnements via les use cases `core` existants.
- [x] Charger les proxies uniquement après sélection explicite organisation + environnement.
- [x] Conserver un contrat identique Demo/Cloud pour le frontend.
- [x] Exposer le rôle et la source de données sans afficher de credential.
- [ ] Compléter les tests succès, vide, permission refusée, expiration de session et erreur réseau.

### Catégorie D — Parcours UI stable

#### M6-Bis-08 — Sélecteur de mode Demo/Live

- [x] Afficher le contrôle de mode dès le démarrage ; Live est le mode par défaut.
- [x] Présenter Demo et Live avec leurs conséquences compréhensibles.
- [x] Ne jamais lancer OAuth automatiquement.
- [x] Permettre de changer de mode depuis la barre de contexte.
- [ ] Tester navigation clavier, focus, libellés et états de sélection dédiés.

#### M6-Bis-09 — Écran Live Login réel

- [ ] En mode Live, n’afficher que l’écran de connexion tant que la session n’est pas authentifiée ; le shell/navigation actuels doivent encore être masqués.
- [x] Afficher la configuration manquante sans exposer de secret.
- [x] Lancer le flux Google OAuth desktop via la commande Tauri.
- [ ] Afficher un vrai écran d’accueil, les spinners de restauration/OAuth et les erreurs dédiées.
- [x] Après connexion, afficher l’écran de sélection d’organisation avant les environnements/proxies.
- [x] Tester le parcours avec un provider OAuth fake ; le test réel Google reste manuel.

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
- [ ] Vérifier manuellement le runtime Demo sans dataset final : lancement, store local et retour au sélecteur.
- [ ] Vérifier manuellement Live sans réseau réel : écran Login et erreurs contrôlées.
- [ ] Vérifier manuellement Live avec le compte Apigee déjà provisionné : login Google, sélection org/env, dashboard et proxies.
- [ ] Reporter le tutoriel Demo et le seeding complet au checkpoint post-MVP dédié.
- [ ] Vérifier qu’aucun changement ne nécessite de refaire M3 ou la validation CLI M4.
- [ ] Marquer M6-Bis terminé dans `ROADMAP.md`.

Commit prévu :

```text
docs(m6-bis): record Demo Cloud GUI stability checkpoint
```

---

## 5. Critères d’acceptation M6-Bis

M6-Bis est terminé lorsque :

1. le premier écran permet de choisir explicitement Demo ou Live ;
2. Demo démarre sans compte, sans réseau et avec un store local prêt ; le dataset complet est post-MVP ;
3. Live ne montre pas le dashboard avant une authentification Google réussie ;
4. la configuration OAuth et les erreurs sont compréhensibles sans exposer de secret ;
5. les deux modes utilisent les mêmes ports/use cases côté `core` ;
6. l’organisation et l’environnement Cloud sont toujours sélectionnés explicitement ;
7. le changement de mode ne mélange jamais les données Demo et Cloud ;
8. le fichier SQLCipher est local, chiffré et absent du repository ;
9. le parcours est couvert par tests avec doubles, sans dépendance réseau CI ;
10. le GUI est une base stable pour M7, sans commencer l’éditeur visuel ni le déploiement GUI.
