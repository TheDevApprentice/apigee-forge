# Apigee Forge — Design

*Tokens visuels validés. Thème clair uniquement — pas de mode sombre pour ce projet.*

> La référence complète du design system GUI, de la page Login vitrine, du motion design et des règles UX est désormais `DOC_PROJECT/GUI_DESIGN_SYSTEM.md`. Ce document conserve les tokens historiques et doit rester cohérent avec cette référence complète.

---

## 1. Direction générale

Sobriété inspirée de Google Cloud Console (structure, densité d'information, navigation latérale), avec une patte propre : accent **teal** (plutôt que le bleu Google), cartes à coins arrondis avec bordure fine plutôt qu'ombre, et le **diagramme de flux de proxy** (PreFlow → Flow conditionnel → PostFlow) comme élément visuel signature de l'outil.

---

## 2. Couleurs

### Surfaces
| Rôle | Hex | Usage |
|---|---|---|
| Fond de page | `#FAFAF9` | Arrière-plan général de la fenêtre |
| Fond sidebar | `#F1F3F2` | Navigation latérale |
| Fond carte | `#FFFFFF` | Cartes de flow, panneaux |
| Bordure par défaut | `#E2E5E3` | Toutes les bordures fines (0.5px) |
| Bordure séparateur | `#E2E5E3` | Barres de séparation (top bar, sidebar) |

### Texte
| Rôle | Hex | Usage |
|---|---|---|
| Texte principal | `#1C2420` | Titres de cartes, labels actifs |
| Texte secondaire | `#5B635F` | Sous-titres, contexte (org/env) |
| Texte muted | `#8C948F` | Hints, labels de section, icônes inactives |

### Accent teal (chips, actions, icône active)
| Rôle | Hex | Usage |
|---|---|---|
| Fond chip accent | `#E1F5EE` | Fond des badges de policy et de rôle |
| Texte sur fond accent | `#085041` | Texte/icônes sur fond `#E1F5EE` — jamais de noir sur ce fond |
| Icône active (sidebar) | `#0F6E56` | Icône de navigation sur l'écran actif |

**Règle stricte** : toujours utiliser `#085041` (jamais noir ni gris) comme couleur de texte sur un fond `#E1F5EE`. C'est la seule paire fond/texte accent du projet — ne pas en introduire d'autres sans mettre à jour ce document.

### Neutres additionnels
| Rôle | Hex | Usage |
|---|---|---|
| Icône inactive | `#8C948F` | Icônes de navigation non sélectionnées |
| Flèche/connecteur | `#C4C9C6` | Flèches entre les étapes du flow |

---

## 3. Typographie

- Police système sans-serif (celle de l'OS via Tauri — pas de police custom embarquée pour le MVP)
- Titres de carte : 12px, weight 500
- Corps / labels de contexte : 13px, weight 400
- Sous-labels / hints : 10–11px, weight 400
- Deux graisses seulement : 400 (normal) et 500 (semi-bold pour les titres/labels actifs) — jamais de 600/700

---

## 4. Espacement et forme

- Rayon des cartes et de la fenêtre app : `8px`
- Rayon des chips/badges : `20px` (forme pilule)
- Rayon des petits tags de policy inline : `4px`
- Bordures : toujours `0.5px solid`, jamais d'ombre portée
- Padding interne des cartes : `10px`
- Padding de la zone de contenu principale : `18px`
- Gap entre éléments de chips : `4–6px`

---

## 5. Iconographie

- Style **outline** uniquement (pas de filled), taille 20px en navigation, 12px dans les chips inline
- Icônes de navigation (sidebar) : dashboard, template, route (proxies), rocket (déploiements), settings
- Icônes de policy : key (API key), shield-check (JWT), arrows-cross (CORS) — une icône cohérente par type de policy, à étendre au même style pour OAuth2/Quota/Spike Arrest/Transform

---

## 6. Patterns de composants

### Sidebar de navigation
Largeur fixe 56px, icônes seules (pas de label texte en MVP), fond `#F1F3F2`, icône de l'écran actif en teal (`#0F6E56`), icônes inactives en `#8C948F`.

### Barre supérieure
Contexte organisation/environnement à gauche (`#5B635F`), badge de rôle Apigee de l'utilisateur à droite (pilule teal).

### Carte d'étape de flow (PreFlow / Conditional Flow / PostFlow)
Fond blanc, bordure fine `#E2E5E3`, rayon 8px, padding 10px. Titre en `#1C2420` (500), condition de route en `#8C948F` si applicable. Policies attachées affichées en petits tags teal (fond `#E1F5EE`, texte `#085041`, rayon 4px).

### Connecteurs de flow
Flèche simple (`ti-arrow-right`), couleur `#C4C9C6`, entre chaque étape — jamais de ligne décorative complexe.

### Palette de policies (ajout)
Chips en pilule (rayon 20px), fond blanc, bordure fine, icône + label, disposées en wrap horizontal sous le diagramme de flow.

---

## 7. Ce qui reste à définir plus tard (hors scope de ce document)

- États d'erreur / validation (couleur danger, affichage des erreurs de policy invalide)
- États de chargement / déploiement en cours
- Responsive / redimensionnement de la fenêtre desktop
- Dark mode : explicitement écarté pour ce projet, à ne pas ajouter sans revalidation
