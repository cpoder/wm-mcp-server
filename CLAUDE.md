# webMethods IS — playbook anti-erreurs (flow services via MCP)

Ce projet pilote un Integration Server via le serveur MCP `webmethods-is`
(binaire `mcp-server-rs/target/release/wm-mcp-server`, cible `http://localhost:5555`).
Objectif de ce fichier : éviter les erreurs 500 récurrentes lors de la
création / modification / suppression de flow services.

## Règle d'or : lire la doc embarquée AVANT d'écrire du JSON

Le serveur MCP expose des **ressources** (`ReadMcpResourceTool`). Avant toute
création de flow non triviale, lis dans l'ordre :

1. `wm://docs/flow-language-reference` — format WmPath, types d'étapes, règles de mapping
2. `wm://docs/putnode-examples` — exemples de JSON `put_node` testés et fonctionnels
3. `wm://docs/builtin-services` — signatures des services `pub.*` que tu vas invoquer
4. `wm://docs/flow-steps-reference` — sémantique INVOKE/BRANCH/LOOP/MAP/SEQUENCE/REPEAT/EXIT
5. `wm://docs/adapter-connection-reference` — connexions adaptateur (JDBC & co.)
6. `wm://docs/adapter-service-reference` — uniquement pour les services adaptateur JDBC

Ne devine JAMAIS un chemin de service IS ni la structure d'un nœud. Si tu n'es
pas sûr d'un format, **lis un nœud existant qui marche** avec `node_get` et
sers-t'en de modèle.

## Modèle mental : `flow_service_create` ≠ un flow utilisable

- `flow_service_create` (et `service_create`) ne crée qu'une **coquille vide**
  (aucune signature, aucune logique). Ce n'est PAS suffisant.
