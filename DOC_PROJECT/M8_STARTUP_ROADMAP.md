# Apigee Forge — Roadmap de démarrage M8

*Jalon de création de proxies, de déploiement de révisions et de suivi depuis le GUI. M8 relie l’éditeur de templates M7 au moteur de génération de bundle et aux opérations Apigee déjà portées par `core`, avec une confirmation explicite avant chaque mutation distante. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M8

Le GUI doit distinguer deux parcours :

1. **Créer un proxy** à partir d’un template ou d’une autre source : préparer les entrées, générer le bundle, l’uploader dans Apigee et obtenir un proxy ou une nouvelle révision non déployée.
2. **Déployer un proxy existant** : sélectionner une révision déjà présente dans Apigee, confirmer la cible puis suivre son déploiement.

Parcours cible :

```text
Templates / autre source → Create proxy
                                  ↓
                       OpenAPI + template + cible
                                  ↓
                       Preview de création
                                  ↓
                     Générer le bundle local
                                  ↓
                       Confirmer l’upload
                                  ↓
                  Importer → proxy / révision créée
                                  ↓
                         Statut : Not deployed

Proxies → sélectionner proxy + révision
       → Review de déploiement
       → Confirmation explicite
       → Déployer → interroger le statut → résultat
```

M8 doit conserver les contrats existants :

- `core::use_cases::GenerateProxyBundleUseCase` pour la génération locale ;
- `ImportProxyBundleUseCase` pour créer un proxy ou une nouvelle révision à partir d’un bundle uploadé ;
- `DeployProxyUseCase` uniquement pour déployer une révision déjà présente dans Apigee sur un environnement ;
- `GetDeploymentStatusUseCase` pour la lecture du statut ;
- `TemplateRepository` et le format de template validé par `core` ;
- les modes Live et Demo du GUI ;
- les conventions de sécurité, d’erreurs et de sérialisation Tauri.

### Hors périmètre M8

- analytics Apigee, métriques de trafic et tableau de bord d’exploitation ;
- rollback automatique ou stratégie canary/blue-green ;
- undeploy, suppression de proxy ou gestion avancée des révisions ;
- édition du template ou ajout de nouvelles policies, déjà traité par M7 ;
- import inversé d’un proxy Apigee existant ;
- support d’autres gateways ;
- polish visuel final et tokens définitifs, prévus en M9 ;
- validation réelle permanente dans la suite automatisée : elle reste manuelle et séparée, sans credential commité.

---

## 2. État de départ

### Déjà disponible

- use cases `GenerateProxyBundleUseCase`, `ImportProxyBundleUseCase`, `DeployProxyUseCase` et `GetDeploymentStatusUseCase` dans `core` ;
- ports `ApigeeProxyBundleGateway` et `ApigeeDeploymentGateway` ;
- implémentations `ReqwestApigeeGateway` et `InMemoryApigeeGateway` ;
- modèles `Deployment`, `DeploymentStatus` et `ProxyRevision` ;
- mapping des endpoints Apigee dans `APIGEE_API_MAP.md` ;
- commandes Tauri de lecture des organisations, environnements, proxies et statuts de révisions ;
- session Live/Demo et sélection d’organisation/environnement ;
- éditeur M7 capable de produire et sauvegarder un template compatible CLI ;
- génération et packaging de bundle déjà testés côté Rust et CLI ;
- écrans GUI `Proxies` et `Deployments` présents ; la création de proxy doit être préparée depuis `Proxies`, tandis que `Deployments` reste réservé aux révisions existantes.

### Gaps à traiter

- contrat DTO Rust ↔ Vue pour les paramètres et résultats de génération/import/déploiement ;
- résolution sûre de la spec OpenAPI et des répertoires temporaires de génération ;
- commandes Tauri de génération locale, import et déploiement ;
- confirmation explicite avant chaque opération distante mutative ;
- état frontend du workflow asynchrone et protection contre les doubles soumissions ;
- polling borné et annulable du statut de déploiement ;
- interface de création/upload, progression et résultat dans le GUI ;
- interface de review et de sélection d’une révision avant déploiement ;
- tests de use cases/commandes/composable avec doubles Demo et sans réseau ;
- procédure de validation manuelle Live sans exposer de secrets.

