# Apigee Forge — Roadmap de démarrage détaillée M4

*Document atomique du jalon M4 — CLI complet. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M4

M4 transforme le CLI minimal M3 en outil utilisable en autonomie :

```text
commande CLI → validation typée → use case core → ports → résultat texte ou JSON
```

Le CLI doit couvrir :

- `login` — authentification desktop ou headless selon le contexte ;
- `template create`, `template list`, `template show`, `template update`, `template delete` ;
- `generate` — spec OpenAPI + template → bundle local ;
- `deploy` — déploiement d’une révision vers une organisation et un environnement ;
- `status` — lecture du statut d’un déploiement ;
- `list-proxies` — liste des proxies accessibles ;
- un mode `--json` stable pour l’intégration CI/CD ;
- des codes de sortie explicites et une absence totale de prompt bloquant en mode non-interactif.

M4 ne doit pas ajouter de logique métier dans `cli/`. Le CLI reste un adaptateur : parsing des arguments, composition root, formatage de sortie et traduction des erreurs en codes de sortie.

---

## 2. État initial vérifié après M3

### Déjà disponible dans `core`

- `AuthProvider` avec `ServiceAccountAuthProvider` et `OAuthDesktopAuthProvider` ;
- configuration headless limitée à `GOOGLE_APPLICATION_CREDENTIALS` ;
- stockage OAuth refresh token dans le keyring OS ;
- `ApigeeGateway` déclarant organisations, environnements, proxies, déploiement, statut et rôles ;
- `InMemoryApigeeGateway` comme double mémoire pour les tests ;
- tests HTTP WireMock de `ReqwestApigeeGateway` ;
- `TemplateRepository` et `FilesystemTemplateRepository` avec CRUD ;
- `CreateTemplateUseCase` ;
- `GenerateProxyBundleUseCase` ;
- renderer, writer filesystem et archiver ZIP M3 ;
- modèles `Organization`, `Environment`, `Proxy`, `Deployment`, `ApigeeRole` ;
- parser OpenAPI et modèle `RenderInput`.

### Gaps à traiter dans M4

- le CLI possède encore un parsing manuel minimal limité à `generate` ;
- aucun arbre de commandes typé ni contrat de sortie commun ;
- aucun `--json` ou code de sortie documenté ;
- aucun use case pour lister les templates, afficher, modifier ou supprimer ;
- aucun use case pour lister organisations/environnements/proxies, déployer ou lire un statut ;
- `ReqwestApigeeGateway` fournit actuellement des méthodes concrètes de lecture mais n’implémente pas encore le trait `ApigeeGateway` complet ;
- aucune composition root CLI pour injecter l’authentification headless ou desktop dans le gateway ;
- `LocalStateStore` est un port sans implémentation CLI nécessaire au périmètre M4 ; le refresh token OAuth reste dans le keyring ;
- les doubles existants ne couvrent pas encore tous les nouveaux use cases ;
- la validation stricte du template contre `schemas/template.schema.json` doit être traitée avant d’accepter des fichiers utilisateur en CLI.

---

## 3. Règles de méthode M4

- Une seule étape atomique à la fois.
- Après chaque étape : tests ciblés, inspection du diff, commit, arrêt au point de validation.
- Aucun `.unwrap()`/`.expect()` dans `core/` ; le CLI doit également retourner une erreur plutôt que paniquer sur une entrée utilisateur.
- Le domaine ne dépend d’aucune crate de CLI.
- Les use cases dépendent uniquement du domaine et des ports.
- `cli/` peut connaître `core::infra` uniquement dans `main.rs`, qui est le composition root.
- Aucun token, refresh token, header Authorization ou contenu de credential dans les sorties, logs, rapports ou erreurs.
- Aucun credential accepté en argument CLI ; le headless utilise uniquement `GOOGLE_APPLICATION_CREDENTIALS`.
- Aucun prompt ne doit apparaître par défaut dans un contexte détecté non-interactif.
- Toute commande doit proposer une sortie humaine et une sortie JSON structurée sans mélange stdout/stderr.
- Les fichiers utilisateur, templates et chemins de sortie sont validés avant toute écriture ou appel réseau.
- Les tests de use cases utilisent des doubles ; les tests du gateway HTTP utilisent WireMock ; les tests de composition CLI utilisent des fixtures sans credential réel.
- Aucun push distant ni merge distant sans demande explicite.

---

## 4. Contrats CLI à décider et conserver

### Parsing

Utiliser un arbre de commandes typé et déclaratif dans `cli/`. Les commandes M4 sont :

```text
cli
├── login
├── template
│   ├── create
│   ├── list
│   ├── show
│   ├── update
│   └── delete
├── generate
├── deploy
├── status
└── list-proxies
```

Les options transverses sont limitées à ce qui est nécessaire :

