### **Audit traduit en français**

---

### **Localhost**
Localhost consiste à créer son propre serveur HTTP et à le tester avec un véritable navigateur.

Prenez le temps nécessaire pour comprendre le projet et le tester. L'exploration du code source vous aidera énormément.

---

## **Fonctionnalités**

L'étudiant est-il capable de justifier ses choix et d'expliquer les points suivants ?  
**Remarque :** Demandez à l'étudiant de montrer l'implémentation dans le code source lorsque cela est nécessaire.

- Comment fonctionne un serveur HTTP ?
- Quelle fonction a été utilisée pour le multiplexage d’E/S et comment fonctionne-t-elle ?
- Le serveur utilise-t-il **un seul** `select` (ou équivalent) pour lire les requêtes des clients et écrire les réponses ?
- Pourquoi est-il important d’utiliser **un seul** `select` et comment cela a-t-il été mis en place ?
- Lisez le code qui va du `select` (ou équivalent) jusqu'à la lecture et l'écriture d'un client. Y a-t-il **une seule** lecture ou écriture par client et par `select` (ou équivalent) ?
- Les valeurs de retour des fonctions d’E/S sont-elles bien vérifiées ?
- Si une erreur est retournée par une de ces fonctions sur un socket, le client est-il supprimé ?
- L’écriture et la lecture passent-elles **toujours** par un `select` (ou équivalent) ?

---

## **Fichier de configuration**

Vérifiez le fichier de configuration et modifiez-le si nécessaire. Les configurations suivantes fonctionnent-elles correctement ?

- **Configuration d'un seul serveur** avec un seul port.
- **Configuration de plusieurs serveurs** avec différents ports.
- **Configuration de plusieurs serveurs avec des noms de domaine différents** (par exemple :
  ```sh
  curl --resolve test.com:80:127.0.0.1 http://test.com/
  ```
  Cela permet de vérifier si votre serveur différencie correctement les requêtes de différents noms de domaine même s'ils pointent vers la même IP et le même port).
- **Configuration de pages d'erreur personnalisées.**
- **Limite de taille du corps des requêtes clients** (par exemple :
  ```sh
  curl -X POST -H "Content-Type: plain/text" --data "TEXTE plus long ou plus court que la limite définie"
  ```
- **Configuration des routes** et vérification de leur prise en compte.
- **Configuration d'un fichier par défaut** lorsque le chemin pointe vers un répertoire.
- **Définition des méthodes HTTP autorisées pour une route** (exemple : essayer de **DELETE** une ressource avec et sans autorisation).

---

## **Méthodes HTTP et cookies**

Pour chaque méthode, vérifiez les codes de statut HTTP (200, 404, etc.) :

- Les requêtes **GET** fonctionnent-elles correctement ?
- Les requêtes **POST** fonctionnent-elles correctement ?
- Les requêtes **DELETE** fonctionnent-elles correctement ?
- **Tester une requête invalide** : le serveur continue-t-il à fonctionner normalement ?
- **Uploader des fichiers** sur le serveur et les récupérer : les fichiers sont-ils intacts et non corrompus ?
- **Le système de sessions et de cookies fonctionne-t-il correctement ?**

---

## **Interaction avec le navigateur**

Ouvrez le navigateur utilisé par l'équipe pour les tests et affichez la console des outils développeur pour faciliter les vérifications.

- Le **navigateur se connecte-t-il au serveur sans problème** ?
- **Les en-têtes des requêtes et des réponses sont-ils corrects ?** (Le serveur doit être capable de **servir un site web statique** sans erreur).
- **Tester une URL erronée** : est-elle bien gérée ?
- **Tester la liste d’un répertoire** : est-elle bien gérée ?
- **Tester une URL redirigée** : est-elle bien gérée ?
- **Tester le CGI implémenté** : fonctionne-t-il correctement avec des données chunkées et non chunkées ?

---

## **Problèmes de ports**

- Configurer **plusieurs ports et sites web** et s’assurer que cela fonctionne correctement.
- Configurer **le même port plusieurs fois** : le serveur doit **détecter l’erreur**.
    - Vérifier comment le serveur gère **les erreurs de configuration**.
    - Si un même port est défini plusieurs fois dans le fichier de configuration, le serveur doit identifier cette erreur et la traiter correctement.
- Configurer **plusieurs serveurs avec des configurations différentes mais partageant le même port**.
    - Si **une configuration est invalide**, le serveur doit continuer de fonctionner avec les configurations valides.
    - **Une erreur dans une configuration ne doit pas faire planter l’ensemble du serveur** si les autres sont correctes.

---

## **Tests de charge et de performance**

- **Tester avec Siege** (`siege -b [IP]:[PORT]`) avec une requête GET sur une page vide.
    - **La disponibilité doit être d'au moins 99,5 %**.
- Vérifier s’il n’y a **aucune fuite mémoire** (avec `top` ou un autre outil).
- Vérifier qu’il n’y a **aucune connexion bloquée**.

---

## **Autres points à vérifier**

✅ **Le serveur supporte plusieurs types de CGI**, comme **Python, C++, Perl, etc.**  
✅ **Il existe une deuxième implémentation du serveur** dans un **langage différent**.  
✅ **Les tests pratiques sont validés sur cette deuxième implémentation.**

---

### **🎯 Conclusion**

Cet audit vise à vérifier **la robustesse, la fiabilité et les performances** du serveur HTTP en conditions réelles. Il couvre les aspects **fonctionnels, de configuration, d'interaction avec un navigateur, de gestion des erreurs, de performances et de sécurité**.

Si tu veux que je développe un point en particulier ou que je t’aide sur une partie spécifique du test, dis-moi ! 😊 🚀