---

## 3. Décisions d’architecture M8

- La génération du bundle reste une opération locale et ne doit jamais appeler Apigee.
- La création/upload et le déploiement sont deux mutations distinctes : l’upload crée un proxy ou une révision, mais ne la déploie jamais.
- Le GUI ne doit jamais présenter une révision importée comme déployée ; son état initial est `Not deployed`.
- La logique métier reste dans `core`; les commandes Tauri valident les DTO, résolvent les dépendances du composition root et délèguent aux use cases.
- Le GUI utilise les mêmes commandes et DTO en mode Demo et en mode Live ; seul le gateway injecté change.
- L’upload nécessite une confirmation explicite contenant au minimum organisation et nom de proxy ; le déploiement nécessite une seconde confirmation contenant organisation, environnement, proxy et révision ciblés.
- Le nom du proxy, l’environnement, la révision et les chemins de génération sont validés côté Rust avant tout appel réseau ou écriture.
- Les fichiers générés sont stockés dans un répertoire de travail temporaire contrôlé ; aucun chemin local sensible ne doit être renvoyé au frontend ou affiché dans une erreur.
- Le suivi utilise un polling borné, avec intervalle raisonnable, arrêt sur `Succeeded`/`Failed`, annulation lors de la navigation et protection contre les requêtes concurrentes obsolètes.
- Les erreurs sont transformées en codes GUI stables et sûrs ; aucun token, header Authorization, corps HTTP ou credential ne doit atteindre Vue, les logs ou les DTO.
- M8 ne modifie pas les permissions IAM et ne fournit pas de mécanisme de contournement d’une autorisation Apigee refusée.

### Choix du suivi temps réel

Pour le MVP, le GUI utilisera un polling Tauri vers `GetDeploymentStatusUseCase` plutôt qu’un WebSocket ou un SSE : l’API Apigee expose déjà une lecture de statut, le flux est unidirectionnel et le polling est plus simple à tester avec le gateway mémoire et à arrêter proprement lors d’un changement de vue. L’intervalle devra être configurable dans le composable ou la commande, borné et documenté ; aucune boucle infinie ni retry agressif ne sera introduit.

---

## 4. Étapes atomiques

### M8-00 — Baseline, contrat et scénario de création/déploiement

- [x] Vérifier que la branche `feature/m8-gui-deployment` est créée depuis `dev` après l’intégration de M7.
- [x] Cartographier le parcours cible et les frontières génération/import/déploiement/statut.
- [x] Définir les DTO sérialisables pour les entrées, résultats, états et erreurs M8.
- [x] Définir les invariants de sécurité : confirmation, validation des cibles, absence de secrets et absence de réseau en Demo.
- [x] Documenter le scénario nominal et les scénarios d’échec qui seront couverts par les étapes suivantes.

#### Contrats DTO proposés

Les types ci-dessous sont le contrat de frontière Rust ↔ Vue de M8. Ils seront implémentés et testés lors des étapes qui exposent réellement les commandes ; cette étape fixe uniquement leur responsabilité et leurs champs non sensibles.

```text
ProxyCreationJobInputDto
├── template_name: String
├── openapi_source: OpenApiSourceDto
├── organization: String
└── proxy_name: String

OpenApiSourceDto
├── display_name: String
└── content: String

BundleGenerationResultDto
├── job_id: String
├── proxy_name: String
├── rendered_file_count: usize
└── state: GenerationStateDto

CreatedProxyRevisionDto
├── organization: String
├── proxy_name: String
├── revision: u32
└── deployed: false

DeploymentDto
├── id: String
├── organization: String
├── environment: String
├── proxy_name: String
├── revision: u32
└── status: DeploymentStatusDto

DeploymentStatusDto = Pending | InProgress | Succeeded | Failed | TimedOut | Cancelled

GuiCommandErrorDto
├── code: String
├── message: String
└── field: Option<String>
```

