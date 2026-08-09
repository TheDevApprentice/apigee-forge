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

- [x] En mode Live, n’afficher que l’écran de connexion tant que la session n’est pas authentifiée ; le shell/navigation actuels doivent encore être masqués.
- [x] Afficher la configuration manquante sans exposer de secret.
- [x] Lancer le flux Google OAuth desktop via la commande Tauri.
- [x] Afficher les spinners de restauration/OAuth et les erreurs dédiées ; l’extraction du shell visuel en écran Login dédié reste à finaliser.
- [x] Après connexion, afficher l’écran de sélection d’organisation avant les environnements/proxies.
- [x] Tester le parcours avec un provider OAuth fake ; le test réel Google reste manuel.

#### M6-Bis-10 — Écran Demo et Dashboard partagé

- [x] Afficher un badge `Demo` persistant et non ambigu.
- [ ] Charger le dataset Demo sans appel réseau — reporté post-MVP avec le seeding.
- [x] Afficher les cartes organisation/environnement/proxies et le rôle Demo.
- [x] Afficher le badge `Live` et l’identité Google en mode Live.
- [x] Partager les composants de liste et d’état entre Demo et Live.
- [x] Préserver les états loading, empty, error et success.

### Catégorie E — Tests et critères d’acceptation

#### M6-Bis-11 — Tests automatisés du parcours

- [x] Ajouter tests Vitest du sélecteur Demo/Live.
- [x] Ajouter tests du démarrage Demo sans réseau ni OAuth.
- [x] Ajouter tests du démarrage Live bloqué sur Login.
- [x] Ajouter tests Live login success/error avec provider Tauri fake.
- [x] Ajouter tests de changement de mode et nettoyage de session.
- [x] Ajouter tests Tauri/Rust du store SQLCipher et des commandes mode.
- [x] Vérifier absence de secrets dans fixtures, logs et sorties de test.

#### M6-Bis-12 — Checkpoint de stabilité GUI

- [x] Exécuter tests workspace, Clippy, tests frontend et build Tauri.
- [x] Vérifier manuellement le runtime Demo sans dataset final : lancement, store local et retour au sélecteur.
- [x] Vérifier manuellement Live sans réseau réel : écran Login et erreurs contrôlées.
- [ ] Vérifier manuellement Live avec le compte Apigee déjà provisionné : login Google, sélection org/env, dashboard et proxies.
- [x] Reporter le tutoriel Demo et le seeding complet au checkpoint post-MVP dédié.
- [x] Vérifier qu’aucun changement ne nécessite de refaire M3 ou la validation CLI M4.
- [x] Marquer M6-Bis terminé dans `ROADMAP.md` après validation manuelle Google et Demo.

Commit prévu :

```text
docs(m6-bis): record Demo Cloud GUI stability checkpoint
```

---

## 5. Étapes supplémentaires réalisées pendant la stabilisation

Ces étapes ne figuraient pas dans le périmètre M6-Bis initial. Elles ont été ajoutées au fil des validations manuelles afin de conserver une continuité entre les décisions réellement prises, les commits de stabilisation et la préparation de M7.

### M6-Bis-13 — OAuth desktop Google réel

- [x] Charger explicitement la configuration OAuth depuis `gui/.env`.
- [x] Supporter le `client_secret` optionnel pour les configurations Google desktop.
- [x] Utiliser le redirect URI loopback desktop conforme à Google.
- [x] Restaurer la session sans relancer inutilement le navigateur.
- [x] Réutiliser l’identité et le token OAuth déjà en cache lors du chargement des rôles.
- [x] Conserver les erreurs OAuth structurées sans exposer de secret.

### M6-Bis-14 — Dashboard, navigation et catalogue des proxies

- [x] Transformer Dashboard, Templates, Proxies, Deployments et Settings en vues distinctes.
- [x] Naviguer vers l’onglet Proxies lorsqu’un proxy est sélectionné depuis le Dashboard.
- [x] Afficher tous les proxies du contexte organisation/environnement courant dans Proxies.
- [x] Ajouter les filtres `All`, `Deployed` et `Not deployed`.
- [x] Afficher les révisions et leur statut réel pour l’environnement sélectionné.
- [x] Retirer le détail du proxy du Dashboard afin d’éviter le scroll inutile.

### M6-Bis-15 — Statuts de déploiement et détail de révision

