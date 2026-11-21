# Démarrer SPACE ERP

Le lot 0 fournit le socle Compose. Le lot 1 couvre les ordres de fabrication et le pointage individuel. Les spécifications initiales restent à la racine avec les instructions pour Codex.

## Démarrage de la démonstration

Depuis la racine, avec Docker Compose et OpenSSL installés.

```sh
./scripts/init-demo.sh
docker compose up --build -d
docker compose logs -f api
```

Ouvrir http://127.0.0.1:8080. La migration SQLx et la création des comptes s’exécutent au démarrage de l’API. Les identifiants sont `superviseur`, `operateur-a` et `operateur-b`. Les mots de passe générés figurent dans `.env` sous `SUPERVISOR_PASSWORD`, `OPERATOR_A_PASSWORD` et `OPERATOR_B_PASSWORD`. Conserver ce fichier hors de Git. Modifier ses valeurs après initialisation ne change pas les mots de passe enregistrés.

## Parcours atelier

Le superviseur crée une révision dans **Articles et révisions**, puis la sélectionne en créant un OF avec un numéro unique, une description et une quantité. Les OF antérieurs à cette migration restent consultables et utilisables pour le pointage. Les opérateurs se connectent dans deux navigateurs ou fenêtres privées distinctes, puis démarrent sur le même OF. Chacun termine sa propre session. Le superviseur clique sur **Actualiser** pour consulter les durées, le total des pointages terminés et l’audit. L’OF reste ouvert.

## API de démonstration

Toutes les routes portent le préfixe `/api/v1`. Après connexion, transmettre le jeton d’une durée de huit heures dans l’en-tête `Authorization: Bearer <jeton>`.

| Route | Usage |
|---|---|
| `GET /health` | Disponibilité de l’API et de PostgreSQL |
| `POST /login` | Connexion avec `username` et `password` |
| `GET /me`, `POST /logout` | Profil et révocation du jeton |
| `GET /item-revisions`, `POST /item-revisions` | Liste des révisions. Création par le superviseur avec `part_number`, `revision` et `description` |
| `GET /orders`, `POST /orders` | Liste des OF. Création par le superviseur avec `number`, `description`, `quantity` et `item_revision_id`. La liste expose `part_number` et `revision` |
| `GET /sessions` | Pointages propres à l’opérateur ou ensemble pour le superviseur |
| `POST /orders/{id}/sessions` | Démarrer sur un OF ouvert. Réponse 409 si l’opérateur pointe déjà |
| `POST /sessions/{id}/stop` | Terminer son propre pointage. Un nouvel arrêt de la même session réussit sans nouvel audit |
| `GET /audit` | Derniers événements, accessibles au superviseur |

## Vérifications

```sh
(cd backend && cargo fmt --check && cargo check --locked && cargo test --locked)
(cd frontend && npm ci && npm run build)
python3 scripts/smoke.py
```

Le test crée un OF fictif dans l’instance démarrée. Il contrôle la création des révisions, le rejet des doublons et des révisions inconnues, la configuration de l’OF, les rôles, deux démarrages concurrents du même opérateur, la simultanéité entre opérateurs, l’arrêt, le total et l’audit. Les procédures d’essai nécessitent uniquement des comptes de démonstration.

## Sauvegarde et restauration

```sh
docker compose exec -T db pg_dump -U space_erp -Fc space_erp > space-erp.dump
```

Restaurer dans une base vide distincte et vérifier le contenu avant toute bascule.

```sh
docker compose exec -T db createdb -U space_erp space_erp_restauration
docker compose exec -T db pg_restore --exit-on-error -U space_erp -d space_erp_restauration < space-erp.dump
```

## Périmètre

Démonstration locale avec comptes fictifs, mots de passe Argon2id et jetons expirants. Le déploiement industriel, le SSO, la gestion des comptes, les corrections de pointage, la clôture d’OF, les nomenclatures et les gammes, le MRP, les FAP et les OMM restent à développer. Le frontal conserve le jeton en mémoire. Un rechargement de page demande une reconnexion et les pointages restent enregistrés.
