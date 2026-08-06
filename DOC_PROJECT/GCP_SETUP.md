# Apigee Forge — Mise en place de l'environnement Google Cloud de test

*Aucun accès entreprise requis. Tout est réalisable avec des comptes Google personnels et gratuits. Référencé depuis ARCHITECTURE.md section 12 (niveau 4 de la stratégie de test).*

---

## 1. Créer le projet Google Cloud

1. Se connecter sur [console.cloud.google.com](https://console.cloud.google.com) avec un compte Google personnel (le compte qui jouera le rôle de **superviseur**).
2. Créer un nouveau projet (sélecteur de projet en haut de la console → "Nouveau projet").
3. Activer la facturation sur ce projet — une carte doit être enregistrée même si l'usage évalué reste gratuit ; ne pas être surpris par cette étape, aucun débit n'a lieu pour l'org d'évaluation.

## 2. Provisionner l'organisation Apigee d'évaluation

1. Dans la console, rechercher "Apigee" ou aller sur la page "Set up Apigee Evaluation".
2. Choisir le plan **Evaluation** (gratuit, 60 jours).
3. Suivre l'assistant : activation des API nécessaires, configuration réseau (VPC), création de l'organisation d'évaluation.
4. Le provisionnement peut prendre jusqu'à 45 minutes — ne pas s'inquiéter si ça semble lent.
5. À l'expiration des 60 jours, l'organisation est supprimée automatiquement. Pour continuer les tests au-delà, répéter cette section sur un nouveau projet (voir ARCHITECTURE.md section 12).

## 3. Confirmer le rôle superviseur (compte principal)

Le compte ayant créé le projet a généralement déjà les pleins droits. Pour le confirmer/assigner explicitement :

1. Console Google Cloud → **IAM et administration** → **IAM**.
2. Vérifier que le compte principal a le rôle **Apigee Admin** (`roles/apigee.admin`) — accès complet, à utiliser pour simuler le profil superviseur dans l'app.

## 4. Créer un deuxième compte pour simuler le profil développeur

Comme il n'y a pas d'organisation d'entreprise ici, le moyen le plus simple est un **second compte Google personnel** (un compte gratuit supplémentaire, par exemple une adresse Gmail secondaire créée pour l'occasion).

1. Créer ce second compte Google s'il n'existe pas déjà.
2. Dans le même projet, aller dans **IAM et administration** → **IAM** → **Accorder l'accès**.
3. Coller l'adresse e-mail du second compte comme nouveau principal.
4. Lui assigner le rôle **Apigee Deployer** (`roles/apigee.deployer`) — un rôle plus restreint que l'admin, adapté pour simuler un profil développeur qui déploie/gère des proxies sans droits d'administration complets sur l'organisation.

**Note sur la granularité** : les rôles Apigee prédéfinis (`apigee.admin`, `apigee.deployer`, `apigee.developerAdmin`, `apigee.readOnlyAdmin`, `apigee.portalAdmin`) sont volontairement larges — Google ne propose pas nativement un rôle "développeur qui édite des templates mais ne déploie pas en prod" par exemple. Si une distinction plus fine est nécessaire plus tard, un **rôle IAM personnalisé** (IAM et administration → Rôles → Créer un rôle) permet de composer un ensemble précis de permissions — mais pas nécessaire pour valider le MVP.

## 5. Répéter pour un deuxième profil superviseur (si besoin de tester plusieurs superviseurs)

Même procédure que l'étape 4, avec un troisième compte Google et le rôle `roles/apigee.admin` au lieu de `apigee.deployer`.

## 6. Se connecter dans l'application avec chaque compte

Le flux OAuth desktop du GUI est par personne, pas par machine — pour tester les deux profils sans conflit :
- Utiliser une fenêtre de navigation privée (ou un profil de navigateur séparé) pour lancer le flux de connexion avec le second compte, sans perturber la session du compte principal.
- Se déconnecter/reconnecter dans l'app entre chaque test si une seule session à la fois suffit.

## 7. Vérifier que le rôle est bien pris en compte

- Le plus simple : observer que l'interface adapte l'affichage/les actions disponibles selon le compte connecté (c'est justement le comportement que l'app doit démontrer).
- Vérification manuelle indépendante si besoin : `gcloud projects get-iam-policy PROJECT_ID` en ligne de commande, ou consulter directement la page IAM de la console.