- [x] Enrichir les révisions avec le statut Apigee réel (`ACTIVE`, `PROGRESSING`, `ERROR`, etc.).
- [x] Ajouter le port `ApigeeRevisionGateway`.
- [x] Brancher `organizations/{org}/apis/{proxy}/revisions/{revision}` derrière une commande Tauri.
- [x] Permettre l’expansion d’une ligne de révision dans la page Proxies.
- [x] Afficher les états loading et erreur de récupération du détail.
- [ ] Compléter le mapping métier de tous les champs de la réponse lorsque le contrat de détail Apigee sera stabilisé.

### M6-Bis-16 — Contexte workspace global et démarrage assisté

- [x] Déplacer les sélecteurs organisation/environnement dans le topbar global.
- [x] Sélectionner automatiquement la première organisation accessible après authentification.
- [x] Sélectionner automatiquement le premier environnement accessible.
- [x] Déclencher automatiquement le chargement des proxies après sélection du contexte.
- [x] Garder le changement de contexte accessible depuis toutes les pages.
- [x] Corriger le scroll pour maintenir sidebar, topbar et titre de page visibles.

### M6-Bis-17 — Dashboard de synthèse et Settings

- [x] Ajouter les cartes de synthèse proxies, révisions et déploiements.
- [x] Ajouter les états vides explicites de Templates, Deployments et Settings.
- [x] Afficher version, build, stack et branche dans Settings.
- [x] Afficher le contexte Live/Demo et la session workspace dans Settings.
- [x] Ajouter les liens GitHub, documentation Apigee, API Management et support.
- [ ] Ajouter les statistiques Analytics Apigee (`environments.stats`) dans une passe dédiée.
- [ ] Ajouter les informations détaillées d’organisation et d’environnement dans une passe dédiée.

### M6-Bis-18 — Profil utilisateur et sidebar

- [x] Ajouter l’indicateur de session connecté/non connecté dans la sidebar.
- [x] Ajouter un avatar circulaire avec fallback initiales.
- [x] Récupérer depuis Google l’email, prénom, nom, nom complet et photo optionnelle.
- [x] Gérer les photos absentes, invalides ou inaccessibles sans afficher d’image cassée.
- [x] Ajouter une bulle utilisateur au survol et au focus clavier.
- [x] Ajouter le profil général dans Settings sans exposer les rôles détaillés ni les credentials.

### M6-Bis-19 — Déconnexion OAuth réelle

- [x] Ajouter un bouton `Sign out` dans la bulle utilisateur de la sidebar.
- [x] Supprimer le refresh token du trousseau OS lors de la déconnexion.
- [x] Vider l’access token et l’identité conservés en mémoire.
- [x] Nettoyer le contexte organisation/environnement/proxies côté Vue.
- [x] Revenir à l’écran Live Login après déconnexion.
- [x] Ne pas afficher le bouton de déconnexion en mode Demo.

### M6-Bis-20 — Décision de sortie vers M7

- [x] Considérer M6-Bis de base comme atteint : modes, OAuth, contexte, navigation, catalogue proxy, états UX et stabilité du shell sont en place.
- [x] Conserver l’éditeur XML/JSON/YAML complet hors de M6-Bis.
- [x] Conserver les actions de déploiement GUI hors de M6-Bis.
- [x] Réserver le polish visuel avancé, les statistiques et les réglages éditables supplémentaires aux étapes suivantes.

---

## 6. Critères d’acceptation M6-Bis

M6-Bis est terminé lorsque :

1. le premier écran permet de choisir explicitement Demo ou Live ;
2. Demo démarre sans compte, sans réseau et avec un store local prêt ; le dataset complet est post-MVP ;
3. Live ne montre pas le dashboard avant une authentification Google réussie ;
4. la configuration OAuth et les erreurs sont compréhensibles sans exposer de secret ;
5. les deux modes utilisent les mêmes ports/use cases côté `core` ;
6. l’organisation et l’environnement Cloud sont sélectionnables depuis le topbar, avec un premier contexte chargé automatiquement ;
7. le changement de mode ne mélange jamais les données Demo et Cloud ;
8. le fichier SQLCipher est local, chiffré et absent du repository ;
9. le parcours est couvert par tests avec doubles, sans dépendance réseau CI ;
10. le GUI est une base stable pour M7, sans commencer l’éditeur visuel ni le déploiement GUI ;
11. l’utilisateur peut se déconnecter réellement de Google et revenir à l’écran Live Login.
