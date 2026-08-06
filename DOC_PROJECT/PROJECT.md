# Apigee Forge — Contexte du projet

*Document de référence à transmettre à toute IA travaillant sur ce projet (Claude Code ou autre). Contient l'origine, les objectifs, et les décisions actées à ne pas remettre en question sans discussion explicite.*

---

## 1. Origine du projet

L'idée vient d'une expérience professionnelle réelle : chez Canam, développement d'un outil interne de migration d'API vers Apigee. Fonctionnement d'origine :
- Application console en C#
- Lecture d'un fichier OpenAPI (routes, sécurité, etc.)
- Génération de fichiers à partir de templates internes, intégrant les standards de gouvernance et de sécurité propres à l'entreprise
- Packaging en zip, envoi vers la plateforme Apigee via API
- Intégration dans une pipeline Azure DevOps, permettant aux équipes de migrer leurs API elles-mêmes, en self-service, en batch

## 2. Constat concurrentiel (analyse faite avant de se lancer)

- Apigee propose **nativement** la génération de proxy à partir d'une spec OpenAPI (wizard "Use OpenAPI Spec")
- Google maintient un outil officiel open source, **apigee-go-gen**, qui fait déjà de la génération de bundle par templates
- Un outil communautaire plus ancien, **openapi2apigee**, fait la même chose

→ **Un simple convertisseur OpenAPI → Apigee n'a aucune valeur différenciante : c'est gratuit et natif.**

La vraie valeur de ce qui a été construit chez Canam n'était pas la conversion elle-même, mais :
1. L'encodage de règles de gouvernance/sécurité **propres à l'organisation** dans les templates (chose qu'aucun outil générique ne peut faire, par nature)
2. Le **self-service en pipeline** (autonomie des équipes, pas de goulot d'étranglement sur une équipe centrale)
3. Aucun outil existant (natif Apigee, apigee-go-gen, openapi2apigee) ne propose d'**éditeur visuel** de template, ni de **gestion des rôles/permissions** adaptée à l'utilisateur connecté, ni de **tableau de bord de suivi des déploiements**

C'est sur ces trois points que ce projet se différencie.

## 3. Objectifs du projet

**Ce n'est pas un projet commercial.** Objectifs actés :
- Vitrine de compétences en développement assisté par IA (l'IA code sous supervision humaine ; l'humain connaît l'architecture, les bonnes pratiques, et apprend au passage — pas de copier-coller aveugle)
- Remise à niveau technique après une pause de plusieurs mois
- Montée en compétence encadrée sur Rust (notions de base au départ)
- Objectif secondaire, non garanti : si l'outil est suffisamment abouti, il pourrait intéresser d'autres développeurs/entreprises utilisant Apigee (potentiel open source)

## 4. Principes directeurs actés (à ne pas remettre en cause sans discussion)

1. **Le CLI doit pouvoir tout faire, seul.** Toute fonctionnalité disponible dans le GUI doit d'abord exister dans le crate `core`, puis être exposée à l'identique en commande CLI. Aucune fonctionnalité ne doit exister uniquement côté GUI. Le CLI doit fonctionner de façon 100% autonome, sur un poste de dev ou dans une pipeline CI/CD, sans aucune interaction graphique.
2. **Scope IAM volontairement limité aux rôles Apigee** (`apigee.admin`, `apigee.readOnlyAdmin`, `apigee.developerAdmin`, `apigee.analyticsViewer`, etc.). Pas de gestion IAM générique Google Cloud (rôles custom, bindings au niveau projet/organisation) — ce périmètre est délibérément laissé à la console Google Cloud native.
3. **Format de template ouvert et documenté** (JSON ou YAML avec schéma explicite), versionnable dans Git, lisible sans l'outil.
4. **Développement assisté par IA avec compréhension systématique** : chaque pattern généré (ownership/lifetimes Rust, IPC Tauri, gestion async) doit être expliqué et compris avant d'être accepté.
5. **Multi-utilisateur nativement supporté, sans gestion d'utilisateurs applicative.** Chaque personne (développeur ou superviseur) s'authentifie avec sa propre identité Google via OAuth desktop — jamais de compte partagé. Google IAM reste l'unique source de vérité des permissions ; l'interface s'adapte au rôle Apigee réel de la personne connectée, sans système de rôles/permissions construit par nous. Voir ARCHITECTURE.md section 4 et GCP_SETUP.md pour la mise en place de comptes de test simulant les deux profils.

## 5. Décisions d'architecture actées

- **Langage unique : Rust**, pour le CLI et le GUI (via Tauri), avec un crate `core` partagé pour éviter toute duplication de logique métier.
- **Workspace Cargo** :
  ```
  apigee-forge/
  ├── core/     # logique métier partagée (auth, API Apigee, parsing OpenAPI, templates, rendu)
  ├── cli/      # binaire CLI, dépend de core
  ├── gui/      # app Tauri (backend Rust src-tauri + frontend Vue/TypeScript), dépend de core
  └── schemas/  # JSON Schema du format de template, documentation indépendante
  ```
- **Authentification** :
  - GUI (interactif) : OAuth2 desktop avec redirection locale, token stocké via `keyring` (trousseau OS)
  - CLI (pipeline, headless) : Service Account JSON ou Workload Identity Federation via `gcp_auth`
- **Crates identifiés** : `oas3`/`openapiv3` (parsing OpenAPI), `tera` (moteur de templates), `reqwest`+`tokio` (HTTP async), `oauth2`+`keyring` (auth desktop), `gcp_auth` (auth headless), `zip` (packaging du bundle), `clap` (CLI), `dialoguer` (prompts interactifs CLI)
- **Frontend Tauri** : Vue 3 (Composition API) + TypeScript — choix assumé pour sa philosophie de composition (composables réutilisables), bien représenté dans l'écosystème Tauri (templates officiels disponibles)

## 6. Direction design

- Inspiration Google Cloud Console : sobriété, typographie sans-serif propre, navigation latérale, espace négatif généreux, palette limitée à un seul accent
- Différenciation : accent teal, **thème clair uniquement** (pas de mode sombre pour ce projet), cartes à coins arrondis avec bordure fine plutôt qu'ombre — voir DESIGN.md pour les tokens exacts validés
- **Signature visuelle** : représentation du flux de proxy (PreFlow → Flows conditionnels → PostFlow) comme diagramme interactif — fonctionnel (reflète le modèle réel Apigee) et immédiatement identifiable

## 7. Pistes explorées et écartées avant ce pivot

Avant d'arriver sur ce projet technique, plusieurs pistes business B2B ont été analysées et écartées faute d'espace libre viable (marché déjà occupé par des acteurs matures/financés) :
- Conformité accessibilité numérique (RGAA/EAA) — occupé par RGAA Checker, Access42, Temesis, etc.
- Facturation électronique — infrastructure réglementée lourde (Plateformes Agréées), dominé par Pennylane/Cegid/Qonto
- Cybersécurité NIS2 / gestion des risques fournisseurs (TPRM) — occupé par nis2facile.fr, Make IT Safe, Vanta/Drata, etc.

Conclusion tirée de ces analyses : les idées "il y a une loi qui arrive, je construis l'outil de conformité" sont visibles par tout le monde en même temps, donc rapidement occupées par des équipes financées. Ce projet technique, basé sur une expérience réelle et vécue, échappe à ce piège.
