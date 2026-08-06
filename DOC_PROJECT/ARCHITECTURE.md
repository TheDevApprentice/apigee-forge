# Apigee Forge — Architecture technique

*Document normatif. Toute IA travaillant sur ce projet doit suivre ces règles précisément — elles existent pour éviter du code Rust non-idiomatique ("C# traduit en Rust de force") et pour garantir une structure cohérente malgré l'absence d'expertise Rust approfondie côté superviseur humain.*

---

## 1. Principe général

**Pas d'OOP classique.** Rust n'a ni classes, ni héritage. On obtient les mêmes bénéfices (séparation des responsabilités, réutilisabilité, modifications cadrées) via :
- des **structs** pour les données
- des **traits** pour les contrats de comportement (équivalent des interfaces — jamais d'implémentation partagée par héritage)
- de la **composition** (une struct contient/utilise d'autres structs, jamais n'"hérite" d'une autre)

Clean Architecture reste l'objectif — elle s'implémente ici via les traits comme mécanisme d'inversion de dépendance, pas via de l'héritage.

**Règle à donner explicitement à l'IA à chaque session : ne jamais essayer de simuler des classes/héritage en Rust (pas de `Rc<RefCell<>>` pour imiter un objet mutable partagé à la C#, pas de sur-usage de trait objects pour singer une hiérarchie de classes). Toujours privilégier structs + traits + composition.**

---

## 2. Structure des dossiers — package Cargo `apigee-forge-core` (dossier `core/`)

```
core/
├── domain/       # entités pures, aucune dépendance externe, aucune I/O
│   ├── proxy.rs         (struct Proxy, struct ProxyRevision)
│   ├── template.rs      (struct Template, enum PolicyType)
│   ├── deployment.rs    (struct Deployment, enum DeploymentStatus)
│   └── role.rs           (enum ApigeeRole)
├── use_cases/    # logique métier, dépend UNIQUEMENT des traits de ports/
│   ├── generate_proxy_bundle.rs
│   ├── deploy_proxy.rs
│   ├── create_template.rs
│   └── list_proxies.rs
├── ports/        # traits = contrats, aucune implémentation ici
│   ├── apigee_gateway.rs      (trait ApigeeGateway)
│   ├── template_repository.rs (trait TemplateRepository)
│   ├── auth_provider.rs        (trait AuthProvider)
│   └── local_state_store.rs    (trait LocalStateStore)
└── infra/        # implémentations concrètes des traits ci-dessus
    ├── reqwest_apigee_gateway.rs
    ├── filesystem_template_repository.rs
    ├── oauth_desktop_auth_provider.rs
    ├── service_account_auth_provider.rs
    └── sqlcipher_local_store.rs
```

Le dossier `core/` contient le package Cargo `apigee-forge-core`, dont le nom de bibliothèque Rust est `apigee_forge_core`. Le nom évite la collision avec le crate standard Rust `core`.

**Règle stricte** : `domain/` ne dépend de rien. `use_cases/` ne dépend que de `domain/` et `ports/` (jamais de `infra/` directement). `infra/` implémente les traits de `ports/` et peut dépendre de crates externes (reqwest, etc.). Cette direction de dépendance ne doit jamais être inversée.

---

## 3. Exemple concret — un port et son implémentation

```rust
// core/ports/apigee_gateway.rs
use async_trait::async_trait;
use crate::domain::{Proxy, Deployment};

#[async_trait]
pub trait ApigeeGateway: Send + Sync {
    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError>;
    async fn deploy(&self, org: &str, bundle: Vec<u8>) -> Result<Deployment, GatewayError>;
    async fn get_deployment_status(&self, deployment_id: &str) -> Result<Deployment, GatewayError>;
}
```

```rust
// core/infra/reqwest_apigee_gateway.rs
pub struct ReqwestApigeeGateway {
    client: reqwest::Client,
    auth: Arc<dyn AuthProvider>,
}

#[async_trait]
impl ApigeeGateway for ReqwestApigeeGateway {
    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError> {
        // appel HTTP réel via reqwest, utilise self.auth pour le token
    }
    // ...
}
```

Notez `#[async_trait]` (crate `async-trait`) : nécessaire pour que les traits avec méthodes async restent utilisables en `dyn Trait` (dispatch dynamique). Sans ça, les traits async natifs de Rust ne sont pas "object-safe" dans notre cas d'usage.

Un `use_case` ne connaît que le trait `ApigeeGateway`, jamais `ReqwestApigeeGateway` :

```rust
// core/use_cases/list_proxies.rs
pub struct ListProxiesUseCase {
    gateway: Arc<dyn ApigeeGateway>,
}

impl ListProxiesUseCase {
    pub async fn execute(&self, org: &str) -> Result<Vec<Proxy>, GatewayError> {
        self.gateway.list_proxies(org).await
    }
}
```

**Référence pour l'implémentation des appels réels** : ne pas deviner ni s'appuyer sur des exemples Postman collectés à la main. La documentation REST officielle d'Apigee est exhaustive et bien structurée — Claude Code doit la lire directement pour mapper chaque endpoint : `https://cloud.google.com/apigee/docs/reference/apis/apigee/rest`. Lire la doc garantit une requête bien écrite ; ça ne remplace pas la stratégie de test (section 12) qui garantit qu'elle fonctionne réellement.

---

## 4. Authentification et résolution de l'organisation

### Les deux modes d'authentification

- **OAuth2 desktop (interactif)** — utilisé par le GUI. Un être humain (développeur ou superviseur) se connecte avec sa propre identité Google. C'est un mécanisme **par personne**, pas par application : deux personnes différentes lancent chacune leur propre flux de connexion, avec leurs propres permissions IAM réelles. Aucune gestion d'utilisateurs applicative à construire — Google IAM reste l'unique source de vérité des permissions.
- **Service Account / Workload Identity Federation (headless)** — utilisé par le CLI en pipeline, où aucune interaction humaine n'est possible.

### Résolution de l'organisation Apigee

Chaque appel à l'API Management Apigee exige l'identifiant de l'organisation dans le chemin de l'URL (`/v1/organizations/{org}/...`). Une organisation Apigee correspond à un seul projet Google Cloud et partage son nom — ce n'est pas un identifiant à part, c'est le nom du projet GCP.

Cet identifiant est toujours requis, mais sa **source diffère selon le mode d'auth** :
- **Service account** : le fichier de clé JSON contient déjà `project_id` — l'organisation est déduite automatiquement des credentials, aucune saisie/sélection nécessaire.
- **OAuth interactif** : l'identité Google connectée peut avoir accès à plusieurs projets/organisations — l'utilisateur doit explicitement sélectionner laquelle cibler après connexion (écran de sélection d'organisation dans le GUI).

### Authentification du CLI en pipeline — une seule voie de résolution

Le CLI ne doit **pas** contenir de logique "essaie WIF, sinon service account". À la place, il lit uniquement la variable d'environnement standard Google **`GOOGLE_APPLICATION_CREDENTIALS`**, qui pointe vers un fichier JSON — que ce fichier soit une clé de service account classique ou une configuration Workload Identity Federation (générée via `gcloud iam workload-identity-pools create-cred-config`) est indifférent au code : la crate `gcp_auth` gère la distinction en interne.

C'est la pipeline (pas notre code) qui décide comment ce fichier est produit :
- Via une action officielle (`google-github-actions/auth`, l'équivalent GitLab, ou une connexion de service fédérée Azure DevOps) qui pose `GOOGLE_APPLICATION_CREDENTIALS` automatiquement après résolution WIF.
- Ou via une clé de service account classique stockée en secret de pipeline, écrite dans un fichier temporaire, avec `GOOGLE_APPLICATION_CREDENTIALS` pointant dessus.

**Ne pas documenter ou inventer un nom de secret propre à notre outil — s'appuyer sur cette convention Google déjà standard.**

---

## 5. Composition root — où tout se branche

Le "composition root" est l'endroit unique où on choisit quelle implémentation concrète brancher derrière chaque trait. **C'est le seul endroit du programme qui doit connaître à la fois les traits (`ports/`) et leurs implémentations (`infra/`).**

- Dans `cli/src/main.rs` : lit `GOOGLE_APPLICATION_CREDENTIALS`, instancie l'`AuthProvider` headless correspondant, l'enveloppe dans `Arc<dyn AuthProvider>`, l'injecte dans les use cases.
- Dans `gui/src-tauri/src/lib.rs` : instancie `OAuthDesktopAuthProvider` (jamais de service account côté interface graphique interactive), ainsi que `SqlCipherLocalStore` pour l'état local.

Utiliser `Arc<dyn Trait + Send + Sync>` plutôt que des génériques : plus simple à écrire, le choix d'implémentation peut se faire à l'exécution, coût de performance négligeable pour ce projet.

---

## 6. Stockage local chiffré

Un composant `LocalStateStore` (port + implémentation `SqlCipherLocalStore` en `infra/`), basé sur SQLite chiffré via **SQLCipher** (crate `rusqlite`, feature `bundled-sqlcipher`) — un seul fichier local, portable, protégé par clé/mot de passe.

**Répartition stricte des responsabilités entre deux coffres différents :**
- **Trousseau OS (`keyring`)** : tokens et secrets d'authentification uniquement. Ne jamais dupliquer de secret dans la base locale, même chiffrée — le trousseau OS est un stockage géré par le système, plus sûr qu'un fichier applicatif.
- **`SqlCipherLocalStore`** : tout le reste de l'état applicatif local et sensible — cache des organisations/environnements/proxies récupérés, historique d'activité local, préférences utilisateur, liste des comptes déjà connectés sur le poste.

**Ce qui ne va jamais dans cette base** : les templates. Ils restent des fichiers ouverts (JSON/YAML, voir `schemas/template.schema.json`), une décision actée dans PROJECT.md précisément pour qu'ils soient versionnables dans Git et lisibles sans l'outil.

---

## 7. Gestion des erreurs

- Dans `core/` : erreurs typées explicites via la crate `thiserror` (un enum d'erreur par module, ex. `GatewayError`, `TemplateError`). Jamais de `.unwrap()` ou `.expect()` dans le code de `core/` — toujours propager via `Result` et l'opérateur `?`.
- Dans `cli/` et `gui/src-tauri/` (les binaires, pas la logique métier) : la crate `anyhow` est acceptable pour la gestion d'erreur au niveau applicatif (moins de boilerplate en bout de chaîne).
- Chaque commande CLI doit se terminer sur un code de sortie explicite (0 = succès, non-zéro = échec) — critique pour l'usage en pipeline CI/CD.

---

## 8. Tests

- `use_cases/` doivent être testables sans aucune dépendance réseau/disque réelle : créer des implémentations "fake" des traits de `ports/` pour les tests (soit à la main, soit via la crate `mockall` qui génère des mocks à partir d'un trait annoté).
- Chaque use case a son test associé démontrant le comportement attendu avec un fake gateway/repository.

---

## 9. Structure `cli/`

```
cli/
└── src/
    ├── main.rs           # composition root du CLI
    ├── commands/         # une fonction par sous-commande clap
    │   ├── login.rs
    │   ├── template.rs
    │   ├── generate.rs
    │   ├── deploy.rs
    │   └── status.rs
    └── output.rs         # formatage de sortie (texte lisible vs --json)
```

Chaque fichier de `commands/` appelle un use case de `core/`, ne contient aucune logique métier propre — uniquement le parsing des arguments (`clap`) et le formatage de la sortie.

---

## 10. Structure `gui/`

```
gui/
├── src-tauri/
│   └── src/
│       ├── lib.rs        # composition root du GUI
│       └── commands/     # une fonction #[tauri::command] par action exposée au frontend
│           ├── auth.rs
│           ├── templates.rs
│           ├── proxies.rs
│           └── deployment.rs
└── src/                  # frontend Vue
    ├── main.ts
    ├── App.vue
    ├── components/        # composants réutilisables et "bêtes" (props in, events out)
    │   ├── base/           # briques de base : BaseDropdown.vue, BaseCard.vue, BaseButton.vue
    │   └── domain/         # composants métier : PolicyForm.vue, FlowDiagram.vue, ProxyList.vue
    ├── composables/       # logique réutilisable (équivalent Vue des hooks React)
    │   ├── useAuth.ts
    │   ├── useProxies.ts
    │   └── useTemplateEditor.ts
    ├── views/             # une vue par écran (LoginView, DashboardView, TemplateEditorView)
    └── stores/            # état global partagé, via Pinia si nécessaire
```

Chaque commande Tauri exposée dans `src-tauri/commands/` est typée des deux côtés : types Rust (via `serde::Serialize`/`Deserialize`) et interfaces TypeScript correspondantes, tenues manuellement synchronisées (ou générées via `tauri-specta` si on veut automatiser cette synchronisation plus tard).

**Règle Vue** : composants "base" = purement présentationnels, aucune logique métier, configurables uniquement via props. Composants "domain" = composent les composants "base", contiennent la logique métier via composables. Un composable ne doit jamais manipuler le DOM directement — uniquement de l'état et des appels à `invoke()`.

---

## 11. Format de communication Rust ↔ Vue

Toutes les données passant par une commande Tauri sont sérialisées en JSON via `serde`. Le schéma du template (défini dans `schemas/`) est la source de vérité unique — le type Rust dans `domain/template.rs` ET le type TypeScript côté Vue doivent tous les deux s'y conformer strictement.

---

## 12. Stratégie de test — développer sans accès entreprise à Apigee

Aucune étape du développement ne doit dépendre d'un accès Apigee entreprise. Quatre niveaux, du plus fréquent au plus rare :

### Niveau 1 — développement quotidien : fake en mémoire
En plus de `ReqwestApigeeGateway` (infra réelle), écrire une implémentation `InMemoryApigeeGateway` du trait `ApigeeGateway` — simule les réponses (liste de proxies, déploiement, statut) sans réseau. Branchée par défaut dans le composition root pendant tout le développement GUI et logique métier. Rapide, sans dépendance externe, sans quota.

### Niveau 2 — génération de bundle
Le rendu template + OpenAPI → bundle proxy XML est de la logique pure, testable localement sans compte Apigee. Valider le XML généré contre les schémas XSD officiels des policies Apigee (publiés sur GitHub) pour garantir la validité structurelle sans jamais déployer.

### Niveau 3 — tests d'intégration HTTP simulés
Pour tester le code réseau réel (`ReqwestApigeeGateway`, gestion d'erreurs, parsing) : serveur mock HTTP local via la crate `wiremock`, simulant les réponses de l'API Management Apigee d'après leur format documenté (voir section 3 pour le lien vers la référence officielle). Utilisé en CI — aucune dépendance à un compte Apigee réel dans les pipelines de test.

### Niveau 4 — validation finale réelle
Organisation d'évaluation Apigee gratuite (60 jours, provisionnable sur un projet Google Cloud personnel, aucun accès entreprise requis — voir GCP_SETUP.md pour la marche à suivre complète). Sert uniquement à la validation d'intégration finale, pas au développement courant. Renouvelable en provisionnant un nouveau projet Google Cloud à l'expiration — chaque cycle repart avec une organisation vide (proxies/déploiements de test à recréer).

| Niveau | Outil | Besoin d'Apigee réel ? |
|---|---|---|
| Dev quotidien | `InMemoryApigeeGateway` | Non |
| Génération de bundle | Tests locaux + validation XSD | Non |
| Intégration HTTP / CI | `wiremock` | Non |
| Validation finale | Eval org gratuite 60 jours | Oui, gratuite et personnelle |

---

## 13. Packaging et distribution

La structure en workspace Cargo (section 2, `core`/`cli`/`gui`) est conçue dès le départ pour permettre deux exports distincts : le CLI seul (package/binaire indépendant) et le GUI+CLI ensemble (installeur desktop Windows/Mac). `cli/` ne dépend que de `core/` — jamais de `gui/` — ce qui garantit qu'il peut être compilé, packagé et publié séparément sans aucune modification de code.

Voir **PACKAGING.md** pour le détail complet de la stratégie de build et de release des deux exports.

---

## 14. Checklist de règles à rappeler à l'IA à chaque session

- Pas d'héritage simulé, pas de `Rc<RefCell<>>` par réflexe — composition et traits uniquement
- Aucune logique métier dans `commands/` (CLI) ou `src-tauri/commands/` (GUI) — uniquement délégation aux use cases
- Aucun `.unwrap()`/`.expect()` dans `core/`
- Toute nouvelle capacité : d'abord un use case dans `core/`, puis exposée en CLI ET en commande Tauri — jamais l'un sans l'autre (voir PROJECT.md, principe n°1)
- Composants Vue "base" sans logique métier ; logique métier dans les composables
- Expliquer chaque pattern Rust non trivial généré (ownership, lifetimes, `Arc<dyn Trait>`, `async-trait`) avant de considérer le code accepté
- Le CLI lit uniquement `GOOGLE_APPLICATION_CREDENTIALS` — jamais de logique applicative pour choisir entre WIF et service account, c'est la pipeline qui en décide
- Jamais de secret/token dans `SqlCipherLocalStore` — uniquement dans le trousseau OS via `keyring`
- Pour tout appel à l'API Apigee, consulter `https://cloud.google.com/apigee/docs/reference/apis/apigee/rest` plutôt que d'improviser le format d'une requête
