# Apigee Forge — Design system GUI et page d’accueil

**Statut : référence normative**  
**Version : 1.0**  
**Dernière mise à jour : 2026-08-30**

Ce document fige la direction visuelle et l’expérience utilisateur du GUI Apigee Forge. Toute évolution frontend doit partir de ces principes. Une divergence volontaire doit être documentée et validée avant implémentation.

---

## 1. Intention produit

Apigee Forge est un atelier calme pour construire, vérifier et livrer des proxies Apigee. L’interface doit rendre un workflow technique complexe immédiatement compréhensible :

```text
Template → OpenAPI → Preview → Bundle → Revision → Deployment → Status
```

Le GUI n’est pas une console d’administration froide et ne doit pas ressembler à un formulaire métier dense. Il doit donner une impression de maîtrise, de confiance et de progression.

### Promesse UX

> **Shape your APIs. Ship with confidence.**

L’utilisateur doit comprendre en quelques secondes :

- ce que fait Apigee Forge ;
- pourquoi Google est utilisé pour se connecter ;
- quelle différence existe entre Demo et Live ;
- comment un template devient une révision Apigee ;
- à quel moment une action devient distante et mutative.

---

## 2. Direction artistique

### Style

Le style est **Google-like, calme, simple et légèrement chaleureux**, sans copier Google Material ni les écrans de Google Cloud.

Principes :

- surfaces claires et respirantes ;
- typographie système lisible ;
- hiérarchie discrète mais nette ;
- accent teal propre à Forge ;
- bleu Google réservé aux éléments liés à la connexion et aux repères d’onboarding ;
- contours doux et rayons généreux sur les surfaces principales ;
- peu d’effets décoratifs ;
- aucun effet néon, glassmorphism excessif ou gradient spectaculaire ;
- la personnalité vient du mouvement du flux, pas d’un excès d’ornement.

### Impression recherchée

L’interface doit être :

- **douce** : contrastes maîtrisés et transitions souples ;
- **mellow** : pas de rupture brutale ni de feedback agressif ;
- **sérieuse** : les actions de déploiement restent explicites ;
- **accessible** : textes, statuts et actions restent compréhensibles sans dépendre de la couleur ;
- **vivante** : le scroll et les micro-interactions donnent une sensation de continuité ;
- **rassurante** : l’utilisateur sait toujours ce qui est local, distant, mutatif ou réversible.

---

## 3. Périmètre du design

Le système s’applique à :

1. la page Login / Welcome, qui fonctionne comme une page vitrine intégrée ;
2. l’onboarding Live et Demo ;
3. le shell applicatif authentifié ;
4. Dashboard, Templates, Proxies, Deployments et Settings ;
5. les drawers, modales et états asynchrones ;
6. l’éditeur de templates et le diagramme de flow.

La page Login est la référence d’intention et de ton. Les vues authentifiées réutilisent ses tokens, ses proportions, ses états et sa grammaire de mouvement, mais restent plus compactes.

---

## 4. Tokens visuels

### 4.1 Couleurs de surface

| Token | Valeur | Usage |
|---|---|---|
| `--page-bg` | `#FAFAF9` | Fond général de l’application |
| `--sidebar-bg` | `#F1F3F2` | Navigation latérale |
| `--card-bg` | `#FFFFFF` | Cartes et panneaux |
| `--surface-tint` | `#F8FBFF` | Teinte froide très légère des zones d’introduction |
| `--border` | `#E2E5E3` | Bordures fines et séparateurs |
| `--connector` | `#C4C9C6` | Connecteurs du workflow |

### 4.2 Couleurs fonctionnelles

| Token | Valeur | Usage |
|---|---|---|
| `--accent-active` | `#0F6E56` | Action Forge, navigation active |
| `--accent-text` | `#085041` | Texte sur fond teal clair |
| `--accent-bg` | `#E1F5EE` | Chips et surfaces teal |
| `--google-blue` | `#1A73E8` | Connexion Google et onboarding |
| `--google-blue-soft` | `#E8F0FE` | Repères Google, illustration Login |
| `--google-green` | `#188038` | Succès et état prêt |
| `--google-yellow` | `#F9AB00` | Attention non bloquante |
| `--google-red` | `#D93025` | Erreur et danger |

Le texte sur `--accent-bg` doit utiliser `--accent-text`, jamais du noir ou du gris. Les statuts ne doivent jamais être communiqués par la couleur seule.

### 4.3 Typographie

