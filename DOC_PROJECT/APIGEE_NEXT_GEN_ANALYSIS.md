# Apigee Forge — Analyse produit Apigee Next Gen

## 1. Distinction environnement du template / environnement Apigee

Le champ `metadata.target_environment` du template (`dev`, `test`, `prod`) est une **cible logique de gouvernance**. Il décrit l’intention du template et peut servir à appliquer des règles, sélectionner une configuration ou proposer une cible.

L’environnement sélectionné dans le topbar (`eval`, `test`, `prod`, etc.) est l’**environnement réel de l’organisation Apigee**. C’est lui qui doit être utilisé pour les appels Management API et les opérations de déploiement.

Ces deux notions ne doivent pas être confondues :

```text
Template target_environment = intention / convention
Apigee selected environment = ressource réelle / destination API
```

Un mapping explicite sera nécessaire dans M8 :

```text
Template target: prod
        ↓
Utilisateur confirme
        ↓
Apigee organization: apigee-forge
Apigee environment: eval
```

Le programme ne doit jamais déployer automatiquement vers `eval` uniquement parce que le template indique `prod`.

## 2. Capacités Apigee utiles au produit

### Création de proxy

Apigee permet de créer un proxy :

- à partir d’un bundle de configuration ZIP ;
- à partir d’une spécification OpenAPI ;
- depuis un proxy vide ou un proxy sans target ;
- avec déploiement immédiat optionnel ou création seule.

API principale :

```text
POST /v1/organizations/{org}/apis?name={proxy}&action=import
```

Pour un bundle, le fichier doit respecter la structure Apigee avec un dossier racine `apiproxy`.

### Révisions

Une modification d’un proxy crée une nouvelle révision. Une révision déployée ne doit plus être éditée directement : il faut créer une nouvelle révision puis la déployer.

Conséquence produit :

```text
Template → génération bundle → création révision → validation → déploiement explicite
```

### Déploiement

Le déploiement est une opération séparée de la création de révision :

```text
POST /v1/organizations/{org}/environments/{env}/apis/{api}/revisions/{rev}/deployments
```

Le bouton de déploiement doit donc ouvrir une étape de confirmation avec :

- organisation ;
- environnement réel ;
- proxy ;
- révision ;
- base paths ;
- impact sur la révision actuellement active ;
- option `override` ;
- compte de service éventuel ;
- option de rollout séquencé.

Avant l’action réelle, Apigee propose aussi des rapports de changement deploy/undeploy. Forge doit les exploiter avant M8 si les permissions sont disponibles.

### États runtime

Le statut ne doit pas être réduit à `deployed: boolean`. Les états utiles sont notamment :

- `ACTIVE` ;
- `PROGRESSING` ;
- `ERROR` ;
- `INACTIVE` ;
- conflits de routage ;
- erreurs runtime par instance.

L’UI doit distinguer :

```text
Revision créée
Revision validée
Revision déployée dans l’environnement
Runtime prêt
Runtime en erreur
```

### OpenAPI

Le workflow OpenAPI doit être guidé :

1. importer ou sélectionner une spec OpenAPI ;
2. valider la spec ;
3. afficher les serveurs, paths et méthodes détectés ;
4. choisir reverse proxy ou proxy sans target ;
5. choisir le nom et la convention du proxy ;
6. appliquer un template de policies ;
7. prévisualiser le flow généré ;
8. générer le bundle ;
9. créer la révision ;
10. proposer le déploiement séparément.

La spec OpenAPI ne doit pas être envoyée directement à Apigee sans aperçu et validation locale.

## 3. Vision produit Forge Next Gen

Forge doit être un orchestrateur guidé, pas seulement un formulaire devant l’API Apigee.

### Parcours Template

```text
Catalogue
  → Nouveau template
  → Metadata
  → Flow
  → Policies
  → Validation
  → Review
  → Sauvegarde locale
```

Le template représente la gouvernance réutilisable :

- sécurité ;
- quotas ;
- CORS ;
- transformation ;
- conventions de nommage ;
- ownership ;
- environnement logique cible.

### Parcours Proxy depuis template

```text
Choisir template
  → Choisir source OpenAPI ou configuration manuelle
  → Valider les entrées
  → Prévisualiser le proxy
  → Générer bundle
  → Créer proxy/révision
  → Vérifier la révision
  → Choisir environnement réel
  → Prévisualiser impact déploiement
  → Confirmer déploiement
  → Suivre runtime
```

### Parcours Upload bundle

```text
Importer ZIP
  → Vérifier dossier apiproxy
  → Inspecter proxy endpoints / target endpoints / policies
  → Afficher les différences éventuelles
  → Choisir nouveau proxy ou nouvelle révision
  → Créer/importer
  → Vérifier
  → Déployer explicitement
```

## 4. Principes UX de sécurité opérationnelle

- Ne jamais confondre création, import, révision et déploiement.
- Ne jamais utiliser automatiquement l’environnement logique du template comme destination réelle.
- Afficher un résumé avant toute opération qui modifie Apigee.
- Toujours indiquer l’organisation et l’environnement réel.
- Afficher les permissions nécessaires avant l’action si elles sont connues.
- Préférer une validation locale et un dry-run avant l’appel mutatif.
- Afficher l’état de l’opération longue et permettre de consulter les erreurs.
- Ne jamais masquer une erreur Apigee derrière un simple booléen.
- Garder les opérations destructives derrière un modal de confirmation explicite.

## 5. Roadmap recommandée après M7

### M7 restant

- fixtures valides/invalides ;
- intégration CLI réelle d’un template créé dans le GUI ;
- tests de parcours complets ;
- revue de l’éditeur et nettoyage du contrat.

### M8 — Orchestration proxy et déploiement

1. modèle de workflow d’opération ;
2. génération bundle depuis template + OpenAPI ;
3. preview du bundle et des endpoints ;
4. création/import de proxy ;
5. création de révision ;
6. rapport de changement deploy ;
7. confirmation environnement réel ;
8. déploiement ;
9. suivi des opérations et runtime ;
10. historique local des actions ;
11. gestion retry/idempotence ;
12. tests avec gateway fake et validation manuelle Apigee.

### M9 — Design system et visualisation

- diagramme de flow avancé ;
- états runtime plus lisibles ;
- design tokens ;
- densité et hiérarchie visuelle ;
- responsive approfondi ;
- cohérence des modals et confirmations.

## 6. Sources officielles

- [Creating an API proxy](https://cloud.google.com/apigee/docs/api-platform/develop/ui-create-proxy)
- [Building a simple API proxy](https://cloud.google.com/apigee/docs/api-platform/fundamentals/build-simple-api-proxy)
- [Editing an API proxy](https://cloud.google.com/apigee/docs/api-platform/develop/ui-edit-proxy)
- [Downloading and uploading API proxy bundles](https://cloud.google.com/apigee/docs/api-platform/fundamentals/download-api-proxies)
- [Deploying an API proxy](https://cloud.google.com/apigee/docs/api-platform/deploy/ui-deploy-new)
- [Apigee environments overview](https://cloud.google.com/apigee/docs/api-platform/fundamentals/environments-overview)
- [Method: organizations.apis.revisions.deployments.deploy](https://cloud.google.com/apigee/docs/reference/apis/apigee/rest/v1/organizations.environments.apis.revisions.deployments/deploy)
