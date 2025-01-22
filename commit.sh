git add .
git commit -m "$1"
#git push

# Pousser vers GitHub
echo "Pushing to GitHub..."
git push github main

echo "Push completed successfully!"