- police système sans-serif : `ui-sans-serif`, `system-ui`, `Segoe UI` ;
- aucun chargement de police externe pour le MVP ;
- graisse normale : `400` ;
- graisse active : `500` ;
- `600` et `700` réservés aux exceptions de marque très localisées ;
- titres Login : fluides mais contenus, généralement `36–52px` ;
- titres de sections : `30–42px` ;
- titres de cartes : `16–20px` ;
- corps : `13–16px` ;
- hints : `10–12px` ;
- hauteur de ligne confortable : `1.45–1.65`.

La typographie doit rester sobre : pas de capitales permanentes pour les titres et pas de texte compressé dans les blocs importants.

### 4.4 Formes et profondeur

- petites cartes applicatives : rayon `8–12px` ;
- grandes surfaces Login : rayon `20–24px` ;
- boutons principaux et badges : forme pilule ;
- champs : rayon `10–12px` ;
- bordure fine : `0.5–1px solid var(--border)` ;
- ombre : légère et fonctionnelle uniquement sur les grandes surfaces ou pour signifier l’élévation ;
- pas d’ombre décorative sur la sidebar, les séparateurs ou les cartes secondaires.

### 4.5 Espacement

Échelle de base : `4 / 8 / 12 / 16 / 24 / 32 / 48 / 72 / 96px`.

- contenu applicatif : `30–36px` sur desktop ;
- largeur de lecture Login : maximum `960px` ;
- largeur hero Login : maximum `960px` ;
- espace entre sections vitrine : `72–120px` ;
- espace entre cartes : `12–16px` ;
- espace intérieur d’une carte compacte : `16px` ;
- espace intérieur d’un hero : `30–40px`.

---

## 5. Architecture de la page Login

La page Login est scrollable et suit une narration progressive.

```text
Header minimal
    ↓
Hero + connexion Google
    ↓
Explication du produit
    ↓
Fonctionnalités principales
    ↓
Core partagé + CLI + GUI
    ↓
Parcours de livraison
    ↓
Sécurité et confiance
    ↓
Choix Live / Demo + CTA final
```

### 5.1 Header

Le header Login doit rester minimal :

- logo Apigee Forge ;
- sélecteur Live / Demo ;
- aucun menu applicatif complet ;
- aucun bouton secondaire inutile ;
- pas de sidebar avant authentification.

### 5.2 Hero

Le hero doit répondre à trois questions :

1. qu’est-ce que Forge apporte ?
2. quelle est l’action principale ?
3. que se passe-t-il après la connexion ?

Éléments obligatoires :

- titre court et mémorisable ;
- description en deux ou trois lignes ;
- bouton `Sign in with Google` ;
- lien vers la section explicative ;
- note sur la restauration automatique de session ;
- illustration du workflow, sans image distante obligatoire.

### 5.3 Contenu vitrine

Le contenu doit présenter le produit par bénéfices, pas par détails d’implémentation uniquement :

- **Compose** : templates visuels et policies de gouvernance réutilisables ;
- **Prepare** : OpenAPI, preview, nommage et génération locale ;
- **Deliver** : import de révision, confirmation, déploiement et statut ;
- **One core, two experiences** : même logique métier dans GUI et CLI ;
- **Built for teams** : versionnable, scriptable, compatible CI/CD ;
- **Safe by default** : séparation Demo/Live, validation, keyring, SQLCipher et confirmations.

### 5.4 Illustrations

Les illustrations doivent être :

- en CSS ou SVG inline lorsque possible ;
- lisibles sans couleur ;
- légères et non indispensables à la compréhension ;
- animées uniquement pour clarifier le flux ;
- sans dépendance à un service distant.

---

## 6. Motion design

### 6.1 Philosophie

Le mouvement représente le chemin d’une API vers sa livraison. Il ne doit jamais distraire ou ralentir une action.

> **Motion explains the flow. It does not decorate the flow.**

### 6.2 Niveaux d’animation

#### Niveau 1 — CSS direct

À utiliser pour :

- hover ;
- focus ;
- active ;
- boutons ;
- badges ;
- champs ;
- petites apparitions.

Durée recommandée : `120–220ms`.

#### Niveau 2 — Entrée des sections au scroll

Les sections Login apparaissent lorsqu’elles entrent dans la fenêtre avec `animation-timeline: view()` lorsque disponible.

Directions autorisées :

- bas vers haut pour les sections centrales ;
- gauche vers centre pour les introductions ;
- droite vers centre pour les cartes alternatives ;
- jamais d’entrée diagonale excessive ou de rotation importante.

Durée visuelle cible : `400–650ms`.

#### Niveau 3 — Workflow et SVG

Les connecteurs peuvent utiliser un léger déplacement de dash ou une pulsation lente. Les cartes du workflow peuvent flotter très légèrement, mais jamais en boucle agressive.

### 6.3 Accessibilité du mouvement

