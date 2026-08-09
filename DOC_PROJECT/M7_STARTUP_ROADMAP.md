# Apigee Forge — Roadmap de démarrage M7

*Jalon de construction de l’éditeur visuel de templates. M7 transforme le contrat de template déjà validé par `core` en une expérience d’édition guidée, persistante, testable et compatible avec le CLI. M7 ne couvre pas encore le déploiement GUI ni le polish visuel final M9.*

---

## 1. Objectif M7

Le GUI doit permettre de créer, ouvrir, modifier, valider et sauvegarder un template Apigee Forge sans obliger l’utilisateur à écrire directement le JSON.

Le template édité doit rester compatible avec :

- `core::domain::Template` ;
- `Template::from_json_value` et ses validations ;
- `TemplateRepository` et le repository filesystem ;
- le CLI de génération de bundle ;
- le schema `schemas/template.schema.json`.

Parcours cible :

```text
Templates → New template → Metadata → Flow → Policies → Validate → Save
                                                          ↓
                                                   JSON compatible CLI
```

### Hors périmètre M7

- déploiement Apigee depuis le GUI, prévu en M8 ;
- suivi runtime et actions deploy/undeploy, prévu en M8 ;
- diff visuel entre révisions, prévu au-delà du MVP ;
- bibliothèque partagée ou marketplace de templates ;
- support de policies Apigee non présentes dans le contrat MVP ;
- refonte graphique complète et tokens finaux, prévue en M9.

---

## 2. État de départ

### Déjà disponible

- modèle Rust `Template` complet ;
- validation métier du template ;
- enum `PolicyType` pour les policies MVP ;
- schema JSON versionné ;
- repository filesystem avec opérations CRUD ;
- use cases `create_template` et `template_crud` ;
- moteur de rendu CLI déjà compatible avec le template ;
- composable Vue `useTemplateEditor` minimal ;
- écran Templates avec état vide ;
- shell Live/Demo stabilisé par M6-Bis.

### À construire

- bridge Tauri entre GUI et repository/use cases ;
- état d’édition réactif et typé ;
- formulaires de metadata ;
- représentation visuelle PreFlow/Flows/PostFlow ;
- éditeur de policies avec formulaires par type ;
- validation inline et synthèse des erreurs ;
- sauvegarde, ouverture, suppression et gestion du template courant ;
- prévention de perte des modifications ;
- tests frontend, Rust et intégration repository.

---

## 3. Décisions d’architecture M7

- Le JSON du template n’est pas la source d’état principale de l’UI ; l’UI manipule un état typé correspondant au domaine.
- La validation finale reste dans `core`, même si l’UI fournit une validation anticipée pour améliorer l’expérience.
- Vue ne connaît ni le filesystem ni le repository concret ; toutes les opérations passent par des commandes Tauri et des DTO.
- Les erreurs de validation sont retournées sous forme de messages sûrs et localisables, sans panic ni fuite de chemin sensible.
- Les policies inconnues ne sont pas silencieusement supprimées lors de l’ouverture d’un template ; elles doivent produire une erreur explicite ou un état non éditable documenté.
- Le nom du template est l’identifiant fonctionnel ; les sauvegardes doivent être atomiques et ne pas laisser de fichier partiellement écrit.
- Le mode Demo peut utiliser le même contrat de commandes avec un repository local ; M7 ne doit pas introduire d’appel réseau Apigee.

---

## 4. Étapes atomiques

### M7-00 — Baseline et contrat de l’éditeur

- [x] Confirmer la branche `feature/m7-template-editor` créée depuis `dev` après clôture M6-Bis.
- [x] Définir le DTO bridge de template sérialisable entre Rust et Vue (`name` + `data` JSON validé par `core`).
- [x] Définir le DTO d’erreur de validation avec code, message et chemin logique optionnel du champ.
- [x] Documenter les invariants qui doivent rester identiques entre UI, `core` et schema JSON.
- [x] Ajouter les critères de non-régression M6-Bis : auth, contexte workspace et déconnexion.

### M7-01 — Port Tauri de templates

