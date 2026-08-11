# Apigee Forge — Roadmap de démarrage M8

*Jalon de déploiement et de suivi depuis le GUI. M8 relie l’éditeur de templates M7 au moteur de génération de bundle et aux opérations Apigee déjà portées par `core`, avec une confirmation explicite avant toute mutation distante. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M8

Le GUI doit permettre de sélectionner un template sauvegardé, fournir la spec OpenAPI correspondante, générer un bundle local, l’importer comme nouvelle révision de proxy, demander son déploiement sur un environnement choisi et suivre l’état de ce déploiement sans bloquer l’interface.

Parcours cible :

```text
Templates → Review & Save → Create proxy
                              ↓
                 OpenAPI + template + cible
                              ↓
                    Preview de génération
                              ↓
                    Générer le bundle local
                              ↓
              Importer une nouvelle révision
                              ↓
           Confirmer explicitement le déploiement
                              ↓
        Déployer → interroger le statut → résultat
```

M8 doit conserver les contrats existants :

- `core::use_cases::GenerateProxyBundleUseCase` pour la génération locale ;
- `ImportProxyBundleUseCase` pour l’import d’une nouvelle révision ;
- `DeployProxyUseCase` pour le déploiement sur un environnement ;
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
- écran GUI `Deployments` présent, mais sans parcours de déploiement sélectionné.

### Gaps à traiter

- contrat DTO Rust ↔ Vue pour les paramètres et résultats de génération/import/déploiement ;
- résolution sûre de la spec OpenAPI et des répertoires temporaires de génération ;
- commandes Tauri de génération locale, import et déploiement ;
- confirmation explicite avant chaque opération distante mutative ;
- état frontend du workflow asynchrone et protection contre les doubles soumissions ;
- polling borné et annulable du statut de déploiement ;
- interface de review, progression et résultat dans le GUI ;
- tests de use cases/commandes/composable avec doubles Demo et sans réseau ;
- procédure de validation manuelle Live sans exposer de secrets.

---

## 3. Décisions d’architecture M8

- La génération du bundle reste une opération locale et ne doit jamais appeler Apigee.
- L’import et le déploiement sont deux mutations distinctes : le GUI ne doit pas les confondre ni présenter un déploiement comme réussi après un simple import.
- La logique métier reste dans `core`; les commandes Tauri valident les DTO, résolvent les dépendances du composition root et délèguent aux use cases.
- Le GUI utilise les mêmes commandes et DTO en mode Demo et en mode Live ; seul le gateway injecté change.
- Le déploiement nécessite une confirmation explicite contenant au minimum organisation, environnement, proxy et révision ciblés.
- Le nom du proxy, l’environnement, la révision et les chemins de génération sont validés côté Rust avant tout appel réseau ou écriture.
- Les fichiers générés sont stockés dans un répertoire de travail temporaire contrôlé ; aucun chemin local sensible ne doit être renvoyé au frontend ou affiché dans une erreur.
- Le suivi utilise un polling borné, avec intervalle raisonnable, arrêt sur `Succeeded`/`Failed`, annulation lors de la navigation et protection contre les requêtes concurrentes obsolètes.
- Les erreurs sont transformées en codes GUI stables et sûrs ; aucun token, header Authorization, corps HTTP ou credential ne doit atteindre Vue, les logs ou les DTO.
- M8 ne modifie pas les permissions IAM et ne fournit pas de mécanisme de contournement d’une autorisation Apigee refusée.

### Choix du suivi temps réel

Pour le MVP, le GUI utilisera un polling Tauri vers `GetDeploymentStatusUseCase` plutôt qu’un WebSocket ou un SSE : l’API Apigee expose déjà une lecture de statut, le flux est unidirectionnel et le polling est plus simple à tester avec le gateway mémoire et à arrêter proprement lors d’un changement de vue. L’intervalle devra être configurable dans le composable ou la commande, borné et documenté ; aucune boucle infinie ni retry agressif ne sera introduit.

---

## 4. Étapes atomiques

### M8-00 — Baseline, contrat et scénario de déploiement

- [x] Vérifier que la branche `feature/m8-gui-deployment` est créée depuis `dev` après l’intégration de M7.
- [x] Cartographier le parcours cible et les frontières génération/import/déploiement/statut.
- [x] Définir les DTO sérialisables pour les entrées, résultats, états et erreurs M8.
- [x] Définir les invariants de sécurité : confirmation, validation des cibles, absence de secrets et absence de réseau en Demo.
- [x] Documenter le scénario nominal et les scénarios d’échec qui seront couverts par les étapes suivantes.

#### Contrats DTO proposés

Les types ci-dessous sont le contrat de frontière Rust ↔ Vue de M8. Ils seront implémentés et testés lors des étapes qui exposent réellement les commandes ; cette étape fixe uniquement leur responsabilité et leurs champs non sensibles.

