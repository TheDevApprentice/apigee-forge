# Apigee Forge — Roadmap de démarrage détaillée

*Complète ROADMAP.md, qui reste au niveau des jalons M1→M10. Ce document zoome sur le tout début (M1) en étapes atomiques, et fixe la méthode à réappliquer pour chaque jalon suivant. Objectif : ne jamais enchaîner "créer le projet, le peupler, et coder" en un seul bloc — chaque étape doit être validée avant la suivante.*

---

## Règle de méthode, valable pour tout le projet

Avant de commencer un jalon (M1, M2, ... M10 dans ROADMAP.md), le découper en étapes atomiques comme ci-dessous — une étape = une action vérifiable, compilable ou testable, jamais un ensemble de fonctionnalités mélangées. Après chaque étape : s'arrêter, présenter ce qui a été fait, attendre validation avant de continuer. Ne jamais enchaîner plusieurs étapes sans point d'arrêt, même si la suite semble évidente.

---

## Étapes atomiques — Jalon M1 (Fondations `core`)

- [x] **Étape 0 — Squelette vide** : créer l'arborescence de dossiers vide (voir STRUCTURE.md), initialiser Git, `Cargo.toml` racine du workspace avec les membres déclarés (`core`, `cli`, `gui/src-tauri`). Vérifier que `cargo build` compile un workspace vide sans erreur. Aucune ligne de logique métier à ce stade.
- [x] **Étape 1 — Dépendances de base `core`** : ajouter `serde`, `thiserror` au `Cargo.toml` de `core`. Rien d'autre.
- [x] **Étape 2 — Entités `domain`** : créer les structs de `domain/` (`Proxy`, `Template`, `Deployment`, `ApigeeRole`) — uniquement les types et leurs dérivations Serde, aucun comportement, aucune méthode complexe.
- [x] **Étape 3 — Validation du schéma de template** : écrire un test qui désérialise `schemas/template.example.json` vers le struct `Template` et vérifie que ça fonctionne sans erreur. Confirme que le modèle Rust colle au schéma JSON déjà défini.
- [x] **Étape 4 — Parsing OpenAPI** : implémenter la lecture d'un fichier OpenAPI minimal (crate `oas3` ou `openapiv3`) et l'extraction des routes + schémas de sécurité déclarés. Tester avec un exemple OpenAPI simple (2-3 routes).
- [x] **Étape 5 — Traits `ports`** : définir les signatures des quatre traits (`ApigeeGateway`, `TemplateRepository`, `AuthProvider`, `LocalStateStore`) — uniquement les signatures de méthodes, sans aucune implémentation.
- [x] **Étape 6 — Fakes `infra`** : implémenter `InMemoryApigeeGateway` et `FilesystemTemplateRepository` — les deux implémentations qui permettent de tester sans réseau ni Apigee réel (niveau 1 de la stratégie de test, ARCHITECTURE.md section 12).
- [x] **Étape 7 — Premier use case** : implémenter `create_template` dans `use_cases/`, avec un test qui l'exerce via les fakes de l'étape 6.
- [x] **Étape 8 — CI minimale** : ajouter `.github/workflows/ci.yml` qui lance `cargo test` et `cargo clippy` à chaque push. Vérifier que ça passe au vert avant de continuer.
- [x] **Étape 9 — Point de contrôle M1** : `cargo test` passe sur le parsing OpenAPI, la validation du schéma de template, et le use case `create_template`. Si tout est vert, M1 est terminé — passer à M2 (ARCHITECTURE.md/ROADMAP.md) en le découpant à son tour selon la même méthode.

---

## Definition of done — critère simple par jalon

Pour chaque jalon (M1 à M10), une phrase de validation simple et vérifiable, sans besoin d'auditer le code en détail :

| Jalon | Condition de complétion |
|---|---|
| M1 | `cargo test` passe sur parsing OpenAPI + validation schéma + premier use case |
| M2 | Le CLI peut lister les proxies d'un compte réel (eval org) via `login` + `list-proxies` |
| M3 | Un bundle `.zip` valide est généré localement à partir d'un exemple OpenAPI + template, sans déploiement |
| M4 | Toutes les commandes CLI listées dans MVP_FEATURES.md fonctionnent en mode non-interactif |
| M5 | Un pipeline CI d'exemple exécute le CLI de bout en bout sur un push de test |
| M6 | Le GUI démarre, l'écran de connexion OAuth fonctionne, la liste des proxies s'affiche |
| M7 | Un template peut être créé et sauvegardé entièrement depuis l'éditeur visuel |
| M8 | Un déploiement lancé depuis le GUI aboutit et le statut s'affiche en temps réel |
| M9 | Le rendu visuel correspond aux tokens de DESIGN.md |
| M10 | Les artefacts de release (CLI + GUI) se génèrent via `PACKAGING.md`, README à jour |
