# Apigee Forge — Roadmap de démarrage M9

*Jalon de polish visuel et de cohérence du design system. M9 transforme l’interface fonctionnelle M8 en une expérience visuelle cohérente avec `DESIGN.md`, sans ajouter de thème sombre ni modifier les contrats métier, les workflows Apigee ou les frontières Tauri. Chaque étape doit être vérifiée et commitée avant de passer à la suivante.*

---

## 1. Objectif M9

M9 doit aligner l’ensemble du GUI sur la direction visuelle validée : thème clair, accent teal, cartes sobres, densité inspirée de Google Cloud Console, composants cohérents et diagramme de flow comme élément visuel signature.

Le résultat attendu :

```text
Tokens DESIGN.md
       ↓
CSS global cohérent
       ↓
Composants base harmonisés
       ↓
Navigation + topbar + états
       ↓
Dashboard / Proxies / Templates / Deployments
       ↓
Flow diagram PreFlow → Conditional Flow → PostFlow
       ↓
Responsive + accessibilité + reduced motion
       ↓
QA visuelle et checkpoint M9
```

### Principes non négociables

- `DESIGN.md` est la source de vérité des tokens visuels.
- Thème clair uniquement ; aucun dark mode ne doit être ajouté.
- Accent teal : `#0F6E56`, fond `#E1F5EE`, texte accent `#085041`.
- Aucun texte noir ou gris sur un fond accent `#E1F5EE`.
- Les cartes utilisent une bordure fine et aucun shadow décoratif.
- Les composants base restent présentationnels ; la logique métier reste dans les composables et l’état parent.
- Les animations doivent respecter `prefers-reduced-motion`.
- M9 ne doit pas modifier les commandes Tauri, les DTO, les use cases ni les contrats de templates.
- Les libellés et actions doivent conserver la distinction entre création de proxy, création de révision et déploiement.

### Hors périmètre M9

- nouvelle fonctionnalité Apigee ou nouveau endpoint ;
- refonte du modèle de template ou des policies MVP ;
- mode sombre ;
- authentification Service Account, traitée dans M8-09-05 ;
- drafts persistants, traités dans M8-09-04 ;
- refonte de l’architecture Rust/Tauri ;
- ajout d’une librairie d’animation sans nécessité démontrée ;
- changement du comportement métier pour obtenir uniquement un résultat visuel.

---

## 2. État de départ et écarts connus

### Déjà disponible

- tokens principaux déjà présents dans `gui/src/style.css` ;
- sidebar icônes seules et topbar workspace ;
- composants base `BaseButton`, `BaseCard`, `BaseChip`, `BaseModal`, `BaseEmptyState`, `BaseErrorState`, `BaseSpinner` ;
- éditeur M7 et parcours de création de proxy M8 ;
- écrans Dashboard, Templates, Proxies, Deployments et Settings ;
- diagramme fonctionnel du flow PreFlow/Conditional Flow/PostFlow ;
- tests Vue et build Vite ;
- support existant de `prefers-reduced-motion`.

### Écarts à traiter

- plusieurs valeurs CSS sont encore codées en dur au lieu d’utiliser les tokens ;
- certaines ombres, tailles, rayons et graisses divergent de `DESIGN.md` ;
- les états loading, error, empty, success et in-progress ne partagent pas encore une hiérarchie visuelle uniforme ;
- certains boutons et libellés historiques ne reflètent plus la distinction proxy/révision/déploiement ;
- les panneaux de création et de review peuvent dépasser la hauteur utile de la fenêtre ;
- le flow visuel doit devenir un élément plus lisible et plus structurant sans devenir décoratif ;
- la densité et l’espacement varient entre Dashboard, Templates, Proxies et Deployments ;
- l’accessibilité visuelle des focus, erreurs et statuts doit être vérifiée après le polish.

---

## 3. Décisions d’architecture visuelle

- Les tokens sont centralisés dans `:root` et utilisés par les composants ; les nouvelles couleurs doivent être ajoutées à `DESIGN.md` avant utilisation.
- Les composants base ne connaissent pas les domaines métier et ne déclenchent aucune commande Tauri.
- Les composants métier composent les composants base et reçoivent leurs données via props/events.
- Le diagramme de flow reste piloté par les données de l’éditeur ; M9 ne duplique pas le modèle métier dans le CSS ou dans une seconde structure frontend.
- Les états asynchrones utilisent une grammaire commune : label de contexte, message court, action de récupération et zone `aria-live` si nécessaire.
- Les transitions visuelles restent courtes et discrètes ; une animation ne doit jamais masquer une information ou empêcher une action clavier.
- Les ombres existantes héritées des prototypes seront supprimées ou remplacées par les bordures normatives de `DESIGN.md`, sauf exception explicitement documentée.

---

## 4. Étapes atomiques M9

### M9-00 — Baseline visuelle et inventaire des écarts