Règles associées :

- `OpenApiSourceDto.content` est consommé uniquement côté Rust et ne doit jamais être réémis dans un résultat de commande ou un log ; `display_name` sert uniquement à l’interface.
- `job_id` est un identifiant opaque généré localement ; il ne doit pas contenir de chemin utilisateur ni de donnée d’authentification.
- `DeploymentDto` répète l’organisation et l’environnement afin que la confirmation et le résultat soient toujours lisibles sans reconstruire la cible côté Vue.
- `TimedOut` et `Cancelled` sont des états de workflow GUI ; ils ne prétendent pas modifier l’état réel Apigee et doivent être distingués d’un `Failed` retourné par Apigee.
- Toute erreur côté Vue est un `GuiCommandErrorDto` sûr ; les détails HTTP, URL internes, tokens et chemins absolus restent confinés à l’infrastructure Rust, avec un mapping vers un code stable.

#### Scénarios de référence

**Nominal Demo/Live**

1. la session est prête et l’organisation/environnement sont sélectionnés ;
2. l’utilisateur choisit un template valide, une spec OpenAPI et un nom de proxy ;
3. la preview de création affiche la cible et les étapes sans mutation ;
4. la génération locale réussit et produit un `job_id` ;
5. l’utilisateur confirme l’upload, qui crée un proxy ou une nouvelle révision ;
6. le résultat affiche le proxy et la révision avec le statut `Not deployed` ;
7. l’utilisateur sélectionne cette révision depuis le catalogue des proxies ;
8. l’utilisateur confirme séparément le déploiement ;
9. le polling termine sur `Succeeded` et actualise les proxies.

**Échecs à couvrir**

- session absente ou organisation/environnement non sélectionnés : aucune écriture ni mutation réseau ;
- template ou spec invalide : échec avant génération ;
- écriture/rendu local impossible : nettoyage et conservation d’une erreur sans chemin sensible ;
- import refusé, timeout ou erreur réseau : aucune tentative automatique de déploiement ;
- révision ou cible invalide : commande refusée côté Rust ;
- utilisateur annule la confirmation d’upload ou de déploiement : aucune mutation correspondante ;
- double soumission : une seule opération mutative active par phase ;
- statut `Failed`, polling annulé ou timeout GUI : état final explicite, sans prétendre que la ressource distante est supprimée.

Commit prévu :

```text
docs(m8): define GUI deployment roadmap and contracts
```

### M8-01 — Préparation et preview de création du proxy

- [x] Permettre de sélectionner un template validé depuis le catalogue M7.
- [x] Ajouter la sélection ou la fourniture contrôlée d’une spec OpenAPI.
- [x] Résoudre organisation, environnement, nom de proxy et convention de nommage sans dupliquer la logique métier.
- [x] Afficher une preview non mutative : template, spec, proxy cible, environnement et étapes à venir.
- [x] Refuser la poursuite si le template, la spec ou la cible sont invalides.

La résolution du nom de proxy dans cette preview est indicative et dérivée des métadonnées du template. Elle ne déclenche aucune mutation et ne remplace pas la validation métier côté Rust qui sera appliquée avant génération/import.

Commit prévu :

```text
feat(gui): prepare proxy creation preview
```

### M8-02 — Génération locale du bundle

- [x] Exposer la génération via un use case/port adapté au composition root GUI, sans appeler Apigee.
- [x] Utiliser des répertoires temporaires contrôlés et nettoyer les artefacts en cas d’échec.
- [x] Retourner un résultat non sensible : proxy, nombre de fichiers et identifiant de job, sans chemin absolu local.
- [x] Afficher la progression et l’erreur de génération dans le GUI.
- [x] Tester template invalide, spec invalide, échec de rendu, échec d’écriture et succès nominal.

