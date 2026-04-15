Le but de cette plateforme est de permettre au bureau & aux 
responsables de l'IE (IsenEngineering) de pouvoir gérer les redirections et liens 
sur le nom de domaine de isenengineering.fr.
*ex: discord.isenengineering.fr -> discord.gg/...*
*ex: event.isenengineering.fr/ndi -> helloasso.com/...* 

## Structure
front: `html` + `tailwindcss`
back: `rust` [(`axum`, `surrealdb`, `tokio`...)](Cargo.toml)

## Build

Pour démarrer un serveur de développement
`cargo run`

Pour construire le binaire
`cargo build --release`

Pour construire l'image
`docker buildx build --platform linux/amd64 -t ghcr.io/isenengineering/link-ie .`

Pour mettre à jour l'image sur github
`docker push ghcr.io/isenengineering/link-ie`
