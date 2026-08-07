# Apigee Forge — Carte des endpoints Apigee API

*Référence de démarrage, pas une source figée. Chaque chemin/paramètre doit être reconfirmé contre `https://docs.cloud.google.com/apigee/docs/reference/apis/apigee/rest` avant implémentation ou modification — règle déjà actée dans ARCHITECTURE.md et rappelée dans M2_STARTUP_ROADMAP.md. Objectif de ce document : donner un point de départ fiable pour ne pas repartir de zéro sur chaque mapping, et éviter d'inventer un endpoint par erreur.*

**Version d'API ciblée : v1 uniquement** (`https://apigee.googleapis.com/$discovery/rest?version=v1`) — il n'existe pas de v2 de l'API de gestion Apigee.

---

## Pourquoi pas de SDK généré

`google-apigee1` (crates.io) existe mais est en maintenance minimale (pas de nouvelle fonctionnalité annoncée, mises à jour occasionnelles). Le SDK officiel `google-cloud-rust` est activement maintenu mais sa couverture de l'API Apigee de gestion des proxies n'est pas confirmée. Le mapping manuel via `reqwest` (déjà construit en M2) reste le choix retenu — ce document sert de documentation de ce mapping, pas de remplacement.

---

## Mapping port → endpoint

| Méthode du port (`ApigeeGateway`) | Endpoint | Notes |
|---|---|---|
| `list_organizations` | `GET /v1/organizations` | Liste les organisations accessibles à l'identité authentifiée. |
| `list_environments(org)` | `GET /v1/organizations/{org}/environments` | |
| `list_proxies(org)` | `GET /v1/organizations/{org}/apis` | Champs à limiter au strict nécessaire MVP (nom, dernière révision) — voir M2-06. |
| `import_bundle(org, proxy_name, zip)` | `POST /v1/organizations/{org}/apis?action=import&name={proxy_name}` | Corps de requête : le bundle `.zip` en `multipart/form-data`. Crée une nouvelle révision du proxy (ou le proxy lui-même s'il n'existe pas). Pas encore implémenté — prévu M4-06. |
| `deploy(org, env, proxy_name, revision)` | `POST /v1/organizations/{org}/environments/{env}/apis/{proxy_name}/revisions/{revision}/deployments` | Distinct de l'import : l'import crée la révision, cet appel la déploie sur un environnement. Ne pas confondre les deux (déjà signalé dans M4-06). |
| `get_deployment_status(org, env, proxy_name, revision)` | `GET /v1/organizations/{org}/environments/{env}/apis/{proxy_name}/revisions/{revision}/deployments` | |
| `get_role(org)` | Pas un endpoint Apigee dédié — lecture via l'API **Cloud Resource Manager** (`projects.getIamPolicy` sur le projet correspondant à l'org, puisqu'org = projet en Apigee X). | Confirmé en M2-07. Ne jamais ajouter de création/modification de binding IAM ici (PROJECT.md principe 2). |

---

## Prochaines vérifications à faire par l'IA avant M4-06

- Confirmer le nom exact du paramètre de requête pour l'import (`action=import` + `name=`) contre la doc officielle — ce document donne la forme attendue, pas une garantie.
- Confirmer le format exact de réponse de l'import (contient la révision créée) pour typer correctement le retour du use case.
- Vérifier si une étape de validation (`?validate=true`) est disponible avant import réel, utile pour un futur mode "dry-run" (hors MVP, à noter seulement).