- [x] Injecter un `TemplateRepository` dans `GuiState` sans exposer son implémentation à Vue.
- [x] Ajouter `list_templates`.
- [x] Ajouter `get_template`.
- [x] Ajouter `create_template`.
- [x] Ajouter `update_template`.
- [x] Ajouter `delete_template` avec confirmation côté UI à brancher dans M7-03.
- [x] Mapper les erreurs repository vers des codes GUI stables.
- [x] Couvrir le contrat repository et les use cases avec les doubles existants de `core`; le test Tauri state fake complet reste à renforcer dans M7-11.

### M7-02 — État d’édition Vue

- [x] Remplacer le placeholder `useTemplateEditor` par un état complet.
- [x] Gérer template courant, état initial, état modifié et état sauvegardé.
- [x] Gérer loading, empty, saving, saved et error.
- [x] Ajouter une action reset/undo vers le dernier état sauvegardé.
- [x] Ajouter une protection contre l’écrasement d’un template ouvert pendant une modification.
- [x] Tester les transitions de l’état d’édition.

### M7-03 — Liste et sélection des templates

- [x] Remplacer l’état vide de l’écran Templates par une liste de templates locaux.
- [x] Afficher nom et owner ; description, environnement cible et dernière action seront enrichis dans M7-04.
- [x] Ajouter recherche ou filtrage local par nom.
- [x] Ajouter sélection d’un template et ouverture de son éditeur.
- [x] Ajouter bouton `New template`.
- [x] Ajouter bouton de suppression avec confirmation.
- [x] Ajouter état vide, erreur, chargement et liste vide filtrée.

### M7-04 — Formulaire Metadata

- [x] Éditer `metadata.name`.
- [x] Éditer `metadata.description`.
- [x] Éditer `metadata.owner`.
- [x] Éditer `metadata.target_environment`.
- [x] Éditer `metadata.naming_convention.prefix`.
- [x] Éditer `metadata.naming_convention.case`.
- [x] Afficher les erreurs de validation au niveau du champ.
- [x] Empêcher une sauvegarde si les champs obligatoires sont invalides.

### M7-05 — Canevas visuel du flux

- [x] Représenter les trois zones `PreFlow`, `Conditional Flows`, `PostFlow`.
- [x] Distinguer request et response dans chaque stage.
- [x] Afficher le nombre de policies par stage.
- [x] Permettre de sélectionner un stage pour préparer l’édition de ses policies.
- [x] Ajouter un flux conditionnel avec sa condition.
- [x] Supprimer un flux conditionnel avec confirmation.
- [x] Préparer une structure compatible avec un diagramme amélioré M9 sans bloquer M7.
- [ ] Sélectionner et éditer une policy individuelle : reporté à M7-06.

### M7-06 — Catalogue des policies MVP

- [x] Ajouter le catalogue des types autorisés par `PolicyType`.
- [x] Créer un formulaire `security_api_key`.
- [x] Créer un formulaire `security_oauth2`.
- [x] Créer un formulaire `security_jwt`.
- [x] Créer un formulaire `quota`.
- [x] Créer un formulaire `spike_arrest`.
- [x] Créer un formulaire `cors`.
- [x] Créer un formulaire `transform`.
- [x] Refuser explicitement l’ajout d’un type non supporté par le catalogue UI.

### M7-07 — Composition et réordonnancement des policies

- [x] Ajouter une policy dans request ou response.
- [x] Supprimer une policy.
- [x] Modifier une policy existante.
- [x] Réordonner les policies à l’intérieur d’un stage.
- [x] Réordonner les policies d’un flow conditionnel.
- [x] Garantir que l’ordre UI est conservé dans le JSON sauvegardé.
- [x] Tester le cycle d’édition et la conservation de l’ordre via les tests frontend du composable ; les scénarios détaillés par policy restent à compléter dans M7-11.

### M7-08 — Validation et compatibilité CLI

