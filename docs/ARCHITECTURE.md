# Architecture technique et fonctionnelle

## Modules cibles
- `core` : utilisateurs, rôles, habilitations, organisations, audit, référentiels, documents, signatures (future).
- `engineering` : projets, articles, révisions, eBOM, configuration de conception, effectivités.
- `methods` : mBOM, gammes, opérations, instructions de travail, moyens et qualifications.
- `planning` : MRP, ordres planifiés, capacité, dates et alertes.
- `procurement` : fournisseurs, demandes d'achat, commandes, réceptions.
- `inventory` : emplacements, lots, numéros de série, statuts qualité, réservations et mouvements.
- `manufacturing` : OF, suivi MES, exécution, pointages par opérateur.
- `quality` : FAP, inspection, dispositions, dérogations, points d'arrêt, libérations.
- `change_control` : OMM, analyse d'impact, autorisations, interventions et vérification.
- `traceability` : généalogie, As-Designed/As-Planned/As-Built, dossier de livraison.
- `analytics` : coûts, charges, retards, non-qualité, tableaux de bord.
- `service` : maintenance, retours, réparations et rétrofits (phase ultérieure).

## Frontières
Frontend → API `/api/v1` → services Rust → SQLx → PostgreSQL. Pas de logique d'autorisation uniquement en UI. Un module ne modifie pas directement les tables privées d'un autre sans contrat de service explicite. Pas de duplication des identifiants maîtres.

## Intégrité et fiabilité
- Contraintes SQL pour invariants locaux ; transactions et verrouillage appropriés pour stocks et opérations concurrentes.
- Transitions de workflow contrôlées par services ; journalisation des décisions et acteur.
- Historique des révisions et de l'applicabilité ; jamais écraser un enregistrement validé sans trace.
- Stockage documentaire avec empreinte SHA-256, version, lien métier et politique de rétention.
- Sauvegardes, restauration vérifiée, stratégie de migration et surveillance avant production.

## Interface par métier
Opérateur : actions tactiles simples, scanner OF, démarrer/arrêter, consignes et anomalies. Méthodes : gammes, documents, OMM. Qualité : FAP, décisions, contrôles. Planning : MRP, manquants et charges. Direction : indicateurs. Composants UI communs, pages métiers spécialisées.

## Sécurité
RBAC initial, journal d'audit, TLS en déploiement, mots de passe hachés via bibliothèque éprouvée, protection CSRF selon stratégie de session, limites de requêtes, secrets par environnement. Les signatures réglementées et l'immutabilité forte sont des sujets de conception/validation séparés.

## Choix de compacité
Monolithe modulaire et workers Rust si nécessaires ; pas de bus externe par défaut. PostgreSQL comme ancrage transactionnel, non comme lieu de tous les algorithmes métier.
