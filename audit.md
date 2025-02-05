Voici la traduction en français de l'audit :

---

**Localhost** consiste à créer son propre serveur HTTP et à le tester avec un navigateur réel.

Prenez le temps nécessaire pour comprendre le projet et le tester. Examiner le code source vous aidera grandement.

### Fonctionnel
L'étudiant est-il capable de justifier ses choix et d'expliquer les points suivants ?  
*Note : Demandez à l'étudiant de vous montrer l'implémentation dans le code source si nécessaire.*

- Comment fonctionne un serveur HTTP ?
- Quelle fonction a été utilisée pour le multiplexage I/O (entrées/sorties) et comment fonctionne-t-elle ?
- Le serveur utilise-t-il un seul `select` (ou équivalent) pour lire les requêtes des clients et écrire les réponses ?
- Pourquoi est-il important d'utiliser un seul `select` et comment cela a-t-il été réalisé ?
- Lisez le code qui va du `select` (ou équivalent) à la lecture et à l'écriture pour un client. Y a-t-il une seule lecture ou écriture par client par `select` (ou équivalent) ?
- Les valeurs de retour des fonctions I/O sont-elles correctement vérifiées ?
- Si une erreur est retournée par les fonctions précédentes sur un socket, le client est-il supprimé ?
- La lecture et l'écriture se font-elles TOUJOURS via un `select` (ou équivalent) ?

### Fichier de configuration
Vérifiez le fichier de configuration et modifiez-le si nécessaire. Les configurations suivantes fonctionnent-elles correctement ?

- Configurer un seul serveur avec un seul port.
- Configurer plusieurs serveurs avec des ports différents.
- Configurer plusieurs serveurs avec des noms d'hôte différents (par exemple : `curl --resolve test.com:80:127.0.0.1 http://test.com/`). Cela vise à confirmer si votre serveur distingue correctement les requêtes pour différents noms d'hôte, même s'ils résolvent vers la même IP et le même port.
- Configurer des pages d'erreur personnalisées.
- Limiter la taille du corps de la requête client (par exemple : `curl -X POST -H "Content-Type: plain/text" --data "CORPS avec quelque chose de plus court ou plus long que la limite du corps"`).
- Configurer des routes et s'assurer qu'elles sont prises en compte.
- Configurer un fichier par défaut si le chemin est un répertoire.
- Configurer une liste de méthodes acceptées pour une route (par exemple : essayer de supprimer quelque chose avec et sans permission).

### Méthodes et cookies
Pour chaque méthode, assurez-vous de vérifier le code de statut (200, 404, etc.) :

- Les requêtes GET fonctionnent-elles correctement ?
- Les requêtes POST fonctionnent-elles correctement ?
- Les requêtes DELETE fonctionnent-elles correctement ?
- Testez une requête INCORRECTE, le serveur fonctionne-t-il toujours correctement ?
- Téléversez quelques fichiers sur le serveur et récupérez-les pour vérifier qu'ils n'ont pas été corrompus.
- Un système de session et de cookies fonctionnel est-il présent sur le serveur ?

### Interaction avec le navigateur
Ouvrez le navigateur utilisé par l'équipe pendant les tests et son panneau d'outils de développement pour vous aider lors des tests.

- Le navigateur se connecte-t-il au serveur sans problème ?
- Les en-têtes de requête et de réponse sont-ils corrects ? (Le serveur devrait servir un site web statique complet sans aucun problème).
- Essayez une URL incorrecte sur le serveur, est-elle gérée correctement ?
- Essayez de lister un répertoire, est-ce géré correctement ?
- Essayez une URL redirigée, est-elle gérée correctement ?
- Vérifiez le CGI implémenté, fonctionne-t-il correctement avec des données découpées en morceaux (chunked) et non découpées (unchunked) ?

### Problèmes de ports
- Configurez plusieurs ports et sites web et assurez-vous que cela fonctionne comme prévu.
- Configurez le même port plusieurs fois. Le serveur devrait détecter l'erreur.

Ce que nous voulons vérifier, c'est comment votre serveur gère les erreurs ou conflits de configuration. Si vous configurez le même port plusieurs fois dans le fichier de configuration de votre serveur pour le même ou des serveurs différents, l'attente est que votre serveur devrait détecter cette erreur et la gérer de manière appropriée. Il s'agit de s'assurer que votre serveur peut identifier les mauvaises configurations.

- Configurez plusieurs serveurs simultanément avec des configurations différentes mais avec des ports communs. Demandez pourquoi le serveur devrait fonctionner si l'une des configurations ne fonctionne pas.

Cela vise à valider la gestion de la configuration de votre serveur. Vous configurerez plusieurs serveurs simultanément, chacun avec des configurations différentes mais avec des ports partagés. Si l'une de ces configurations n'est pas valide ou rencontre un problème, votre serveur devrait continuer à fonctionner pour les autres configurations sans être entièrement perturbé. L'objectif est de s'assurer qu'une erreur dans la configuration d'un serveur ne fasse pas tomber l'ensemble du serveur si les autres configurations sont correctement configurées.

### Test de charge (Siege) et stress
- Utilisez `siege` avec une méthode GET sur une page vide. La disponibilité devrait être d'au moins 99,5 % avec la commande `siege -b [IP]:[PORT]`.
- Vérifiez qu'il n'y a pas de fuite de mémoire (vous pouvez utiliser des outils comme `top`).
- Vérifiez qu'il n'y a pas de connexion suspendue.

### Général
- Il existe plus d'un système CGI, comme [Python, C++, Perl].
- Il existe une deuxième implémentation du serveur dans un langage différent (répétez les tests pratiques dessus avant de valider).

--- 

J'espère que cette traduction vous sera utile !