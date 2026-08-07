# Exemple CI — CLI Apigee Forge

Cet exemple décrit la forme attendue d’un job non-interactif sans stocker de credential dans le repository.

Le workflow doit préparer un fichier de credential Google via le mécanisme secret du CI, puis définir uniquement le chemin standard :

```yaml
- name: Generate proxy bundle
  env:
    GOOGLE_APPLICATION_CREDENTIALS: ${{ runner.temp }}/gcp-credentials.json
  run: >-
    cargo run --locked -p cli --
    --json generate
    --spec examples/helloworld/openapi.yaml
    --template examples/helloworld/template.json
    --proxy-name helloworld
    --output target/helloworld-output
    --archive target/helloworld.zip
```

Règles :

- la valeur du credential n’est jamais écrite dans le YAML ou le repository ;
- aucun token n’est passé en argument CLI ;
- `--json` produit une sortie machine lisible sur stdout ;
- les erreurs et codes de sortie doivent interrompre le job ;
- les bundles générés restent sous `target/` et ne sont pas commités ;
- les commandes nécessitant Apigee utilisent le même mode headless explicite :

```text
cargo run --locked -p cli -- --json list-proxies --headless --org <organization>
```

Le placeholder `<organization>` doit être fourni par la configuration du job, jamais deviné par le workflow.
