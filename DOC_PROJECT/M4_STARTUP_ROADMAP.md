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
- `deploy` — déploiement d'une révision vers une organisation et un environnement ;
- `status` — lecture du statut d'un déploiement ;
- `list-proxies` — liste des proxies accessibles ;
- un mode `--json` stable pour l'intégration CI/CD ;
- des codes de sortie explicites et une absence totale de prompt bloquant en mode non-interactif.

M4 ne doit pas ajouter de logique métier dans `cli/`. Le CLI reste un adaptateur : parsing des arguments, composition root, formatage de sortie et traduction des erreurs en codes de sortie.

**Ajout important à ce jalon** : M4 est aussi le premier point du projet où le CLI est validé contre un environnement Apigee **réel**, pas seulement contre des doubles (`InMemoryApigeeGateway`) ou des réponses simulées (WireMock). Voir section 7 pour la méthode complète, `DOC_PROJECT/GCP_SETUP.md` pour le provisionnement, et `DOC_PROJECT/APIGEE_API_MAP.md` pour la référence des endpoints. **Aucun mécanisme de bascule fake/réel n'est à coder dans le CLI** : le binaire livré utilise toujours `ReqwestApigeeGateway` — les fakes restent confinés à la suite de tests automatisée (ARCHITECTURE.md section 12, niveau 1). "Tester en réel" signifie exécuter le vrai CLI, une fois l'environnement provisionné, pas ajouter un flag de simulation au produit.

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
- `ReqwestApigeeGateway` fournit actuellement des méthodes concrètes de lecture mais n'implémente pas encore le trait `ApigeeGateway` complet (import de bundle et déploiement manquants — voir `APIGEE_API_MAP.md`) ;
- aucune composition root CLI pour injecter l'authentification headless ou desktop dans le gateway ;
- `LocalStateStore` est un port sans implémentation CLI nécessaire au périmètre M4 ; le refresh token OAuth reste dans le keyring ;
- les doubles existants ne couvrent pas encore tous les nouveaux use cases ;
- la validation stricte du template contre `schemas/template.schema.json` doit être traitée avant d'accepter des fichiers utilisateur en CLI ;
- **aucun test n'a encore été exécuté contre un environnement Apigee réel** — tout ce qui existe à ce stade est validé par doubles/WireMock uniquement.

---

## 3. Règles de méthode M4

- Une seule étape atomique à la fois.
- Après chaque étape : tests ciblés, inspection du diff, commit, arrêt au point de validation.
- Aucun `.unwrap()`/`.expect()` dans `core/` ; le CLI doit également retourner une erreur plutôt que paniquer sur une entrée utilisateur.
- Le domaine ne dépend d'aucune crate de CLI.
- Les use cases dépendent uniquement du domaine et des ports.
- `cli/` peut connaître `core::infra` uniquement dans `main.rs`, qui est le composition root.
- Aucun token, refresh token, header Authorization ou contenu de credential dans les sorties, logs, rapports ou erreurs.
- Aucun credential accepté en argument CLI ; le headless utilise uniquement `GOOGLE_APPLICATION_CREDENTIALS`.
- Aucun prompt ne doit apparaître par défaut dans un contexte détecté non-interactif.
- Toute commande doit proposer une sortie humaine et une sortie JSON structurée sans mélange stdout/stderr.
- Les fichiers utilisateur, templates et chemins de sortie sont validés avant toute écriture ou appel réseau.
- Les tests de use cases utilisent des doubles ; les tests du gateway HTTP utilisent WireMock ; les tests de composition CLI utilisent des fixtures sans credential réel.
- **La validation contre l'environnement Apigee réel (section 7) est manuelle, exécutée par l'humain, et n'entre jamais dans la suite `cargo test` automatisée ou la CI.**
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
- organisation/environnement explicitement sélectionnés quand l'authentification ne permet pas de les déduire ;
- chemins de fichiers explicites ;
- `--non-interactive` ou détection documentée de l'absence de terminal si une voie interactive existe.

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

Le JSON ne doit jamais contenir de source error brute susceptible d'exposer une URL interne, un token ou une réponse HTTP.

### Codes de sortie proposés

- `0` : succès ;
- `1` : erreur d'entrée ou d'exécution générique ;
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
- [x] Vérifier que le working tree est propre à l'entrée du jalon.
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
- [x] Ne jamais afficher les sources d'erreur contenant des secrets ou des corps HTTP.
- [x] Tester chaque catégorie d'erreur en mode texte et JSON.
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

