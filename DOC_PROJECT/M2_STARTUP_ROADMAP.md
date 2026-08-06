# Apigee Forge — Roadmap de démarrage détaillée M2

*Document atomique du jalon M2 — Client Apigee. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M2

M2 implémente dans le package partagé `apigee-forge-core` les frontières d'authentification et de communication avec l'API Management Apigee :

- authentification headless via `gcp_auth` pour le CLI et les pipelines ;
- authentification OAuth2 desktop avec PKCE pour le GUI futur ;
- stockage du refresh token dans le trousseau OS ;
- récupération des organisations, environnements et proxies accessibles ;
- lecture du rôle Apigee de l'identité connectée ;
- client HTTP réutilisable, sécurisé, testable et indépendant des use cases.

M2 reste **core-only**. Les commandes CLI complètes prévues par M4 et les commandes Tauri prévues par M6/M8 ne sont pas implémentées dans ce jalon.

---

## 2. Règles de méthode

- Une seule étape atomique à la fois.
- Après chaque étape : vérification, présentation du résultat, commit, arrêt pour validation.
- Aucun credential réel requis pour les tests unitaires ou CI.
- Toute requête Apigee doit être implémentée à partir de la documentation REST officielle, jamais d'un exemple improvisé.
- Aucun token, refresh token, credential ou header Authorization dans les logs, rapports ou fichiers committés.
- Aucun `.unwrap()`/`.expect()` dans `core/`.
- Les use cases dépendent uniquement des ports ; le composition root est le seul endroit qui branche les implémentations concrètes.
- Tout pattern Rust non trivial doit être expliqué avant acceptation.

---

## 3. Organisation Git M2

M1 a été fusionné dans `dev` avec un merge commit explicite :

```text
6045bd5 merge: integrate M1 core foundations
```

La branche de travail M2 est :

```text
feature/m2-apigee-client
```

Aucune sous-branche M2 n'est prévue pour le moment. Les séparations seront réalisées par des commits atomiques. Aucun push distant ne doit être effectué sans demande explicite.

---

## 4. Pré-requis techniques

- Workspace Cargo compilable avec `Cargo.lock` commité.
- Ports existants dans `core/src/ports/`.
- Erreurs typées dans `core/src/error.rs`.
- `async-trait` disponible pour les ports asynchrones.
- `reqwest` avec TLS vérifié et timeout explicite.
- `gcp_auth` pour le mode headless.
- `oauth2` pour Authorization Code + PKCE.
- `keyring` pour le refresh token desktop.
- `wiremock` uniquement en dépendance de test pour les appels HTTP simulés.

---

## 5. Étapes atomiques M2

### M2-00 — Documentation et baseline Git

- [x] Fusionner M1 dans `dev` avec `--no-ff`.
- [x] Créer `feature/m2-apigee-client` depuis `dev`.
- [x] Créer ce document.
- [x] Référencer ce document dans `STRUCTURE.md` et `PROMPT.md`.
- [x] Committer la documentation M2.

Commit prévu :

```text
docs(m2): add detailed client roadmap
```

### M2-01 — Refinement des contrats d'authentification

- [x] Examiner `AuthProvider` et éviter les `String` ambigus pour le contexte d'identité.
- [x] Décider comment représenter project ID, organisation sélectionnée, identité et expiration.
- [x] Séparer access token temporaire et refresh token persistant.
- [x] Conserver les secrets hors du domaine et hors de `LocalStateStore`.
- [x] Ajouter les erreurs typées nécessaires sans contenu sensible.
- [x] Ajouter les tests de contrat pour absence, expiration et token indisponible.
- [x] Vérifier tests et Clippy.

Commit prévu :

```text
refactor(core): refine M2 authentication contracts
```

### M2-02 — Configuration headless sécurisée

- [x] Lire uniquement `GOOGLE_APPLICATION_CREDENTIALS`.
- [x] Refuser les credentials en argument de commande.
- [x] Ne pas distinguer applicativement WIF et clé de service account.
- [x] Documenter que la pipeline produit le fichier de credentials.
- [x] Tester l'absence et l'invalidité de la variable sans appeler Google.
- [x] Vérifier l'absence de credentials dans les erreurs et logs.

