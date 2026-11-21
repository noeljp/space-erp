# space-erp

## SPACE ERP — Dossier de lancement Codex

**Statut :** lots 0 et 1 implémentés pour la démonstration. Le lot 2 couvre déjà les révisions d’article et leur association aux OF. **Langue produit :** français. **Domaine :** fabrication d'équipements électroniques spatiaux.

Voir le [guide de démarrage](docs/DEMARRAGE.md) pour lancer et tester le pointage.

## Objectif
Construire une plateforme métier autonome et unifiée, sans dépendance à SAP, Jira, PALMA, Windchill, Suivat ou à un MES externe. Les bibliothèques techniques open source sont autorisées, après vérification de leurs licences et de leur maintenance.

## Stack cible
- Backend : Rust stable, Axum, Tokio, SQLx, Serde, tracing, utoipa.
- Frontend : React, TypeScript strict, Vite, shadcn/ui, TanStack Table et Query, React Hook Form, Zod.
- Données : PostgreSQL, migrations SQLx ; documents sur volume local avec métadonnées et empreintes en base.
- Déploiement : Docker Compose, Linux, configuration par variables d'environnement.
- Architecture : monolithe modulaire, une API versionnée `/api/v1`, aucun accès direct du navigateur à PostgreSQL.

## Démarrage avec Codex
1. Lire `AGENTS.md`, `docs/ARCHITECTURE.md`, `docs/MODELE_DONNEES.md` et `docs/PLAN_IMPLEMENTATION.md`.
2. Exécuter le prompt de `CODEX_START.md`.
3. Construire un premier incrément réellement utilisable : utilisateurs de démonstration, OF, sessions de pointage, vue superviseur, journal d'audit.
4. Produire un README d'exécution avec commandes exactes, migrations et limites connues.

## Principes non négociables
- Les règles d'intégrité persistantes sont garanties par PostgreSQL, les workflows par le backend.
- Une session de pointage appartient à un opérateur et à un OF. Plusieurs opérateurs peuvent pointer simultanément sur un même OF ; au plus une session active par opérateur (règle initiale configurable par évolution contrôlée).
- Arrêter un pointage ne clôture pas l'OF.
- OMM = Ordre de Modification Matériel ; FAP = Fiche d'Anomalies de Production.
- La conception, la configuration planifiée et la configuration réellement fabriquée sont distinguées.
- Aucune prétention de conformité ECSS ou de qualification industrielle avant validation formelle.

## Livrables attendus
Code, migrations, compose.yaml, .env.example, scripts d'initialisation, jeu de démonstration fictif, documentation d'API, vérifications ciblées, guide opérateur minimal.
