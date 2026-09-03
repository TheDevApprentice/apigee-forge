# Apigee Forge — Roadmap de finalisation du MVP

*Roadmap consolidée des travaux restant à réaliser avant de considérer Apigee Forge comme un MVP solide et prêt à être déployé. Ce document remplace les anciennes roadmaps opérationnelles pour la suite du projet, sans les supprimer : les documents M1 à M10 restent l'historique des décisions et de la construction.*

---

## 1. Objectif de cette roadmap

Les roadmaps M1 à M10 ont permis de construire les fondations, le client Apigee, le moteur de rendu, le CLI, le GUI, l'éditeur de templates et le parcours de déploiement.

Cette nouvelle roadmap a un objectif différent : **fermer proprement les écarts restants et transformer le prototype avancé en MVP fiable**.

Le MVP final doit permettre de réaliser le parcours suivant sans intervention technique manuelle dans le code :

```text
OpenAPI valide + template validé
              ↓
Mapping explicite vers le modèle Apigee
              ↓
Prévisualisation de la cible et des policies
              ↓
Génération locale reproductible
              ↓
Validation du bundle
              ↓
Import d'une révision non déployée
              ↓
Revue organisation / environnement / proxy / révision
              ↓
Déploiement confirmé
              ↓
Suivi borné du statut
              ↓
Validation réelle Apigee
              ↓
Livrable CLI ou installeur GUI prêt à distribuer
```

Le but n'est pas d'ajouter rapidement de nouvelles fonctionnalités, mais de fiabiliser le chemin critique. Chaque étape doit produire une preuve vérifiable : test automatisé, fixture, validation réelle, documentation ou artefact de build.

---

## 2. État de départ consolidé

### Déjà construit

- Workspace Cargo Rust avec `core`, `cli` et `gui/src-tauri`.
- Bibliothèque métier partagée `apigee-forge-core`.
- Modèles de domaine pour l'authentification, les organisations, les proxies, les révisions, les déploiements, les rôles et les templates.
- Parsing OpenAPI 3.x.
- Validation métier des templates.
- Schéma JSON versionné.
- Authentification OAuth desktop avec PKCE et keyring.
- Authentification headless via `GOOGLE_APPLICATION_CREDENTIALS`.
- Client HTTP Apigee avec timeout, retries et mapping d'erreurs.
- Gateway mémoire Demo.
- Repository filesystem des templates.
- Stockage local SQLCipher.
- Rendu Tera/XML.
- Packaging ZIP déterministe.
- CRUD CLI des templates.
- Génération CLI non interactive.
- Import, déploiement et lecture de statut côté core et CLI.
- Mode Live/Demo côté GUI.
- Dashboard, catalogue de proxies, catalogue de templates et Settings.
- Éditeur visuel de templates.
- Préparation de création de proxy.
- Revue et déploiement depuis le GUI.
- Polling borné et annulable côté Vue.
- Tests Rust, tests CLI, tests Vitest et CI de génération de bundle.

### Écarts connus à fermer

Les écarts sont regroupés par priorité dans les phases ci-dessous. Les points historiquement non cochés dans les documents M2 à M10 sont inclus, notamment :

- pagination/réponses Apigee volumineuses ;
- états complets du gateway mémoire ;
- couverture d'intégration du CLI avec les doubles ;
- validation réelle Live du GUI ;
- seeding Demo ;
- intégration GUI → CLI ;
- tests détaillés de l'éditeur ;
- checkpoint QA visuel M9 ;
- packaging release et README.

---

## 3. Règles de travail pour la finalisation

1. **Une étape atomique à la fois.** Chaque tâche doit être compilable, testable ou vérifiable indépendamment.
2. **Pas de nouvelle fonctionnalité avant fermeture du chemin critique.** Analytics, marketplace et autres extensions restent hors MVP.
3. **Le core reste la source de vérité métier.** Le GUI et le CLI ne doivent pas implémenter de logique concurrente.
4. **Toute mutation distante doit être explicite.** Génération, import et déploiement restent trois étapes distinctes.
5. **Aucun credential réel dans le dépôt, les fixtures, les logs ou les rapports.**
6. **La validation réelle est manuelle et séparée de la CI.** Elle utilise un environnement d'évaluation contrôlé et documente uniquement des résultats non sensibles.
7. **Les tests doivent couvrir le comportement, pas uniquement la présence des fonctions.**
8. **Toute modification de contrat doit être versionnée et accompagnée d'une migration ou d'un plan de compatibilité.**
9. **Le working tree doit être propre avant chaque validation de phase.** Les modifications locales non liées doivent être identifiées et conservées hors du périmètre de la tâche.
10. **Chaque phase se termine par une décision explicite : terminée, reportée avec justification, ou bloquée avec cause documentée.**