- [x] Définir le choix d'authentification CLI : headless via `GOOGLE_APPLICATION_CREDENTIALS`, desktop OAuth uniquement sur demande interactive explicite.
- [x] Composer `AuthProvider` dans la composition root CLI sans exposer les credentials.
- [x] Implémenter `login` avec résultat non sensible et comportement clair en mode headless.
- [x] Résoudre l'organisation depuis le project ID headless ou une sélection/option explicite en desktop.
- [x] Refuser toute ambiguïté d'organisation au lieu de deviner.
- [x] Tester avec un double de `AuthProvider` ; les doubles `BrowserLauncher` et `RefreshTokenStore` existants de M2 couvrent le provider OAuth.
- [ ] **Checkpoint de provisionnement différé** : suivre `M4-04_checkpoint.md` et `GCP_SETUP.md` lorsque l'environnement réel sera nécessaire. Ce checkpoint ne bloque pas l'implémentation automatisée de M4 ; il est requis avant la première validation réelle et tout déploiement réel.

Commit prévu :

```text
feat(cli): add secure authentication composition
```

### M4-05 — Use cases de lecture Apigee

- [x] Implémenter `ListOrganizationsUseCase`.
- [x] Implémenter `ListEnvironmentsUseCase`.
- [x] Implémenter `ListProxiesUseCase`.
- [x] Implémenter l'adaptation `ReqwestApigeeGateway: ApigeeGateway` pour ces opérations (endpoints confirmés dans `APIGEE_API_MAP.md`).
- [x] Conserver `InMemoryApigeeGateway` comme double et compléter ses données de test si nécessaire.
- [x] Ajouter tests de use cases avec fake gateway et tests HTTP WireMock déjà alignés sur les endpoints officiels.
- [x] Brancher `list-proxies` avec org explicite ou résolue par auth.
- [ ] **Checkpoint de connectivité réelle (première validation manuelle contre Apigee)** : une fois l'implémentation testée par doubles/WireMock, exécuter manuellement `login` puis `list-proxies` contre l'organisation d'évaluation réelle provisionnée en M4-04. Objectif : confirmer que l'authentification et la lecture fonctionnent réellement avant de construire l'écriture (M4-06) par-dessus une fondation non vérifiée. Ne pas automatiser ce test en CI — c'est une vérification manuelle, ponctuelle, documentée dans le commit ou une note (sans credential ni détail sensible).

Commit possible :

```text
feat(core): add Apigee read use cases
feat(cli): add list proxies command
```

### M4-06 — Déploiement et statut

- [ ] Vérifier et compléter le contrat réel de déploiement Apigee : upload/import du bundle, révision et déploiement ne doivent pas être confondus (voir `APIGEE_API_MAP.md` — import et déploiement sont deux appels distincts).
- [ ] Étendre `ApigeeGateway` ou créer des ports séparés si le trait devient trop large.
- [ ] Implémenter `DeployProxyUseCase` et `GetDeploymentStatusUseCase`.
- [ ] Implémenter les méthodes HTTP manquantes de `ReqwestApigeeGateway` selon la documentation officielle (import bundle, déploiement, statut).
- [ ] Compléter `InMemoryApigeeGateway` pour les scénarios pending/in-progress/succeeded/failed.
- [ ] Tester erreurs d'authentification, permission, ressource absente, timeout, rate limit et serveur.
- [ ] Ajouter `deploy` et `status` sans afficher de credential ni de corps HTTP.
- [ ] **Checkpoint de connectivité réelle (écriture)** : une fois testé par doubles/WireMock, exécuter manuellement un déploiement contre l'org réelle avec un bundle trivial (pas nécessairement Helloworld à ce stade — un bundle minimal suffit pour confirmer que l'import et le déploiement fonctionnent réellement). Le test Helloworld complet est fait en M4-11, une fois `generate` migré en M4-07.

Commit possible :

```text
feat(core): add deployment and status use cases
feat(cli): add deploy and status commands
```

### M4-07 — Génération CLI complète

- [ ] Migrer le flux M3 `generate` vers l'arbre de commandes typé.
- [ ] Supporter template inline validé ou template référencé par le repository local.
- [ ] Préserver l'écriture atomique du bundle et le packaging ZIP M3.
- [ ] Produire un résultat humain/JSON stable.
- [ ] Tester les entrées invalides, template absent, spec invalide, sortie existante et succès complet.

Commit prévu :

```text
feat(cli): complete generate command
```

### M4-08 — Non-interactif et sorties scriptables

