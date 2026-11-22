# Décisions et questions ouvertes

## Décisions actées
- Application métier unique, sans dépendance à des logiciels métiers externes.
- Rust backend, React frontend, PostgreSQL référentiel transactionnel.
- Monolithe modulaire, API REST, bibliothèques open source maintenues.
- OMM = Ordre de Modification Matériel ; FAP = Fiche d'Anomalies de Production.
- Pointage sur OF : démarrer/terminer ; chronomètre individuel ; simultanéité multi-opérateurs.
- Développement incrémental et tests ciblés ; intégrité non négociable.

## Hypothèses initiales, à confirmer
- Un opérateur ne peut avoir qu'une session active à la fois.
- L'interface est en français ; temps stockés en UTC.
- Première installation sur serveur Linux avec Docker Compose.
- Jeu de données de démonstration fictif ; pas de données industrielles réelles dans les tests.

## Décisions différées
Authentification SSO ou locale ; rôles précis ; workflow exact OMM/FAP ; règles ECSS applicables ; identification des équipements et révisions ; politiques de rétention et signatures ; hébergement et sauvegardes ; interfaces scanner et matériel d'atelier ; niveau de confidentialité.
