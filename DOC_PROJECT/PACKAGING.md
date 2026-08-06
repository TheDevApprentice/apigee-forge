# Apigee Forge — Packaging et distribution

*Comment le projet s'exporte : le CLI seul (package indépendant) et le GUI+CLI ensemble (installeur desktop Windows/Mac). Référencé depuis ARCHITECTURE.md section 13.*

---

## 1. Pourquoi c'est possible sans effort de refonte

`cli/` ne dépend que de `core/` — jamais de `gui/` (voir ARCHITECTURE.md section 2). Cette règle de dépendance, posée dès le départ, est ce qui rend les deux exports possibles sans code séparé à maintenir : le CLI se compile et se distribue seul, tel quel.

---

## 2. Export 1 — le CLI seul

### Cibles de compilation
- `x86_64-pc-windows-msvc` (Windows)
- `x86_64-apple-darwin` (Mac Intel)
- `aarch64-apple-darwin` (Mac Apple Silicon)
- `x86_64-unknown-linux-gnu` — **indispensable**, même si le GUI ne vise que Windows/Mac : les runners GitHub Actions/GitLab CI/Azure Pipelines par défaut tournent sous Linux, c'est la cible réellement utilisée en pipeline.

### Process de release
Un seul repo, un seul workflow GitHub Actions déclenché sur un tag de version (`vX.Y.Z`) :
- Matrice de build : `windows-latest`, `macos-latest` (avec les deux targets Mac ajoutées via `rustup target add`), `ubuntu-latest`
- Chaque job compile `cargo build --release -p cli --target <cible>`
- Chaque binaire est renommé clairement (ex. `apigee-forge-cli-x86_64-pc-windows-msvc.exe`) et publié comme asset séparé sur une **GitHub Release** unique correspondant au tag
- Résultat pour l'utilisateur : il télécharge uniquement le binaire CLI correspondant à son OS, sans rien d'autre

---

## 3. Export 2 — GUI + CLI ensemble

### Bundler Tauri
`tauri build` produit nativement :
- macOS : `.dmg` contenant un `.app`
- Windows : `.msi` ou `.exe` (NSIS) — NSIS est le défaut Tauri le plus léger

### Inclure le CLI dans le bundle GUI — mécanisme "sidecar"
Tauri permet d'embarquer un binaire externe dans le bundle de l'application via `bundle.externalBin` dans `tauri.conf.json`. Concrètement : le binaire CLI compilé est copié dans les ressources de l'app au moment du build, donc **installer le GUI installe aussi physiquement le CLI**, accessible depuis le dossier de ressources de l'app (chemin exact documenté dans le README utilisateur final, à ajouter au jalon M10). Ajout au PATH système : optionnel, à proposer à l'installation plutôt qu'imposé.

### Même workflow de release
Le même tag de version déclenche, en plus des builds CLI, les jobs `tauri build` par OS (`windows-latest`, `macos-latest`), dont les installeurs sont publiés comme assets supplémentaires sur la **même GitHub Release**. Un seul cycle de release, deux catégories d'assets.

---

## 4. Point d'attention — signature de code

Sans certificat de signature payant (~99 USD/an pour Apple Developer, ~200-400 USD/an pour un certificat Windows), les installeurs déclenchent des avertissements du système :
- **macOS** : Gatekeeper affiche "développeur non identifié" — contournement utilisateur : clic droit → Ouvrir
- **Windows** : SmartScreen affiche un avertissement — contournement utilisateur : "Plus d'infos" → "Exécuter quand même"

C'est un comportement attendu et acceptable pour un projet portfolio/open source sans budget de certificat — à documenter clairement dans le README plutôt qu'à essayer de le masquer. Peut être reconsidéré plus tard si le projet gagne en traction.

---

## 5. Versioning

Un seul numéro de version au niveau du workspace Cargo, tenu synchronisé entre `core`, `cli`, et `gui`. Un tag Git `vX.Y.Z` déclenche l'unique workflow de release décrit ci-dessus, produisant en une seule fois tous les assets (CLI × 4 cibles, installeurs GUI × 2 OS).