- [x] Sérialiser l’état Vue vers le format JSON officiel du template.
- [x] Appeler la validation métier `core` via Tauri avant sauvegarde.
- [x] Afficher une synthèse des erreurs avec champ logique lorsqu’il est identifiable.
- [x] Vérifier les champs inconnus et les enums invalides via `Template::from_json_value`.
- [x] Vérifier la compatibilité avec le schema `schemas/template.schema.json` via le contrat `core`.
- [ ] Vérifier qu’un template créé dans l’UI peut être consommé par la commande CLI `generate` — checkpoint d’intégration à compléter.
- [ ] Ajouter fixtures JSON valides et invalides sans secrets — complétion prévue dans M7-11.

### M7-09 — Sauvegarde et cycle de vie

- [x] Sauvegarder un nouveau template dans le repository filesystem.
- [x] Mettre à jour un template existant.
- [x] Afficher un feedback de sauvegarde réussi.
- [x] Gérer une erreur d’écriture sans perdre l’état local.
- [x] Afficher un indicateur `Unsaved changes`.
- [x] Demander confirmation avant remplacement ou abandon d’un template modifié.
- [x] Écrire via un fichier temporaire synchronisé puis renommé, avec nettoyage en cas d’erreur.
- [x] Tester le cycle CRUD via repository fake ; les scénarios d’erreur filesystem détaillés restent à compléter dans M7-11.

### M7-10 — UX responsive et accessibilité

- [x] Garantir la navigation clavier dans la liste et l’éditeur.
- [x] Associer chaque champ à un label accessible.
- [x] Rendre les erreurs lisibles par les technologies d’assistance via `role=alert` et `aria-live`.
- [x] Prévoir une disposition utilisable avec une fenêtre plus étroite.
- [x] Éviter les composants qui dépendent uniquement du drag-and-drop ; les boutons haut/bas restent disponibles.
- [x] Préserver les tokens existants sans lancer le polish M9.

### M7-11 — Tests de parcours

- [ ] Tester création d’un template minimal.
- [ ] Tester ouverture et modification d’un template existant.
- [ ] Tester chaque formulaire de policy MVP.
- [ ] Tester ajout, suppression et réordonnancement.
- [ ] Tester validation invalide et correction inline.
- [ ] Tester sauvegarde et rechargement.
- [ ] Tester perte de modifications et confirmation.
- [ ] Tester absence de réseau pour toutes les opérations locales.

### M7-12 — Checkpoint de sortie M7

- [ ] Exécuter tests workspace, Clippy, tests frontend et build Tauri.
- [ ] Vérifier manuellement création, édition, validation et sauvegarde d’un template.
- [ ] Générer un bundle CLI à partir d’un template créé dans le GUI.
- [ ] Vérifier qu’aucun secret ou chemin local sensible n’est affiché ou persisté dans les fixtures.
- [ ] Vérifier que M8 peut consommer le template sauvegardé sans refonte du contrat.
- [ ] Mettre à jour `ROADMAP.md` et préparer la branche suivante.

---

## 5. Premier point recommandé

Le premier travail M7 doit être **M7-01 — Port Tauri de templates**.

Sans ce port, l’interface ne pourra utiliser que des fixtures locales et le reste de l’éditeur risquerait de construire un état parallèle au repository réel. Ce point réduit le risque d’intégration avant de travailler le visuel et les formulaires.

---

## 6. Critères d’acceptation M7

M7 sera terminé lorsque :

1. l’utilisateur peut créer, ouvrir, modifier, valider et supprimer un template depuis le GUI ;
2. les metadata et les flows sont éditables sans écrire directement le JSON ;
3. les sept types de policies MVP sont représentés par des formulaires guidés ;
4. l’ordre des policies et des flows est conservé ;
5. les erreurs de validation sont compréhensibles et reliées aux champs ;
6. les templates sauvegardés sont compatibles avec `core` et le CLI ;
7. les modifications non sauvegardées sont signalées et protégées ;
8. le parcours fonctionne sans réseau en mode local ;
9. les tests couvrent le cycle CRUD, la validation et le parcours d’édition ;
10. M8 peut ajouter la génération et le déploiement sans réécrire l’éditeur.