- `--json` ;
- `--quiet` si nécessaire pour CI ;
- organisation/environnement explicitement sélectionnés quand l’authentification ne permet pas de les déduire ;
- chemins de fichiers explicites ;
- `--non-interactive` ou détection documentée de l’absence de terminal si une voie interactive existe.

### Sortie

Définir un contrat sérialisable, non sensible :

```json
{
  "ok": true,
  "command": "list-proxies",
  "data": {},
  "error": null
}
```

En erreur :

```json
{
  "ok": false,
  "command": "deploy",
  "data": null,
  "error": {
    "code": "AUTH_REQUIRED",
    "message": "authentication is required"
  }
}
```

Le JSON ne doit jamais contenir de source error brute susceptible d’exposer une URL interne, un token ou une réponse HTTP.

### Codes de sortie proposés

- `0` : succès ;
- `1` : erreur d’entrée ou d’exécution générique ;
- `2` : arguments/usage invalides ;
- `3` : configuration ou authentification absente/invalide ;
- `4` : accès refusé ou ressource inexistante ;
- `5` : erreur réseau/gateway ;
- `6` : erreur filesystem ou packaging.

Les valeurs devront être figées dans un module CLI testé.

---

## 5. Étapes atomiques M4

### M4-00 — Baseline Git et documentation

- [x] Merger `feature/m3-rendering-engine` dans `dev` avec un merge commit explicite.
- [x] Créer `feature/m4-cli` depuis `dev` et basculer dessus.
- [x] Vérifier que le working tree est propre à l’entrée du jalon.
- [x] Créer ce document et le référencer dans `STRUCTURE.md` et `PROMPT.md`.
- [x] Committer uniquement la documentation M4.

Commit prévu :

```text
docs(m4): add detailed CLI roadmap
```

### M4-01 — Arbre de commandes et composition root

- [x] Remplacer le parsing manuel M3 par un parser CLI typé.
- [x] Définir les commandes et sous-commandes M4 sans implémenter leur logique métier dans `cli`.
- [x] Conserver `generate` comme commande fonctionnelle pendant la migration.
- [x] Centraliser `--json`, usage et erreurs de parsing.
- [ ] Définir les codes de sortie stables — prévu dans M4-02.
- [x] Ajouter des tests de parsing des commandes valides, options manquantes, doublons et commandes hors périmètre.
- [x] Vérifier que le CLI ne dépend pas de `gui` et que `core` ne dépend pas de `cli`.

Commit prévu :

```text
feat(cli): define typed M4 command tree
```

### M4-02 — Contrat de sortie, erreurs et codes de sortie

- [x] Définir les enveloppes stdout humaine/JSON.
- [x] Mapper les erreurs `AuthError`, `GatewayError`, `TemplateError`, erreurs OpenAPI et erreurs filesystem vers des codes stables.
- [x] Ne jamais afficher les sources d’erreur contenant des secrets ou des corps HTTP.
- [x] Tester chaque catégorie d’erreur en mode texte et JSON.
- [x] Vérifier que stdout reste parseable en `--json` et que les diagnostics vont sur stderr.

Commit prévu :

```text
feat(cli): add stable output and exit code contracts
```

### M4-03 — Use cases de gestion des templates

- [x] Ajouter les use cases `list`, `get/show`, `update` et `delete` derrière `TemplateRepository`.
- [x] Définir la validation de nom et de contenu avant écriture.
- [x] Ajouter la validation stricte contre `schemas/template.schema.json` avant create/update/generate.
- [x] Supporter `template create --from <file>` comme voie non-interactive.
- [x] Garder la création importée non-interactive et sans prompt bloquant ; les prompts guidés restent optionnels et sont reportés.
- [x] Tester avec `FilesystemTemplateRepository` isolé et un double mémoire de repository.

Commit possible :

```text
feat(core): add template CRUD use cases
feat(cli): add template management commands
```

### M4-04 — Authentification CLI et résolution du contexte

- [ ] Définir le choix d’authentification CLI : headless via `GOOGLE_APPLICATION_CREDENTIALS`, desktop OAuth uniquement sur demande interactive explicite.
- [ ] Composer `AuthProvider` dans `cli/main.rs` sans exposer les credentials.
- [ ] Implémenter `login` avec résultat non sensible et comportement clair en mode headless.
- [ ] Résoudre l’organisation depuis le project ID headless ou une sélection/option explicite en desktop.
- [ ] Refuser toute ambiguïté d’organisation au lieu de deviner.
- [ ] Tester avec doubles de `AuthProvider`, `BrowserLauncher` et `RefreshTokenStore`.

Commit prévu :

```text
feat(cli): add secure authentication composition
```

### M4-05 — Use cases de lecture Apigee