- Le vrai outil est **`put_node`** (API IS `putNode`) : il crée/met à jour le
  service complet (signature `sig_in`/`sig_out` + arbre `flow`). Un seul
  `put_node` suffit : **l'outil crée lui-même la coquille** quand le nœud
  n'existe pas (`serviceAdd` pour un flow, `makeNode` pour un doc type), parce
  que sur IS 12.1 (`watt.server.ns.lockingMode=full`) `putNode` verrouille le
  nœud avant d'écrire et échoue sinon avec `[ISS.0081.9001] Node ... does not
  exist` à `nsimpl.lockNode`. Les **dossiers** parents, eux, doivent exister.
- Après l'écriture, `put_node` **relit le nœud et compare le nombre d'étapes
  par type** avec ce qui a été envoyé (`verification` dans la réponse). Un
  déficit est une erreur : IS supprime en silence toute étape dont il ne
  connaît pas le `type` ou les clés (voir RETRY / `evaluate-labels` plus bas).
  L'arbre est aussi validé AVANT l'envoi : `REPEAT`, `TRY`, `label-expressions`…
  sont refusés avec la bonne orthographe.
- `node_get` renvoie le vrai arbre `flow.nodes` (lecture en XML côté IS) : c'est
  le modèle à copier quand un format est incertain.

## Ordre des opérations (checklist — respecte-la, c'est ce qui évite les 500)

1. **Package** : vérifie qu'il existe, est activé et inscriptible
   (`package_list` / `package_info`). Sinon `package_create`.
2. **Dossiers parents** : crée chaque dossier de l'arborescence AVANT le service
   (`folder_create`, un appel par niveau — les parents ne sont pas créés
   implicitement). Un `put_node` dans un dossier inexistant échoue avec
   `[ISS.0081.9001] Node ... does not exist`.
   **Arborescence** : le namespace IS est **commun à tous les packages**, donc un
   package possède exactement **un dossier racine, son nom en minuscules**, et
   tout le reste est imbriqué dessous :
   `PetstoreAPI` → `petstoreapi`, puis `petstoreapi.api`, `petstoreapi.adapter`.
   Un nom composé se découpe en segments (`WxEdiAddon` → `wx.edi.addon`).
   Créer `api` ou `services` directement à la racine du package est une erreur :
   ces noms génériques entrent en collision avec les autres packages.
3. **Document types** : si ta signature ou tes mappings utilisent des champs
   RecordRef (type 4) ou des doc types, **crée-les d'abord**
   (`document_type_create` puis `put_node` pour les champs). Référencer un doc
   type inexistant fait planter le compilateur de flow → 500.
4. **Service** : `put_node` avec le `node_data` complet (voir règles ci-dessous).
5. **Vérifie** : la réponse de `put_node` contient `verification.status = ok`
   (comptage des étapes relues) ; puis `service_invoke` avec un jeu d'essai
   (une erreur revient avec `isError` et un corps structuré `error` /
   `errorType` / `at` / `cause`). Ne considère jamais « créé » = « marche ».

## Règles de contenu `put_node` (sources fréquentes de 500)

- **Jamais de préfixe avec le nom du package (LA cause des 500)** : `node_nsName`
  est un chemin de **dossiers** + service (`dossier.sousDossier:service`) ; il ne
  contient JAMAIS le nom du package, lequel va uniquement dans `node_pkg`.
  Correct : `node_nsName = monpackage.commandes.api:creer` + `node_pkg = MonPackage`.
  FAUX : `node_nsName = MonPackage.commandes.api:creer` → 500. Même règle pour
  `folder_create`, `document_type_create`, `node_delete` : le chemin ne contient
  jamais le package.
  La raison : chaque segment du chemin est un **dossier qui doit déjà exister**.
  `MonPackage` (PascalCase) est un package, pas un dossier → `[ISS.0081.9001]`.
  `monpackage` en minuscules marche parce que c'est le dossier racine que tu as
  créé à l'étape 2.
- **Identité** : `node_nsName` = `"dossier.sousDossier:nomService"`, `node_pkg`
  = nom du package, `node_type` = `"service"`, `svc_type` = `"flow"`,
  `svc_subtype` = `"default"`, `svc_sigtype` = `"java 3.5"`.
- **Signature** : `sig_in` et `sig_out` DOIVENT porter
  `javaclass: "com.wm.util.Values"`.
- **WmPath** : `/champ;type;dim` où **type** = `1` String, `2` Record (anonyme),
  `3` Object, `4` RecordRef (doc type typé) ; **dim** = `0` scalaire, `1` tableau,
  `2` table 2D. Exemples : `/nom;1;0`, `/lignes;2;1`,
  `/comptes;4;1;pkg.doctypes:compte`.
- **LOOP sur tableau de records** : le MAPCOPY/MAPSET interne DOIT utiliser le
  type **4 (RecordRef)** avec le qualificateur de doc type, pas le type 2 :
  `"/comptes;4;0;pkg.doctypes:compte/nomClient;1;0"` (dim=0 = élément courant
  de l'itération).
- **MAPSET constante** : fournis la charge XML
  `data: "<Values version=\"2.0\"><value name=\"xml\">valeur</value></Values>"`
  avec `d_enc: "XMLValues"`, `mapseti18n: "true"`.
- **INVOKE** : mets `validate-in: "$none"` et `validate-out: "$none"` sauf besoin
  contraire. Les mappings INPUT/OUTPUT vont dans des `MAP` mode `INPUT`/`OUTPUT`
  enfants du nœud INVOKE.
- **TRY/CATCH** : ce sont des SEQUENCE **frères adjacents** (`"form": "TRY"` /
  `"form": "CATCH"`) ; à l'intérieur du TRY, déclenche le CATCH avec
  `EXIT from="$parent" signal="FAILURE"`. Les types `TRY`/`CATCH` n'existent pas.
- **REPEAT = type `RETRY`** : `{"type":"RETRY","count":"3","backoff":"5",
  "repeat-on":"FAILURE"|"SUCCESS"}` (flow.xml `<RETRY COUNT BACK-OFF LOOP-ON>`).
  `"type":"REPEAT"`, `repeat-interval`, `back-off` sont ignorés en silence
  (le nœud ET son sous-arbre disparaissent) ; `put_node` les refuse désormais.
- **BRANCH sur expressions** : clé **`evaluate-labels: "true"`** (pas
  `label-expressions`, ignorée → `[ISC.0049.9009] Missing required property
  switch` à l'exécution) ; pas de `switch` en mode expression ; `EXIT
  from="$loop"` dans une BRANCH marche dans LOOP comme dans RETRY.
- **Nettoyage** : après un LOOP, `MAPDELETE` les tableaux temporaires hors de la
  sortie.

## Quand un appel échoue : LIS le message, ne réessaie pas à l'aveugle

Le serveur MCP remonte désormais le **corps de la réponse IS** dans l'erreur
(format `HTTP 500: <détail webMethods>`). Ce détail est la cause réelle —
exploite-le au lieu de relancer un JSON identique. Correspondances fréquentes :

| Indice dans le corps | Cause probable | Correctif |
|---|---|---|
| `... does not exist` / `unknown node` sur un doc type | RecordRef vers doc type absent | crée le doc type d'abord (étape 3) |
| `folder` / parent introuvable | dossier parent manquant | `folder_create` d'abord |
| `NullPointerException` côté compilateur flow | WmPath mal formé / type incohérent | revérifie `;type;dim` et `javaclass` |
| `already exists` | nœud déjà présent | `node_get` pour comparer, ou supprime/mets à jour |
| `not writable` / package désactivé | package read-only/désactivé | active le package, ou choisis-en un autre |
| `has dependents` à la suppression | d'autres nœuds référencent celui-ci | `ns_dep_get_dependents` avant `node_delete` |
| `[ISC.0049.9009] Missing required property switch` à l'exécution | BRANCH écrite avec `label-expressions` (clé ignorée) | utilise `evaluate-labels` |
| `verification.status = mismatch` dans la réponse `put_node` | étape/clé inconnue d'IS supprimée en silence | lis `missing`, corrige le type/la clé (RETRY, evaluate-labels…) |
| `[ART.117.4030] Unable to create adapter service` (server.log) après un 200 | propriété du mauvais type Java (tableau vide → `Object[]`, nombre JSON) | omets les tableaux vides, chaînes pour int/boolean ; `adapter_service_create` vérifie maintenant l'existence |
| `[ART.114.243] ... "original" is null` sur un lookup `updateColumnNames` | dépendance `*tables.columnInfo` passée en chaîne brute | `values: [["<columnInfo>"]]` (tableau imbriqué) |

## Connexions adaptateur JDBC (≠ pools JDBC)

Un **pool JDBC** (`jdbc_pool_*`) sert à l'IS lui-même (ISCoreAudit, TN, xref).
Une **connexion adaptateur** (`adapter_connection_*`) est un nœud JCA dans un
package : c'est la seule chose sur laquelle `adapter_service_create` peut
s'appuyer. Le vocabulaire des pools (url, uid, pwd, drivers, mincon/maxcon)
n'existe pas dans `connection_settings`.

Séquence obligatoire — détails dans `wm://docs/adapter-connection-reference` :

