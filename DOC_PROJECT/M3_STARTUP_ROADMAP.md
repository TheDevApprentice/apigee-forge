# Apigee Forge — Roadmap de démarrage détaillée M3

*Document atomique du jalon M3 — Moteur de rendu. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M3

M3 transforme une spec OpenAPI et un template Apigee Forge en un bundle de proxy Apigee valide :

```text
spec OpenAPI + Template → modèle de rendu → fichiers XML apiproxy → archive .zip
```

Le bundle doit respecter la structure Apigee officielle avec `apiproxy/` comme répertoire racine de l’archive.

M3 reste local et sans déploiement réel. Le déploiement et les appels Apigee déjà préparés en M2 ne sont pas utilisés pour produire le bundle.

---

## 2. Périmètre

Inclus :

- extraction des routes, méthodes, serveurs et sécurités nécessaires depuis OpenAPI ;
- validation du template avant rendu ;
- modèle interne de rendu indépendant des crates externes ;
- génération des ProxyEndpoint et TargetEndpoint XML ;
- génération des policies MVP ;
- écriture contrôlée du bundle sur disque ;
- packaging ZIP avec `apiproxy/` comme racine ;
- validation XML et tests golden/fixtures ;
- vérification du contenu ZIP et des chemins générés ;
- sortie locale utilisable par un test ou un CLI minimal.

Hors scope :

- déploiement vers Apigee ;
- appels réseau Apigee ;
- policies avancées hors MVP ;
- éditeur GUI ;
- commandes CLI complètes prévues par M4 ;
- génération de code client ;
- exécution de scripts ou de templates arbitraires fournis par l’utilisateur ;
- écriture de fichiers en dehors du répertoire de sortie explicitement choisi.

---

## 3. Règles de méthode

- Une seule étape atomique à la fois.
- Après chaque étape : tests, inspection du diff, commit, arrêt pour validation.
- Aucun `.unwrap()`/`.expect()` dans `core/`.
- Le domaine ne dépend ni de Tera, ni de ZIP, ni de `quick-xml`.
- Les use cases dépendent uniquement de ports et du domaine.
- Les implémentations de rendu et de packaging restent dans `infra/`.
- Aucun contenu XML ou chemin dérivé d’une entrée utilisateur ne doit contourner la validation.
- Toute erreur de template, XML, fichier ou archive doit être typée et propagée par `Result`.
- Les rapports de tests restent dans `target/test-results/` et ne contiennent aucun secret.

---

## 4. Structure cible M3

Fichiers à ajouter ou à confirmer, après mise à jour de `STRUCTURE.md` :

```text
core/src/
├── domain/
│   └── render.rs                         # modèles internes de rendu
├── use_cases/
│   └── generate_proxy_bundle.rs          # orchestration métier du rendu
├── ports/
│   ├── bundle_renderer.rs                # contrat de génération de fichiers
│   └── bundle_archiver.rs                # contrat de packaging ZIP
└── infra/
    ├── tera_bundle_renderer.rs           # rendu Tera/XML concret
    ├── zip_bundle_archiver.rs            # packaging ZIP concret
    └── templates/                        # templates XML internes versionnés
```

Les chemins exacts devront être validés contre `STRUCTURE.md` avant création. Toute nouvelle abstraction non prévue devra d’abord être documentée dans la structure canonique.

---

## 5. Dépendances envisagées

Versions à vérifier avec Cargo et le toolchain Rust avant ajout :

- `tera = { version = "=1.20.1", default-features = false }` pour le rendu ;
- `zip = { version = "=7.2.0", default-features = false, features = ["deflate"] }` avec uniquement les features nécessaires, afin de rester compatible avec Rust 1.85.1 ;
- `quick-xml = "=0.40.1"` pour parser/valider la structure XML dans les tests ;
- aucune dépendance réseau ou Apigee supplémentaire dans M3.

Les versions doivent être verrouillées dans `Cargo.lock` et auditées par la CI.

---

## 6. Étapes atomiques M3

### M3-00 — Baseline Git et documentation

- [x] Vérifier que la branche `feature/m3-rendering-engine` est issue de `dev` après le merge M2.
- [x] Vérifier `cargo test --workspace --locked` et Clippy avant tout code.
- [x] Créer ce document.
- [x] Mettre à jour `STRUCTURE.md` avec les nouveaux emplacements M3.
- [x] Référencer ce document dans `PROMPT.md`.
- [x] Committer uniquement la documentation.

Commit prévu :

```text
docs(m3): add detailed rendering roadmap
```

### M3-01 — Contrat des entrées de rendu

- [x] Examiner le parseur OpenAPI existant : il expose actuellement routes et sécurités, mais pas nécessairement tous les champs requis par le rendu.
- [x] Définir les types internes de rendu : proxy name, target URL, routes, méthodes, conditions et contexte de sécurité.
- [x] Ne pas utiliser directement `openapiv3::OpenAPI` dans les use cases de rendu.
- [x] Définir explicitement les règles de sélection du serveur OpenAPI.
- [x] Refuser une spec sans serveur/target exploitable selon la règle retenue.
- [x] Borner la taille des specs et templates chargés.
- [x] Ajouter tests de cas nominal, absence de serveur, taille excessive et identifiants/URLs invalides.