- [ ] Implémenter `ListOrganizationsUseCase`.
- [ ] Implémenter `ListEnvironmentsUseCase`.
- [ ] Implémenter `ListProxiesUseCase`.
- [ ] Implémenter l’adaptation `ReqwestApigeeGateway: ApigeeGateway` pour ces opérations.
- [ ] Conserver `InMemoryApigeeGateway` comme double et compléter ses données de test si nécessaire.
- [ ] Ajouter tests de use cases avec fake gateway et tests HTTP WireMock déjà alignés sur les endpoints officiels.
- [ ] Brancher `list-proxies` avec org explicite ou résolue par auth.

Commit possible :

```text
feat(core): add Apigee read use cases
feat(cli): add list proxies command
```

### M4-06 — Déploiement et statut

- [ ] Vérifier et compléter le contrat réel de déploiement Apigee : upload/import du bundle, révision et déploiement ne doivent pas être confondus.
- [ ] Étendre `ApigeeGateway` ou créer des ports séparés si le trait devient trop large.
- [ ] Implémenter `DeployProxyUseCase` et `GetDeploymentStatusUseCase`.
- [ ] Implémenter les méthodes HTTP manquantes de `ReqwestApigeeGateway` selon la documentation officielle.
- [ ] Compléter `InMemoryApigeeGateway` pour les scénarios pending/in-progress/succeeded/failed.
- [ ] Tester erreurs d’authentification, permission, ressource absente, timeout, rate limit et serveur.
- [ ] Ajouter `deploy` et `status` sans afficher de credential ni de corps HTTP.

Commit possible :

```text
feat(core): add deployment and status use cases
feat(cli): add deploy and status commands
```

### M4-07 — Génération CLI complète

- [ ] Migrer le flux M3 `generate` vers l’arbre de commandes typé.
- [ ] Supporter template inline validé ou template référencé par le repository local.
- [ ] Préserver l’écriture atomique du bundle et le packaging ZIP M3.
- [ ] Produire un résultat humain/JSON stable.
- [ ] Tester les entrées invalides, template absent, spec invalide, sortie existante et succès complet.

Commit prévu :

```text
feat(cli): complete generate command
```

### M4-08 — Non-interactif et sorties scriptables

- [ ] Vérifier que chaque commande fonctionne avec flags et variables d’environnement sans prompt.
- [ ] Définir explicitement les valeurs interdites en pipeline : credential en argument, sélection implicite, confirmation interactive.
- [ ] Ajouter tests de subprocess CLI pour succès, erreur et JSON.
- [ ] Vérifier stdout/stderr, codes de sortie et stabilité des clés JSON.
- [ ] Documenter un exemple CI sans secret dans le repository.

Commit prévu :

```text
test(cli): validate non-interactive command behavior
```

### M4-09 — Doubles, intégration et matrice de commandes

- [ ] Ajouter un fake de sortie/runner pour tester le CLI sans réseau ni credential.
- [ ] Couvrir les commandes avec `InMemoryApigeeGateway`, repository filesystem isolé, auth doubles et WireMock.
- [ ] Produire un rapport de matrice des commandes et catégories d’erreurs dans `target/test-results/`.
- [ ] Vérifier qu’aucun bundle, token ou credential n’est versionné.
- [ ] Tester les commandes inconnues et les chemins dangereux.

Commit prévu :

```text
test(cli): cover complete command matrix
```

### M4-10 — Point de contrôle final M4

- [ ] Exécuter `cargo fmt --all -- --check`.
- [ ] Exécuter `cargo test --workspace --locked`.
- [ ] Exécuter Clippy avec `-D warnings`.
- [ ] Exécuter `cargo audit`.
- [ ] Vérifier les builds CLI ciblés prévus par `PACKAGING.md` sans lancer de release.
- [ ] Vérifier l’absence de secrets dans code, fixtures, rapports et sorties capturées.
- [ ] Marquer M4 terminé dans `ROADMAP.md` uniquement après validation de toutes les commandes.

Commit prévu :

```text
docs(m4): record complete CLI validation
```

---

## 6. Critères d’acceptation M4

M4 sera considéré terminé lorsque :

1. chaque commande documentée possède un chemin heureux et des erreurs typées ;
2. le CLI fonctionne sans GUI et sans credentials réels dans les tests ;
3. le mode headless utilise uniquement `GOOGLE_APPLICATION_CREDENTIALS` ;
4. `--json` est parseable et ne contient aucun secret ;
5. les codes de sortie sont stables et testés ;
6. les opérations Apigee réelles passent par `ApigeeGateway` et non par des appels HTTP dans `cli` ;
7. les fakes et WireMock couvrent les scénarios d’erreur importants ;
8. la suite workspace, Clippy et audit passent ;
9. aucun changement GUI, déploiement réel ou release packaging M10 n’est ajouté par erreur au jalon.