Chaque animation doit fonctionner avec :

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    scroll-behavior: auto !important;
    transition-duration: 0.01ms !important;
  }
}
```

Le contenu ne doit jamais être masqué en l’absence de support de `animation-timeline`.

### 6.4 Performance

- animer prioritairement `opacity` et `transform` ;
- éviter les animations permanentes hors illustration Login ;
- éviter les recalculs JavaScript au scroll lorsque CSS suffit ;
- ne pas introduire une librairie d’animation sans besoin démontré ;
- mesurer avant d’ajouter Anime.js ou une solution équivalente.

---

## 7. UX d’authentification et onboarding

### Live

```text
Ouverture → restauration de session → identité Google → organisation → environnement → workspace
```

États visibles :

- `Preparing your workspace` ;
- `Restoring your Google workspace` ;
- `Connected` ;
- `Choose your organization` ;
- `Choose your environment` ;
- `Sign in again` si le refresh token n’est plus valide.

Le refresh token reste dans le keyring OS côté Rust. L’access token reste en mémoire et n’est jamais affiché ni persisté dans le frontend.

### Demo

Le mode Demo doit être présenté comme un choix volontaire :

- aucune connexion Google ;
- aucune requête réseau ;
- données explicitement locales ;
- badge Demo toujours visible ;
- possibilité de revenir à Live.

### Erreurs

Une erreur doit expliquer :

1. ce qui s’est passé ;
2. si l’utilisateur peut réessayer ;
3. si l’action a eu lieu ou non ;
4. quelle étape est concernée.

---

## 8. Grammaire des actions

Les libellés doivent conserver les distinctions suivantes :

| Action | Signification |
|---|---|
| `Create template` | Crée une définition locale réutilisable |
| `Create proxy` | Prépare un nouveau parcours proxy |
| `Generate bundle` | Génère localement sans mutation Apigee |
| `Upload and create proxy` | Crée un proxy ou une révision dans Apigee |
| `Review deployment` | Prépare la confirmation de déploiement |
| `Deploy revision` | Déclenche une mutation sur l’environnement ciblé |
| `Check status` | Lit l’état distant sans mutation |

Les actions mutatives doivent rester plus visibles que les actions de lecture et demander une confirmation lorsque le workflow l’exige.

---

## 9. Responsive et accessibilité

- largeur minimale historique du desktop : `960px` ;
- aucune information essentielle ne doit dépendre d’un hover ;
- focus visible sur chaque élément interactif ;
- labels explicites sur les champs ;
- `aria-live` pour les états asynchrones ;
- `role="alert"` pour les erreurs bloquantes ;
- contraste WCAG AA minimum ;
- pas de couleur seule pour distinguer un statut ;
- le scroll Login doit rester naturel et ne pas être enfermé dans plusieurs conteneurs concurrents ;
- les liens d’ancrage doivent fonctionner au clavier ;
- les illustrations ont un texte alternatif ou `aria-hidden` si elles sont décoratives.

---

## 10. Ce qui est interdit

- ajouter un dark mode sans décision explicite ;
- réintroduire une interface dense de type formulaire sur la page Login ;
- utiliser un gradient comme information fonctionnelle ;
- cacher une erreur dans une animation ;
- animer les déploiements de manière ludique ou trompeuse ;
- présenter un template comme une ressource directement déployée ;
- afficher un access token, un refresh token, un chemin local ou un header HTTP ;
- ajouter une dépendance d’animation lourde sans mesure ni justification ;
- modifier les contrats Tauri uniquement pour obtenir un effet visuel.

---

## 11. Critères de validation design

Une évolution est conforme lorsque :

- elle conserve la lisibilité et la sobriété Google-like ;
- elle renforce la compréhension du flux produit ;
- elle utilise les tokens existants ou les documente ici ;
- elle fonctionne à 960px et à 1200px ;
- elle reste utilisable au clavier ;
- elle respecte `prefers-reduced-motion` ;
- elle ne révèle aucun secret ;
- elle ne confond jamais génération, upload et déploiement ;
- elle conserve les tests et le comportement Demo/Live ;
- elle possède une justification UX, et pas seulement décorative.

---

## 12. Sources de vérité liées

- `DOC_PROJECT/DESIGN.md` : tokens et conventions historiques de l’application ;
- `DOC_PROJECT/PROJECT.md` : positionnement et objectifs ;
- `DOC_PROJECT/MVP_FEATURES.md` : périmètre fonctionnel ;
- `DOC_PROJECT/M8_STARTUP_ROADMAP.md` : workflows de création et déploiement GUI ;
- `DOC_PROJECT/M9_STARTUP_ROADMAP.md` : polish visuel et accessibilité ;
- `gui/src/App.vue` : composition actuelle des vues ;
- `gui/src/style.css` : implémentation des tokens et composants visuels.
