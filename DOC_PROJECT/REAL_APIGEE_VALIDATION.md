# Validation réelle Apigee — Hello World

Cette procédure est exécutée uniquement sur la branche `feature/real-apigee-validation`. Elle ne doit pas être ajoutée à la CI standard et aucun credential ne doit être versionné.

## Informations nécessaires

### Valeurs non secrètes à fournir au CLI

- `GCP_PROJECT_ID` : project ID Google Cloud utilisé pour l'organisation Apigee ;
- `APIGEE_ORG` : identifiant de l'organisation Apigee, généralement égal au project ID ;
- `APIGEE_ENVIRONMENT` : nom exact d'un environnement provisionné, par exemple `test` ou `eval` ;
- `APIGEE_PROXY_NAME` : nom temporaire unique, par exemple `forge-helloworld-20260808`.

Ces valeurs peuvent être communiquées dans la conversation si nécessaire. Ne pas communiquer de credential.

### Credential headless requis localement

Le CLI utilise le mode headless avec `GOOGLE_APPLICATION_CREDENTIALS`. Il faut :

1. un compte de service Google Cloud autorisé à utiliser Apigee ;
2. son fichier JSON de clé enregistré localement ;
3. uniquement le chemin du fichier placé dans la variable d'environnement PowerShell.

Le contenu JSON, `private_key`, token OAuth et secret ne doivent jamais être collés dans la conversation, un ticket ou un commit.

Le rôle minimal dépend de l'action testée :

- lecture : rôle Apigee en lecture adapté ;
- import/déploiement : `roles/apigee.deployer` ou rôle équivalent accordé au compte de service.

Pour le premier test, vérifier dans IAM que le principal utilisé possède bien le rôle effectif sur le projet/org d'évaluation.

## Préparation PowerShell locale

```powershell
$env:GOOGLE_APPLICATION_CREDENTIALS = 'c:\Users\Utilisateur\Documents\apigee-forge\apigee-forge-b1c0171ad15f.json'
$env:GCP_PROJECT_ID = 'apigee-forge'
$env:APIGEE_ORG = 'apigee-forge'
$env:APIGEE_ENVIRONMENT = 'eval'
$env:APIGEE_PROXY_NAME = 'forge-helloworld-unique'
```

La variable `GOOGLE_APPLICATION_CREDENTIALS` ne doit pas être ajoutée à `.env`, au YAML, à Git ou à une capture d'écran.

## Parcours réel prévu

1. vérifier la configuration avec `login --headless` ;
2. vérifier la lecture avec `list-proxies --headless` ;
3. générer le bundle Hello World localement ;
4. importer et déployer avec `deploy --headless` ;
5. lire le statut avec `status --headless` ;
6. confirmer avec `list-proxies --headless` ;
7. vérifier le proxy dans la console Apigee ;
8. documenter les résultats sans données sensibles.

Commandes types :

```powershell
cargo run -p cli --locked -- --json login --headless --org $env:APIGEE_ORG
cargo run -p cli --locked -- --json list-proxies --headless --org $env:APIGEE_ORG
cargo run -p cli --locked -- --json generate --spec examples/helloworld/openapi.yaml --template examples/helloworld/template.json --proxy-name $env:APIGEE_PROXY_NAME --output "target/${env:APIGEE_PROXY_NAME}-output" --archive "target/${env:APIGEE_PROXY_NAME}.zip"
cargo run -p cli --locked -- --json deploy --headless --org $env:APIGEE_ORG --environment $env:APIGEE_ENVIRONMENT --proxy-name $env:APIGEE_PROXY_NAME --bundle "target/${env:APIGEE_PROXY_NAME}.zip"
cargo run -p cli --locked -- --json status --headless --org $env:APIGEE_ORG --environment $env:APIGEE_ENVIRONMENT --proxy-name $env:APIGEE_PROXY_NAME --revision <REVISION>
cargo run -p cli --locked -- --json list-proxies --headless --org $env:APIGEE_ORG
```

Le numéro de révision doit être pris dans la sortie JSON de `deploy` ; il ne doit pas être deviné.

## Ce qui doit être observé

- `login` confirme le mode headless et l'organisation sans exposer de token ;
- `list-proxies` retourne une enveloppe JSON `ok: true` ou une erreur Apigee explicite ;
- `generate` ne contacte pas Apigee et produit uniquement le bundle local ;
- `deploy` réalise deux opérations côté gateway : import du bundle puis déploiement de la révision ;
- `status` lit l'état de cette révision ;
- `list-proxies` confirme que le proxy est visible dans l'organisation.

Toute divergence entre l'API réelle et `APIGEE_API_MAP.md` doit être corrigée dans le code et les tests WireMock avant de cocher le checkpoint.
