# Apigee Forge — Structure du projet

*Référence canonique de l'arborescence complète. En cas de doute sur où placer un fichier, ce document fait autorité — ne pas improviser une structure différente. ARCHITECTURE.md explique le *pourquoi* de cette organisation ; ce document montre le *où* exact.*

---

## Arborescence racine

```
apigee-forge/
├── DOC_PROJECT/              # tous les documents de cadrage (ce dossier)
├── core/                     # package Cargo apigee-forge-core — logique métier partagée
├── cli/                      # binaire CLI, dépend de core uniquement
├── gui/                      # app Tauri (GUI), dépend de core uniquement
├── schemas/                  # JSON Schema du format de template + exemples
├── .github/
│   └── workflows/
│       ├── ci.yml            # tests + lint sur chaque push/PR
│       └── release.yml       # build + publication CLI et GUI sur tag de version
├── Cargo.toml                # workspace root, liste les membres (core, cli, gui/src-tauri)
├── .gitignore
├── LICENSE
└── README.md                 # présentation du projet, instructions d'installation/usage
```

---

## `core/` — détail complet

```
core/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── domain/
    │   ├── mod.rs
    │   ├── auth.rs            # AuthContext, AuthMode, GoogleIdentity, ProjectId, OrganizationId
    │   ├── organization.rs    # Organization, Environment
    │   ├── render.rs          # modèles internes de rendu M3
    │   ├── proxy.rs           # struct Proxy, struct ProxyRevision
    │   ├── template.rs        # struct Template, enum PolicyType (conforme à schemas/template.schema.json)
    │   ├── deployment.rs      # struct Deployment, enum DeploymentStatus
    │   └── role.rs            # enum ApigeeRole
    ├── openapi.rs             # parsing OpenAPI et extraction routes/sécurité
    ├── use_cases/
    │   ├── mod.rs
    │   ├── apigee_read.rs             # use cases de lecture Apigee M4
    │   ├── apigee_write.rs            # import, déploiement et statut Apigee M4
    │   ├── generate_proxy_bundle.rs
    │   ├── deploy_proxy.rs
    │   ├── create_template.rs
    │   └── list_proxies.rs
    ├── ports/
    │   ├── mod.rs
    │   ├── apigee_gateway.rs       # trait ApigeeGateway lecture
    │   ├── apigee_deployment_gateway.rs # trait déploiement/statut Apigee
    │   ├── apigee_proxy_bundle_gateway.rs # trait import bundle Apigee
    │   ├── bundle_renderer.rs      # port de rendu M3
    │   ├── bundle_writer.rs        # port d’écriture contrôlée du bundle M3
    │   ├── bundle_archiver.rs      # port de packaging M3
    │   ├── template_repository.rs  # trait TemplateRepository
    │   ├── auth_provider.rs        # trait AuthProvider
    │   └── local_state_store.rs    # trait LocalStateStore
    ├── infra/
    │   ├── mod.rs
    │   ├── headless_auth_config.rs
    │   ├── service_account_auth_provider.rs
    │   ├── oauth_desktop_auth_provider.rs
    │   ├── reqwest_apigee_gateway.rs       # transport HTTP partagé Apigee
    │   ├── in_memory_apigee_gateway.rs     # fake, niveau 1 de la stratégie de test
    │   ├── filesystem_bundle_writer.rs     # écriture transactionnelle du bundle M3
    │   ├── filesystem_template_repository.rs
    │   ├── tera_bundle_renderer.rs       # moteur Tera/XML M3
    │   ├── zip_bundle_archiver.rs        # packaging ZIP M3
    │   ├── templates/                    # templates XML internes M3
    │   └── sqlcipher_local_store.rs
    └── error.rs                # types d'erreur partagés (thiserror)
└── tests/
    └── use_cases/              # tests d'intégration des use cases avec les fakes
```

---

## `cli/` — détail complet

```
cli/
├── Cargo.toml                 # dépend de core
└── src/
    ├── main.rs                # composition root du CLI
    ├── commands/
    │   ├── mod.rs
    │   ├── login.rs
    │   ├── template.rs
    │   ├── generate.rs
    │   ├── deploy.rs
    │   ├── status.rs
    │   └── list_proxies.rs
    └── output.rs               # formatage texte lisible vs --json
```

---

## `gui/` — détail complet

```
gui/
├── src-tauri/
│   ├── Cargo.toml              # dépend de core
│   ├── tauri.conf.json         # config bundle, dont externalBin (sidecar CLI, voir PACKAGING.md)
│   └── src/
│       ├── main.rs
│       ├── lib.rs               # composition root du GUI
│       └── commands/
│           ├── mod.rs
│           ├── auth.rs
│           ├── templates.rs
│           ├── proxies.rs
│           └── deployment.rs
├── src/                         # frontend Vue 3 + TypeScript
│   ├── main.ts
│   ├── App.vue
│   ├── components/
│   │   ├── base/                # BaseDropdown.vue, BaseCard.vue, BaseButton.vue, BaseChip.vue
│   │   └── domain/               # PolicyForm.vue, FlowDiagram.vue, ProxyList.vue, RoleBadge.vue
│   ├── composables/
│   │   ├── useAuth.ts
│   │   ├── useProxies.ts
│   │   └── useTemplateEditor.ts
│   ├── views/
│   │   ├── LoginView.vue
│   │   ├── DashboardView.vue
│   │   └── TemplateEditorView.vue
│   ├── stores/                   # Pinia, si état global partagé nécessaire
│   └── types/                    # interfaces TypeScript miroir des types Rust exposés via Tauri
├── package.json
├── vite.config.ts
└── index.html
```

---

## `schemas/`

```
schemas/
├── template.schema.json        # déjà créé
└── template.example.json       # déjà créé
```

---

## `DOC_PROJECT/`

```
DOC_PROJECT/
├── PROJECT.md
├── MVP_FEATURES.md
├── ARCHITECTURE.md
├── DESIGN.md
├── ROADMAP.md
├── STARTUP_ROADMAP.md
├── M2_STARTUP_ROADMAP.md
├── M3_STARTUP_ROADMAP.md
├── M4_STARTUP_ROADMAP.md
├── M4-04_checkpoint.md
├── CI_CLI_EXAMPLE.md
├── SECURITY.md
├── STRUCTURE.md                 # ce document
├── PACKAGING.md
├── GCP_SETUP.md
└── PROMPT.md
```

---

## Règle de placement

Avant de créer un nouveau fichier, vérifier qu'un emplacement n'est pas déjà prévu ci-dessus. Si un besoin ne correspond à aucun emplacement existant, s'arrêter et demander plutôt que de créer une structure ad hoc — c'est un signal que la structure doit être mise à jour consciemment (et donc ce document avec elle), pas contournée silencieusement.