Commit prévu :

```text
refactor(core): define rendering input model
```

### M3-02 — Contrat du moteur de rendu et validation du template

- [ ] Définir `BundleRenderer` comme port indépendant de Tera.
- [ ] Définir `BundleArchiver` comme port indépendant de ZIP.
- [ ] Définir les erreurs `RenderError`, `BundleError` ou types équivalents.
- [ ] Valider le template contre `schemas/template.schema.json` avant tout rendu.
- [ ] Refuser les policies ou champs inconnus au lieu de les ignorer silencieusement.
- [ ] Définir un contexte de rendu sérialisable et stable.
- [ ] Tester les erreurs de validation sans écriture partielle.

Commit prévu :

```text
feat(core): define bundle rendering contracts
```

### M3-03 — Dépendances et templates XML internes

- [x] Ajouter Tera, `zip` et `quick-xml` avec versions compatibles et lockfile mis à jour.
- [x] Créer les templates XML internes versionnés sous `core/src/infra/templates/`.
- [ ] Activer un mode de rendu strict et éviter les insertions qui paniquent sur une erreur de sérialisation.
  - Note : cette décision reste volontairement ouverte et devra être tranchée avant la clôture M3, lors du renderer concret et de la validation des templates utilisateur.
- [x] Utiliser `Context::try_insert` ou `Context::from_serialize` avec propagation d’erreur.
- [x] Ne pas permettre l’exécution de fonctions arbitraires dans les templates utilisateurs.
- [x] Définir les échappements XML attendus pour les noms, URLs, conditions et valeurs de policy.
- [x] Ajouter un test de syntaxe Tera et un test d’échappement XML.

Commit prévu :

```text
chore(core): add rendering dependencies and XML templates
```

### M3-04 — Rendu ProxyEndpoint et TargetEndpoint

- [x] Générer `apiproxy/proxies/default.xml`.
- [x] Générer `apiproxy/targets/default.xml`.
- [x] Définir la cible backend à partir du modèle OpenAPI validé.
- [x] Générer PreFlow, flows conditionnels et PostFlow selon le modèle de template.
- [x] Produire les conditions de chemin/verbe avec échappement XML.
- [x] Refuser un nom de proxy ou de fichier dangereux.
- [x] Tester le rendu XML sur une spec + template de référence.

Commit prévu :

```text
feat(core): render proxy and target endpoints
```

### M3-05 — Rendu des policies MVP

- [x] Implémenter le rendu API Key.
- [x] Implémenter le rendu OAuth2.
- [x] Implémenter le rendu JWT.
- [x] Implémenter le rendu Quota.
- [x] Implémenter le rendu Spike Arrest.
- [x] Implémenter le rendu CORS.
- [x] Implémenter le rendu XML↔JSON basique.
- [x] Utiliser une stratégie additive par policy, sans modifier le cœur du moteur pour chaque nouvelle variante.
- [x] Vérifier les paramètres requis et les valeurs interdites avant génération.
- [x] Tester chaque policy avec un fixture XML attendu.

Commit possible par groupe cohérent de policies si le changement devient trop large :

```text
feat(core): render MVP security policies
feat(core): render MVP traffic policies
feat(core): render MVP transformation policies
```

### M3-06 — Écriture du bundle sur disque

- [x] Implémenter un writer contrôlé du répertoire `apiproxy/`.
- [x] Créer uniquement les chemins autorisés : `proxies/`, `targets/`, `policies/`, `resources/` si nécessaire.
- [x] Refuser toute traversée de chemin.
- [x] Écrire via des fichiers temporaires ou une stratégie évitant un bundle partiellement valide.
- [x] Borner la taille des fichiers générés.
- [x] Tester un nom de proxy malveillant, un nom de policy invalide et une erreur d’écriture.

Commit prévu :

```text
feat(core): write validated proxy bundle directory
```

### M3-07 — Packaging ZIP

- [ ] Implémenter `ZipWriter` derrière `BundleArchiver`.
- [ ] Garantir que le premier niveau de l’archive est `apiproxy/`.
- [ ] Écrire les fichiers avec des chemins relatifs normalisés.
- [ ] Ne jamais inclure un chemin absolu ou `..` dans l’archive.
- [ ] Utiliser une compression déterministe et compatible avec Apigee.
- [ ] Éviter de construire inutilement une archive complète en mémoire ; écrire vers un `File` ou writer.
- [ ] Tester la liste exacte des entrées ZIP et extraire les fichiers de contrôle.
- [ ] Tester archive vide/incomplète et erreur de finalisation.

Commit prévu :

```text
feat(core): package proxy bundle as ZIP
```

### M3-08 — Use case `generate_proxy_bundle`

