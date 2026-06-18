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
5. `wm://docs/adapter-service-reference` — uniquement pour les services adaptateur JDBC

Ne devine JAMAIS un chemin de service IS ni la structure d'un nœud. Si tu n'es
pas sûr d'un format, **lis un nœud existant qui marche** avec `node_get` et
sers-t'en de modèle.

## Modèle mental : `flow_service_create` ≠ un flow utilisable

- `flow_service_create` (et `service_create`) ne crée qu'une **coquille vide**
  (aucune signature, aucune logique). Ce n'est PAS suffisant.
- Le vrai outil est **`put_node`** (API IS `putNode`) : il crée/met à jour le
  service complet (signature `sig_in`/`sig_out` + arbre `flow`). Tu peux
  d'ailleurs créer directement le service final en un seul `put_node`, sans
  passer par `flow_service_create`.

## Ordre des opérations (checklist — respecte-la, c'est ce qui évite les 500)

1. **Package** : vérifie qu'il existe, est activé et inscriptible
   (`package_list` / `package_info`). Sinon `package_create`.
2. **Dossiers parents** : crée chaque dossier de l'arborescence AVANT le service
   (`folder_create`). Un `put_node` dans un dossier inexistant échoue.
3. **Document types** : si ta signature ou tes mappings utilisent des champs
   RecordRef (type 4) ou des doc types, **crée-les d'abord**
   (`document_type_create` puis `put_node` pour les champs). Référencer un doc
   type inexistant fait planter le compilateur de flow → 500.
4. **Service** : `put_node` avec le `node_data` complet (voir règles ci-dessous).
5. **Vérifie** : `node_get` (le nœud est bien là et complet) puis
   `service_invoke` avec un jeu d'essai. Ne considère jamais « créé » = « marche ».

## Règles de contenu `put_node` (sources fréquentes de 500)

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
- **TRY/CATCH** : ce sont des SEQUENCE **frères adjacents** ; à l'intérieur du
  TRY, déclenche le CATCH avec `EXIT from="$parent" signal="FAILURE"`.
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
