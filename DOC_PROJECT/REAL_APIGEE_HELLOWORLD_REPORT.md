# Rapport de validation réelle — Hello World

Date : 2026-08-08
Branche : `feature/real-apigee-validation`
Organisation : `apigee-forge`
Environnement : `eval`
Proxy : `forge-helloworld-unique`

Aucun credential, token ou détail privé n'est inclus dans ce rapport.

## Résultat

Validation réelle réussie jusqu'au statut runtime Apigee :

```text
login headless       : OK
list-proxies lecture : OK
generate local       : OK
deploy réel          : OK
status initial       : InProgress
status final         : Succeeded
```

Révision finalement déployée : `8`.

## Déroulé détaillé

### 1. Démarrage du CLI et authentification

La commande compose `ServiceAccountAuthProvider` depuis `GOOGLE_APPLICATION_CREDENTIALS`. Le fichier JSON reste local et n'est jamais lu dans un log par le CLI.

`login --headless --org apigee-forge` :

- charge le credential local via Application Default Credentials ;
- obtient un contexte headless ;
- récupère le project ID `apigee-forge` ;
- vérifie que l'organisation demandée correspond au project ID ;
- retourne uniquement une enveloppe JSON non sensible.

### 2. Lecture initiale

`list-proxies --headless --org apigee-forge` a confirmé que l'organisation était accessible et contenait déjà le proxy de démonstration `helloworld`.

### 3. Génération locale

`generate` ne contacte pas Apigee. Il :

1. lit `examples/helloworld/openapi.yaml` ;
2. lit `examples/helloworld/template.json` ;
3. rend `ProxyEndpoint` et `TargetEndpoint` ;
4. écrit le bundle sous `target/` ;
5. crée `target/forge-helloworld-unique.zip` avec `apiproxy/` comme racine.

### 4. Import réel

`deploy` lit le ZIP local puis appelle l'endpoint d'import Apigee :

```text
POST /v1/organizations/apigee-forge/apis?action=import&name=forge-helloworld-unique
```

Le bundle est envoyé en `multipart/form-data` dans le champ `file`. Apigee crée une nouvelle révision. Plusieurs révisions de test ont été créées pendant le diagnostic ; la révision finalement déployée est la `8`.

L'API réelle renvoie `revision` comme une chaîne (`"8"`) dans la réponse d'import. Le mapping a été rendu compatible avec les réponses de liste (`["1", "2"]`) et d'import (`"8"`).

### 5. Déploiement réel

Le CLI appelle ensuite :

```text
POST /v1/organizations/apigee-forge/environments/eval/apis/forge-helloworld-unique/revisions/8/deployments?override=false
```

Apigee exige un `Content-Length: 0` explicite pour ce POST sans payload. Le client envoyait auparavant un POST sans longueur et recevait `411 Length Required`. Le gateway envoie maintenant un body vide avec cet en-tête.

La commande a retourné :

```json
{
  "ok": true,
  "command": "deploy",
  "data": {
    "proxy_name": "forge-helloworld-unique",
    "environment": "eval",
    "revision": 8,
    "status": "Pending"
  },
  "error": null
}
```

`Pending` signifie que l'appel de déploiement a été accepté mais que le runtime n'a pas encore fini la propagation.

### 6. Lecture du statut

Le CLI appelle :

```text
GET /v1/organizations/apigee-forge/environments/eval/apis/forge-helloworld-unique/revisions/8/deployments
```

Le premier statut observé était `InProgress`, puis après propagation :

```json
{
  "ok": true,
  "command": "status",
  "data": {
    "proxy_name": "forge-helloworld-unique",
    "environment": "eval",
    "revision": 8,
    "status": "Succeeded"
  },
  "error": null
}
```

L'API Apigee utilise l'état réel `READY`. Le mapping CLI convertit désormais `READY` en `Succeeded`.

### 7. Visibilité du proxy

`list-proxies` confirme que `forge-helloworld-unique` et sa révision `8` sont visibles dans l'organisation.

La commande `list-proxies` actuelle ne déduit pas l'état de déploiement : son champ `deployed` reste structurellement à `false`. La preuve de déploiement utilisée pour ce checkpoint est donc la commande `status`, qui retourne `Succeeded`.

## Corrections apportées par le test réel

1. réponse d'import Apigee : `revision` scalaire accepté en plus du tableau de révisions ;
2. état runtime Apigee `READY` mappé vers `Succeeded` ;
3. POST de déploiement sans payload envoyé avec `Content-Length: 0` ;
4. classification des erreurs HTTP 400 et réponses gateway invalides rendue plus explicite sans afficher le corps HTTP.

## Limites restantes

- l'appel HTTP de déploiement est validé ;
- le statut final `Succeeded` est validé ;
- le test de trafic fonctionnel via l'URL publique du runtime nécessite le hostname de l'environnement/groupe d'environnement Apigee et n'est pas inclus ici ;
- la couverture automatisée de tous les états `InMemoryApigeeGateway` reste un point séparé de M4-09.