1. `adapter_type_list` → prends `adapterName` **tel quel**. JDBC = `JDBCAdapter`.
   `WmJDBCAdapter` est le **package** → 500 `[ART.114.232] Unable to get the
   adapter type`.
2. `adapter_connection_metadata` → le `systemName` de chaque propriété EST la clé
   de `connection_settings`. JDBC : `datasourceClass` (obligatoire), `serverName`,
   `portNumber`, `databaseName`, `user`, `password`, plus `transactionType` /
   `driverType` optionnels. Aucune URL JDBC n'est acceptée ici.
   La classe DataSource s'écrit `com.wm.dd.jdbcx.*` — avec un x : les
   `com.wm.dd.jdbc.*` que renvoie `jdbc_driver_list` sont des Driver, réservés
   aux pools.
3. `adapter_connection_create` avec `connection_alias` = chemin de dossiers
   partant de la racine minuscule du package (`petstoreapi.connections:petstore`)
   — **jamais** `PetstoreAPI.connections:petstore` : contrairement à `put_node`,
   `createConnectionNode` crée les dossiers manquants, donc ça ne lève aucune
   erreur mais fabrique un dossier nommé comme le package, et l'alias réel n'est
   plus celui que tu attends.
4. `adapter_connection_enable` : la création n'est **pas** validée (même `{}`
   passe en HTTP 200, le nœud naît désactivé). C'est l'activation qui teste
   vraiment la configuration.
5. `adapter_connection_state` → `connectionState: enabled`, `hasError: false`,
   puis `adapter_resource_domain_lookup` (`catalogNames`) pour prouver que la
   base répond.

## Services adaptateur JDBC : passe par les outils de haut niveau

- **CustomSQL** → `jdbc_custom_sql_create(service_name, package_name,
  connection_alias, sql, inputs=[{name, jdbc_type}], outputs=[…]?,
  result_row_field?)` ; **BatchInsert** → `jdbc_batch_insert_create(…, schema,
  table, exclude_columns=[clés serial])`. Ils construisent les ~30 propriétés
  du template, créent le nœud et **vérifient qu'il existe** (l'ART refuse des
  nœuds en silence, HTTP 200 quand même).
- Avec `adapter_service_create` brut : jamais de tableau JSON vide, entiers et
  booléens en chaînes, longueurs de tableaux cohérentes ; `customSQLcolInfo`
  renvoie `-1` dès qu'il y a jointure/sous-requête/fonction → fournis les
  colonnes toi-même. Détails : `wm://docs/adapter-service-reference`.
- FSL (`fsl_deploy`) : à réserver aux services sans listes de documents ni
  boucles ni `pub.date` — le compilateur IS 12.1 supprime les copies de
  recordList et le corps des WHILE sans erreur. Sinon `put_node`.

## Suppression sûre

Avant `node_delete`, appelle `ns_dep_get_dependents` : supprimer un nœud
référencé ailleurs échoue (et peut casser d'autres services). Après une
création/suppression, si le namespace semble incohérent, `package_reload`.

## Garde-fous généraux

- Une seule source de vérité pour les chemins : les outils MCP. Pas de chemin de
  service inventé dans `service_invoke`.
- Lis le schéma de chaque outil (champs requis vs optionnels) avant l'appel.
- Multi-instances : `list_instances` puis passe `instance` si besoin.
- Vérifie systématiquement après mutation (`node_get`, `service_invoke`).
