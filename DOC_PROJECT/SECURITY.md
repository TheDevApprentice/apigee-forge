# Apigee Forge — Guidelines de cybersécurité

*Normatif au même titre qu'ARCHITECTURE.md. S'applique à tout code généré, CLI comme GUI.*

---

## 1. Principes généraux

- **Moindre privilège** : chaque composant (rôle IAM assigné, permission Tauri, portée d'un token) ne doit avoir accès qu'au strict nécessaire — jamais "large par prudence".
- **Ne jamais faire confiance à une entrée** : toute donnée venant de l'extérieur du processus (fichier OpenAPI, template chargé, réponse HTTP, entrée utilisateur dans le GUI) est non fiable jusqu'à validation explicite.
- **Échec sécurisé** : en cas d'erreur ou d'ambiguïté, le programme doit refuser l'action plutôt que de deviner ou continuer avec un état partiel.

---

## 2. Rust — pratiques obligatoires

- **Pas d'`unsafe`** sans justification écrite en commentaire au-dessus du bloc, expliquant pourquoi c'est nécessaire et quelles invariants sont garantis manuellement. À éviter par défaut dans ce projet — aucun cas d'usage prévu ne devrait en avoir besoin.
- **Aucun `.unwrap()`/`.expect()` dans `core/`** (déjà acté dans ARCHITECTURE.md section 7) — un panic sur une entrée non prévue est une défaillance de disponibilité, pas seulement un problème de style.
- **`Cargo.lock` commité dans le repo** — garantit des versions de dépendances reproductibles, y compris en CI.
- **`cargo audit` en CI** (`.github/workflows/ci.yml`) — scanne les dépendances contre la base de vulnérabilités RustSec à chaque push. Un audit qui échoue bloque le merge.
- **`cargo clippy` avec les lints par défaut activés** en CI — beaucoup de lints clippy attrapent des erreurs qui ont des implications sécurité (comparaisons suspectes, conversions de types risquées).

---

## 3. Secrets et données sensibles

- **Jamais de secret en dur dans le code source**, ni dans un fichier commité (y compris de config d'exemple — utiliser des placeholders explicites du type `<YOUR_TOKEN_HERE>`).
- **Jamais de secret dans les logs**, y compris les logs d'erreur/debug. Vérifier explicitement qu'un message d'erreur qui inclut le contenu d'une requête HTTP échouée ne fait pas fuiter un header d'autorisation.
- **Répartition stricte déjà actée** (ARCHITECTURE.md section 6) : tokens/credentials → trousseau OS (`keyring`) uniquement. Base locale chiffrée (`SqlCipherLocalStore`) → jamais de secret dedans, même chiffrée.
- **Clé de chiffrement SQLCipher** : ne doit jamais être stockée en clair à côté du fichier `.db` qu'elle protège — dérivée depuis le trousseau OS ou depuis une entrée utilisateur, jamais codée en dur.

---

## 4. Réseau et communication avec Apigee

- **TLS toujours vérifié** — ne jamais désactiver la validation de certificat (`danger_accept_invalid_certs` ou équivalent), même temporairement pour déboguer.
- **Timeout explicite sur tout appel HTTP** (`reqwest::Client` configuré avec un timeout) — un appel qui ne répond jamais ne doit pas bloquer indéfiniment un CLI en pipeline.
- **Retry avec backoff raisonnable** sur les erreurs réseau transitoires, jamais de boucle de retry infinie ou agressive qui pourrait ressembler à un abus de l'API Apigee.

---

## 5. Authentification

- **Flux OAuth desktop avec PKCE obligatoire** — protège contre l'interception du code d'autorisation, standard pour les applications desktop qui ne peuvent pas garder un secret client confidentiel.
- **Tokens d'accès de courte durée** — ne jamais persister un access token longue durée ; seul le refresh token (dans le trousseau OS) doit survivre entre les sessions.
- **CLI en pipeline** : résolution unique via `GOOGLE_APPLICATION_CREDENTIALS` (ARCHITECTURE.md section 4) — ne jamais accepter de credentials passés en argument de ligne de commande en clair (visibles dans l'historique shell / les logs de pipeline).

---

## 6. Validation des entrées

- **Spec OpenAPI et templates chargés** : valider strictement contre `schemas/template.schema.json` avant tout traitement — ne jamais faire confiance à un fichier fourni par l'utilisateur ou récupéré depuis un chemin/URL externe.
- **Écriture de fichiers générés (bundle proxy) sur disque** : se prémunir contre la traversée de chemin (path traversal) — un nom de proxy ou de fichier dérivé d'une entrée utilisateur ne doit jamais permettre d'écrire en dehors du répertoire de sortie prévu.
- **Taille des entrées** : borner la taille des fichiers acceptés (spec OpenAPI, template) pour éviter qu'un fichier malveillant ou corrompu ne consomme une mémoire excessive.

---

## 7. Spécifique Tauri (GUI)

- **Content Security Policy stricte** dans `tauri.conf.json` — pas de `unsafe-inline`/`unsafe-eval` sans raison documentée.
- **Devtools désactivés en build de production** — activés uniquement en développement.
- **Allowlist/capabilities Tauri réduite au strict nécessaire** — n'exposer au frontend que les commandes réellement utilisées (`auth`, `templates`, `proxies`, `deployment`), jamais un accès filesystem ou shell large par défaut.
- **Toute donnée reçue du frontend (webview) est validée côté Rust avant usage** — le frontend n'est pas une zone de confiance, même si c'est notre propre code : traiter la frontière IPC Tauri comme une frontière de sécurité réelle.

---

## 8. CI/CD

- **Pas de credentials statiques quand Workload Identity Federation est disponible** (déjà acté, ARCHITECTURE.md section 4).
- **Aucun secret affiché dans les logs de pipeline** — attention particulière aux commandes `--verbose`/`--debug` qui pourraient échapper cette règle par accident.
- **`cargo audit` bloquant** intégré au pipeline CI, pas seulement recommandé en local.