Commit prévu :

```text
feat(core): define headless authentication configuration
```

### M2-03 — Provider headless `gcp_auth`

- [ ] Ajouter `gcp_auth` au package `apigee-forge-core`.
- [ ] Créer `core/src/infra/service_account_auth_provider.rs`.
- [ ] Utiliser `gcp_auth::provider()`.
- [ ] Demander le scope `https://www.googleapis.com/auth/cloud-platform`.
- [ ] Récupérer l'access token court uniquement en mémoire.
- [ ] Utiliser `project_id()` pour la résolution headless de l'organisation.
- [ ] Mapper les erreurs `gcp_auth` vers `AuthError` sans exposer le détail sensible.
- [ ] Tester la configuration sans credential réel.

Commit prévu :

```text
feat(core): add headless GCP auth provider
```

### M2-04 — Provider OAuth desktop PKCE

- [ ] Ajouter `oauth2` et `keyring` selon les versions validées.
- [ ] Créer `core/src/infra/oauth_desktop_auth_provider.rs`.
- [ ] Implémenter Authorization Code + PKCE.
- [ ] Générer et vérifier le state CSRF.
- [ ] Utiliser une redirection localhost limitée à la session.
- [ ] Configurer l'échange OAuth sans redirections HTTP automatiques.
- [ ] Stocker uniquement le refresh token dans le trousseau OS.
- [ ] Conserver l'access token et son expiration uniquement en mémoire.
- [ ] Gérer l'absence, la révocation et la suppression du refresh token.
- [ ] Ne jamais mettre de secret client confidentiel en dur.
- [ ] Tester les transitions avec des doubles contrôlables, sans navigateur ni compte réel.

Commit prévu :

```text
feat(core): add desktop OAuth provider
```

### M2-05 — Contrat et transport HTTP Apigee

- [ ] Raffiner les modèles de domaine si les réponses Apigee ne tiennent pas dans `Vec<String>`, `Proxy` et `Deployment` actuels.
- [ ] Confirmer les DTO internes nécessaires sans exposer les types de `reqwest` ou de l'API dans le domaine.
- [ ] Ajouter `reqwest` avec TLS et timeout explicite.
- [ ] Créer `core/src/infra/reqwest_apigee_gateway.rs`.
- [ ] Injecter `Arc<dyn AuthProvider>`.
- [ ] Réutiliser une seule instance de `reqwest::Client`.
- [ ] Ajouter le header Bearer sans jamais le journaliser.
- [ ] Mapper les statuts HTTP et réponses invalides vers `GatewayError`.
- [ ] Définir retries et backoff bornés uniquement pour les erreurs transitoires.

Commit prévu :

```text
feat(core): add Apigee HTTP gateway
```

### M2-06 — Organisations, environnements et proxies

- [ ] Implémenter le mapping de `organizations.list`.
- [ ] Implémenter le mapping de `organizations.environments.list`.
- [ ] Implémenter le mapping de `organizations.apis.list` avec les champs nécessaires aux révisions.
- [ ] Gérer les réponses vides, champs absents et pagination si applicable.
- [ ] Ne charger que les métadonnées nécessaires au MVP.
- [ ] Tester chaque mapping avec des réponses JSON WireMock documentées.

Endpoints de référence à consulter avant chaque implémentation :

- `GET https://apigee.googleapis.com/v1/organizations`
- `GET https://apigee.googleapis.com/v1/organizations/{org}/environments`
- `GET https://apigee.googleapis.com/v1/organizations/{org}/apis`

Commit possible par frontière fonctionnelle si le changement devient trop large.

### M2-07 — Lecture du rôle Apigee