- [x] Vérifier que `feature/m9-design-polish` est créée depuis `dev` après intégration de M8.
- [ ] Capturer les vues de référence Dashboard, Templates, Proxies, Deployments et Settings en mode Live/Demo ; captures manuelles restantes à faire pendant la validation visuelle.
- [x] Inventorier couleurs, rayons, bordures, ombres, espacements, tailles et graisses actuellement divergents.
- [x] Identifier les composants et sélecteurs CSS à migrer vers les tokens sans modifier leur comportement.
- [x] Définir une checklist de validation visuelle et une largeur de fenêtre de référence.

#### Baseline M9 enregistrée

- **Branche** : `feature/m9-design-polish`, créée depuis le commit M8 intégré dans `dev`.
- **Fenêtre de référence** : 1200 × 760 px, correspondant à `tauri.conf.json` ; vérifier également une largeur étroite jusqu’à la largeur minimale de 960 px.
- **Vues à comparer** : Dashboard connecté, Templates catalogue/éditeur/review, Proxies catalogue/détails/création, Deployments review/succès/erreur, Settings, en modes Demo et Live lorsque le contexte est disponible.
- **Écarts CSS relevés** : ombres présentes sur plusieurs surfaces, rayons `7px`, `9px`, `10px` et `14px`, couleurs d’erreur/succès/warning codées en dur, quelques tailles et graisses hors des tokens documentés.
- **Sélecteurs prioritaires** : `BaseCard`, `BaseButton`, `BaseChip`, `BaseModal`, états empty/error/loading, `workspace-selectors`, `review-grid`, `deployment-preparation`, `proxy-detail`, `proxy-revisions`, `flow-canvas`.

#### Checklist de comparaison visuelle

- [ ] Aucun dark mode ou style de surface non prévu par `DESIGN.md`.
- [ ] Les surfaces principales utilisent `#FAFAF9`, `#F1F3F2`, `#FFFFFF` et `#E2E5E3` selon leur rôle.
- [ ] Les textes sur `#E1F5EE` utilisent `#085041`.
- [ ] Les actions actives utilisent `#0F6E56` sans dépendre uniquement de la couleur.
- [ ] Les cartes principales n’ont pas d’ombre décorative.
- [ ] Les focus, erreurs, loading states et statuts restent lisibles et accessibles.
- [ ] Les écrans restent utilisables à 960 px et sans débordement horizontal.

Commit prévu :

```text
docs(m9): define visual baseline and polish checklist
```

### M9-01 — Tokens exacts et fondations CSS

- [x] Aligner les tokens `:root` exactement sur `DESIGN.md`.
- [x] Ajouter les tokens manquants uniquement après validation documentaire : spacing, border width, radius, focus, states.
- [x] Remplacer les couleurs codées en dur par les tokens lorsqu’elles correspondent à un rôle existant.
- [x] Supprimer les ombres et rayons non conformes des surfaces principales.
- [x] Harmoniser la typographie sur les graisses 400/500 uniquement.
- [x] Tester que le thème clair reste le seul thème disponible.

Les captures visuelles comparatives restent à réaliser manuellement sur les vues de référence ; les fondations CSS sont alignées et couvertes par le build/tests frontend.

Commit prévu :

```text
refactor(gui): align CSS tokens with design specification
```

### M9-02 — Composants base et grammaire des états

- [x] Harmoniser `BaseButton` : variantes, états disabled, focus-visible, action primaire/secondaire.
- [x] Harmoniser `BaseCard`, `BaseChip`, `BaseModal`, `BaseEmptyState`, `BaseErrorState` et `BaseSpinner`.
- [x] Définir les styles communs loading, success, warning, error et not-deployed.
- [x] Vérifier les contrastes et la lisibilité des textes sur fond accent.
- [x] Conserver les composants base sans logique de domaine ni appel Tauri.
- [x] Ajouter les tests de rendu/comportement des variantes accessibles.

Commit prévu :

```text
refactor(gui): harmonize base components and state styles
```

### M9-03 — Sidebar, topbar et navigation

- [x] Aligner la sidebar sur 56px, les icônes outline 20px et les couleurs active/inactive normatives.
- [x] Harmoniser les tooltips, labels ARIA et focus clavier des entrées de navigation.
- [x] Améliorer la hiérarchie de la topbar workspace sans augmenter inutilement sa hauteur.
- [x] Harmoniser les sélecteurs organisation/environnement et le switch Live/Demo.
- [x] Corriger les libellés historiques pour distinguer `Create proxy`, `Create revision`, `Review deployment` et `Deploy revision`.
- [x] Tester la navigation au clavier et le redimensionnement horizontal.

Commit prévu :

```text
feat(gui): polish navigation and workspace chrome
```

### M9-04 — Dashboard, Templates, Proxies et Deployments

