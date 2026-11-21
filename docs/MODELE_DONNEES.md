# Modèle de données initial et règles d'intégrité

## Entités de départ
`core.users(id, username, display_name, password_hash, active)` ; `core.roles` ; `core.user_roles` ; `core.audit_events(id, actor_id, action, entity_type, entity_id, occurred_at, payload)`.

`engineering.items(id, part_number, description)` ; `engineering.item_revisions(id, item_id, revision, status)` ; `engineering.bom_headers` ; `engineering.bom_lines(parent_revision_id, child_revision_id, quantity, effectivity)`.

`manufacturing.work_orders(id, number, item_revision_id, serial_number, quantity, status, created_at)` ; `manufacturing.work_sessions(id, operator_id, work_order_id, started_at, ended_at, created_at)`.

Évolutions : `manufacturing.operations`, `quality.production_anomalies`, `change_control.material_change_orders`, `inventory.stock_movements`, `traceability.installed_components` et tables de documents/versionnement.

## Règles indispensables
- PK UUID ou bigint cohérentes ; FK et contraintes NOT NULL ; numéros d'OF uniques.
- `ended_at IS NULL OR ended_at > started_at`.
- Index unique partiel sur `operator_id WHERE ended_at IS NULL`.
- Les sessions terminées sont conservées ; corrections par mécanisme tracé, pas d'écrasement silencieux.
- Plusieurs opérateurs peuvent avoir une session active sur le même OF.
- Début/fin horodatés côté serveur ; UTC en stockage.
- Le calcul des heures est la somme des durées individuelles, pas l'intervalle min/max des sessions.
- Le pointage terminé n'entraîne pas la clôture de l'OF.
- Démarrage seulement sur OF dans un état autorisé ; contrôler les droits de l'opérateur.
- Prévenir les doubles clics/requêtes répétées par contrainte SQL et réponses d'API idempotentes quand approprié.

## Exemple de contrainte
```sql
CREATE UNIQUE INDEX one_active_session_per_operator
ON manufacturing.work_sessions (operator_id)
WHERE ended_at IS NULL;
```

## Points à préciser avant modules avancés
Numérotation multi-site ; effectivités série/date ; règles de statut qualité ; variantes de nomenclature ; approbations OMM/FAP ; gestion des pauses et pointages rétroactifs ; unités de mesure ; devise et taux horaires ; règles d'accès aux coûts.