- [ ] Ajouter `generate_proxy_bundle.rs` dans `use_cases/`.
- [ ] Injecter les ports de rendu et d’archivage, jamais Tera/ZIP directement.
- [ ] Orchestrer validation → modèle interne → rendu → écriture → packaging.
- [ ] Éviter toute écriture partielle si une étape échoue.
- [ ] Retourner un résultat typé contenant le chemin du bundle et ses métadonnées non sensibles.
- [ ] Tester le use case avec fakes des ports, sans disque réel si possible.
- [ ] Ajouter un rapport de test spécifique.

Commit prévu :

```text
feat(core): add generate proxy bundle use case
```

### M3-09 — Vérification XML, ZIP et fixture de référence

- [ ] Vérifier que chaque XML généré est bien formé avec `quick-xml`.
- [ ] Vérifier les éléments obligatoires ProxyEndpoint/TargetEndpoint.
- [ ] Vérifier la présence des policies attendues.
- [ ] Vérifier que l’archive se décompresse et possède la racine `apiproxy/`.
- [ ] Vérifier qu’aucun secret ou contenu non prévu n’est rendu.
- [ ] Produire les rapports d’artefacts sous `target/test-results/` sans commit des bundles générés.
- [ ] Définir un fixture golden stable et lisible pour les revues.

Commit prévu :

```text
test(core): validate generated Apigee bundle fixtures
```

### M3-10 — CLI minimal de génération et point de contrôle M3

- [ ] Décider si le CLI minimal de génération est nécessaire avant M4 ou si le use case suffit pour M3.
- [ ] Si nécessaire, ajouter uniquement une commande thin adapter sans logique métier.
- [ ] Ne pas implémenter les commandes CLI complètes de M4.
- [ ] Exécuter `cargo test --workspace --locked`.
- [ ] Exécuter Clippy avec `-D warnings`.
- [ ] Exécuter `cargo audit`.
- [ ] Vérifier l’absence de secrets et chemins dangereux dans les fixtures/rapports.
- [ ] Marquer M3 terminé dans `ROADMAP.md` et ce document seulement quand le bundle ZIP de référence est validé.

Commit prévu :

```text
docs(m3): record M3 validation
```

---

## 7. Règles SOLID, performance et sécurité

- **Single Responsibility** : parsing OpenAPI, construction du modèle de rendu, rendu Tera, écriture de fichiers et ZIP restent séparés.
- **Open/Closed** : chaque policy MVP est un module/renderer additif ; le moteur central ne doit pas devenir une suite de branches impossible à maintenir.
- **Liskov** : les fakes et implémentations de `BundleRenderer`/`BundleArchiver` doivent produire les mêmes garanties de validité et d’erreur.
- **Interface Segregation** : ne pas exposer un port qui mélange rendu, écriture et déploiement.
- **Dependency Inversion** : les use cases ne connaissent ni Tera, ni ZIP, ni `reqwest`.
- Utiliser `render_to` et des writers pour limiter les copies de grosses chaînes.
- Écrire le ZIP en streaming vers un fichier plutôt que construire toute l’archive en mémoire.
- Trier les entrées générées pour obtenir des archives reproductibles.
- Définir des timestamps/permissions déterministes si la crate ZIP le permet sans compromettre la compatibilité.
- Échapper toutes les valeurs XML issues d’OpenAPI ou du template.
- Refuser les chemins absolus, `..`, séparateurs inattendus et noms vides.
- Borner taille des specs, templates, XML individuels et bundle final.
- Ne jamais rendre ou recopier des credentials dans les XML, logs ou rapports.

---

## 8. Definition of Done M3

M3 sera terminé lorsque :

1. une spec OpenAPI et un template valides produisent un répertoire `apiproxy/` cohérent ;
2. les XML ProxyEndpoint/TargetEndpoint sont bien formés ;
3. les policies MVP attendues sont générées avec les paramètres du template ;
4. le bundle ZIP a `apiproxy/` comme racine et se décompresse correctement ;
5. les validations path traversal, XML, taille et erreurs sont couvertes ;
6. un use case orchestre le rendu sans dépendre des implémentations infra ;
7. les tests golden et rapports sont inspectables sans secrets ;
8. `cargo test`, Clippy, `cargo audit` et le lockfile passent ;
9. aucun déploiement Apigee réel n’est déclenché par M3 ;
10. aucune commande CLI complète de M4 n’est implémentée prématurément.

---

## 9. Questions à résoudre avant l’implémentation

- Quels champs OpenAPI supplémentaires faut-il exposer pour sélectionner le backend et construire les conditions ?
- Le template JSON reste-t-il uniquement la source de configuration, ou des templates Tera internes doivent-ils être versionnés séparément ?
- Quelle convention de target endpoint doit être appliquée quand la spec contient plusieurs servers ?
- Les policies doivent-elles être générées dans des fichiers XML séparés avec des noms déterministes dérivés du template ?
- Quelle politique de compression et de métadonnées ZIP garantit la reproductibilité sans réduire la compatibilité Apigee ?
- Le CLI minimal est-il requis en M3, ou le use case testable suffit-il jusqu’à M4 ?
