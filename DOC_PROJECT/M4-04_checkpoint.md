# M4-04 — Checkpoint d’authentification et validation réelle

## Décision

Le provisionnement immédiat d’un projet Google Cloud et d’une organisation Apigee **n’est pas nécessaire pour continuer l’implémentation technique de M4**.

La CLI peut être construite et vérifiée sans environnement réel grâce à :

- la documentation REST officielle Apigee v1 ;
- le document Discovery API v1 ;
- les contrats `ApigeeGateway` ;
- `InMemoryApigeeGateway` ;
- les tests HTTP WireMock ;
- les doubles `AuthProvider`, `BrowserLauncher` et `RefreshTokenStore`.

Le test réel est requis plus tard pour obtenir une preuve d’intégration, pas pour écrire les use cases et l’adaptateur CLI.

## Statut actualisé — 2026-08-08

Le projet Google Cloud et l’organisation Apigee d’évaluation ont été provisionnés par l’utilisateur. Le provisionnement est donc validé ; les vérifications de connectivité, de lecture et d’écriture restent à exécuter sur `feature/real-apigee-validation`.

## Ce qui peut être terminé sans environnement réel

Les points suivants peuvent être implémentés et testés localement :

1. composition headless/OAuth de la CLI ;
2. résolution et validation de l’organisation ;
3. use cases de lecture organisations/environnements/proxies ;
4. mapping des réponses Apigee vers le domaine ;
5. mapping des erreurs HTTP, timeouts, retries et rate limits ;
6. use cases d’import, déploiement et statut ;
7. sortie humaine/JSON et codes de sortie ;
8. mode non-interactif ;
9. tests WireMock des contrats HTTP ;
10. tests CLI avec doubles, sans token ni credential réel.

Les sources de référence sont :

- `https://apigee.googleapis.com/$discovery/rest?version=v1`
- `https://docs.cloud.google.com/apigee/docs/reference/apis/apigee/rest`
- `DOC_PROJECT/APIGEE_API_MAP.md`

Chaque endpoint d’écriture devra toutefois être reconfirmé dans la documentation officielle au moment de son implémentation, en particulier l’import multipart du bundle et le format exact de sa réponse.

## Ce que seul un test réel peut confirmer

Un environnement réel est nécessaire pour confirmer :

- que le projet GCP et l’organisation Apigee sont correctement provisionnés ;
- que le compte utilisé possède exactement les permissions attendues ;
- que `gcp_auth` résout correctement le credential fourni ;
- que l’organisation déduite du project ID est bien celle ciblée ;
- que les réponses réelles d’import et de déploiement correspondent aux contrats documentés ;
- que les statuts et délais de propagation réels sont correctement gérés ;
- que les restrictions de l’organisation d’évaluation ne modifient pas le comportement attendu.

Ces vérifications ne doivent pas être transformées en tests CI et ne doivent jamais nécessiter de credential commité.

## Plan de validation différée

### Après M4-05

Lorsque l’environnement sera provisionné, exécuter manuellement :

```text
login --headless
list-proxies --org <organization>
```

Cette vérification confirme l’authentification et la lecture avant d’utiliser les opérations d’écriture.

### Après M4-06

Exécuter manuellement un import puis un déploiement avec un bundle trivial. Confirmer :

- la révision créée ;
- l’appel de déploiement ;
- le statut retourné ;
- l’absence de fuite de token dans la sortie.

### M4-11

Faire le parcours complet Helloworld :

```text
authentification → generate → import/deploy → status → list-proxies
```

Ce parcours devient le checkpoint de preuve avant le premier usage réel, mais ne bloque pas la poursuite du développement local et automatisé de M4.

## Règles de sécurité

- Ne jamais passer un credential ou token en argument CLI.
- Utiliser uniquement `GOOGLE_APPLICATION_CREDENTIALS` pour le mode headless.
- Ne jamais écrire de token dans les rapports, logs ou fixtures.
- Ne jamais ajouter de flag fake/réel dans le produit.
- Les fakes restent confinés aux tests ; le binaire réel compose toujours les providers et gateways réels.
- Documenter uniquement le résultat du test réel, sans credential, token ou détail sensible.

## M4-04 terminé côté code

### Sélection explicite du mode d’authentification

Le CLI exige maintenant un mode explicite pour `login` :

```text
login --headless
login --interactive --org <organization>
```

Cas refusés :

```text
login
login --headless --interactive
```

Le mode headless utilise uniquement :

```text
GOOGLE_APPLICATION_CREDENTIALS
```

Le mode OAuth desktop utilise :

```text
APIGEE_FORGE_OAUTH_CLIENT_ID
```

`APIGEE_FORGE_OAUTH_USERNAME` reste optionnel et sert uniquement d’alias local du trousseau OS ; sa valeur par défaut est `desktop`. Aucun credential ou token n’est accepté en argument CLI.

### Composition root

La composition auth est isolée dans :

```text
cli/src/auth.rs
```

Le CLI branche uniquement les providers réels :

- `ServiceAccountAuthProvider` pour headless ;
- `OAuthDesktopAuthProvider` pour OAuth desktop.

Les doubles restent confinés aux tests.

### Résolution de l’organisation

Les règles implémentées sont :

- en headless, l’organisation est déduite du project ID ;
- une organisation passée explicitement doit correspondre au project ID ;
- en desktop, `--org` est obligatoire tant qu’aucune organisation n’est sélectionnée ;
- toute ambiguïté est refusée ;
- les identifiants d’organisation dangereux sont rejetés.

### Résultat `login`

Le résultat ne contient aucune donnée sensible :

```json
{
  "ok": true,
  "command": "login",
  "data": {
    "mode": "headless",
    "identity": null,
    "project_id": "project-id",
    "selected_organization": "project-id"
  },
  "error": null
}
```

Un appel sans mode explicite produit une erreur JSON sûre :

```json
{
  "ok": false,
  "command": "login",
  "data": null,
  "error": {
    "code": "INVALID_AUTH_MODE",
    "message": "select exactly one explicit authentication mode"
  }
}
```

### Tests M4-04

Les tests couvrent :

- un double `AuthProvider` ;
- la résolution headless ;
- la détection des conflits d’organisation ;
- le refus d’une organisation desktop absente ;
- les modes d’authentification explicites ;
- le résumé non sensible.

Les doubles `BrowserLauncher` et `RefreshTokenStore` existants de M2 couvrent déjà le provider OAuth desktop.

### Validation automatisée

```text
cargo test --workspace --locked
55 tests core
1 test golden
11 tests CLI
0 échec

cargo clippy --workspace --all-targets --locked -- -D warnings
OK

cargo audit
OK
```

## État au checkpoint M4-04

- Composition auth CLI : implémentée.
- Mode headless explicite : implémenté.
- Mode OAuth desktop explicite : implémenté.
- Résolution d’organisation : implémentée et testée avec doubles.
- Résultat `login` humain/JSON non sensible : implémenté.
- Test réel Apigee : volontairement différé.
- Provisionnement GCP/Apigee : à effectuer avant la validation M4-11 et avant tout premier déploiement réel.
