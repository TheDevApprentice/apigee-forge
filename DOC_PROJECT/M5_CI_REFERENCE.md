# M5 — Pipeline CI/CD de référence

## Objectif

Le workflow GitHub Actions valide le cas d’usage self-service du CLI en mode non-interactif, sans contacter Apigee et sans credential réel.

Le job `cli-smoke` :

1. compile et exécute le vrai binaire CLI ;
2. utilise la spec `examples/helloworld/openapi.yaml` ;
3. utilise le template `examples/helloworld/template.json` ;
4. génère un bundle local sous `target/` ;
5. vérifie la sortie JSON ;
6. vérifie la liste exacte des entrées ZIP ;
7. publie le ZIP et le résultat JSON comme artefacts du workflow.

## Commande validée

```text
cargo run --locked -p cli -- --json generate \
  --spec examples/helloworld/openapi.yaml \
  --template examples/helloworld/template.json \
  --proxy-name helloworld \
  --output target/helloworld-output \
  --archive target/helloworld.zip
```

Dans le fichier YAML GitHub Actions, cette commande est écrite au format multi-ligne shell compatible Ubuntu.

## Limites volontaires

- aucun appel réseau Apigee dans le workflow ;
- aucun provisioning GCP ;
- aucun token ou credential dans le repository ;
- aucun flag fake/réel dans la CLI ;
- le job vérifie le rendu et le packaging, pas le déploiement réel.

Le parcours réel `login → generate → deploy → status → list-proxies` reste le checkpoint M4-11 et sera exécuté manuellement après provisionnement.
