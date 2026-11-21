# Instructions permanentes pour Codex — SPACE ERP

## Mission
Livrer rapidement des incréments verticaux fonctionnels d'un ERP/MRP/MES spatial autonome, maintenable et robuste. Favoriser la simplicité du code, la réutilisation des bibliothèques éprouvées et l'intégrité PostgreSQL.

## Discipline de développement
1. Lire les documents dans `docs/` avant toute modification structurante ; conserver les décisions déjà validées.
2. Ne jamais réécrire un module entier si une modification locale suffit. Éviter le code dupliqué et les abstractions spéculatives.
3. Livrer à chaque incrément : schéma/migration, API, interface, documentation courte et vérification du scénario métier concerné.
4. Garder les modules métier séparés ; leurs interactions passent par des services explicites, pas par des écritures arbitraires dans leurs tables.
5. PostgreSQL protège les clés, relations, unicités, contraintes et opérations atomiques ; Rust protège les règles métiers, droits, transitions et orchestration.
6. Aucun trigger complexe sans justification écrite. Aucune suppression silencieuse de données industrielles approuvées.
7. SQLx avec requêtes paramétrées, transactions explicites, migrations versionnées ; ne jamais interpoler des entrées utilisateur dans du SQL.
8. Authentification serveur, permissions vérifiées dans les handlers/services, audit des changements sensibles ; aucune confiance dans les contrôles frontend.
9. Utiliser des dépendances open source stables et justifiées ; vérifier licence, activité, vulnérabilités, maintenance et compatibilité avant adoption. Éviter deux bibliothèques pour le même besoin.
10. Garder une interface commune, mais des vues spécialisées pour opérateur, méthodes, qualité, planning et direction ; accessibilité et écrans utilisables sur tablette d'atelier.
11. Ne pas ajouter de microservices, Kafka, Redis ou moteur BPM tant qu'un besoin mesuré ne le justifie.
12. Ne pas affirmer qu'une fonctionnalité est opérationnelle si elle n'est pas réellement exécutée ; signaler explicitement les placeholders.

## Politique de tests ciblés
- À chaque changement : formatage, compilation/typecheck et tests liés aux modules touchés.
- À chaque migration : migration sur base vierge et vérification des contraintes impactées.
- À chaque scénario critique : 1 à 3 tests d'intégration couvrant le chemin nominal, une erreur et une concurrence si pertinente.
- Avant livraison : smoke test API + frontend + PostgreSQL sous Compose.
- Pas d'objectif arbitraire de couverture à 100 %, pas de batterie E2E géante à chaque commit.
- Ne jamais supprimer un test de sécurité/intégrité simplement pour accélérer un build.

## Premier parcours métier
Créer OF → opérateurs A/B démarrent des sessions simultanées sur le même OF → A/B terminent → heures individuelles et cumulées exactes → trace d'audit → aucune clôture automatique de l'OF. Une double session active pour le même opérateur doit être rejetée, y compris en concurrence.

## Critères de qualité
Code explicite, modules courts, erreurs structurées, logs sans secrets, temps serveur en UTC, affichage localisé, migrations reproductibles, versions épinglées, .env.example sans secret, sauvegarde/restauration documentées.

## Réponse attendue de Codex après chaque lot
Résumé de ce qui fonctionne ; fichiers changés ; commandes de lancement ; vérifications réellement exécutées ; limites et décisions ouvertes ; prochaine étape. Ne pas réclamer de confirmation pour chaque détail mineur.