M8-02 et M8-03 sont intégrés dans un même flux afin que le bundle généré soit conservé temporairement côté Rust et transmis à l’upload par un `job_id`, sans exposer de chemin local ni renvoyer inutilement le ZIP au frontend.

Commit prévu avec M8-03 :

```text
feat(gui): generate and upload proxy bundle
```

### M8-03 — Upload et création du proxy/révision

- [x] Exposer `ImportProxyBundleUseCase` via une commande Tauri dédiée.
- [x] Valider organisation et nom de proxy côté Rust avant l’appel gateway.
- [x] Présenter l’upload comme une opération de création de proxy ou de nouvelle révision, jamais comme un déploiement.
- [x] Retourner un `CreatedProxyRevisionDto` avec `deployed: false` sans exposer de réponse HTTP brute.
- [x] Rafraîchir le catalogue des proxies après création et rendre la révision sélectionnable pour un déploiement ultérieur.
- [x] Couvrir succès, authentification absente, accès refusé, timeout, erreur gateway et mode Demo.

Commit partagé avec M8-02 :

```text
feat(gui): generate and upload proxy bundle
```

### M8-04 — Sélection et review d’une révision à déployer

- [x] Présenter les proxies et leurs révisions existantes dans le catalogue `Proxies`.
- [x] Afficher explicitement organisation, environnement, proxy, révision, statut actuel et mode Live/Demo.
- [x] Refuser la sélection d’une révision absente d’Apigee ou déjà déployée sans action explicite de remplacement.
- [x] Exiger une confirmation distincte de l’upload qui a créé la révision.
- [x] Empêcher double clic, soumission concurrente et déploiement sans révision sélectionnée.
- [x] Conserver un état local récupérable en cas d’annulation ou d’erreur avant mutation.

La confirmation M8-04 prépare l’état de déploiement mais n’appelle volontairement pas encore `DeployProxyUseCase`; la mutation et son résultat appartiennent à M8-05.

Commit prévu :

```text
feat(gui): review proxy revision deployment
```

### M8-05 — Déploiement d’une révision existante et contrat de résultat

- [ ] Exposer `DeployProxyUseCase` via une commande Tauri dédiée.
- [ ] Accepter uniquement une révision existante retournée par Apigee ou par l’étape d’upload M8-03.
- [ ] Valider la révision, l’organisation, l’environnement, le proxy et `override_existing` côté Rust.
- [ ] Conserver `override_existing` explicite et jamais activé implicitement.
- [ ] Retourner un DTO de déploiement stable avec identifiant, cible, révision et statut.
- [ ] Tester le mapping des statuts `Pending`, `InProgress`, `Succeeded` et `Failed`, ainsi que les erreurs sûres.

Commit prévu :

```text
feat(gui): deploy imported proxy revision
```

### M8-06 — Suivi de statut borné et annulable

- [ ] Exposer `GetDeploymentStatusUseCase` avec un DTO de statut stable.
- [ ] Implémenter le polling dans un composable Vue dédié, sans manipulation directe du DOM.
- [ ] Arrêter automatiquement le polling sur succès, échec, annulation ou timeout global.
- [ ] Empêcher qu’une réponse ancienne écrase le statut d’un nouveau job.
- [ ] Prévoir un bouton d’arrêt/retry contrôlé et une indication claire de la dernière mise à jour.
- [ ] Tester transitions, timeout, erreur transitoire, annulation et changement de vue.

Commit prévu :

```text
feat(gui): track deployment status with bounded polling
```

### M8-07 — Écran de déploiement et intégration au dashboard