---

# 4. Phase 0 — Audit de clôture et contrats de référence

**Priorité : haute**

Cette phase transforme les constats des anciennes roadmaps en une liste de critères contrôlables. Elle doit être terminée avant les changements fonctionnels importants.

## 4.1 Inventaire technique final

- [ ] Comparer le code présent avec `MVP_FEATURES.md`, `ROADMAP.md`, `ARCHITECTURE.md`, `PACKAGING.md` et les roadmaps M2 à M10.
- [ ] Marquer chaque ancienne tâche comme `terminée`, `partiellement terminée`, `reportée` ou `abandonnée`.
- [ ] Identifier les divergences entre `STRUCTURE.md` et l'arborescence réelle.
- [ ] Vérifier les contrats réellement utilisés par le GUI et le CLI.
- [ ] Identifier les endpoints Apigee effectivement utilisés et ceux seulement documentés.
- [ ] Produire une matrice des fonctionnalités avec leurs preuves : fichier, test, fixture ou validation manuelle.

## 4.2 Baseline de qualité

- [ ] Exécuter et archiver les résultats de `cargo fmt --all -- --check`.
- [ ] Exécuter `cargo test --workspace --locked`.
- [ ] Exécuter `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [ ] Exécuter `cargo audit`.
- [ ] Exécuter les tests GUI et le build Vite.
- [ ] Vérifier l'état Git et séparer les modifications de polish GUI déjà présentes des travaux de finalisation.
- [ ] Définir les commandes officielles de validation dans la documentation projet.

## 4.3 Definition of Done

La phase est terminée lorsque :

- tous les écarts restants sont listés dans ce document ;
- chaque écart possède une phase cible ;
- les commandes de validation de référence passent ;
- le contrat de succès du MVP est accepté et compréhensible.

---

# 5. Phase 1 — Contrat template v1 et mapping OpenAPI/Apigee

**Priorité : haute — bloque la fiabilité fonctionnelle du MVP**

Le parsing OpenAPI existe déjà, mais les routes et exigences de sécurité ne sont pas encore transformées de façon complète en flows conditionnels Apigee. Cette phase est la priorité fonctionnelle numéro un.

## 5.1 Stabiliser le contrat JSON du template

- [ ] Définir officiellement `template.schema.json` comme contrat public v1.
- [ ] Comparer systématiquement le JSON Schema, `domain::Template`, les formulaires Vue et les fixtures.
- [ ] Corriger les différences sur les champs obligatoires, facultatifs et valeurs par défaut.
- [ ] Décider si les flows vides peuvent être représentés par `{}` ou doivent toujours contenir `request` et `response`.
- [ ] Documenter la version du format, la compatibilité ascendante et la stratégie de migration.
- [ ] Ajouter un champ de version si cela est nécessaire sans casser les templates existants.
- [ ] Ajouter des exemples valides et invalides pour chaque policy MVP.
- [ ] Documenter la correspondance entre les champs JSON et les éléments XML Apigee.
- [ ] Définir une politique pour les champs inconnus : rejet systématique en v1.
- [ ] Vérifier que le CLI, le GUI et le renderer consomment le même contrat.

## 5.2 Définir le mapping OpenAPI → modèle interne

- [ ] Définir les règles pour le serveur principal OpenAPI lorsqu'il y en a plusieurs.
- [ ] Définir la représentation des chemins, paramètres et verbes HTTP dans le modèle interne.
- [ ] Définir la traduction des security schemes OpenAPI vers les policies Forge.
- [ ] Décider le comportement pour API Key header, query et cookie.
- [ ] Décider le comportement pour bearer HTTP, OAuth2 et OpenID Connect.
- [ ] Produire des erreurs explicites lorsqu'un mécanisme OpenAPI ne possède pas d'équivalent MVP.
- [ ] Ne jamais déduire silencieusement une policy de sécurité ambiguë.
- [ ] Définir les règles lorsqu'une sécurité globale est remplacée par une sécurité spécifique à une opération.

## 5.3 Générer réellement les flows conditionnels

- [ ] Transformer chaque route OpenAPI en condition Apigee déterministe.
- [ ] Générer les conditions combinant `proxy.pathsuffix` et `request.verb`.
- [ ] Garantir l'échappement XML des chemins et des conditions.
- [ ] Définir la stratégie pour les paramètres de chemin et les wildcards.
- [ ] Définir le comportement lorsqu'une route possède plusieurs méthodes.
- [ ] Définir le lien entre les security requirements de la route et les policies du flow.
- [ ] Éviter les doublons de flows lorsque deux opérations produisent la même condition.
- [ ] Conserver la possibilité de policies globales dans PreFlow/PostFlow.
- [ ] Ajouter un aperçu des flows générés avant toute écriture ou mutation distante.

## 5.4 Tests de la phase

- [ ] Ajouter des fixtures OpenAPI avec plusieurs routes et verbes.
- [ ] Ajouter une fixture avec sécurité globale et sécurité par opération.
- [ ] Ajouter une fixture avec API Key, OAuth2, bearer/JWT et cas non supporté.
- [ ] Vérifier les conditions XML générées.
- [ ] Vérifier l'absence de flow fantôme ou de policy non référencée.
- [ ] Vérifier le rendu déterministe à entrée identique.
- [ ] Vérifier le rendu depuis un template créé par le GUI puis consommé par le CLI.
- [ ] Vérifier les erreurs avant toute écriture de fichier.

## 5.5 Definition of Done

Une spec OpenAPI représentative du MVP, combinée à un template valide, produit un bundle Apigee dont :

- les endpoints sont valides ;
- chaque route attendue possède un flow ou une règle documentée ;
- les policies sont générées et référencées correctement ;
- les cas non supportés sont refusés clairement ;
- le résultat est consommable par le CLI et le GUI.

---

# 6. Phase 2 — Validation Apigee réelle et robustesse du gateway

**Priorité : haute — preuve nécessaire avant déploiement final**

Cette phase ferme les validations réelles différées dans M2, M4, M6-Bis et M8.

## 6.1 Préparation de l'environnement d'évaluation

- [ ] Vérifier que le projet GCP et l'organisation Apigee d'évaluation sont disponibles.
- [ ] Vérifier les environnements et permissions réellement associés au compte de test.
- [ ] Préparer un credential headless local ou une fédération conforme à la documentation existante.
- [ ] Ne jamais ajouter le fichier de credential, son contenu ou un token au dépôt.
- [ ] Définir un nom de proxy temporaire et unique pour les essais.
- [ ] Définir une procédure de nettoyage manuelle documentée sans suppression automatique dangereuse.

## 6.2 Parcours CLI réel

- [ ] Exécuter `login --headless`.
- [ ] Exécuter `list-proxies --headless`.
- [ ] Générer un bundle à partir de la fixture Hello World.
- [ ] Importer et déployer le bundle avec `deploy --headless`.
- [ ] Lire le statut avec `status --headless`.
- [ ] Vérifier le proxy avec `list-proxies --headless`.
- [ ] Vérifier les réponses réelles, la révision retournée et les délais de propagation.
- [ ] Comparer les réponses réelles aux mappings WireMock.
- [ ] Documenter le résultat dans `REAL_APIGEE_HELLOWORLD_REPORT.md` sans donnée sensible.

## 6.3 Parcours GUI Live réel

- [ ] Démarrer le GUI en mode Live.
- [ ] Restaurer ou réaliser la connexion OAuth desktop.
- [ ] Sélectionner explicitement l'organisation.
- [ ] Sélectionner explicitement l'environnement.
- [ ] Charger les proxies réels.
- [ ] Afficher les statuts de révisions réels.
- [ ] Générer localement un bundle depuis le GUI.
- [ ] Importer une révision depuis le GUI.
- [ ] Vérifier que la révision importée est affichée `Not deployed`.
- [ ] Confirmer le déploiement depuis la revue GUI.
- [ ] Vérifier le polling jusqu'à `Succeeded` ou `Failed`.
- [ ] Vérifier le comportement d'une permission refusée, d'un timeout et d'une session expirée.

## 6.4 Robustesse du gateway

- [ ] Gérer proprement les réponses vides.
- [ ] Décider et documenter la pagination si les volumes Apigee l'exigent.
- [ ] Vérifier les champs absents dans les réponses organisations, environnements, proxies et déploiements.
- [ ] Compléter les états `Pending`, `InProgress`, `Succeeded` et `Failed` du gateway mémoire.
- [ ] Vérifier le mapping de tous les états réels observés.
- [ ] Vérifier qu'un statut inconnu ne devient pas silencieusement un succès.
- [ ] Vérifier les retries uniquement sur les erreurs transitoires.
- [ ] Vérifier l'absence de retry sur une mutation lorsque cela pourrait créer un doublon non idempotent.

## 6.5 Definition of Done

Le CLI et le GUI ont chacun réalisé un parcours réel documenté sur l'organisation d'évaluation, sans secret exposé, et les différences entre API réelle et doubles/tests sont corrigées ou explicitement documentées.

---

# 7. Phase 3 — Permissions, statuts et déploiements sûrs

**Priorité : haute**

## 7.1 Finaliser la gestion des rôles côté GUI

- [ ] Charger les rôles après authentification et sélection d'organisation.
- [ ] Afficher le rôle ou les rôles dans un composant dédié.
- [ ] Distinguer clairement identité, organisation et permissions.
- [ ] Adapter les actions disponibles au rôle effectivement retourné.
- [ ] Masquer ou désactiver les actions interdites sans présenter cela comme une erreur technique.
- [ ] Conserver le contrôle côté Rust/Apigee : le GUI ne doit jamais être la seule barrière de sécurité.
- [ ] Gérer plusieurs rôles, absence de rôle reconnu et rôle inconnu.
- [ ] Tester les profils lecture seule, développeur et déployeur.
- [ ] Réutiliser `useRoles.ts` ou le remplacer par une abstraction réellement branchée dans le flux applicatif.

## 7.2 Réduire le risque N+1 des statuts

Le chargement actuel peut interroger le statut de chaque révision séparément. Il faut éviter que le temps et le nombre de requêtes augmentent linéairement de façon incontrôlée.

- [ ] Mesurer le nombre d'appels pour 1, 10, 50 et 100 révisions.
- [ ] Vérifier si l'API Apigee permet un chargement groupé ou une réponse enrichie.
- [ ] Si aucun endpoint groupé n'est disponible, limiter la concurrence avec un nombre borné de requêtes.
- [ ] Mettre en cache les statuts pendant la durée d'un chargement.
- [ ] Ne pas relancer inutilement les statuts inchangés lors d'un changement d'écran.
- [ ] Prévoir un chargement à la demande des détails de révision.
- [ ] Afficher un statut provisoire sans bloquer toute la liste.
- [ ] Ajouter une métrique/test comptant les appels réseau attendus.
- [ ] Vérifier les comportements de timeout partiel et de statut indisponible.

## 7.3 Finaliser les révisions et le remplacement explicite

- [ ] Définir précisément la différence entre création de proxy, création de révision et déploiement.
- [ ] Bloquer par défaut le déploiement d'une révision déjà active.
- [ ] Ajouter une action de remplacement explicite et séparée.
- [ ] Afficher une confirmation renforcée pour un remplacement.
- [ ] Vérifier la cible complète : organisation, environnement, proxy et révision.
- [ ] Vérifier le statut actuel avant mutation.
- [ ] Gérer les courses entre deux utilisateurs ou deux actions GUI.
- [ ] Rafraîchir les proxies après mutation.
- [ ] Conserver le statut `Not deployed` pour toute révision nouvellement importée.
- [ ] Tester import, déploiement, remplacement refusé, remplacement confirmé, échec et retry.

## 7.4 Definition of Done

Les actions proposées par le GUI correspondent aux permissions et aux statuts réels, le chargement reste maîtrisé avec un grand nombre de révisions et aucune mutation ambiguë ne peut être lancée par erreur.

---

# 8. Phase 4 — Tests d'intégration et non-régression

**Priorité : haute**

Cette phase transforme les scénarios actuellement testés séparément en preuves de parcours complets.

## 8.1 Tests core et CLI

- [ ] Ajouter les états complets du `InMemoryApigeeGateway`.
- [ ] Tester les use cases avec gateway mémoire et repository isolé.
- [ ] Tester l'intégration template GUI → fichier local → CLI `generate`.
- [ ] Ajouter des fixtures valides et invalides sans secrets.
- [ ] Ajouter des tests de pagination ou de volume si retenue en Phase 2.
- [ ] Tester l'idempotence et les erreurs des opérations mutatives.
- [ ] Vérifier les codes de sortie et enveloppes JSON pour chaque commande.
- [ ] Vérifier qu'aucun chemin local sensible n'apparaît dans les erreurs.

## 8.2 Tests GUI/Tauri

- [ ] Ajouter un test de state Tauri complet avec dépendances injectées.
- [ ] Tester les transitions Live/Demo avec gateway et repository fakes.
- [ ] Tester les erreurs d'authentification, permission refusée, expiration et réseau.
- [ ] Tester l'appel réel des rôles côté frontend.
- [ ] Tester les appels de statut bornés et annulables.
- [ ] Tester les doubles soumissions sur génération, upload et déploiement.
- [ ] Tester la navigation pendant un chargement asynchrone.
- [ ] Ajouter, si l'outillage le permet, un test de démarrage Tauri sans compte réel.

## 8.3 Tests éditeur de templates

- [ ] Tester la création d'un template minimal.
- [ ] Tester l'ouverture et la modification d'un template existant.
- [ ] Tester chaque formulaire de policy MVP.
- [ ] Tester ajout, suppression et réordonnancement.
- [ ] Tester la validation invalide et la correction inline.
- [ ] Tester la sauvegarde et le rechargement.
- [ ] Tester la perte de modifications et la confirmation.
- [ ] Tester les noms longs, listes longues et contenus volumineux.
- [ ] Tester les templates incompatibles ou contenant des champs inconnus.

## 8.4 Definition of Done

Tous les parcours critiques possèdent au moins :

- un test nominal ;
- un test d'erreur ;
- un test d'annulation ou de concurrence lorsqu'il y a une opération asynchrone ;
- une vérification de non-exposition des secrets.

---

# 9. Phase 5 — Maintenabilité et découpage du code

**Priorité : moyenne, à réaliser avant release**

## 9.1 Découper `App.vue`

- [ ] Extraire le shell applicatif et la navigation.
- [ ] Extraire le dashboard.
- [ ] Extraire le catalogue de templates.
- [ ] Extraire le catalogue de proxies.
- [ ] Extraire la page Deployments.
- [ ] Extraire Settings et Support si cela réduit réellement la complexité.
- [ ] Conserver dans `App.vue` uniquement la composition et les transitions globales.
- [ ] Déplacer les règles métier frontend dans des composables dédiés.
- [ ] Éviter de dupliquer les validations déjà garanties par `core`.
- [ ] Préserver les contrats et les tests existants pendant chaque extraction.

## 9.2 Découper `src-tauri/src/commands/mod.rs`

Organiser les commandes par responsabilité, par exemple :

```text
gui/src-tauri/src/commands/
├── auth.rs
├── session.rs
├── templates.rs
├── organizations.rs
├── proxies.rs
├── revisions.rs
├── generation.rs
└── deployment.rs
```

- [ ] Extraire les DTO dans des modules cohérents.
- [ ] Extraire les mappings d'erreurs.
- [ ] Conserver un seul composition root dans `lib.rs`.
- [ ] Ajouter des tests ciblés pour chaque groupe de commandes.
- [ ] Ne pas introduire de logique métier dans les handlers Tauri.

## 9.3 Definition of Done

Le découpage diminue la taille et la responsabilité des fichiers sans changer le comportement. Tous les tests existants passent avant et après extraction.

---

# 10. Phase 6 — Dataset Demo et expérience de démonstration

**Priorité : moyenne — clôture de M6-Bis**

Le mode Demo doit devenir une démonstration cohérente du produit, tout en restant clairement fictif et sans réseau.

- [ ] Définir un dataset versionné et explicitement fictif.
- [ ] Inclure une organisation Demo.
- [ ] Inclure au moins deux environnements.
- [ ] Inclure au moins deux proxies.
- [ ] Inclure plusieurs révisions.
- [ ] Inclure des statuts `NotDeployed`, `Pending`, `InProgress`, `Succeeded` et `Failed`.
- [ ] Inclure des rôles Demo représentatifs.
- [ ] Seed uniquement à la première initialisation.
- [ ] Ajouter un reset Demo explicite et confirmé.
- [ ] Vérifier qu'aucun nom, identifiant ou résultat de l'organisation réelle ne figure dans les fixtures.
- [ ] Tester que le mode Demo ne déclenche aucun appel réseau.
- [ ] Tester le parcours complet création → import → déploiement → statut.
- [ ] Ajouter un tutoriel de démonstration reproductible.

## Definition of Done

Un utilisateur peut démarrer le GUI en Demo, comprendre le produit, parcourir un workspace crédible et réaliser le parcours principal sans configuration Google ni réseau.

---

# 11. Phase 7 — Validation frontend, accessibilité et polish M9

**Priorité : moyenne**

Le polish visuel ne doit pas modifier les contrats métier, mais il doit être validé avant release.

- [ ] Capturer les vues de référence Dashboard, Templates, Proxies, Deployments et Settings.
- [ ] Comparer les vues en mode Demo et Live lorsque disponible.
- [ ] Vérifier les tokens contre `DESIGN.md`.
- [ ] Vérifier l'absence de dark mode non prévu.
- [ ] Vérifier les surfaces, bordures, rayons, contrastes et typographies.
- [ ] Vérifier les états loading, empty, error, success, pending et in-progress.
- [ ] Vérifier l'absence de débordement à 960 px et à la taille Tauri nominale.
- [ ] Vérifier le focus clavier sur tous les contrôles.
- [ ] Vérifier `aria-live`, `role=alert`, labels et descriptions.
- [ ] Vérifier `prefers-reduced-motion`.
- [ ] Vérifier la restitution du focus après fermeture des modals.
- [ ] Réévaluer les modifications locales non commitées de `BaseCard.vue`, `App.vue` et `TemplateEditorShell.vue`.
- [ ] Corriger ou documenter les éventuelles régressions de hiérarchie visuelle.

## Definition of Done

Les captures de référence sont validées, les exceptions sont documentées et le polish ne réintroduit aucune régression fonctionnelle ou d'accessibilité.

---

# 12. Phase 8 — Documentation et packaging de release

**Priorité : moyenne à haute — dernier verrou avant diffusion**

## 12.1 README

- [ ] Présenter le projet et sa proposition de valeur.
- [ ] Décrire l'architecture `core` / `cli` / `gui`.
- [ ] Documenter les prérequis Rust, Node, Tauri et Google Cloud.
- [ ] Documenter le mode Demo.
- [ ] Documenter le mode Live.
- [ ] Documenter OAuth desktop sans exposer de secret.
- [ ] Documenter `GOOGLE_APPLICATION_CREDENTIALS` pour le CLI headless.
- [ ] Documenter les commandes CLI avec exemples sûrs.
- [ ] Documenter la génération locale, l'import et le déploiement.
- [ ] Documenter le format JSON du template et renvoyer vers le schema.
- [ ] Documenter les erreurs, codes de sortie et sortie `--json`.
- [ ] Documenter la validation réelle Apigee.
- [ ] Documenter les avertissements de signature Windows/macOS.
- [ ] Ajouter des captures d'écran si elles ne contiennent aucune donnée sensible.

## 12.2 Packaging CLI

- [ ] Vérifier le build release Windows.
- [ ] Vérifier le build release macOS Intel.
- [ ] Vérifier le build release macOS Apple Silicon.
- [ ] Vérifier le build release Linux GNU pour les pipelines.
- [ ] Nommer les binaires de façon stable.
- [ ] Vérifier les artefacts sur une GitHub Release de test ou un dry-run.

## 12.3 Packaging GUI + CLI

- [ ] Finaliser la configuration Tauri.
- [ ] Vérifier l'embarquement du CLI sidecar.
- [ ] Vérifier le build Windows `.msi` ou NSIS.
- [ ] Vérifier le build macOS `.dmg`.
- [ ] Vérifier le contenu et les chemins du bundle.
- [ ] Documenter l'emplacement du CLI embarqué.
- [ ] Documenter les avertissements de signature non payée.
- [ ] Préparer le workflow de release déclenché par tag.
- [ ] Vérifier que la version Cargo, GUI et CLI est synchronisée.

## 12.4 Definition of Done

Une version taguée produit les artefacts CLI et GUI prévus, et un nouvel utilisateur peut installer, configurer ou lancer le projet en suivant uniquement le README.

---

# 13. Phase 9 — Gate finale du MVP

Cette phase ne développe pas de fonctionnalité. Elle vérifie que toutes les phases précédentes sont réellement terminées.

## 13.1 Parcours local Demo

- [ ] Démarrage Demo sans réseau.
- [ ] Dataset chargé.
- [ ] Création ou sélection d'un template.
- [ ] Préparation OpenAPI.
- [ ] Génération locale.
- [ ] Upload Demo.
- [ ] Révision non déployée.
- [ ] Revue de déploiement.
- [ ] Confirmation.
- [ ] Statut final affiché.

## 13.2 Parcours CLI local

- [ ] `template create/list/show/update/delete`.
- [ ] `generate` avec template fichier.
- [ ] `generate` avec template repository.
- [ ] Sortie humaine.
- [ ] Sortie JSON.
- [ ] Codes de sortie.
- [ ] Erreurs sans fuite de chemins ou secrets.

## 13.3 Parcours Live manuel

- [ ] OAuth ou headless fonctionnel.
- [ ] Organisation correcte.
- [ ] Environnement correct.
- [ ] Liste des proxies correcte.
- [ ] Génération sans appel distant.
- [ ] Import réel.
- [ ] Déploiement réel.
- [ ] Statut réel.
- [ ] Rôle et actions GUI cohérents.
- [ ] Aucun secret dans les sorties ou captures.

## 13.4 Qualité automatisée

- [ ] `cargo fmt --all -- --check`.
- [ ] `cargo test --workspace --locked`.
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- [ ] `cargo audit`.
- [ ] `npm test`.
- [ ] `npm run build`.
- [ ] Build Tauri sans bundle.
- [ ] Tests CLI de génération et d'erreurs.
- [ ] Tests de non-régression GUI.

## 13.5 Décision de sortie

Le MVP est considéré comme terminé uniquement si :

1. le mapping OpenAPI → flows Apigee est implémenté et testé ;
2. le contrat template v1 est documenté et stable ;
3. le CLI et le GUI utilisent le même core et le même format ;
4. les parcours Demo et Live sont séparés et sûrs ;
5. les permissions et statuts sont cohérents ;
6. le risque N+1 est mesuré et maîtrisé ;
7. l'import et le déploiement sont explicitement séparés ;
8. les tests critiques passent ;
9. le parcours réel Apigee est documenté ;
10. le README et les artefacts de distribution sont prêts.

---

# 14. Hors périmètre jusqu'à la fin du MVP

Les éléments suivants ne doivent pas détourner l'effort de finalisation :

- analytics Apigee avancées ;
- tableau de bord de trafic et latence ;
- versioning visuel entre révisions ;
- rollback automatique ;
- stratégies canary ou blue/green ;
- policies Apigee avancées ;
- support de Kong, AWS API Gateway ou Azure APIM ;
- bibliothèque ou marketplace de templates ;
- multi-tenant ;
- import inversé depuis un proxy Apigee ;
- connecteur Layer7 ;
- publication open source complète avant la stabilisation du MVP.

Ces sujets pourront constituer une roadmap post-MVP dédiée après la gate finale.

---

# 15. Ordre recommandé d'exécution

L'ordre suivant minimise le risque de construire du polish ou de la documentation sur des contrats encore instables :

```text
Phase 0  Audit et baseline
   ↓
Phase 1  Contrat template + mapping OpenAPI/Apigee
   ↓
Phase 2  Validation réelle et robustesse gateway
   ↓
Phase 3  Rôles + statuts + remplacement explicite
   ↓
Phase 4  Tests d'intégration et non-régression
   ↓
Phase 5  Découpage App.vue / commandes Tauri
   ↓
Phase 6  Dataset Demo
   ↓
Phase 7  QA visuelle et accessibilité
   ↓
Phase 8  README et packaging
   ↓
Phase 9  Gate finale du MVP
```

La priorité immédiate est donc la **Phase 0**, suivie sans détour par la **Phase 1**. Tant que le mapping OpenAPI et le contrat de template ne sont pas stabilisés, une validation de déploiement final resterait incomplète, même si le parcours GUI actuel fonctionne avec des fixtures.
