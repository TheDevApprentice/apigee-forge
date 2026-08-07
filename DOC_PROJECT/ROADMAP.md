# Apigee Forge — Roadmap

---

## Partie 1 — Roadmap de construction du MVP

Séquence recommandée : chaque jalon doit être fonctionnel et testable avant de passer au suivant. Le CLI complet (jalons 1 à 5) doit être utilisable en autonomie totale avant même de commencer le GUI — c'est la meilleure façon de vérifier que le principe "le CLI doit tout pouvoir faire" est respecté par construction.

- [x] **M1 — Fondations `core`** : setup du workspace Cargo, parsing OpenAPI, définition du schéma de template (JSON Schema dans `schemas/`). Pas encore d'appel réseau réel.
- [x] **M2 — Client Apigee** : authentification (OAuth2 desktop + service account/WIF via `gcp_auth`), appels API de base (liste orgs, environnements, proxies, lecture du rôle IAM).
- [ ] **M3 — Moteur de rendu** : spec OpenAPI + template → bundle proxy (fichiers XML + packaging zip), testé uniquement via tests unitaires/CLI minimal, sans déploiement réel.
- [ ] **M4 — CLI complet** : toutes les commandes (`login`, `template`, `generate`, `deploy`, `status`, `list-proxies`), mode non-interactif complet, sortie `--json`. À ce stade, l'outil doit être 100% utilisable en ligne de commande, sans GUI.
- [ ] **M5 — Intégration CI/CD de référence** : exemple de pipeline (GitHub Actions et/ou Azure Pipelines) utilisant le CLI en mode non-interactif, pour valider le cas d'usage self-service en pipeline.
- [ ] **M6 — Squelette GUI Tauri** : écran de connexion OAuth, vue liste orgs/environnements/proxies (lecture seule pour commencer).
- [ ] **M7 — Éditeur visuel de template** : formulaires guidés pour les policies MVP, représentation visuelle du flux PreFlow/Flow/PostFlow, sauvegarde compatible avec le format lu par le CLI.
- [ ] **M8 — Déploiement et suivi depuis le GUI** : génération + déploiement + suivi de statut en temps réel.
- [ ] **M9 — Polish design** : mise en cohérence visuelle (thème clair, accent teal, diagramme de flux soigné), tokens exacts dans DESIGN.md.
- [ ] **M10 — Packaging et documentation** : mise en place du build/release CLI seul + installeur GUI+CLI (voir PACKAGING.md), README, captures d'écran, préparation du projet pour présentation en portfolio (et publication open source si souhaité à ce stade).

---

## Partie 2 — Au-delà du MVP (v1.1, v2...)

*Idées capturées pour ne pas les perdre, mais volontairement exclues du MVP pour éviter la dérive de scope.*

### Policies et fonctionnalités Apigee avancées
- Callouts JavaScript/Java
- Politiques de monétisation
- Politiques de cache
- Politiques de transformation avancées (au-delà du JSON↔XML basique)

### Gestion et collaboration
- Diff/versioning visuel entre révisions de proxy
- Bibliothèque de templates partageables (import/export, éventuelle logique de "marketplace" communautaire)
- Mode multi-organisation / multi-tenant

### Extension du périmètre
- Abstraction dans `core` pour supporter d'autres gateways (Kong, AWS API Gateway, Azure APIM) — nécessiterait une refonte du client API en couche d'abstraction
- Génération automatique de template à partir d'un proxy Apigee existant ("import inversé" d'une convention déjà en place)
- **Connecteur de source additionnel — Layer7** : au-delà des deux sources MVP (nouveau template créé dans l'éditeur, spec OpenAPI), ajouter une troisième source : connexion directe à un serveur Layer7 (CA API Gateway) via identifiants fournis par l'utilisateur, pour identifier et remonter le contenu à migrer — reflète un cas réel vécu (migration Layer7 → Apigee chez Canam). Implique un nouveau modèle d'auth, un nouveau format à interpréter, et une gestion de credentials supplémentaire : à traiter comme un module d'extension à part entière, pas une simple option en plus.

### Observabilité
- Tableau de bord analytics (usage, erreurs, latence agrégée via l'API Apigee Analytics)

### Écosystème
- Publication open source avec guide de contribution
- Éventuelle proposition d'interopérabilité ou de dialogue avec la communauté apigee-go-gen / Apigee elle-même
