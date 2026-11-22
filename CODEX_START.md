# Prompt à transmettre à Codex

Tu es l'ingénieur principal chargé de créer SPACE ERP à partir de ce dépôt. Lis d'abord AGENTS.md et tous les documents de docs/. Ne te limite pas à produire une architecture ou des maquettes : implémente le **lot 0 et le lot 1** du plan, avec application démarrable sous Docker Compose.

Exigences prioritaires : Rust/Axum/SQLx/PostgreSQL, React/TypeScript ; base PostgreSQL avec migrations ; utilisateurs fictifs et rôles ; OF et sessions de pointage individuelles ; plusieurs opérateurs sur le même OF ; interdiction d'une double session active pour un même opérateur garantie par index SQL ; démarrage/arrêt horodatés côté serveur ; total des heures par OF ; audit ; frontend réellement relié à l'API. Authentification de démonstration sécurisée à remplacer avant production, sans identifiants hardcodés dans le code source.

Commence par inspecter le dépôt. Propose brièvement le plan, puis exécute-le. Réutilise des bibliothèques open source maintenues ; vérifie leurs licences et évite les dépendances superflues. Effectue uniquement les vérifications ciblées et le smoke test requis. Si une fonctionnalité est reportée, marque-la explicitement comme non implémentée. À la fin, donne les commandes exactes pour démarrer, initialiser et tester.
