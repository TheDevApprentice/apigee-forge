# Apigee Forge — Roadmap de démarrage M6

*Document de cadrage du jalon M6 — squelette GUI Tauri/Vue. Les étapes sont volontairement regroupées : M6 est un jalon court centré sur une première interface fonctionnelle, sans éditeur complet ni déploiement réel obligatoire.*

---

## 1. Objectif M6

M6 fournit le premier parcours GUI utilisable :

```text
Tauri/Vue → commandes Tauri typées → core → providers/gateways → état UI
```

Le GUI M6 couvre :

- écran de connexion OAuth desktop ;
- sélection explicite de l’organisation et de l’environnement ;
- dashboard en lecture seule ;
- liste des proxies accessibles ;
- affichage du rôle Apigee ;
- états loading, erreur, vide et succès ;
- structure prête à accueillir l’éditeur de templates M7 ;
- respect des tokens visuels de `DESIGN.md`.

M6 ne couvre pas encore :

- éditeur visuel complet de template ;
- déploiement depuis l’interface ;
- suivi temps réel de déploiement ;
- validation Apigee réelle obligatoire ;
- modification des fichiers depuis le frontend sans commande Tauri validée.

---

## 2. Contraintes et décisions

- Le GUI utilise OAuth desktop ; aucun service account dans le parcours interactif.
- Les commandes Tauri restent la seule frontière frontend/backend.
- Le frontend ne connaît ni `reqwest`, ni `keyring`, ni les types d’infrastructure.
- Les données exposées à Vue sont des DTO sérialisables et non sensibles.
- Les tests GUI et Tauri utilisent des doubles ou des données fixtures ; aucun compte Apigee réel n’est requis pour M6.
- Les composants `base` sont présentationnels ; les composants `domain` utilisent des composables.
- Le thème reste clair uniquement.
- Les erreurs Tauri sont typées côté Rust et affichées sans token, URL interne ou corps HTTP.
- Les transitions et micro-interactions restent sobres ; CSS en priorité, Anime.js seulement si une animation coordonnée apporte une valeur claire.
- Toute étape est commitée avant de passer à la suivante, mais chaque étape M6 peut regrouper plusieurs fichiers cohérents.

---

## 3. État initial

- `gui/src-tauri/src/main.rs` est encore un binaire minimal.
- `gui/src-tauri/src/lib.rs` est le composition root prévu mais ne contient pas encore les commandes GUI.
- Le frontend Vue n’est pas encore présent dans le repository.
- `core` possède déjà les providers OAuth/headless, les modèles d’authentification, les repositories templates et les gateways Apigee.
- `InMemoryApigeeGateway`, les doubles OAuth et WireMock permettent de développer sans réseau réel.
- `DESIGN.md` définit les couleurs, espacements, composants, iconographie et règles d’accessibilité visuelle.

---

## 4. Étapes regroupées M6

### M6-00 — Baseline Git, structure et décision d’architecture

- [x] Créer `feature/m6-gui` depuis `dev` après le merge M4.
- [x] Créer ce document et le référencer dans `STRUCTURE.md` et `PROMPT.md`.
- [x] Confirmer le périmètre M6 et les éléments reportés à M7/M8.
- [x] Vérifier la baseline workspace avant le code GUI.
- [x] Commiter uniquement la documentation et la baseline M6.

Commit prévu :

```text
docs(m6): define Tauri Vue GUI roadmap
```

### M6-01 — Shell GUI et fondation visuelle

Créer le squelette frontend et la structure Tauri minimale :

- [x] layout principal avec sidebar 56px et topbar ;
- [x] navigation visuelle vers Login, Dashboard, Templates, Proxies et Deployments ;
- [x] tokens `DESIGN.md` centralisés en CSS ;
- [x] composants base : bouton, carte, badge/chip, état vide, état erreur, spinner ;
- [x] responsive desktop minimal et focus clavier visible ;
- [x] aucun appel métier directement depuis les composants.

Validation regroupée :

- [x] build frontend ;
- [x] compilation et démarrage Tauri vérifiés ;
- [x] inspection visuelle du shell ;
- [x] contrastes et navigation clavier traités dans les styles ;
- [ ] validation interactive complète à reprendre lors de M6-04.

Commit possible :

```text
feat(gui): add M6 visual application shell
```

### M6-02 — Bridge Tauri et composition root

Brancher les commandes Tauri et les types frontend :

- [x] composition root OAuth desktop dans `gui/src-tauri/src/lib.rs` ;
- [x] commande `auth` pour login/logout/contexte courant ;
- [x] commande `organizations`/`environments` pour la sélection de contexte ;
- [x] commande `proxies` pour la lecture des proxies ;
- [x] DTO Rust sérialisables et interfaces TypeScript correspondantes ;
- [x] injection par ports (`Arc<dyn Trait>`) et doubles disponibles pour les tests ;
- [x] aucune logique `reqwest` ou `keyring` dans le frontend.

Validation regroupée :

- [x] tests Rust des commandes et de sérialisation DTO ;
- [x] build frontend et compilation Tauri ;
- [ ] test frontend des états invoke succès/erreur à compléter dans M6-03.

Commit possible :

```text
feat(gui): add typed Tauri core bridge
```

### M6-03 — Parcours Login → Dashboard → Proxies

Implémenter le parcours visible :

- [x] écran Login avec action OAuth explicite ;
- [x] affichage du contexte organisation/environnement ;
- [x] sélection d’organisation sans sélection implicite ;
- [x] dashboard avec cartes de contexte et rôle ;
- [x] liste de proxies avec états loading, empty, error et success ;
- [x] composables `useAuth`, `useProxies` et `useTemplateEditor` initialisé pour M7 ;
- [x] affichage d’erreurs sûres et récupération/retry contrôlée ;
- [x] aucun appel Apigee réel requis.

Validation regroupée :

- [x] tests de composants/composables avec Vitest ;
- [x] parcours UI avec doubles Tauri ;
- [x] contrôle accessibilité de base : labels, rôles, focus visible et navigation explicite.

Commit possible :

```text
feat(gui): add authentication dashboard and proxy views
```

### M6-04 — Point de contrôle GUI

- [x] Exécuter les tests workspace et les tests frontend.
- [x] Vérifier le build Tauri sans compte Apigee réel (`npm run tauri build -- --no-bundle`).
- [x] Vérifier la séparation CLI/core/GUI.
- [x] Vérifier l’absence de secrets dans les DTO, fixtures et logs.
- [x] Vérifier le respect des tokens `DESIGN.md`.
- [x] Marquer M6 terminé dans `ROADMAP.md` puisque le parcours Login → Dashboard → Proxies fonctionne avec des doubles.

Commit prévu :

```text
docs(m6): record GUI skeleton validation
```

---

## 5. Critères d’acceptation M6

M6 est terminé lorsque :

1. l’application Tauri démarre et affiche le shell visuel ;
2. le parcours Login → Dashboard → Proxies fonctionne avec des doubles ;
3. les commandes Tauri sont typées et n’exposent aucune infrastructure au frontend ;
4. les états loading, erreur et vide sont traités ;
5. la navigation clavier et les contrastes principaux sont acceptables ;
6. aucun compte Apigee réel n’est requis pour les tests M6 ;
7. l’éditeur complet reste explicitement reporté à M7.