- [ ] Vérifier que chaque commande fonctionne avec flags et variables d'environnement sans prompt.
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
- [ ] Produire un rapport de matrice des commandes et catégories d'erreurs dans `target/test-results/`.
- [ ] Vérifier qu'aucun bundle, token ou credential n'est versionné.
- [ ] Tester les commandes inconnues et les chemins dangereux.

Commit prévu :

```text
test(cli): cover complete command matrix
```

### M4-10 — Point de contrôle final M4 (suite automatisée)

- [ ] Exécuter `cargo fmt --all -- --check`.
- [ ] Exécuter `cargo test --workspace --locked`.
- [ ] Exécuter Clippy avec `-D warnings`.
- [ ] Exécuter `cargo audit`.
- [ ] Vérifier les builds CLI ciblés prévus par `PACKAGING.md` sans lancer de release.
- [ ] Vérifier l'absence de secrets dans code, fixtures, rapports et sorties capturées.

Commit prévu :

```text
docs(m4): record automated CLI validation
```

### M4-11 — Validation end-to-end réelle : proxy Helloworld

*Checkpoint réel différé, distinct de M4-10 : cette étape est manuelle et ne bloque ni la CI ni la poursuite de l'implémentation automatisée de M4. Elle constitue la preuve concrète avant le premier usage réel, pas un prérequis pour coder les contrats et use cases.*

- [ ] Confirmer que le projet GCP et l'organisation d'évaluation sont bien provisionnés (M4-04) et toujours dans leur fenêtre de 60 jours.
- [ ] Créer une spec OpenAPI minimale Helloworld (une seule route `GET /hello`) sous `examples/helloworld/openapi.yaml`.
- [ ] Créer un template minimal correspondant sous `examples/helloworld/template.json`, conforme à `schemas/template.schema.json` (sécurité API Key suffit pour ce premier test — pas besoin de couvrir toutes les policies MVP ici).
- [ ] Exécuter dans l'ordre, avec le CLI réel (pas de double) : `login`, `generate`, `deploy`, `status`, `list-proxies`.
- [ ] Confirmer que le proxy apparaît bien dans la console Apigee (ou via `list-proxies`) avec le statut déployé attendu.
- [ ] Documenter le résultat dans un court rapport (succès/échec, sans aucun credential ni détail sensible) — sert de preuve de validation MVP, pas seulement de checklist cochée.
- [ ] Si un écart apparaît entre le comportement réel et ce que WireMock simulait, corriger le mapping dans `APIGEE_API_MAP.md` et le code correspondant avant de considérer M4 terminé.

Commit prévu :

```text
docs(m4): record real Apigee end-to-end validation (helloworld)
```

La fin technique automatisée de M4 peut être marquée après M4-10. M4-11 reste un checkpoint de preuve réelle à valider avant le premier déploiement réel ; il ne bloque pas les étapes de développement local.

---

## 6. Critères d'acceptation M4

M4 sera considéré terminé lorsque :

1. chaque commande documentée possède un chemin heureux et des erreurs typées ;
2. le CLI fonctionne sans GUI et sans credentials réels dans les tests automatisés ;
3. le mode headless utilise uniquement `GOOGLE_APPLICATION_CREDENTIALS` ;
4. `--json` est parseable et ne contient aucun secret ;
5. les codes de sortie sont stables et testés ;
6. les opérations Apigee réelles passent par `ApigeeGateway` et non par des appels HTTP dans `cli` ;
7. les fakes et WireMock couvrent les scénarios d'erreur importants ;
8. la suite workspace, Clippy et audit passent ;
9. le checkpoint réel M4-11 est documenté comme prérequis avant le premier déploiement réel, sans être requis pour la clôture technique automatisée ;
10. aucun changement GUI, déploiement réel automatisé en CI, ou release packaging M10 n'est ajouté par erreur au jalon.

---

## 7. Méthode de validation contre Apigee réel — résumé

| Étape | Ce qui est validé | Contre quoi |
|---|---|---|
| M4-04 | Provisionnement de l'environnement | GCP_SETUP.md, projet + org eval réels |
| M4-05 | Authentification + lecture | Org réelle (`login`, `list-proxies`) |
| M4-06 | Écriture (import/déploiement) | Org réelle, bundle trivial |
| M4-11 | Bout en bout complet | Org réelle, proxy Helloworld généré par le CLI lui-même |

Aucune de ces validations n'entre dans `cargo test` ou la CI — elles restent manuelles, exécutées par l'humain, documentées sans secret. La CI continue de reposer uniquement sur les niveaux 1 à 3 de la stratégie de test (ARCHITECTURE.md section 12).