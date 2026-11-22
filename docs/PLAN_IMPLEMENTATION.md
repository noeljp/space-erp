# Plan de développement rapide

## Lot 0 — Socle exécutable
Monorepo, Compose PostgreSQL/backend/frontend, variables d'environnement, migrations SQLx, endpoint santé, gestion d'erreurs, logging, documentation d'exécution, configuration de démonstration. **Acceptation :** démarrage reproductible sur machine vierge équipée de Docker.

## Lot 1 — OF et pointage Suivat intégré
Utilisateurs/rôles de démonstration, création/consultation OF, démarrage/arrêt individuel, plusieurs techniciens sur un même OF, total heures, historique et audit, vue opérateur et superviseur. **Acceptation :** scénario complet réellement démontrable ; refus de deux sessions actives pour le même opérateur même sous concurrence.

## Lot 2 — Articles, révisions, eBOM/mBOM et gammes
Référentiel produits, nomenclatures multi-niveaux, révisions, instructions de travail, opérations d'OF. **Acceptation :** un OF référence une configuration produit versionnée.

## Lot 3 — FAP / OMM / qualité
Création, liens OF/équipement, états, rôles, décisions, approbations et audit ; modification matérielle avec trace des composants. **Acceptation :** anomalie → disposition → éventuel OMM → vérification.

## Lot 4 — Achats, stocks et MRP
Réception, lots, statuts qualité, réservations, mouvements, explosion mBOM, besoins nets datés, propositions d'achat/fabrication. **Acceptation :** stock bloqué exclu de la disponibilité ; aucune surconsommation concurrente.

## Lot 5 — MES et As-Built
Exécution de gamme, contrôles, enregistrements, généalogie sérialisée, dossier de fabrication et livraison. **Acceptation :** reconstitution de l'historique d'un équipement et de ses modifications.

## Lot 6 — Planification et pilotage avancés
Capacité, charges, coûts, KPI, maintenance et rétrofit selon besoins validés.

## Vérifications minimales à chaque lot
`cargo fmt --check`, `cargo check`, `cargo test` ciblé, `tsc --noEmit`, build frontend, migration sur base vierge, smoke test des routes modifiées. Tests de concurrence et intégrité seulement pour les scénarios qui en ont besoin. Avant mise en production : campagne renforcée sécurité, restauration, droits et traçabilité.

## Critère de sortie
Une fonctionnalité n'est « terminée » que si l'UI appelle réellement l'API, l'API persiste les données, les contraintes critiques sont en base, et un scénario métier est vérifié.