```text
DeploymentJobInputDto
├── template_name: String
├── openapi_source: OpenApiSourceDto
├── organization: String
├── environment: String
├── proxy_name: String
└── override_existing: bool

OpenApiSourceDto
├── display_name: String
└── content: String

BundleGenerationResultDto
├── job_id: String
├── proxy_name: String
├── rendered_file_count: usize
└── state: GenerationStateDto

ImportedRevisionDto
├── organization: String
├── proxy_name: String
└── revision: u32

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
3. la preview affiche la cible et les étapes sans mutation ;
4. la génération locale réussit et produit un `job_id` ;
5. l’utilisateur demande l’import, qui retourne une révision ;
6. la review affiche organisation, environnement, proxy et révision ;
7. l’utilisateur confirme le déploiement ;
8. le polling termine sur `Succeeded` et actualise les proxies.

**Échecs à couvrir**

- session absente ou organisation/environnement non sélectionnés : aucune écriture ni mutation réseau ;
- template ou spec invalide : échec avant génération ;
- écriture/rendu local impossible : nettoyage et conservation d’une erreur sans chemin sensible ;
- import refusé, timeout ou erreur réseau : aucune tentative automatique de déploiement ;
- révision ou cible invalide : commande refusée côté Rust ;
- utilisateur annule la confirmation : aucune mutation distante ;
- double soumission : une seule opération mutative active ;
- statut `Failed`, polling annulé ou timeout GUI : état final explicite, sans prétendre que la ressource distante est supprimée.

Commit prévu :

```text
docs(m8): define GUI deployment roadmap and contracts
```

### M8-01 — Préparation et preview du job local

- [x] Permettre de sélectionner un template validé depuis le catalogue M7.
- [x] Ajouter la sélection ou la fourniture contrôlée d’une spec OpenAPI.
- [x] Résoudre organisation, environnement, nom de proxy et convention de nommage sans dupliquer la logique métier.
- [x] Afficher une preview non mutative : template, spec, proxy cible, environnement et étapes à venir.
- [x] Refuser la poursuite si le template, la spec ou la cible sont invalides.

La résolution du nom de proxy dans cette preview est indicative et dérivée des métadonnées du template. Elle ne déclenche aucune mutation et ne remplace pas la validation métier côté Rust qui sera appliquée avant génération/import.

Commit prévu :

```text
feat(gui): prepare deployment job preview
```

### M8-02 — Génération locale du bundle

- [ ] Exposer la génération via un use case/port adapté au composition root GUI, sans appeler Apigee.
- [ ] Utiliser des répertoires temporaires contrôlés et nettoyer les artefacts en cas d’échec.
- [ ] Retourner un résultat non sensible : proxy, nombre de fichiers et identifiant de job, sans chemin absolu local.
- [ ] Afficher la progression et l’erreur de génération dans le GUI.
- [ ] Tester template invalide, spec invalide, échec de rendu, échec d’écriture et succès nominal.

Commit prévu :

```text
feat(gui): generate proxy bundle locally
```

### M8-03 — Import de la révision Apigee

- [ ] Exposer `ImportProxyBundleUseCase` via une commande Tauri dédiée.
- [ ] Valider organisation et nom de proxy côté Rust avant l’appel gateway.
- [ ] Distinguer clairement l’état `bundle generated` de l’état `revision imported`.
- [ ] Mapper le résultat vers une `ProxyRevision` DTO sans exposer de réponse HTTP brute.
- [ ] Couvrir succès, authentification absente, accès refusé, timeout, erreur gateway et mode Demo.

Commit prévu :

```text
feat(gui): import generated proxy revision
```

### M8-04 — Review et confirmation du déploiement

- [ ] Présenter une étape de review dédiée avant l’appel `deploy`.
- [ ] Afficher explicitement organisation, environnement, proxy, révision et mode Live/Demo.
- [ ] Exiger une action de confirmation distincte de la génération et de l’import.
- [ ] Empêcher double clic, soumission concurrente et déploiement sans révision importée.
- [ ] Conserver un état local récupérable en cas d’annulation ou d’erreur avant mutation.

Commit prévu :

```text
feat(gui): add explicit deployment confirmation
```

### M8-05 — Déploiement et contrat de résultat

- [ ] Exposer `DeployProxyUseCase` via une commande Tauri dédiée.
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
3. le GUI peut importer ce bundle comme nouvelle révision Apigee ;
4. le GUI demande une confirmation explicite avant le déploiement ;
5. le GUI déploie la révision sur l’organisation et l’environnement affichés ;
6. le statut est suivi automatiquement avec un polling borné et annulable ;
7. les statuts de succès, d’échec, de timeout et d’accès refusé sont compréhensibles ;
8. le parcours fonctionne en mode Demo sans réseau et reste compatible avec le mode Live ;
9. les tests couvrent les use cases, la frontière Tauri et le parcours Vue ;
10. aucune sortie, fixture ou erreur ne contient de secret, token, réponse HTTP brute ou chemin local sensible.

---

## 6. Premier point recommandé

Le premier travail M8 doit être **M8-00 — Baseline, contrat et scénario de déploiement**.

Les use cases métier existent déjà, mais leur contrat GUI, le cycle de vie des artefacts locaux et la séparation stricte entre import et déploiement doivent être figés avant d’ajouter des commandes Tauri ou une interface mutative. Cette étape évite de construire un parcours frontend qui mélangerait génération locale, import de révision et déploiement distant.

---

## 7. Validation manuelle Live

La validation Live ne fait pas partie des tests automatisés et ne doit jamais être exécutée avec un credential présent dans le repository.

Le scénario manuel doit utiliser l’organisation d’évaluation documentée dans `REAL_APIGEE_VALIDATION.md` :

1. ouvrir une session OAuth desktop ;
2. sélectionner explicitement organisation et environnement ;
3. utiliser une spec et un template Hello World sans secret ;
4. générer le bundle localement ;
5. importer la révision ;
6. confirmer le déploiement après vérification de la cible ;
7. suivre le statut jusqu’à `Succeeded` ou `Failed` ;
8. conserver uniquement un rapport non sensible.

Toute réponse inattendue de l’API doit être traitée comme un échec à analyser, pas comme une raison d’affaiblir la validation, les permissions ou la sécurité.