- [ ] Remplacer l’état vide de `Deployments` par le parcours complet de review et d’exécution.
- [ ] Ajouter résumé de la cible, progression, statut courant, erreurs et résultat final.
- [ ] Permettre de revenir au template ou aux proxies sans perdre un état utile ni relancer une mutation.
- [ ] Actualiser la liste des proxies/révisions après import ou déploiement réussi.
- [ ] Garantir navigation clavier, labels accessibles, annonces `aria-live` et affichage responsive.
- [ ] Tester le parcours Vue nominal et les branches d’erreur avec `InMemoryApigeeGateway`.

Commit prévu :

```text
feat(gui): integrate deployment workflow into dashboard
```

### M8-08 — Tests de parcours, sécurité et checkpoint de sortie

- [ ] Tester le parcours complet Demo : template → preview → génération → import → déploiement → statut final.
- [ ] Tester l’absence de réseau pour toutes les opérations Demo et locales.
- [ ] Ajouter les tests Rust des commandes et des use cases avec doubles, sans credential réel.
- [ ] Ajouter les tests frontend du composable et du parcours UI, y compris les doubles soumissions et annulations.
- [ ] Vérifier `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features` et les tests/build frontend.
- [ ] Vérifier que les erreurs et fixtures ne contiennent ni secret ni chemin local sensible.
- [ ] Exécuter séparément la validation Live Hello World documentée, uniquement avec l’environnement de test prévu.
- [ ] Mettre à jour `ROADMAP.md`, documenter les reports éventuels et préparer l’intégration de M8 dans `dev`.

Commit prévu :

```text
test(m8): validate GUI deployment workflow
```

---

## 5. Critères d’acceptation M8

M8 sera terminé lorsque :

1. l’utilisateur peut sélectionner un template M7 et une spec OpenAPI sans écrire de JSON ;
2. le GUI génère un bundle local valide sans appel réseau ;
3. le GUI peut uploader ce bundle pour créer un proxy ou une nouvelle révision ;
4. la révision créée apparaît explicitement comme `Not deployed` ;
5. le GUI ne permet de déployer qu’une révision existante et sélectionnée ;
6. le GUI demande une confirmation explicite avant l’upload puis une confirmation distincte avant le déploiement ;
7. le GUI déploie la révision sur l’organisation et l’environnement affichés ;
8. le statut est suivi automatiquement avec un polling borné et annulable ;
9. le parcours fonctionne en mode Demo sans réseau et reste compatible avec le mode Live ;
10. les tests couvrent les use cases, la frontière Tauri et le parcours Vue ;
11. aucune sortie, fixture ou erreur ne contient de secret, token, réponse HTTP brute ou chemin local sensible.

---

## 6. Premier point recommandé

Le premier travail M8 doit être **M8-00 — Baseline, contrat et scénario de création/déploiement**.

Les use cases métier existent déjà, mais leur contrat GUI, le cycle de vie des artefacts locaux et la séparation stricte entre création/upload et déploiement doivent être figés avant d’ajouter des commandes Tauri ou une interface mutative. Cette étape évite de construire un parcours frontend qui traiterait un template comme une ressource déployable ou mélangerait la création d’une révision et son déploiement.

---

## 7. Validation manuelle Live

La validation Live ne fait pas partie des tests automatisés et ne doit jamais être exécutée avec un credential présent dans le repository.

Le scénario manuel doit utiliser l’organisation d’évaluation documentée dans `REAL_APIGEE_VALIDATION.md` :

1. ouvrir une session OAuth desktop ;
2. sélectionner explicitement organisation et environnement ;
3. utiliser une spec et un template Hello World sans secret ;
4. générer le bundle localement ;
5. confirmer l’upload et créer le proxy ou la révision ;
6. vérifier que la révision apparaît comme `Not deployed` ;
7. sélectionner cette révision et confirmer séparément le déploiement ;
8. suivre le statut jusqu’à `Succeeded` ou `Failed` ;
8. conserver uniquement un rapport non sensible.

Toute réponse inattendue de l’API doit être traitée comme un échec à analyser, pas comme une raison d’affaiblir la validation, les permissions ou la sécurité.
