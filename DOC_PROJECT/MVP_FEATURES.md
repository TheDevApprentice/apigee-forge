# Apigee Forge — Fonctionnalités MVP

*Périmètre v1. Voir ROADMAP.md pour la séquence de construction et les fonctionnalités post-MVP.*

---

## Core (package Cargo `apigee-forge-core`, dossier `core/`)

### Parsing OpenAPI
- [ ] Parsing OpenAPI 3.x (routes, verbes HTTP, paramètres, schémas de sécurité déclarés)
- [ ] Validation de la spec avant traitement (erreurs claires si spec invalide/incomplète)

### Modèle de template
- [ ] Schéma de template documenté (JSON ou YAML), versionné dans `schemas/`
- [ ] Policies couvertes en MVP :
  - Sécurité : OAuth2, API Key, JWT
  - Quota
  - Spike Arrest
  - CORS
  - Transformation basique JSON ↔ XML
- [ ] Règles de mapping : quel type de sécurité détecté dans l'OpenAPI → quelle policy appliquer
- [ ] Conventions de nommage imposées (préfixes, structure des noms de proxy)
- [ ] Métadonnées de gouvernance (propriétaire, environnement cible)
- [ ] Schéma conçu de façon extensible (chaque policy = module additif, pour ajout futur sans refonte)

### Moteur de rendu
- [ ] Génération du bundle proxy Apigee (structure XML conforme) à partir de : spec OpenAPI + template sélectionné
- [ ] Packaging du bundle en `.zip`

### Client API Apigee
- [ ] Authentification (OAuth2 desktop ET service account/WIF)
- [ ] Liste des organisations/environnements accessibles
- [ ] Liste des proxies existants
- [ ] Déploiement d'une révision de proxy
- [ ] Récupération du statut de déploiement
- [ ] Lecture du rôle Apigee de l'utilisateur/compte connecté

### Gestion des templates
- [ ] Create / Read / Update / Delete d'un template
- [ ] Stockage local (fichiers sur disque) — pas de backend distant en v1

---

## CLI (`cli`)

- [ ] `login` — authentification interactive (OAuth desktop) ou non-interactive (service account/WIF)
- [ ] `template create` — création guidée via prompts (`dialoguer`) ou import d'un fichier existant
- [ ] `template list` / `template show`
- [ ] `generate` — spec OpenAPI + template → bundle proxy (sortie locale, sans déploiement)
- [ ] `deploy` — envoi du bundle vers Apigee
- [ ] `status` — suivi de l'état d'un déploiement
- [ ] `list-proxies` — liste des proxies déployés
- [ ] **Mode 100% non-interactif** disponible pour chaque commande (flags/variables d'environnement), aucun prompt bloquant possible en pipeline
- [ ] Sortie `--json` pour intégration scriptée (lecture machine)
- [ ] Codes de sortie explicites (succès/échec/erreur de config) pour usage en pipeline CI/CD

---

## GUI (Tauri)

- [ ] Écran de connexion (OAuth2 desktop)
- [ ] Vue d'ensemble : organisations / environnements / proxies accessibles, adaptée selon le rôle Apigee détecté
- [ ] **Éditeur visuel de template** :
  - Représentation du flux PreFlow → Flows conditionnels → PostFlow
  - Ajout de policies MVP via formulaires guidés (listes déroulantes, pas de XML à écrire à la main)
  - Sauvegarde au format de template standard (compatible CLI)
- [ ] Génération de proxy depuis l'UI : sélection d'une spec OpenAPI (fichier local ou URL) + template → prévisualisation du bundle
- [ ] Déploiement depuis l'UI + suivi de statut en temps réel
- [ ] Thème sombre par défaut, direction visuelle définie dans PROJECT.md

---

## Explicitement hors scope MVP

*(à ne pas implémenter maintenant, même si proposé par l'IA en cours de route — voir ROADMAP.md)*

- Policies avancées (callouts JavaScript/Java, monétisation, cache)
- Support d'autres API gateways que Apigee (Kong, AWS API Gateway, Azure APIM)
- Gestion IAM générique Google Cloud (rôles custom, bindings)
- Multi-tenant / partage de templates entre organisations
- Tableau de bord analytics d'usage API
- Diff/versioning visuel entre révisions de proxy