- [ ] Identifier l'endpoint officiel et les permissions exactes.
- [ ] Confirmer s'il s'agit d'une lecture IAM ou d'un endpoint Apigee dédié.
- [ ] Ne pas ajouter de création ou modification de bindings IAM.
- [ ] Mapper uniquement les rôles Apigee autorisés par `ApigeeRole`.
- [ ] Refuser explicitement un rôle inconnu.
- [ ] Tester admin, read-only/deployer et réponse inconnue.

Commit prévu :

```text
feat(core): resolve Apigee role
```

### M2-08 — Tests HTTP simulés et sécurité

- [ ] Ajouter `wiremock` en dépendance de test uniquement.
- [ ] Simuler les endpoints selon les réponses de la documentation officielle.
- [ ] Vérifier les headers sans imprimer de token.
- [ ] Tester 200, 401, 403, 404, 429, 5xx et JSON invalide.
- [ ] Tester les timeouts et le nombre maximal de retries.
- [ ] Vérifier le backoff sans boucle infinie.
- [ ] Produire un rapport spécifique par test dans `target/test-results/`.
- [ ] Vérifier qu'aucun rapport ne contient de secret.

Commit prévu :

```text
test(core): add WireMock Apigee coverage
```

### M2-09 — Validation finale M2

- [ ] `cargo test --workspace --locked` passe.
- [ ] `cargo clippy --workspace --all-targets --locked -- -D warnings` passe.
- [ ] `cargo audit` passe dans la CI.
- [ ] Les logs et rapports ne contiennent aucun credential.
- [ ] La validation manuelle de l'organisation d'évaluation est documentée mais n'est pas requise par la CI.
- [ ] Aucun CLI complet n'est ajouté avant M4.
- [ ] Mettre à jour `ROADMAP.md` et ce document.
- [ ] Committer la clôture M2.

Commit prévu :

```text
docs(m2): record M2 validation
```

---

## 6. Règles SOLID et optimisation

- **Single Responsibility** : séparer providers de credentials, trousseau, transport HTTP, DTO mapping, erreurs et composition root.
- **Open/Closed** : ajouter les providers OAuth/headless derrière `AuthProvider` sans modifier les use cases.
- **Liskov** : les providers doivent respecter les mêmes contrats d'expiration, d'erreur et de non-exposition des secrets.
- **Interface Segregation** : ne pas transformer `ApigeeGateway` en god trait ; réévaluer le découpage si le périmètre augmente.
- **Dependency Inversion** : les use cases ne dépendent que des ports ; l'infrastructure est branchée uniquement dans les composition roots.
- Réutiliser les clients HTTP.
- Éviter les clones inutiles de tokens et de grosses réponses.
- Ne jamais tenir un verrou ou une opération bloquante à travers un `.await`.
- Borner taille des entrées, réponses et retries.
- TLS activé, timeout obligatoire, backoff borné.
- Aucun secret dans code, logs, rapports, `SqlCipherLocalStore` ou fichiers committés.

---

## 7. Definition of Done M2

M2 sera terminé lorsque :

1. les providers headless et desktop respectent les contrats validés ;
2. le client Apigee couvre organisations, environnements, proxies et lecture du rôle ;
3. les appels sont testés avec WireMock sans compte Apigee réel ;
4. erreurs réseau, timeouts, retries et réponses invalides sont couverts ;
5. aucun secret n'apparaît dans le code, les logs ou les rapports ;
6. tests, Clippy et audit CI passent ;
7. la validation manuelle GCP est documentée mais ne bloque pas la CI ;
8. les commandes CLI restent réservées au jalon M4.

---

## 8. Questions à résoudre au début de l'implémentation

- Le contrat `AuthProvider` doit-il retourner un contexte project ID/organisation plutôt qu'un simple `()` et `String` ?
- Quels champs d'organisation et d'environnement sont nécessaires au futur GUI ?
- Quelle API officielle permet la lecture du rôle Apigee connecté sans gestion IAM générique ?
- Quelle configuration non secrète porte le client OAuth desktop ?
- Que faire lorsqu'aucun refresh token n'est disponible ?
- Quel niveau de retry est acceptable pour Apigee et les tests WireMock ?