- [ ] Harmoniser la grille, les métriques, les cartes d’action et les empty states du Dashboard.
- [ ] Harmoniser le catalogue Templates, l’éditeur, les drafts et les actions de sauvegarde sans modifier leur logique.
- [ ] Harmoniser le catalogue Proxies, les lignes de révisions, les badges de statut et le parcours de création.
- [ ] Harmoniser la review Deployment, la progression, le polling, le succès, l’échec, le retry et l’arrêt.
- [ ] Vérifier que chaque action destructive ou mutative conserve sa confirmation modale.
- [ ] Tester qu’un polish CSS ne réintroduit pas de double soumission ou de perte d’état.

Commit prévu :

```text
feat(gui): polish application workflow surfaces
```

### M9-05 — Diagramme de flow signature

- [ ] Extraire ou stabiliser un composant de diagramme de flow piloté par les données existantes.
- [ ] Représenter clairement `PreFlow → Conditional Flow(s) → PostFlow`.
- [ ] Utiliser les cartes blanches, bordures `#E2E5E3`, rayon 8px et connecteurs `#C4C9C6` prévus par `DESIGN.md`.
- [ ] Afficher les policies sous forme de tags teal avec `#E1F5EE` + `#085041`.
- [ ] Distinguer request/response sans dépendre uniquement de la couleur.
- [ ] Prévoir une disposition responsive et une alternative clavier aux interactions visuelles.
- [ ] Respecter `prefers-reduced-motion` si une transition de diagramme est ajoutée.

Commit prévu :

```text
feat(gui): refine proxy flow diagram
```

### M9-06 — Policies, formulaires et feedback visuel

- [ ] Harmoniser les chips de policies et leurs icônes outline.
- [ ] Harmoniser les formulaires metadata, OpenAPI, proxy creation et review.
- [ ] Rendre les erreurs inline cohérentes, localisables et visibles par lecteur d’écran.
- [ ] Améliorer les états de validation, sauvegarde, génération, upload et déploiement.
- [ ] Vérifier que les textes restent lisibles avec des noms de proxy, templates et organisations longs.
- [ ] Tester le comportement avec listes vides, erreurs et contenus volumineux.

Commit prévu :

```text
refactor(gui): harmonize policy forms and feedback states
```

### M9-07 — Responsive, focus et motion

- [ ] Vérifier les largeurs 960px minimum, fenêtres étroites et contenu scrollable.
- [ ] Éviter les débordements horizontaux du flow, des reviews et des formulaires.
- [ ] Vérifier les focus-visible sur tous les contrôles interactifs.
- [ ] Vérifier `aria-live`, `role=alert`, labels et descriptions après les changements visuels.
- [ ] Ajouter uniquement des transitions utiles et respecter `prefers-reduced-motion`.
- [ ] Vérifier que les modals restituent le focus à leur déclencheur après fermeture.

Commit prévu :

```text
feat(gui): finalize responsive accessibility and motion polish
```

### M9-08 — QA visuelle et checkpoint de sortie

- [ ] Comparer les vues de référence avant/après sur Dashboard, Templates, Proxies, Deployments et Settings.
- [ ] Vérifier les tokens utilisés contre `DESIGN.md` et supprimer les écarts non documentés.
- [ ] Exécuter `cargo test --workspace`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, tests frontend et build Vite.
- [ ] Vérifier l’absence de dark mode, de dépendance d’animation inutile et de secret dans les fixtures.
- [ ] Tester Demo et Live sans modifier les contrats métier.
- [ ] Documenter les exceptions visuelles restantes et les reports post-MVP.
- [ ] Mettre à jour `ROADMAP.md` et préparer l’intégration de M9 dans `dev`.

Commit prévu :

```text
test(m9): validate visual polish checkpoint
```

---

## 5. Critères d’acceptation M9

M9 sera terminé lorsque :

1. les tokens visuels utilisés par le GUI correspondent à `DESIGN.md` ;
2. le thème clair et l’accent teal sont cohérents sur toutes les vues ;
3. les composants base partagent les mêmes bordures, rayons, typographie et états ;
4. Dashboard, Templates, Proxies et Deployments présentent une hiérarchie visuelle cohérente ;
5. le diagramme PreFlow → Conditional Flow → PostFlow est lisible, responsive et accessible ;
6. les formulaires, erreurs, loading states et statuts de déploiement sont visuellement cohérents ;
7. la navigation clavier, les focus, les annonces et le reduced motion sont préservés ;
8. aucun comportement métier M8 n’est régressé ;
9. les tests, clippy et build passent ;
10. les écarts restants sont documentés avant l’intégration dans `dev`.

---

## 6. Premier point recommandé

Le premier travail M9 doit être **M9-00 — Baseline visuelle et inventaire des écarts**.

Il faut d’abord mesurer les divergences réelles entre `DESIGN.md` et le CSS actuel avant de modifier les composants. Cette étape évite un polish subjectif, limite les régressions visuelles et permet de répartir les commits par fondation, composants et vues.
