Pour ajouter un `hostname` au fichier `/etc/hosts` sur un système Linux, vous devez éditer ce fichier en tant qu'administrateur (root) et ajouter une ligne qui associe une adresse IP à un nom d'hôte.

Voici les étapes à suivre :

1. **Ouvrir le fichier `/etc/hosts` avec un éditeur de texte** :
   Vous pouvez utiliser `nano`, `vim`, ou tout autre éditeur de texte. Par exemple, avec `nano` :

   ```bash
   sudo nano /etc/hosts
   ```

2. **Ajouter une ligne pour associer l'adresse IP au hostname** :
   Dans le fichier, ajoutez une ligne au format suivant :

   ```
   <adresse_IP> <hostname>
   ```

   Par exemple, si vous voulez associer l'adresse IP `192.168.1.100` au hostname `mon-serveur`, vous ajouteriez la ligne suivante :

   ```
   192.168.1.100 mon-serveur
   ```

3. **Sauvegarder et quitter l'éditeur** :
    - Si vous utilisez `nano`, appuyez sur `CTRL + O` pour sauvegarder, puis `CTRL + X` pour quitter.
    - Si vous utilisez `vim`, appuyez sur `ESC`, puis tapez `:wq` et appuyez sur `Entrée` pour sauvegarder et quitter.

4. **Vérifier que le changement a bien été pris en compte** :
   Vous pouvez utiliser la commande `ping` pour vérifier que le hostname est bien résolu vers l'adresse IP que vous avez spécifiée :

   ```bash
   ping mon-serveur
   ```

   Vous devriez voir des réponses provenant de l'adresse IP `128.0.0.10`.

### Exemple complet

Supposons que vous vouliez ajouter l'entrée suivante :

- Adresse IP : `128.0.0.10`
- Hostname : `mon-serveur`

Vous feriez :

```bash
sudo nano /etc/hosts
```

Ajoutez la ligne :

```
128.0.0.10 mon-serveur
```

Sauvegardez et quittez, puis testez avec :

```bash
ping mon-serveur
```

Cela devrait renvoyer des réponses de `128.0.0.10`.

### Remarque
- Le fichier `/etc/hosts` est utilisé pour la résolution de noms locale, avant que le système ne consulte un serveur DNS.
- Les modifications apportées à ce fichier sont immédiatement effectives, sans besoin de redémarrer le système.